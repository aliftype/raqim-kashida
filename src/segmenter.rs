//! Finding joined run boundaries.

use crate::grapheme::{analyze_grapheme, is_joining_type};
use icu_properties::props::JoiningType;
use icu_segmenter::GraphemeClusterSegmenter;

/// Splits text into runs of joined letters.
///
/// Returns boundary offsets, including 0 and the end of the string.
/// An empty string has one boundary, at 0.
///
/// # Example
///
/// ```
/// use kashida::JoinedRunSegmenter;
///
/// let text = "خط الزوال";
/// let boundaries: Vec<usize> = JoinedRunSegmenter::new().segment_str(text).collect();
/// let runs: Vec<&str> = boundaries.windows(2).map(|w| &text[w[0]..w[1]]).collect();
/// assert_eq!(runs, ["خط", " ", "ا", "لز", "و", "ا", "ل"]);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct JoinedRunSegmenter;

impl JoinedRunSegmenter {
    /// Creates a joined run segmenter.
    pub const fn new() -> Self {
        Self
    }

    /// Returns joined run boundaries as UTF-8 byte offsets.
    pub fn segment_str(self, input: &str) -> impl Iterator<Item = usize> {
        let grapheme_boundaries: Vec<usize> =
            GraphemeClusterSegmenter::new().segment_str(input).collect();
        let mut boundaries = vec![0];
        let mut previous_type: Option<JoiningType> = None;
        for pair in grapheme_boundaries.windows(2) {
            let grapheme = analyze_grapheme(&input[pair[0]..pair[1]]);
            // A transparent cluster stays in the segment before it.
            if grapheme.joining_type == JoiningType::Transparent {
                continue;
            }
            // Continue the run if the two clusters join each other.
            let can_join = is_joining_type(grapheme.joining_type);
            let continues_run = can_join
                && previous_type.is_some_and(|previous| {
                    previous != JoiningType::RightJoining
                        && grapheme.joining_type != JoiningType::LeftJoining
                });
            if !continues_run && pair[0] > 0 {
                boundaries.push(pair[0]);
            }
            previous_type = can_join.then_some(grapheme.joining_type);
        }
        if !input.is_empty() {
            boundaries.push(input.len());
        }
        boundaries.into_iter()
    }

    /// Returns joined run boundaries as UTF-16 code unit offsets.
    ///
    /// Unpaired surrogates are treated as U+FFFD (the replacement character).
    pub fn segment_utf16(self, input: &[u16]) -> impl Iterator<Item = usize> {
        let text: String = char::decode_utf16(input.iter().copied())
            .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
        // Replacing an unpaired surrogate preserves its UTF-16 length.
        let mut chars = text.chars();
        let (mut byte_offset, mut code_unit_offset) = (0, 0);
        let boundaries: Vec<usize> = self
            .segment_str(&text)
            .map(|boundary| {
                while byte_offset < boundary {
                    let c = chars.next().expect("boundaries fall between characters");
                    byte_offset += c.len_utf8();
                    code_unit_offset += c.len_utf16();
                }
                code_unit_offset
            })
            .collect();
        boundaries.into_iter()
    }
}
