//! A library for finding _kashida_ (_tatweel_) insertion points and
//! priorities, driven by a small pattern language.
//!
//! Given a word and a compiled pattern set, the crate returns the possible
//! _kashida_ insertion points and their priorities.

#![forbid(unsafe_code)]
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

pub use builtin::{builtin_pattern_set, builtin_pattern_set_names, is_builtin_pattern_set};
pub use error::{CompileError, CompileErrorKind};
pub use pattern::{compile_pattern_text, PatternSet};

use grapheme::{is_bare_tatweel_at, joined_runs, split_graphemes, KASHIDA};
use resolve::resolve_run;

/// A point where a kashida may be inserted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KashidaPoint {
    /// The grapheme cluster index the kashida goes after.
    pub index: u32,
    /// The kashida point priority, from 0–9, higher priority means a more
    /// preferable insertion point.
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
/// Any **bare** kashida already in the text is stripped first, unless
/// `remove_existing_kashida` is `false`. A kashida carrying a mark serves as
/// a seat for it, so it is always kept.
///
/// Returns the (possibly stripped) text along with the points, whose
/// indices refer to it.
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
