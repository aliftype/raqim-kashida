//! Grapheme clustering and Arabic-like joining analysis.
//!
//! Splits text into UAX #29 grapheme clusters and determines their joining
//! types, joining groups, and positional forms.

use icu_properties::props::{JoiningGroup, JoiningType};
use icu_properties::CodePointMapData;
use icu_segmenter::GraphemeClusterSegmenter;

pub(crate) const KASHIDA: char = '\u{0640}';

#[derive(Clone, Copy, Debug)]
pub(crate) struct Grapheme {
    pub(crate) base_codepoint: u32,
    pub(crate) joining_group: JoiningGroup,
    pub(crate) joining_type: JoiningType,
    pub(crate) is_mark_seat: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoiningForm {
    Isolated,
    Initial,
    Medial,
    Final,
}

// A bare tatweel that carries no mark is an elongation and can be stripped.
// One that carries a mark serves as a seat for it and must be kept.
pub(crate) fn is_bare_tatweel_at(chars: &[char], k: usize) -> bool {
    if chars[k] != KASHIDA {
        return false;
    }
    let joining_types = CodePointMapData::<JoiningType>::new();
    !matches!(chars.get(k + 1), Some(&c) if joining_types.get(c) == JoiningType::Transparent)
}

pub(crate) fn split_graphemes(text: &str) -> Vec<Grapheme> {
    let boundaries: Vec<usize> = GraphemeClusterSegmenter::new().segment_str(text).collect();
    boundaries
        .windows(2)
        .map(|pair| analyze_grapheme(&text[pair[0]..pair[1]]))
        .collect()
}

pub(crate) fn analyze_grapheme(cluster: &str) -> Grapheme {
    let joining_types = CodePointMapData::<JoiningType>::new();
    let joining_groups = CodePointMapData::<JoiningGroup>::new();
    let base = cluster.chars().next().expect("non-empty cluster");
    let mut joining_type = joining_types.get(base);
    // ZWJ and ZWNJ can change the cluster's joining type.
    if cluster
        .chars()
        .skip(1)
        .any(|c| joining_types.get(c) == JoiningType::NonJoining)
    {
        joining_type = match joining_type {
            JoiningType::DualJoining | JoiningType::JoinCausing => JoiningType::RightJoining,
            JoiningType::LeftJoining | JoiningType::Transparent => JoiningType::NonJoining,
            other => other,
        };
    } else if joining_type == JoiningType::DualJoining
        && cluster
            .chars()
            .skip(1)
            .any(|c| joining_types.get(c) == JoiningType::JoinCausing)
    {
        joining_type = JoiningType::JoinCausing;
    }
    Grapheme {
        base_codepoint: base as u32,
        joining_group: joining_groups.get(base),
        joining_type,
        is_mark_seat: base == KASHIDA
            && cluster
                .chars()
                .any(|c| joining_types.get(c) == JoiningType::Transparent),
    }
}

pub(crate) fn is_joining_type(joining_type: JoiningType) -> bool {
    matches!(
        joining_type,
        JoiningType::DualJoining
            | JoiningType::RightJoining
            | JoiningType::LeftJoining
            | JoiningType::JoinCausing
    )
}

pub(crate) fn joins_left(graphemes: &[Grapheme], index: usize) -> bool {
    let cur = &graphemes[index];
    if !is_joining_type(cur.joining_type) || cur.joining_type == JoiningType::RightJoining {
        return false;
    }
    match graphemes.get(index + 1) {
        Some(next) => {
            is_joining_type(next.joining_type) && next.joining_type != JoiningType::LeftJoining
        }
        None => false,
    }
}

pub(crate) fn joins_right(graphemes: &[Grapheme], index: usize) -> bool {
    let cur = &graphemes[index];
    if !is_joining_type(cur.joining_type) || cur.joining_type == JoiningType::LeftJoining {
        return false;
    }
    let prev = index.checked_sub(1).map(|i| &graphemes[i]);
    match prev {
        Some(p) if is_joining_type(p.joining_type) => p.joining_type != JoiningType::RightJoining,
        _ => false,
    }
}

pub(crate) fn form_of(graphemes: &[Grapheme], index: usize) -> JoiningForm {
    let right = joins_right(graphemes, index);
    let left = joins_left(graphemes, index);
    match (right, left) {
        (true, true) => JoiningForm::Medial,
        (false, true) => JoiningForm::Initial,
        (true, false) => JoiningForm::Final,
        (false, false) => JoiningForm::Isolated,
    }
}

// Grapheme indices used for pattern matching within one segment.
// Tatweels carrying marks are skipped.
pub(crate) fn find_run_indices(graphemes: &[Grapheme]) -> Vec<usize> {
    (0..graphemes.len())
        .filter(|&i| is_joining_type(graphemes[i].joining_type) && !graphemes[i].is_mark_seat)
        .collect()
}
