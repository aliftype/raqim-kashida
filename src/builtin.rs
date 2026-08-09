//! The built-in pattern sets, compiled on first use.

use crate::pattern::{compile_pattern_text, PatternSet};
use std::sync::OnceLock;

const ARABIC_SIMPLE_TEXT: &str = include_str!("../data/arabic-simple.pat");
const ARABIC_NASKH_TEXT: &str = include_str!("../data/arabic-naskh.pat");

fn arabic_simple_set() -> &'static PatternSet {
    static SET: OnceLock<PatternSet> = OnceLock::new();
    SET.get_or_init(|| {
        compile_pattern_text(ARABIC_SIMPLE_TEXT).expect("built-in \"arabic-simple\" compiles")
    })
}

fn arabic_naskh_set() -> &'static PatternSet {
    static SET: OnceLock<PatternSet> = OnceLock::new();
    SET.get_or_init(|| {
        compile_pattern_text(ARABIC_NASKH_TEXT).expect("built-in \"arabic-naskh\" compiles")
    })
}

/// Whether `name` refers to a built-in pattern set, without compiling it,
/// unlike [`builtin_pattern_set`].
pub fn is_builtin_pattern_set(name: &str) -> bool {
    matches!(name, "arabic-simple" | "arabic-naskh")
}

/// The built-in pattern set of that name.
pub fn builtin_pattern_set(name: &str) -> Option<&'static PatternSet> {
    match name {
        "arabic-simple" => Some(arabic_simple_set()),
        "arabic-naskh" => Some(arabic_naskh_set()),
        _ => None,
    }
}
