//! wasm-bindgen surface: JS names for the crate's API, over an opaque
//! compiled-pattern-set handle.

use crate::{
    builtin_pattern_set, builtin_pattern_set_names, compile_pattern_text, find_kashida_points,
    find_kashida_points_patterns, is_builtin_pattern_set, PatternSet,
};
use wasm_bindgen::prelude::*;

/// An opaque handle over a compiled pattern set.
#[wasm_bindgen]
pub struct CompiledPatternSet {
    inner: PatternSet,
}

/// The version of the library.
#[wasm_bindgen(js_name = version)]
pub fn version_js() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Compiles `.pat` pattern text into an opaque handle.
#[wasm_bindgen(js_name = compilePatternText)]
pub fn compile_pattern_text_js(text: &str) -> Result<CompiledPatternSet, JsError> {
    compile_pattern_text(text)
        .map(|inner| CompiledPatternSet { inner })
        .map_err(|error| JsError::new(&error.to_string()))
}

/// The names of the built-in pattern sets.
#[wasm_bindgen(js_name = builtinPatternSetNames)]
pub fn builtin_pattern_set_names_js() -> Vec<String> {
    builtin_pattern_set_names()
        .iter()
        .map(|name| name.to_string())
        .collect()
}

/// The built-in pattern set of that name, or `undefined` if there is none.
/// `builtinPatternSetNames` lists the names.
#[wasm_bindgen(js_name = builtinPatternSet)]
pub fn builtin_pattern_set_js(name: &str) -> Option<CompiledPatternSet> {
    builtin_pattern_set(name).map(|set| CompiledPatternSet { inner: set.clone() })
}

/// Whether `name` refers to a built-in pattern set, without compiling it.
#[wasm_bindgen(js_name = isBuiltinPatternSet)]
pub fn is_builtin_pattern_set_js(name: &str) -> bool {
    is_builtin_pattern_set(name)
}

/// Returns `[cleaned: string, { index, priority }[]]`.
#[wasm_bindgen(js_name = findKashidaPoints)]
pub fn find_kashida_points_js(
    word: &str,
    set: &CompiledPatternSet,
    remove_existing_kashida: bool,
) -> Result<JsValue, JsValue> {
    let cleaned_and_points = find_kashida_points(word, &set.inner, remove_existing_kashida);
    serde_wasm_bindgen::to_value(&cleaned_and_points).map_err(Into::into)
}

/// Returns `{ index, priority }[]`.
#[wasm_bindgen(js_name = findKashidaPointsPatterns)]
pub fn find_kashida_points_patterns_js(
    word: &str,
    set: &CompiledPatternSet,
) -> Result<JsValue, JsValue> {
    let points = find_kashida_points_patterns(word, &set.inner);
    serde_wasm_bindgen::to_value(&points).map_err(Into::into)
}
