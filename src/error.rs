//! Pattern-compilation errors.

use core::fmt;

/// An error while compiling pattern text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CompileError {
    /// What went wrong.
    pub kind: CompileErrorKind,
    /// 1-based line number into the compiled pattern text.
    pub line_number: usize,
}

/// What went wrong on a pattern line.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompileErrorKind {
    /// A `[…]` length guard that is malformed or that no run can satisfy.
    InvalidLengthGuard(String),
    /// A `[` length guard with no closing `]`.
    UnterminatedLengthGuard,
    /// A `{` group set with no closing `}`.
    UnterminatedGroupSet,
    /// A `{}` group set with no group names.
    EmptyGroupSet,
    /// A group name that is not a Unicode Joining_Group long name.
    UnknownGroupName(String),
    /// A token after the trailing `.` run boundary.
    TokenAfterTrailingBoundary,
    /// A `\` in a two-digit priority not followed by a digit.
    ExpectedDigitAfterBackslash,
    /// A two-digit priority whose second digit is greater than its first.
    IncreasingPriority {
        /// The starting priority.
        base: u8,
        /// The end priority it drops to.
        min: u8,
    },
    /// A `\` that does not follow a priority digit.
    BackslashWithoutDigit,
    /// A `^` not followed by `{`, `@`, or `=`.
    CaretNotFollowed,
    /// An `@` or `=` with no group name after it.
    EmptyGroupName,
    /// A pattern line with no letter tokens.
    NoLetters,
    /// A character that is neither pattern syntax nor a joining letter.
    StrayCharacter(char),
    /// Two weights (digits or `!`) in the same inter-token gap.
    ConflictingWeights,
    /// A weight in the gap between a token and a `.`, where no junction
    /// exists.
    WeightOutsideRun,
}

impl fmt::Display for CompileErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileErrorKind::InvalidLengthGuard(body) => {
                write!(f, "Invalid length guard “[{body}]”")
            }
            CompileErrorKind::UnterminatedLengthGuard => write!(f, "Unterminated length guard"),
            CompileErrorKind::UnterminatedGroupSet => write!(f, "Unterminated “{{” group set"),
            CompileErrorKind::EmptyGroupSet => write!(f, "Empty “{{}}” group set"),
            CompileErrorKind::UnknownGroupName(name) => {
                write!(f, "Unknown Unicode Joining_Group name “{name}”")
            }
            CompileErrorKind::TokenAfterTrailingBoundary => {
                write!(f, "Token after a trailing “.” boundary")
            }
            CompileErrorKind::ExpectedDigitAfterBackslash => {
                write!(f, "Expected a digit after “\\”")
            }
            CompileErrorKind::IncreasingPriority { base, min } => {
                write!(f, "Priority must not increase ({base}\\{min})")
            }
            CompileErrorKind::BackslashWithoutDigit => {
                write!(f, "“\\” must follow a priority digit")
            }
            CompileErrorKind::CaretNotFollowed => {
                write!(f, "“^” must be followed by “{{”, “@”, or “=”")
            }
            CompileErrorKind::EmptyGroupName => write!(f, "Empty group name"),
            CompileErrorKind::NoLetters => write!(f, "Pattern has no letters"),
            CompileErrorKind::StrayCharacter(ch) => write!(f, "Stray character {ch:?}"),
            CompileErrorKind::ConflictingWeights => {
                write!(f, "Conflicting weights at one junction")
            }
            CompileErrorKind::WeightOutsideRun => {
                write!(f, "Weight outside the run at a “.” boundary")
            }
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line_number, self.kind)
    }
}

impl std::error::Error for CompileError {}
