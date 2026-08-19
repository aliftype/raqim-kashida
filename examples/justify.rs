use icu_segmenter::{
    options::{LineBreakOptions, WordBreakInvariantOptions},
    GraphemeClusterSegmenter, LineSegmenter, WordSegmenter,
};
use kashida::{builtin_pattern_set, find_kashida_points, PatternSet};

const KASHIDA: &str = "\u{0640}";
const WIDTH: usize = 32;

const MAX_KASHIDA: usize = 2;

const TEXT: &str = concat!(
    "قال أفلاطون: «الخط عقال العقل». وقال إقليدس ",
    "الإغريقي: «الخط هندسة روحانية وإن ظهرت بآلة ",
    "جسمانية». وقال أبو دلف رحالة القرن العاشر ",
    "الميلادي: «الخط رياض العلوم». وقال النظام المعتزلي: ",
    "«الخط أصيل في الروح وإن ظهر بحواس البدن»."
);

fn justify(line: &str, set: &PatternSet) -> String {
    // Split line into words, and find the highest-priority kashida point in
    // each word.
    let mut words = Vec::new();
    let mut prev = 0;
    for end in WordSegmenter::new_auto(WordBreakInvariantOptions::default()).segment_str(line) {
        let (cleaned, points) = find_kashida_points(&line[prev..end], set, true);
        prev = end;
        words.push((
            cleaned,
            points.into_iter().max_by_key(|point| point.priority),
        ));
    }

    // Insert kashidas until the line is filled, starting with highest-priority
    // points across the line.
    let mut room = WIDTH - line.chars().count();
    for priority in (0..=9).rev() {
        for (word, point) in &mut words {
            // No more space left to fill.
            if room == 0 {
                break;
            }
            if let Some(point) = point.filter(|point| point.priority == priority) {
                // The kashida goes after the grapheme cluster at point.index.
                let mut boundaries = GraphemeClusterSegmenter::new().segment_str(word);
                let index = boundaries.nth(point.index as usize + 1).unwrap();
                // Do not insert more than MAX_KASHIDA at each point.
                let count = MAX_KASHIDA.min(room);
                word.insert_str(index, &KASHIDA.repeat(count));
                room -= count;
            }
        }
    }
    words.into_iter().map(|(word, _)| word).collect()
}

fn main() {
    // Break the text into lines.
    let (mut lines, mut start, mut prev) = (Vec::new(), 0, 0);
    for brk in LineSegmenter::new_auto(LineBreakOptions::default()).segment_str(TEXT) {
        if TEXT[start..brk].trim_end().chars().count() > WIDTH {
            lines.push(TEXT[start..prev].trim_end());
            start = prev;
        }
        prev = brk;
    }
    lines.push(TEXT[start..].trim_end());

    // Print the unjustified lines.
    println!("unjustified");
    for line in &lines {
        println!("{line}");
    }

    // Print justified lines, the last line is unjustified.
    let pattern_set = builtin_pattern_set("arabic-naskh").unwrap();
    println!("\njustified");
    let last = lines.pop().unwrap();
    for line in lines {
        println!("{}", justify(line, pattern_set));
    }
    println!("{last}");
}
