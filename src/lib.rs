//! A library for finding _kashida_ (_tatweel_) insertion points and
//! priorities, driven by a small pattern language.
//!
//! Given a text and a compiled pattern set, the crate returns the possible
//! _kashida_ insertion points and their priorities.
//!
//! # Example
//!
//! Here is an example that breaks a text into lines and justifies them by
//! inserting _kashida_. For simplicity, the example assumes a monospaced
//! font where every character has the same width.
//!
//! 1. Break text into lines, greedily
//! 2. For each line, insert the highest priority _kashidas_ first across the
//!    line:
//!    1. Only the highest _kashida_ point in any given word
//!    2. Up to `MAX_KASHIDA` _kashidas_ at each point
//! 4. Repeat with the next priority, until the line is filled, or there are no
//!    more _kashida_ points to fill.
//!
//! The example is also runnable with
//! `cargo run --example justify`:
//!
//! ```
#![doc = include_str!("../examples/justify.rs")]
//! ```
//!
//! Which prints:
//!
//! <pre dir="rtl">
//! unjustified
//! قال أفلاطون: «الخط عقال العقل».
//! وقال إقليدس الإغريقي: «الخط
//! هندسة روحانية وإن ظهرت بآلة
//! جسمانية». وقال أبو دلف رحالة
//! القرن العاشر الميلادي: «الخط
//! رياض العلوم». وقال النظام
//! المعتزلي: «الخط أصيل في الروح
//! وإن ظهر بحواس البدن».
//!
//! justified
//! قال أفلاطون: «الخـط عقال العقل».
//! وقــال إقليـدس الإغريقي: «الخــط
//! هندســة روحانيــة وإن ظهرت بـآلة
//! جسمانيــة». وقــال أبو دلف رحالة
//! القرن العاشــر الميلادي: «الخــط
//! ريــاض العـلوم». وقــال النــظام
//! المعتزلي: «الخــط أصـيل في الروح
//! وإن ظهر بحواس البدن».
//! </pre>

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(unreachable_pub)]

mod builtin;
mod error;
mod grapheme;
mod pattern;
mod rasm;
mod resolve;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod wasm;

#[cfg(test)]
mod tests;

pub use builtin::{builtin_pattern_set, builtin_pattern_set_names, is_builtin_pattern_set};
pub use error::{CompileError, CompileErrorKind};
pub use pattern::{compile_pattern_text, PatternSet};

use grapheme::{is_bare_tatweel_at, joined_runs, split_graphemes, KASHIDA};
use resolve::resolve_run;

/// A point where a kashida may be inserted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(serde::Serialize)
)]
pub struct KashidaPoint {
    /// The grapheme cluster index the kashida goes after.
    pub index: u32,
    /// The kashida point priority, from 0–9, higher priority means a more
    /// preferable insertion point.
    pub priority: u8,
}

/// Kashida insertion points for `text` from the pattern set alone.
pub fn find_kashida_points_patterns(text: &str, set: &PatternSet) -> Vec<KashidaPoint> {
    let graphemes = split_graphemes(text);
    let mut out = Vec::new();
    for run in joined_runs(&graphemes) {
        out.extend(resolve_run(&graphemes, &run, set));
    }
    out
}

fn strip_bare_tatweel(text: &str) -> String {
    if !text.contains(KASHIDA) {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for k in 0..chars.len() {
        if is_bare_tatweel_at(&chars, k) {
            continue;
        }
        out.push(chars[k]);
    }
    out
}

/// Kashida insertion points for `text` under the given pattern set.
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
    text: &str,
    set: &PatternSet,
    remove_existing_kashida: bool,
) -> (String, Vec<KashidaPoint>) {
    let cleaned = if remove_existing_kashida {
        strip_bare_tatweel(text)
    } else {
        text.to_string()
    };
    let points = find_kashida_points_patterns(&cleaned, set);
    (cleaned, points)
}
