use crate::builtin::{builtin_pattern_set, builtin_pattern_set_names, is_builtin_pattern_set};
use crate::grapheme::{
    form_of, joined_runs, joins_left, joins_right, split_graphemes, JoiningForm,
};
use crate::pattern::compile_pattern_text;
use crate::rasm::{rasm_matches, resolve_group_name};
use crate::{find_kashida_points, find_kashida_points_patterns};
use icu_properties::props::JoiningGroup;

fn points(word: &str, text: &str) -> Vec<(u32, u8)> {
    let set = compile_pattern_text(text).expect("pattern compiles");
    find_kashida_points_patterns(word, &set)
        .iter()
        .map(|k| (k.index, k.priority))
        .collect()
}

fn builtin_points(name: &str, word: &str) -> Vec<(u32, u8)> {
    let set = builtin_pattern_set(name).expect("built-in set exists");
    find_kashida_points_patterns(word, set)
        .iter()
        .map(|k| (k.index, k.priority))
        .collect()
}

fn err_msg(text: &str) -> String {
    compile_pattern_text(text)
        .expect_err("pattern should fail to compile")
        .to_string()
}

#[test]
fn resolve_group_name_matches_long_names_only() {
    assert_eq!(resolve_group_name("@Beh").unwrap(), JoiningGroup::Beh);
    assert_eq!(
        resolve_group_name("@Teh_Marbuta").unwrap(),
        JoiningGroup::TehMarbuta
    );
    assert_eq!(
        resolve_group_name("@Farsi_Yeh").unwrap(),
        JoiningGroup::FarsiYeh
    );

    // Un-prefixed / mis-cased / icu-style / unknown names are rejected.
    assert!(resolve_group_name("Beh").is_err());
    assert!(resolve_group_name("@TehMarbuta").is_err());
    assert!(resolve_group_name("@teh_marbuta").is_err());
    assert!(resolve_group_name("@beh").is_err());
    assert!(resolve_group_name("@Nope").is_err());
    assert!(resolve_group_name("@No_Joining_Group").is_err());
}

#[test]
fn form_of_derives_positional_form() {
    let g = split_graphemes("بنت"); // beh-noon-teh, all dual-joining
    assert_eq!(form_of(&g, 0), JoiningForm::Initial);
    assert_eq!(form_of(&g, 1), JoiningForm::Medial);
    assert_eq!(form_of(&g, 2), JoiningForm::Final);
    assert_eq!(form_of(&split_graphemes("ب"), 0), JoiningForm::Isolated);
}

#[test]
fn joins_left_tests() {
    let cases: &[(&str, usize, bool)] = &[
        ("بيت", 2, false),  // final teh, nothing to its left
        ("Test", 0, false), // non-joining
        ("aب", 0, false),   // non-joining before joining
        ("بa", 0, false),   // non-joining after joining
        ("بaب", 0, false),
        ("نص", 0, true),
        ("نَص", 0, true),  // skips one marks
        ("نَّص", 0, true),  // skips more than one mark
        ("أب", 0, false), // alef is right-joining
        ("أَب", 0, false), // skips one mark
    ];
    for &(word, index, expected) in cases {
        assert_eq!(
            joins_left(&split_graphemes(word), index),
            expected,
            "{word:?} @ {index}"
        );
    }
}

#[test]
fn joins_right_tests() {
    let cases: &[(&str, usize, bool)] = &[
        ("بيت", 0, false), // initial beh, nothing to its right
        ("بيت", 2, true),
        ("Test", 0, false), // non-joining
        ("بa", 1, false),   // non-joining after joining
        ("معطار", 3, true), // skips one mark
        ("معطَار", 3, true), // skips one mark
        ("معطَّار", 3, true), // skips more than one mark
        ("ار", 1, false),   // reh is right-joining
        ("اَر", 1, false),   // skips one mark
    ];
    for &(word, index, expected) in cases {
        assert_eq!(
            joins_right(&split_graphemes(word), index),
            expected,
            "{word:?} @ {index}"
        );
    }
}

#[test]
fn rasm_folding() {
    assert!(rasm_matches(
        JoiningGroup::Beh,
        JoiningGroup::Noon,
        JoiningForm::Medial
    ));
    assert!(rasm_matches(
        JoiningGroup::Beh,
        JoiningGroup::Yeh,
        JoiningForm::Initial
    ));
    assert!(!rasm_matches(
        JoiningGroup::Beh,
        JoiningGroup::Noon,
        JoiningForm::Final
    ));
    assert!(!rasm_matches(
        JoiningGroup::Beh,
        JoiningGroup::Yeh,
        JoiningForm::Isolated
    ));

    // The identical group always matches.
    assert!(rasm_matches(
        JoiningGroup::Beh,
        JoiningGroup::Beh,
        JoiningForm::Final
    ));

    // Feh and qaf merge in initial/medial only.
    assert!(rasm_matches(
        JoiningGroup::Feh,
        JoiningGroup::Qaf,
        JoiningForm::Medial
    ));
    assert!(!rasm_matches(
        JoiningGroup::Feh,
        JoiningGroup::Qaf,
        JoiningForm::Final
    ));

    // Folding is directional: a group whose skeleton belongs to another
    // group in some forms does not match in those forms, not even itself.
    assert!(!rasm_matches(
        JoiningGroup::Noon,
        JoiningGroup::Beh,
        JoiningForm::Medial
    ));
    assert!(!rasm_matches(
        JoiningGroup::Noon,
        JoiningGroup::Noon,
        JoiningForm::Medial
    ));
    assert!(rasm_matches(
        JoiningGroup::Noon,
        JoiningGroup::Nya,
        JoiningForm::Final
    ));
    assert!(!rasm_matches(
        JoiningGroup::Qaf,
        JoiningGroup::Feh,
        JoiningForm::Medial
    ));
}

#[test]
fn letters_match_only_themselves() {
    assert_eq!(points("بت", "ب2ت"), vec![(0, 2)]);
    assert_eq!(points("نت", "ب2ت"), Vec::new()); // noon is not beh
    assert_eq!(points("نت", "ٮ2ت"), Vec::new()); // dotless beh does not fold
}

#[test]
fn the_last_rule_wins_at_a_connection() {
    assert_eq!(points("بت", "ب2ت\nب5ت"), vec![(0, 5)]);
    // Not the highest: order alone decides.
    assert_eq!(points("بت", "ب5ت\nب2ت"), vec![(0, 2)]);
}

#[test]
fn absent_digit_is_no_candidate_explicit_zero_is_weakest() {
    assert_eq!(points("بت", "بت"), Vec::new()); // no digit
    assert_eq!(points("بت", "ب0ت"), vec![(0, 0)]); // priority 0
}

#[test]
fn a_suppression_holds_until_a_later_rule_speaks() {
    assert_eq!(points("بت", "ب9ت\nب!ت"), Vec::new());
    assert_eq!(points("بت", "ب!ت\nب9ت"), vec![(0, 9)]);
}

#[test]
fn length_guards_gate_on_joined_run_length() {
    assert_eq!(points("بتر", "[3]ب2ت"), vec![(0, 2)]);
    assert_eq!(points("بتر", "[4]ب2ت"), Vec::new());
    assert_eq!(points("بتر", "[2:3]ب2ت"), vec![(0, 2)]);
    assert_eq!(points("بتبت", "[2:3]ب2ت"), Vec::new());
    assert_eq!(points("بت", "[2:]ب2ت"), vec![(0, 2)]);
    assert_eq!(points("بت", "[3:]ب2ت"), Vec::new());
}

#[test]
fn priority_steps_down_as_run_grows() {
    let steps = |word: &str| points(word, "[4:]ب6\\3ت");
    assert_eq!(steps("بتنن"), vec![(0, 6)]); // len 4
    assert_eq!(steps("بتننن"), vec![(0, 5)]); // len 5
    assert_eq!(steps("بتنننن"), vec![(0, 4)]); // len 6
    assert_eq!(steps("بتننننن"), vec![(0, 3)]); // len 7
    assert_eq!(steps("بتنننننن"), vec![(0, 3)]); // len 8: holds at 3
}

#[test]
fn an_open_guard_matches_any_length_and_drops_off_either_way() {
    let steps = |word: &str| points(word, "[:4:]ب6\\3ت");
    assert_eq!(steps("بت"), vec![(0, 4)]); // len 2, two letters from 4
    assert_eq!(steps("بتن"), vec![(0, 5)]); // len 3
    assert_eq!(steps("بتنن"), vec![(0, 6)]); // len 4, where the priority is highest
    assert_eq!(steps("بتننن"), vec![(0, 5)]); // len 5
    assert_eq!(steps("بتنننن"), vec![(0, 4)]); // len 6
    assert_eq!(steps("بتننننن"), vec![(0, 3)]); // len 7, reaches the second digit
    assert_eq!(steps("بتنننننن"), vec![(0, 3)]); // len 8, holds
}

#[test]
fn an_open_guard_clamps_a_short_run_at_the_second_digit() {
    // Two letters from 6 would give 1, but the priority never drops below the
    // second digit.
    assert_eq!(points("بت", "[:6:]ب3\\2ت"), vec![(0, 2)]);
    // A one-digit priority is the same at every length.
    assert_eq!(points("بت", "[:4:]ب6ت"), vec![(0, 6)]);
    assert_eq!(points("بتنن", "[:4:]ب6ت"), vec![(0, 6)]);
}

#[test]
fn an_open_guard_bound_is_still_checked() {
    let err_msg = |text: &str| compile_pattern_text(text).unwrap_err().to_string();
    assert!(err_msg("[:1:]ب2ت").contains("Invalid length guard"));
    assert!(err_msg("[:x:]ب2ت").contains("Invalid length guard"));
    // Without the closing `:` a leading one is still an invalid range.
    assert!(err_msg("[:4]ب2ت").contains("Invalid length guard"));
}

#[test]
fn lam_alef_is_suppressed_by_the_pattern_sets() {
    // The pattern allows kashida before any alef, lam-alef has no special
    // treatment.
    assert_eq!(points("لا", "2ا"), vec![(0, 2)]);
    assert_eq!(points("با", "2ا"), vec![(0, 2)]);
    // But built-in patterns suppress kashida in lam-lef
    assert_eq!(builtin_points("arabic-simple", "لا"), Vec::new());
    assert_eq!(builtin_points("arabic-naskh", "لا"), Vec::new());
    // Only before an alef in simple pattern (in naskh no kashida after any lam).
    assert_eq!(builtin_points("arabic-simple", "لب"), vec![(0, 3)]);
}

#[test]
fn inline_group_set_matches_any_of_its_groups() {
    let set = "{@Beh @Noon @Yeh} 5 ت";
    assert_eq!(points("بت", set), vec![(0, 5)]);
    assert_eq!(points("نت", set), vec![(0, 5)]);
    assert_eq!(points("صت", set), Vec::new());
}

#[test]
fn group_sets_accept_literals_and_tatweel() {
    let set = "{=Seen ب} 5 ت";
    assert_eq!(points("ست", set), vec![(0, 5)]); // group member
    assert_eq!(points("بت", set), vec![(0, 5)]); // literal member
    assert_eq!(points("نت", set), Vec::new());
    assert_eq!(points("بـت", "{@Tatweel} 9"), vec![(1, 9)]);
}

#[test]
fn set_members_behave_as_they_do_outside() {
    // A one-element set is the same as the bare reference.
    assert_eq!(points("نت", "@Beh 5 ت"), points("نت", "{@Beh} 5 ت"));
    assert_eq!(points("نت", "{@Beh} 5 ت"), vec![(0, 5)]);
}

#[test]
fn exact_group_reference_does_not_fold() {
    // `=Name` matches that Joining_Group alone. `@Name` folds through the rasm
    // classes.
    assert_eq!(points("بت", "=Beh 5 ت"), vec![(0, 5)]);
    assert_eq!(points("نت", "=Beh 5 ت"), Vec::new()); // initial noon: no fold
    assert_eq!(points("نت", "{=Beh} 5 ت"), Vec::new()); // nor inside a set
    assert_eq!(points("نت", "^=Beh 5 ت"), vec![(0, 5)]); // complement form
    assert!(err_msg("=Foo 5 ت").contains("Unknown"));
    // Tatweel is a literal, so folding does not apply: both prefixes name it.
    assert_eq!(points("بـت", "=Tatweel 9"), points("بـت", "@Tatweel 9"));
}

#[test]
fn group_in_no_rasm_class_matches_itself_alone() {
    // Seen folds through no rasm class, so `@Seen` and `=Seen` are the same.
    assert_eq!(points("ست", "@Seen 5 *"), vec![(0, 5)]);
    assert_eq!(points("ست", "=Seen 5 *"), vec![(0, 5)]);
}

#[test]
fn not_group_set_matches_any_joining_letter_not_in_set() {
    let set = "^{@Beh @Noon} 5 ت";
    assert_eq!(points("صت", set), vec![(0, 5)]);
    assert_eq!(points("بت", set), Vec::new());
    assert_eq!(points("نت", set), Vec::new());
    // ^@Name without braces is the lone-group complement.
    assert_eq!(points("صت", "^@Beh 5 ت"), vec![(0, 5)]);
    assert_eq!(points("بت", "^@Beh 5 ت"), Vec::new());
}

#[test]
fn group_folds_to_tooth_initial_medial_strict_when_final() {
    assert_eq!(points("نت", "@Beh 5 ت"), vec![(0, 5)]); // initial noon
    assert_eq!(points("يت", "@Beh 5 ت"), vec![(0, 5)]); // initial yeh
    assert_eq!(points("بنت", "@Beh 5 ت"), vec![(1, 5)]); // medial noon
    assert_eq!(points("تب", "ت 5 @Beh ."), vec![(0, 5)]); // final beh matches
    assert_eq!(points("تن", "ت 5 @Beh ."), Vec::new()); // final noon does not
}

#[test]
fn ignores_comments_and_blank_lines() {
    assert_eq!(
        points("بت", "# a comment\n\nب2ت  # trailing\n"),
        vec![(0, 2)]
    );
}

#[test]
fn rejects_malformed_pattern_lines() {
    assert!(err_msg("[3ب2ت").contains("Unterminated length guard"));
    assert!(err_msg("[x]ب2ت").contains("Invalid length guard"));
    assert!(err_msg("[x:]ب2ت").contains("Invalid length guard"));
    assert!(err_msg("[2:y]ب2ت").contains("Invalid length guard"));
    assert!(err_msg("[4.5]ب2ت").contains("Invalid length guard"));
    assert!(err_msg("[:3]ب2ت").contains("Invalid length guard"));
    assert!(err_msg("@").contains("Empty group name"));
    assert!(err_msg("=").contains("Empty group name"));
    assert!(err_msg("ب.ت").contains("Token after a trailing"));
    assert!(err_msg("5").contains("Pattern has no letters"));
    assert!(err_msg("@Nope 2 ت").contains("Unknown Unicode Joining_Group name"));
    assert!(err_msg("ب3\\6ت").contains("must not increase"));
    assert!(err_msg("ب9\\خت").contains("Expected a digit after"));
    assert!(err_msg("ب\\3ت").contains("must follow a priority digit"));
    assert!(err_msg("{@Beh @Nope} 2 ت").contains("Unknown Unicode Joining_Group"));
    assert!(err_msg("{@Beh Noon} 2 ت").contains("Stray character")); // un-prefixed = literals
    assert!(err_msg("> ب2ت").contains("Stray character")); // no run-claiming marker
    assert!(err_msg("{@Beh 2 ت").contains("Unterminated “{”"));
    assert!(err_msg("{} 2 ت").contains("Empty “{}”"));
    assert!(err_msg("ب 2 ^ت").contains("must be followed by “{”, “@”, or “=”"));
}

#[test]
fn builtin_sets_resolve_and_are_named() {
    assert_eq!(
        builtin_pattern_set_names(),
        ["arabic-naskh", "arabic-nastaliq", "arabic-simple", "syriac"]
    );
    // Every listed name resolves, and nothing else does.
    for name in builtin_pattern_set_names() {
        assert!(is_builtin_pattern_set(name), "{name}");
        assert!(builtin_pattern_set(name).is_some(), "{name}");
    }
    assert!(!is_builtin_pattern_set("nope"));
    assert!(builtin_pattern_set("nope").is_none());
}

#[test]
fn the_naskh_matrix_reaches_short_runs() {
    let p = |word: &str| builtin_points("arabic-naskh", word);

    // beh-tah is the matrix's 9\6 pairing. The priority is highest in a
    // four-letter run and drops by one for every letter away from that.
    assert_eq!(p("بط"), vec![(0, 7)]); // len 2
    assert_eq!(p("مبط"), vec![(1, 8)]); // len 3
    assert_eq!(p("ممبط"), vec![(0, 3), (2, 9)]); // len 4
    assert_eq!(p("مممبط"), vec![(0, 2), (1, 2), (3, 8)]); // len 5

    // beh-meem is the 6\3 pairing, three lower at every length.
    assert_eq!(p("بم"), vec![(0, 4)]); // len 2
    assert_eq!(p("ممبم"), vec![(0, 3), (2, 6)]); // len 4

    // beh-beh is an empty cell in figure 25, so it gets no point at any length.
    assert_eq!(p("بب"), Vec::new()); // len 2
    assert_eq!(p("ممبب"), vec![(0, 3)]); // len 4, nothing at the beh-beh
}

#[test]
fn nastaliq_forbids_a_kashida_after_an_initial_beh() {
    // Naskh allows it, Nastaliq does not.
    assert_eq!(
        builtin_points("arabic-naskh", "يهتم"),
        vec![(0, 6), (1, 6), (2, 6)]
    );
    assert_eq!(
        builtin_points("arabic-nastaliq", "يهتم"),
        vec![(1, 6), (2, 6)]
    );
    // The tooth folds, so an initial noon counts too.
    assert_eq!(
        builtin_points("arabic-nastaliq", "نهتم"),
        vec![(1, 6), (2, 6)]
    );
    // Only an initial one: a medial beh is untouched.
    assert_eq!(
        builtin_points("arabic-nastaliq", "فبتم"),
        builtin_points("arabic-naskh", "فبتم")
    );
}

#[test]
fn nastaliq_forbids_a_kashida_before_a_thin_join() {
    // Medial feh and qaf, which naskh allows.
    assert_eq!(builtin_points("arabic-naskh", "سقتم"), vec![(0, 3), (2, 6)]);
    assert_eq!(builtin_points("arabic-nastaliq", "سقتم"), vec![(2, 6)]);
    // Tah.
    assert_eq!(
        builtin_points("arabic-naskh", "متظلم"),
        vec![(1, 8), (2, 5)]
    );
    assert_eq!(builtin_points("arabic-nastaliq", "متظلم"), vec![(2, 5)]);
    // A medial heh, but a final one still takes the strongest point.
    assert_eq!(builtin_points("arabic-naskh", "متهم"), vec![(1, 6)]);
    assert_eq!(builtin_points("arabic-nastaliq", "متهم"), Vec::new());
    assert_eq!(builtin_points("arabic-nastaliq", "بحه"), vec![(1, 5)]);
}

#[test]
fn nastaliq_forbids_a_kashida_after_a_lam_or_kaf() {
    for word in ["كلمة", "معلم", "يكتم"] {
        let points = builtin_points("arabic-nastaliq", word);
        let graphemes: Vec<char> = word.chars().collect();
        for &(index, _) in &points {
            assert!(
                !matches!(graphemes[index as usize], 'ل' | 'ك'),
                "{word}: {points:?}"
            );
        }
    }
}

#[test]
fn syriac_ranks_points_outside_in() {
    // LibreOffice picks Syriac positions outside-in: from the letter before
    // the last toward the word's midpoint, then from the start toward it.
    // These are the orders GetWordKashidaPositionSyriac yields, most
    // preferred first.
    let expected: &[(usize, &[u32])] = &[
        (2, &[0]),
        (3, &[1, 0]),
        (4, &[2, 1, 0]),
        (5, &[3, 2, 0, 1]),
        (6, &[4, 3, 2, 0, 1]),
        (7, &[5, 4, 3, 0, 1, 2]),
        (8, &[6, 5, 4, 3, 0, 1, 2]),
        (9, &[7, 6, 5, 4, 0, 1, 2, 3]),
        (10, &[8, 7, 6, 5, 4, 0, 1, 2, 3]),
    ];
    for &(letters, order) in expected {
        // Beth is dual-joining, so the whole word is one run.
        let word = "ܒ".repeat(letters);
        let mut points = builtin_points("syriac", &word);
        points.sort_by_key(|&(index, priority)| (std::cmp::Reverse(priority), index));
        let ranked: Vec<u32> = points.iter().map(|&(index, _)| index).collect();
        assert_eq!(ranked, order, "run of {letters} letters");
    }
}

#[test]
fn syriac_matches_the_libreoffice_test_vectors() {
    // i18nutil/qa/cppunit/test_kashida.cxx walks the whole preference order
    // by disabling each position in turn. Its words are used verbatim here.
    let ranked = |word: &str| {
        let mut points = builtin_points("syriac", word);
        points.sort_by_key(|&(index, priority)| (std::cmp::Reverse(priority), index));
        points.iter().map(|&(index, _)| index).collect::<Vec<_>>()
    };
    // testSyriac(): seven letters yield 5, 4, 3, 0, 1, 2.
    assert_eq!(ranked("ܥܥܥܥܥܥܥ"), vec![5, 4, 3, 0, 1, 2]);
    // testSyriacVowelMarks(): the same seven letters carrying vowel marks
    // yield the same order (“the midpoint counts letters only” rule).
    assert_eq!(
        ranked("ܥܥܥܥܥ\u{073F}\u{073E}ܥ\u{073F}\u{073E}ܥ\u{073F}\u{073E}"),
        vec![5, 4, 3, 0, 1, 2]
    );
    // testSyriac() also checks that a kashida the user typed wins, at its
    // own index.
    assert_eq!(ranked("ܥܥـܥܥܥܥ")[0], 2);
}

#[test]
fn syriac_never_breaks_lomadh_olaph() {
    // "No Kashida character should be inserted between the letter sequence:
    // Lomadh, Olaph." LibreOffice's Syriac path never checks for it.
    assert_eq!(builtin_points("syriac", "ܠܐ"), Vec::new());
    // Only that pair: a lomadh before anything else still elongates.
    assert_eq!(builtin_points("syriac", "ܠܒ"), vec![(0, 8)]);
}

#[test]
fn syriac_letters_that_take_no_kashida_after_them() {
    // "The following letters should not receive a kashida after them:
    // Olaph; Dolath; He; Waw; Zayn; Sodhe; Rish; Taw; Dotless Dolath Rish."
    // None of them joins forward, so no connection ever follows one.
    for word in ["ܐܒ", "ܕܒ", "ܗܒ", "ܘܒ", "ܙܒ", "ܨܒ", "ܪܒ", "ܬܒ", "ܖܒ"] {
        assert_eq!(builtin_points("syriac", word), Vec::new(), "{word}");
    }
}

#[test]
fn syriac_ladders_apply_to_each_joined_run() {
    // Rish does not join forward, so ܡܪܝܡ is two runs of two letters and
    // each gets its own ladder.
    assert_eq!(builtin_points("syriac", "ܡܪܝܡ"), vec![(0, 8), (2, 8)]);
    // A user-inserted kashida still outranks every position.
    assert_eq!(
        builtin_points("syriac", "ܡـܝܡ"),
        vec![(0, 3), (1, 9), (2, 8)]
    );
}

#[test]
fn naskh_pattern_tests() {
    // The heh-ending rule lands a kashida before the final heh.
    assert_eq!(builtin_points("arabic-naskh", "بحه"), vec![(1, 9)]);
    // No kashida before an ain.
    assert_eq!(builtin_points("arabic-naskh", "مسعد"), vec![(2, 3)]);
    // None after a kaf or lam.
    assert_eq!(builtin_points("arabic-naskh", "كلمة"), vec![(2, 9)]);
    // None before a final yeh.
    assert_eq!(builtin_points("arabic-naskh", "سعي"), Vec::new());
    // The heh ending is the one naskh rule with no length gate.
    assert_eq!(builtin_points("arabic-naskh", "به"), vec![(0, 9)]);
}

#[test]
fn simple_pattern_tests() {
    // Rule 2 (after initial seen) at point 0, rule 7 before the
    // final teh at point 1.
    assert_eq!(builtin_points("arabic-simple", "سبت"), vec![(0, 8), (1, 3)]);
    // Rule 2 carries no final-yeh exception: seen before a final yeh takes 8.
    assert_eq!(builtin_points("arabic-simple", "سي"), vec![(0, 8)]);
    // بيبت has no final reh/yeh, so rule 5 must stay silent; only rule 7
    // applies, before the final teh.
    assert_eq!(builtin_points("arabic-simple", "بيبت"), vec![(2, 3)]);
    // The genuine shape still fires: a medial tooth before a final yeh.
    assert_eq!(builtin_points("arabic-simple", "بني"), vec![(0, 5), (1, 3)]);
}

#[test]
fn zwnj_breaks_the_join() {
    // The ZWNJ clusters into the beh grapheme; the connection it suppresses
    // must not host a kashida, and both letters shape isolated.
    assert_eq!(points("ب\u{200C}ت", "ب2ت"), Vec::new());
    assert_eq!(builtin_points("arabic-simple", "ب\u{200C}ت"), Vec::new());
    let g = split_graphemes("ب\u{200C}ت");
    assert_eq!(form_of(&g, 0), JoiningForm::Isolated);
    assert_eq!(form_of(&g, 1), JoiningForm::Isolated);
}

#[test]
fn stray_pattern_characters_are_rejected() {
    assert!(err_msg("{=Seen x} 5 ت").contains("Stray character")); // Non_Joining
    assert!(err_msg("ب\u{0662}ت").contains("Stray character")); // Arabic-Indic ٢
    assert!(err_msg("ب2ت]").contains("Stray character"));
    assert!(err_msg("[2] > ب2ت").contains("Stray character")); // '>' after guard
    assert!(err_msg("ب2ت // note").contains("Stray character")); // '#' comments only
}

#[test]
fn zwj_zwnj() {
    // ZWJ makes beh take a final form but dal is right-joining still.
    assert_eq!(points("د\u{200D}ب", "د2ب"), Vec::new());
    // Between two dual joiners it changes nothing.
    assert_eq!(points("ب\u{200D}ت", "ب2ت"), vec![(0, 2)]);
    // A ZWNJ severs the join no matter where a ZWJ sits around it.
    assert_eq!(points("ب\u{200D}\u{200C}ت", "ب2ت"), Vec::new());
    assert_eq!(points("ب\u{200C}\u{200D}ت", "ب2ت"), Vec::new());
    // A ZWJ after feh makes it medial, so a final-matching pattern
    // must not fire.
    assert_eq!(points("سف", "8 ف ."), vec![(0, 8)]);
    assert_eq!(points("سف\u{200D}", "8 ف ."), Vec::new());
}

#[test]
fn manual_tatweel() {
    let set = builtin_pattern_set("arabic-simple").unwrap();
    // NFC puts a vowel before a tatweel-seated hamza. The tatweel still
    // carries the hamza and must survive stripping.
    let seated = "ب\u{0640}\u{064E}\u{0654}ت";
    assert_eq!(find_kashida_points(seated, set, true).0, seated);
    // Hamza below is a seat as well.
    let below = "ب\u{0640}\u{0655}ت";
    assert_eq!(find_kashida_points(below, set, true).0, below);
    // A harakah makes a seat as well, so the tatweel under it is kept.
    let harakah = "ب\u{0640}\u{064E}ت";
    assert_eq!(find_kashida_points(harakah, set, true).0, harakah);
    // A kept bare kashida is a run letter: it counts toward length guards
    // and matches `*` like anything else.
    assert_eq!(points("بـتر", "[4]ت2ر"), vec![(2, 2)]);
    assert_eq!(points("بـت", "ب2*"), vec![(0, 2)]);
    // The tatweel is an ordinary literal token: `ـ 9` lands after an
    // existing kashida, i.e. at the kashida's own index.
    assert_eq!(points("بـت", "ـ 9"), vec![(1, 9)]);
    // `@Tatweel` is the readable spelling of the same literal.
    assert_eq!(points("بـت", "@Tatweel 9"), vec![(1, 9)]);
    // naskh has no rule targeting it, so a kept kashida just stays in the
    // text.
    let set = builtin_pattern_set("arabic-naskh").unwrap();
    let (_, merged) = find_kashida_points("سـبل", set, false);
    let merged: Vec<_> = merged.iter().map(|k| (k.index, k.priority)).collect();
    assert_eq!(merged, vec![(2, 3)]);
}

#[test]
fn existing_kashida_matches_like_any_letter() {
    let set = compile_pattern_text("ـ 5").expect("pattern compiles");
    let (_, pts) = find_kashida_points("بـت", &set, false);
    let pts: Vec<_> = pts.iter().map(|k| (k.index, k.priority)).collect();
    assert_eq!(pts, vec![(1, 5)]);
    // Stripped by default, there is nothing left for it to match.
    assert_eq!(find_kashida_points("بـت", &set, true).1, Vec::new());
}

#[test]
fn conflicting_weights_at_one_connection_are_rejected() {
    assert!(err_msg("ب2 3ت").contains("Conflicting weights"));
    // '!' silently overwritten by a digit was the worst case.
    assert!(err_msg("ب!2ت").contains("Conflicting weights"));
}

#[test]
fn degenerate_length_guards_are_rejected() {
    // No run of fewer than 2 letters has a connection; empty ranges match
    // nothing. All of these compiled silently dead before.
    assert!(err_msg("[0]ب2ت").contains("Invalid length guard"));
    assert!(err_msg("[1]ب2ت").contains("Invalid length guard"));
    assert!(err_msg("[1:]ب2ت").contains("Invalid length guard"));
    assert!(err_msg("[3:2]ب2ت").contains("Invalid length guard"));
}

#[test]
fn boundary_edge_weights_are_rejected() {
    // A weight in the gap between a token and a `.` can never land.
    assert!(err_msg(". 5 ب ت").contains("Weight outside the run"));
    assert!(err_msg("ب 5 .").contains("Weight outside the run"));
}

#[test]
fn group_name_stops_at_non_name_characters() {
    // '@Behت' is '@Beh' followed by a teh literal, not a group-name typo.
    assert_eq!(points("بتت", "@Behت 2 ت"), vec![(1, 2)]);
}

#[test]
fn a_seat_kashida_is_transparent() {
    let set = builtin_pattern_set("arabic-naskh").unwrap();
    for (word, seated, bare) in [
        ("ٱلرَّحۡمَـٰنِ", vec![(3, 2), (5, 2)], vec![(3, 2), (4, 2)]),
        ("ٱلۡعَـٰلَمِینَ", vec![(3, 1), (6, 4)], vec![(2, 1), (5, 4)]),
        ("وَبِٱلۡـَٔاخِرَةِ", vec![(1, 4), (6, 1)], vec![(1, 4), (5, 1)]),
        ("ٱلنَّبِیِّـۧنَ", vec![(5, 5)], vec![(4, 5)]),
        ("تَأۡمَـ۫نَّا", vec![(0, 4), (4, 5)], vec![(0, 4), (3, 5)]),
        ("لِیَسُـࣳۤـُٔوا۟", Vec::new(), Vec::new()),
        ("ٱلصَّـٰلِحَـٰتِ", vec![(3, 5)], vec![(2, 5)]),
    ] {
        assert_eq!(find_kashida_points(word, set, true).0, word, "{word}");
        assert_eq!(builtin_points("arabic-naskh", word), seated, "{word}");
        let stripped: String = word.chars().filter(|c| *c != 'ـ').collect();
        assert_eq!(builtin_points("arabic-naskh", &stripped), bare, "{word}");
    }

    let seated = "ٱلۡعَـٰلَمِینَ";
    assert_eq!(points(seated, "[6]م2ی"), vec![(5, 2)]);
    assert_eq!(points(seated, "[7]م2ی"), Vec::new());
    assert_eq!(points(seated, "ـ 9"), Vec::new());
    let typed = "ٱلۡعَـٰلَـمِینَ";
    assert_eq!(points(typed, "[7]م2ی"), vec![(6, 2)]);
    assert_eq!(points(typed, "ـ 9"), vec![(5, 9)]);
}

#[test]
fn find_kashida_points_strips_or_keeps_user_tatweel() {
    let set = builtin_pattern_set("arabic-simple").unwrap();
    // A bare tatweel is stripped when removeExisting is set.
    let (cleaned, _) = find_kashida_points("بـت", set, true);
    assert_eq!(cleaned, "بت");
    // Kept, rule 1 (`ـ 9`) targets it: priority 9 at the kashida's own index.
    let (cleaned, pts) = find_kashida_points("بـت", set, false);
    assert_eq!(cleaned, "بـت");
    assert!(pts.iter().any(|k| k.index == 1 && k.priority == 9));
}

#[test]
fn readme_length_ladder_words() {
    // The README's length-dependent priority walkthrough. In the مبتعث words
    // the connection is teh–ain: teh is in the beh Joining_Group.
    let p = |w: &str| points(w, "[4:] @Beh 9\\6 @Ain");
    assert_eq!(p("بعثة"), vec![(0, 9)]); // four-letter run, the floor
    assert_eq!(p("مبتعث"), vec![(2, 8)]); // five
    assert_eq!(p("المبتعث"), vec![(4, 7)]); // six: ال's alef stands apart
    assert_eq!(p("المبتعثة"), vec![(4, 6)]); // seven: reaches the second digit
}

#[test]
fn every_connection_in_a_run_already_joins() {
    // A run ends at the first letter that cannot join on.
    let words =
        "بيت لا دب بد مررت الله سـبح ب\u{200C}ت ب\u{200D}ت نَّص \u{064E}بت بـ وا أبد كتاب لآ دا";
    for word in words.split(' ') {
        let graphemes = split_graphemes(word);
        for run in joined_runs(&graphemes) {
            // Every member but the last one hosts a connection.
            let (_, hosts) = run.split_last().expect("a run is never empty");
            for &index in hosts {
                assert!(
                    joins_left(&graphemes, index),
                    "{word:?}: grapheme {index} does not join forward"
                );
            }
        }
    }
}

#[test]
fn a_later_rule_overrides_an_earlier_one() {
    // Lowered to 1, which no single rule could do before; the teh-meem
    // connection is untouched.
    assert_eq!(points("بتم", "ب3ت\nت5م\nب1ت"), vec![(0, 1), (1, 5)]);
}

#[test]
fn a_later_rule_can_undo_a_suppression_and_impose_one() {
    assert_eq!(points("بت", "ب!ت\nب5ت"), vec![(0, 5)]);
    assert_eq!(points("بت", "ب5ت\nب!ت"), Vec::new());
}

#[test]
fn use_splices_a_builtin_in() {
    // naskh on its own puts 9 before the final heh.
    assert_eq!(builtin_points("arabic-naskh", "بحه"), vec![(1, 9)]);
    // Importing it and softening that one point leaves the rest alone.
    assert_eq!(points("بحه", "use arabic-naskh\n* 2 @Heh ."), vec![(1, 2)]);
    // With no rules of its own an import is just the set itself.
    assert_eq!(points("بحه", "use arabic-naskh"), vec![(1, 9)]);
}

#[test]
fn rejects_malformed_imports() {
    assert!(err_msg("use nope").contains("Unknown pattern set"));
    // Only a whole `use` word counts; anything else is an ordinary line.
    assert!(err_msg("used arabic-naskh").contains("Stray character"));
}
