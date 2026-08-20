//! The built-in pattern sets, compiled on first use.

use crate::pattern::{compile_pattern_text, PatternSet};
use std::sync::OnceLock;

const ARABIC_SIMPLE_TEXT: &str = include_str!("../data/arabic-simple.pat");
const ARABIC_NASKH_TEXT: &str = include_str!("../data/arabic-naskh.pat");
const ARABIC_NASTALIQ_TEXT: &str = include_str!("../data/arabic-nastaliq.pat");
const SYRIAC_TEXT: &str = include_str!("../data/syriac.pat");

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

fn arabic_nastaliq_set() -> &'static PatternSet {
    static SET: OnceLock<PatternSet> = OnceLock::new();
    SET.get_or_init(|| {
        compile_pattern_text(ARABIC_NASTALIQ_TEXT).expect("built-in \"arabic-nastaliq\" compiles")
    })
}

fn syriac_set() -> &'static PatternSet {
    static SET: OnceLock<PatternSet> = OnceLock::new();
    SET.get_or_init(|| compile_pattern_text(SYRIAC_TEXT).expect("built-in \"syriac\" compiles"))
}

/// The names of the built-in pattern sets:
///
/// - `"arabic-naskh"`: Arabic _kashida_ insertion rules suitable for classical
///   _naskh_ and _naskh_-like typefaces (e.g. _thuluth_) that follow the
///   classical rules of Arabic calligraphy and have advanced relations between
///   letters.
///   The rules are derived mainly from the publications of [Benatia et al.]
///   on the subject, but overridden where they disagree with Fawzi Salim
///   Afifi’s _Taallum al-khatt al-arabi_, part 3 (تعلم الخط العربي، الجزء
///   الثالث).
/// - `"arabic-nastaliq"`: Arabic _kashida_ insertion rules suitable for
///   classical _nastaliq_ typefaces. Uses the `arabic-naskh` rules as a base
///   with some tailoring for _nastaliq_.
///   Currently based on Afifi’s _Silsilat taalim al-khatt al-arabi_, part 9:
///   _al-khatt al-farisi_ (سلسلة تعليم الخط العربي، الجزء التاسع: الخط
///   الفارسي).
/// - `"arabic-simple"`: Arabic _kashida_ insertion rules suitable for “simple”
///   typefaces, i.e. those where letters have only the basic forms with no or
///   very limited relations between letters (no or very few ligatures,
///   contextual alternates, and so on). It is also suitable for _kufic_ styles
///   of Arabic in general.
///   The rules are based on [Microsoft justification rules], which are rooted
///   in Arabic newspaper typesetting.
/// - `"syriac"`: Syriac _kashida_ insertion rules, following the [guidelines
///   proposed for justified Syriac].
///
/// See the `.pat` sources for the rules themselves and their citations.
///
/// [Microsoft justification rules]: https://web.archive.org/web/20130308140133/microsoft.com/middleeast/msdn/JustifyingText-CSS.aspx
/// [Benatia et al.]: https://www.tug.org/tugboat/tb27-2/tb87benatia.pdf
/// [guidelines proposed for justified Syriac]: https://bugs.documentfoundation.org/show_bug.cgi?id=140767
pub fn builtin_pattern_set_names() -> &'static [&'static str] {
    &["arabic-naskh", "arabic-nastaliq", "arabic-simple", "syriac"]
}

/// Whether `name` refers to a built-in pattern set, without compiling it,
/// unlike [`builtin_pattern_set`].
pub fn is_builtin_pattern_set(name: &str) -> bool {
    builtin_pattern_set_names().contains(&name)
}

/// The built-in pattern set of that name, compiled on first use, or `None`
/// if there is none. [`builtin_pattern_set_names`] lists and describes them.
pub fn builtin_pattern_set(name: &str) -> Option<&'static PatternSet> {
    match name {
        "arabic-naskh" => Some(arabic_naskh_set()),
        "arabic-nastaliq" => Some(arabic_nastaliq_set()),
        "arabic-simple" => Some(arabic_simple_set()),
        "syriac" => Some(syriac_set()),
        _ => None,
    }
}
