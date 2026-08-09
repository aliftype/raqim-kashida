//! Rasm (skeleton) folding for group tokens.
//!
//! Some joining groups share a skeleton with others in certain positional
//! forms (the medial tooth, the feh/qaf head, the heh bowl, …). `@Name` and
//! `{…}` tokens fold through this table.

use crate::error::CompileErrorKind;
use crate::grapheme::JoiningForm;
use icu_properties::props::JoiningGroup;
use icu_properties::{PropertyNamesLong, PropertyParser};

pub(crate) struct RasmClass {
    pub(crate) groups: &'static [JoiningGroup],
    pub(crate) forms: &'static [JoiningForm],
}

// Classes that share the same rasm skeleton. This is per positional form,
// since some letters share one skeleton in initial/medial and another in
// isolated/final.
pub(crate) const RASM_CLASSES: &[RasmClass] = &[
    // Yeh barree and yeh-with-tail are excluded since they have no
    // initial/medial form.
    RasmClass {
        groups: &[
            JoiningGroup::Beh,
            JoiningGroup::Noon,
            JoiningGroup::AfricanNoon,
            JoiningGroup::Nya,
            JoiningGroup::Yeh,
            JoiningGroup::FarsiYeh,
        ],
        forms: &[JoiningForm::Initial, JoiningForm::Medial],
    },
    RasmClass {
        groups: &[
            JoiningGroup::Feh,
            JoiningGroup::AfricanFeh,
            JoiningGroup::Qaf,
            JoiningGroup::AfricanQaf,
        ],
        forms: &[JoiningForm::Initial, JoiningForm::Medial],
    },
    RasmClass {
        groups: &[JoiningGroup::Feh, JoiningGroup::AfricanFeh],
        forms: &[JoiningForm::Final, JoiningForm::Isolated],
    },
    RasmClass {
        groups: &[JoiningGroup::Qaf, JoiningGroup::AfricanQaf],
        forms: &[JoiningForm::Final, JoiningForm::Isolated],
    },
    RasmClass {
        groups: &[
            JoiningGroup::Heh,
            JoiningGroup::HehGoal,
            JoiningGroup::TehMarbuta,
            JoiningGroup::TehMarbutaGoal,
        ],
        forms: &[JoiningForm::Final, JoiningForm::Isolated],
    },
    RasmClass {
        groups: &[
            JoiningGroup::Noon,
            JoiningGroup::AfricanNoon,
            JoiningGroup::Nya,
        ],
        forms: &[JoiningForm::Final, JoiningForm::Isolated],
    },
    RasmClass {
        groups: &[
            JoiningGroup::Yeh,
            JoiningGroup::FarsiYeh,
            JoiningGroup::YehWithTail,
        ],
        forms: &[JoiningForm::Final, JoiningForm::Isolated],
    },
    RasmClass {
        groups: &[JoiningGroup::YehBarree, JoiningGroup::BurushaskiYehBarree],
        forms: &[JoiningForm::Final, JoiningForm::Isolated],
    },
    RasmClass {
        groups: &[JoiningGroup::Kaf, JoiningGroup::Gaf],
        forms: &[JoiningForm::Initial, JoiningForm::Medial],
    },
];

pub(crate) fn rasm_matches(
    token_group: JoiningGroup,
    grapheme_group: JoiningGroup,
    form: JoiningForm,
) -> bool {
    if token_group == grapheme_group {
        return true;
    }
    for cls in RASM_CLASSES {
        if !cls.forms.contains(&form) {
            continue;
        }
        if cls.groups.contains(&token_group) && cls.groups.contains(&grapheme_group) {
            return true;
        }
    }
    false
}

// A group reference is `@Name` where Name is a canonical Unicode Joining_Group
// long name. No_Joining_Group can never match a real letter, so it is rejected
// like any unknown name.
pub(crate) fn resolve_group_name(name: &str) -> Result<JoiningGroup, CompileErrorKind> {
    let unknown = || CompileErrorKind::UnknownGroupName(name.to_string());
    let bare = name.strip_prefix(['@', '=']).ok_or_else(unknown)?;
    let parser = PropertyParser::<JoiningGroup>::new();
    let long = PropertyNamesLong::<JoiningGroup>::new();
    match parser.get_strict(bare) {
        Some(group) if group != JoiningGroup::NoJoiningGroup && long.get(group) == Some(bare) => {
            Ok(group)
        }
        _ => Err(unknown()),
    }
}
