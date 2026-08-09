//! Arabic kashida (tatweel) insertion-point finding.
//!
//! Given text plus a compiled pattern set, produce the junctions where a
//! kashida may be inserted, each with a priority (0–9, higher = stronger).

#![warn(missing_docs)]
#![warn(unreachable_pub)]

mod builtin;
mod error;
mod grapheme;
mod pattern;
mod rasm;
mod resolve;

#[cfg(test)]
mod tests;

pub use builtin::{builtin_pattern_set, is_builtin_pattern_set};
pub use error::{CompileError, CompileErrorKind};
pub use pattern::{compile_pattern_text, PatternSet};

use grapheme::{is_bare_tatweel_at, joined_runs, split_graphemes, KASHIDA};
use resolve::resolve_run;

/// A junction where a kashida may be inserted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KashidaPoint {
    /// Insert a kashida after this grapheme-cluster index.
    pub index: u32,
    /// 0–9, higher = stronger, filled first.
    pub priority: u8,
}

/// Kashida insertion points for `word` from the pattern set alone.
pub fn find_kashida_points_patterns(word: &str, set: &PatternSet) -> Vec<KashidaPoint> {
    let graphemes = split_graphemes(word);
    let mut out = Vec::new();
    for run in joined_runs(&graphemes) {
        out.extend(resolve_run(&graphemes, &run, set));
    }
    out
}

fn strip_bare_tatweel(word: &str) -> String {
    if !word.contains(KASHIDA) {
        return word.to_string();
    }
    let chars: Vec<char> = word.chars().collect();
    let mut out = String::with_capacity(word.len());
    for k in 0..chars.len() {
        if is_bare_tatweel_at(&chars, k) {
            continue;
        }
        out.push(chars[k]);
    }
    out
}

/// Kashida insertion points for `word` under the given pattern set.
///
/// Bare user kashidas are stripped first (unless asked not to); a kept one
/// is an ordinary run letter that patterns can target, like
/// the built-in simple set's `@Tatweel 9` rule. Returns the (possibly stripped)
/// text along with the points, whose indices refer to it.
///
/// # Example
///
/// ```
/// use kashida::{builtin_pattern_set, find_kashida_points};
///
/// let set = builtin_pattern_set("arabic-simple").unwrap();
/// let (cleaned, points) = find_kashida_points("بيت", set, true);
/// for point in points {
///     // Insert a kashida after grapheme cluster `point.index`.
///     println!("{} @ {}", point.priority, point.index);
/// }
/// ```
pub fn find_kashida_points(
    word: &str,
    set: &PatternSet,
    remove_existing_kashida: bool,
) -> (String, Vec<KashidaPoint>) {
    let cleaned = if remove_existing_kashida {
        strip_bare_tatweel(word)
    } else {
        word.to_string()
    };
    let points = find_kashida_points_patterns(&cleaned, set);
    (cleaned, points)
}
