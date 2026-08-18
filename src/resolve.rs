//! Matching a compiled pattern set against the joined runs of the text.

use crate::grapheme::{form_of, is_joining_type, Grapheme};
use crate::pattern::{LengthGuard, PatternSet, Token, Weight};
use crate::rasm::rasm_matches;
use crate::KashidaPoint;
use icu_properties::props::JoiningType;

fn match_token(token: &Token, graphemes: &[Grapheme], index: usize) -> bool {
    let g = &graphemes[index];
    match token {
        Token::Any => is_joining_type(g.joining_type),
        Token::Literal(cp) => g.base_codepoint == *cp,
        Token::Group(group) => rasm_matches(*group, g.joining_group, form_of(graphemes, index)),
        Token::ExactGroup(group) => g.joining_group == *group,
        Token::GroupSet(members) => members.iter().any(|m| match_token(m, graphemes, index)),
        Token::NotGroupSet(members) => {
            is_joining_type(g.joining_type)
                && !members.iter().any(|m| match_token(m, graphemes, index))
        }
    }
}

fn guard_matches(guard: Option<LengthGuard>, len: usize) -> bool {
    match guard {
        None => true,
        Some(LengthGuard::Exact(n)) => len == n,
        Some(LengthGuard::Min(n)) => len >= n,
        Some(LengthGuard::Range { lo, hi }) => len >= lo && len <= hi,
        Some(LengthGuard::Open(_)) => true,
    }
}

// The run length where a two-digit priority is at its first digit. With no
// guard it is 2, the shortest run that has a connection.
fn guard_floor(guard: Option<LengthGuard>) -> usize {
    match guard {
        None => 2,
        Some(LengthGuard::Range { lo, .. }) => lo,
        Some(LengthGuard::Exact(n) | LengthGuard::Min(n) | LengthGuard::Open(n)) => n,
    }
}

// The priority in a run of `len` letters. It is `base` when `len` is `floor`,
// drops by one for every letter of difference either way, and never goes below
// `min`. Only `Open` matches a run shorter than its floor, so for every other
// guard `len` is at least `floor` and the direction never comes up.
fn effective_priority(base: u8, min: u8, len: usize, floor: usize) -> u8 {
    let value = base as i32 - (len as i32 - floor as i32).abs();
    value.max(min as i32) as u8
}

pub(crate) fn resolve_run(
    graphemes: &[Grapheme],
    run: &[usize],
    set: &PatternSet,
) -> Vec<KashidaPoint> {
    let len = run.len();
    if len < 2 {
        return Vec::new();
    }

    // The last priority for an given connection wins.
    let mut priorities: Vec<Option<u8>> = vec![None; len - 1];

    for pattern in &set.patterns {
        if !guard_matches(pattern.guard, len) {
            continue;
        }
        let floor = guard_floor(pattern.guard);
        let m = pattern.tokens.len();
        if m > len {
            continue;
        }
        for start in 0..=(len - m) {
            if pattern.leading_boundary && start != 0 {
                continue;
            }
            // A join-causing last member (a tatweel, or a ZWJ tail) still
            // joins forward, so nothing in that run is final.
            if pattern.trailing_boundary
                && (start + m != len
                    || graphemes[run[len - 1]].joining_type == JoiningType::JoinCausing)
            {
                continue;
            }
            let matched =
                (0..m).all(|k| match_token(&pattern.tokens[k], graphemes, run[start + k]));
            if !matched {
                continue;
            }

            for gap in 0..=m {
                let weight = match &pattern.weights[gap] {
                    Some(w) => w,
                    None => continue,
                };
                let point = start as isize + gap as isize - 1;
                if point < 0 || point > len as isize - 2 {
                    continue;
                }
                let point = point as usize;
                priorities[point] = match weight {
                    Weight::Suppress => None,
                    Weight::Priority { base, min } => {
                        Some(effective_priority(*base, *min, len, floor))
                    }
                };
            }
        }
    }

    let mut out = Vec::new();
    for point in 0..len - 1 {
        if let Some(priority) = priorities[point] {
            // If there is a mark seat, the kashida point goes after it (in
            // other words the mark seat is effectively part of the previous
            // cluster).
            let mut index = run[point];
            while graphemes.get(index + 1).is_some_and(|g| g.is_mark_seat) {
                index += 1;
            }
            out.push(KashidaPoint {
                index: index as u32,
                priority,
            });
        }
    }
    out
}
