//! The pattern language: IR types and the compiler.

use crate::error::{CompileError, CompileErrorKind};
use crate::grapheme::{is_joining_type, KASHIDA};
use crate::rasm::resolve_group_name;
use icu_properties::props::{JoiningGroup, JoiningType};
use icu_properties::CodePointMapData;

#[derive(Clone, Debug)]
pub(crate) enum Token {
    Group(JoiningGroup),      // @Name, positional rasm by group
    ExactGroup(JoiningGroup), // =Name, that Joining_Group alone, no folding
    GroupSet(Vec<Token>),     // {…}, any of its members
    NotGroupSet(Vec<Token>),  // ^{…}, none of its members
    Literal(u32),             // a letter, exact codepoint
    Any,                      // * wildcard
}

// `base` at the guard's floor length, stepping down 1 per extra letter to
// `min`, then holding (`min == base` is constant).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Weight {
    Priority { base: u8, min: u8 },
    Suppress,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum LengthGuard {
    Exact(usize),
    Min(usize),
    Range { lo: usize, hi: usize },
}

#[derive(Clone, Debug)]
pub(crate) struct CompiledPattern {
    pub(crate) guard: Option<LengthGuard>,
    pub(crate) tokens: Vec<Token>,
    // weights[k] is the contribution at the gap before token k; index
    // tokens.len() is the gap after the last token.
    pub(crate) weights: Vec<Option<Weight>>,
    pub(crate) leading_boundary: bool,
    pub(crate) trailing_boundary: bool,
}

/// A compiled set of kashida insertion patterns.
#[derive(Clone, Debug)]
pub struct PatternSet {
    pub(crate) patterns: Vec<CompiledPattern>,
}

fn strip_comment(raw: &str) -> String {
    let body = match raw.find('#') {
        Some(hash) => &raw[..hash],
        None => raw,
    };
    body.trim().to_string()
}

// letter ::= a codepoint with a joining Joining_Type
fn is_letter(ch: char) -> bool {
    is_joining_type(CodePointMapData::<JoiningType>::new().get(ch))
}

// An `@` reference folds positionally through the rasm classes. A group in
// none of them just matches itself alone. An `=` reference matches its
// Joining_Group alone in any position. Under either prefix, `Tatweel` names
// U+0640 ARABIC TATWEEL itself.
fn resolve_reference(name: &str) -> Result<Token, CompileErrorKind> {
    if name.strip_prefix(['@', '=']) == Some("Tatweel") {
        return Ok(Token::Literal(KASHIDA as u32));
    }
    let group = resolve_group_name(name)?;
    if name.starts_with('@') {
        Ok(Token::Group(group))
    } else {
        Ok(Token::ExactGroup(group))
    }
}

fn set_weight(
    weights: &mut Vec<Option<Weight>>,
    k: usize,
    weight: Weight,
) -> Result<(), CompileErrorKind> {
    if k >= weights.len() {
        weights.resize(k + 1, None);
    }
    // Two weights in one gap make no sense.
    if weights[k].is_some() {
        return Err(CompileErrorKind::ConflictingWeights);
    }
    weights[k] = Some(weight);
    Ok(())
}

// A recursive-descent parser over one comment-stripped line:
//
//   line      ::= (use | pattern)? comment?
//   use       ::= "use" set_name
//   pattern   ::= guard? element+
//   guard     ::= "[" bound ("+" | "-" bound)? "]"
//   element   ::= token | weight | "."
//   token     ::= reference | set | "^" (set | reference) | letter | "*"
//   set       ::= "{" member+ "}"
//   member    ::= reference | letter
//   reference ::= ("@" | "=") name
//   weight    ::= digit ("\" digit)? | "!"
struct Parser<'a> {
    chars: &'a [char],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn digit(&mut self) -> Option<u8> {
        let digit = self.peek()?.to_digit(10)?;
        self.pos += 1;
        Some(digit as u8)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t')) {
            self.pos += 1;
        }
    }

    // pattern ::= guard? element+
    fn pattern(&mut self) -> Result<CompiledPattern, CompileErrorKind> {
        let guard = if self.peek() == Some('[') {
            Some(self.guard()?)
        } else {
            None
        };

        let mut tokens: Vec<Token> = Vec::new();
        let mut weights: Vec<Option<Weight>> = Vec::new();
        let mut leading_boundary = false;
        let mut trailing_boundary = false;

        // element ::= token | weight | "."
        loop {
            self.skip_whitespace();
            let Some(ch) = self.peek() else { break };
            if ch == '.' {
                self.pos += 1;
                if tokens.is_empty() {
                    leading_boundary = true;
                } else {
                    trailing_boundary = true;
                }
                continue;
            }
            if trailing_boundary {
                return Err(CompileErrorKind::TokenAfterTrailingBoundary);
            }
            if ch.is_ascii_digit() || ch == '!' || ch == '\\' {
                set_weight(&mut weights, tokens.len(), self.weight()?)?;
                continue;
            }
            tokens.push(self.token(ch)?);
        }

        if tokens.is_empty() {
            return Err(CompileErrorKind::NoLetters);
        }
        weights.resize(tokens.len() + 1, None);
        // A weight in the gap between a token and a `.` can never land on a
        // connection: no connection exists at a run's edge.
        if (leading_boundary && weights[0].is_some())
            || (trailing_boundary && weights[tokens.len()].is_some())
        {
            return Err(CompileErrorKind::WeightOutsideRun);
        }
        Ok(CompiledPattern {
            guard,
            tokens,
            weights,
            leading_boundary,
            trailing_boundary,
        })
    }

    // guard ::= "[" (bound | bound ":" | bound ":" bound) "]"
    fn guard(&mut self) -> Result<LengthGuard, CompileErrorKind> {
        self.pos += 1; // the `[`
        let start = self.pos;
        while self.peek().is_some_and(|c| c != ']') {
            self.pos += 1;
        }
        if !self.eat(']') {
            return Err(CompileErrorKind::UnterminatedLengthGuard);
        }
        let body: String = self.chars[start..self.pos - 1].iter().collect();
        let trimmed = body.trim();
        let invalid = || CompileErrorKind::InvalidLengthGuard(body.clone());
        // bound ::= digit+.
        let bound = |s: &str| -> Result<usize, CompileErrorKind> {
            if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                return Err(invalid());
            }
            s.parse::<usize>().map_err(|_| invalid())
        };
        let guard = if let Some(stripped) = trimmed.strip_suffix(':') {
            LengthGuard::Min(bound(stripped)?)
        } else if let Some(colon) = trimmed.find(':') {
            LengthGuard::Range {
                lo: bound(&trimmed[..colon])?,
                hi: bound(&trimmed[colon + 1..])?,
            }
        } else {
            LengthGuard::Exact(bound(trimmed)?)
        };
        // Reject guards no run can satisfy: a connection needs two letters, and
        // a range must not be empty.
        let bounds_ok = match guard {
            LengthGuard::Exact(n) | LengthGuard::Min(n) => n >= 2,
            LengthGuard::Range { lo, hi } => lo >= 2 && lo <= hi,
        };
        if bounds_ok {
            Ok(guard)
        } else {
            Err(invalid())
        }
    }

    // weight ::= digit ("\" digit)? | "!"
    fn weight(&mut self) -> Result<Weight, CompileErrorKind> {
        if self.eat('!') {
            return Ok(Weight::Suppress);
        }
        let Some(base) = self.digit() else {
            // Only a lone `\` lands here.
            return Err(CompileErrorKind::BackslashWithoutDigit);
        };
        let mut min = base;
        if self.eat('\\') {
            // The priority drops from the first digit down to the second as
            // the run grows.
            match self.digit() {
                Some(end) => {
                    min = end;
                    if min > base {
                        return Err(CompileErrorKind::IncreasingPriority { base, min });
                    }
                }
                None => return Err(CompileErrorKind::ExpectedDigitAfterBackslash),
            }
        }
        Ok(Weight::Priority { base, min })
    }

    // token ::= reference | set | "^" (set | reference) | letter | "*"
    fn token(&mut self, ch: char) -> Result<Token, CompileErrorKind> {
        match ch {
            '*' => {
                self.pos += 1;
                Ok(Token::Any)
            }
            '{' => Ok(Token::GroupSet(self.set()?)),
            '^' => {
                self.pos += 1;
                match self.peek() {
                    Some('{') => Ok(Token::NotGroupSet(self.set()?)),
                    Some('@' | '=') => Ok(Token::NotGroupSet(vec![self.reference()?])),
                    _ => Err(CompileErrorKind::CaretNotFollowed),
                }
            }
            '@' | '=' => self.reference(),
            _ => {
                self.pos += 1;
                // A letter stands for itself alone. Anything else is an error,
                // not a token.
                if is_letter(ch) {
                    Ok(Token::Literal(ch as u32))
                } else {
                    Err(CompileErrorKind::StrayCharacter(ch))
                }
            }
        }
    }

    // set ::= "{" member+ "}"
    // member ::= reference | letter
    fn set(&mut self) -> Result<Vec<Token>, CompileErrorKind> {
        self.pos += 1; // the `{`
        let start = self.pos;
        while self.peek().is_some_and(|c| c != '}') {
            self.pos += 1;
        }
        if !self.eat('}') {
            return Err(CompileErrorKind::UnterminatedGroupSet);
        }
        let body: String = self.chars[start..self.pos - 1].iter().collect();
        if body.trim().is_empty() {
            return Err(CompileErrorKind::EmptyGroupSet);
        }
        let mut members = Vec::new();
        for part in body.split_whitespace() {
            if part.starts_with('@') || part.starts_with('=') {
                members.push(resolve_reference(part)?);
            } else {
                for ch in part.chars() {
                    if is_letter(ch) {
                        members.push(Token::Literal(ch as u32));
                    } else {
                        return Err(CompileErrorKind::StrayCharacter(ch));
                    }
                }
            }
        }
        Ok(members)
    }

    // reference ::= ("@" | "=") name
    // name ::= (ALPHA | "_")+
    fn reference(&mut self) -> Result<Token, CompileErrorKind> {
        let mut name = String::from(self.chars[self.pos]); // the `@` or `=`
        self.pos += 1;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() || c == '_' {
                name.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        if name.len() == 1 {
            return Err(CompileErrorKind::EmptyGroupName);
        }
        resolve_reference(&name)
    }
}

// line ::= pattern? comment?
fn parse_line(raw: &str) -> Result<Option<CompiledPattern>, CompileErrorKind> {
    let line = strip_comment(raw);
    if line.is_empty() {
        return Ok(None);
    }
    let chars: Vec<char> = line.chars().collect();
    Parser {
        chars: &chars,
        pos: 0,
    }
    .pattern()
    .map(Some)
}

// use ::= "use" set_name
// for the rules after it to override.
fn parse_use(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("use")?;
    if rest.starts_with([' ', '\t']) {
        Some(rest.trim())
    } else {
        None
    }
}

/// Compiles pattern text into a [`PatternSet`].
pub fn compile_pattern_text(text: &str) -> Result<PatternSet, CompileError> {
    let mut patterns = Vec::new();
    for (index, raw) in text.split('\n').enumerate() {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let context = |kind| CompileError {
            kind,
            line_number: index + 1,
        };
        if let Some(name) = parse_use(&strip_comment(raw)) {
            let imported = crate::builtin::builtin_pattern_set(name)
                .ok_or_else(|| context(CompileErrorKind::UnknownImport(name.to_string())))?;
            patterns.extend(imported.patterns.iter().cloned());
            continue;
        }
        if let Some(pattern) = parse_line(raw).map_err(context)? {
            patterns.push(pattern);
        }
    }
    Ok(PatternSet { patterns })
}
