use zwipe_core::http::contracts::collection::UnmatchedRow;

use super::*;

fn summary(unmatched: Vec<UnmatchedRow>, total: u32) -> HttpCollectionImportSummary {
    HttpCollectionImportSummary {
        source: CollectionSource::ManaBox,
        rows: 4,
        added: 2,
        updated: 1,
        removed: 0,
        cards: 7,
        unmatched,
        unmatched_total: total,
    }
}

fn picked(state: &mut State) {
    let effects = state.update(Action::PickFile {
        name: "manabox.csv".into(),
        text: "Name,Set code,Quantity\nSol Ring,c21,1\n".into(),
    });
    assert_eq!(effects, []);
}

fn sent(effects: &[Effect]) -> Ticket {
    match effects {
        [Effect::ImportCollection { ticket, .. }] => *ticket,
        other => panic!("one import: {other:?}"),
    }
}

#[test]
fn nothing_sends_without_a_file() {
    let mut state = State::new();
    assert!(!state.can_submit());
    assert_eq!(state.update(Action::Submit), []);
    assert_eq!(state.submit_label(), IMPORT);
}

#[test]
fn an_add_sends_the_file_at_once() {
    let mut state = State::new();
    picked(&mut state);
    assert_eq!(state.file_name(), Some("manabox.csv"));
    let effects = state.update(Action::Submit);
    let Some(Effect::ImportCollection { csv, mode, .. }) = effects.first() else {
        panic!("{effects:?}")
    };
    assert_eq!(
        (csv.as_str(), *mode),
        ("Name,Set code,Quantity\nSol Ring,c21,1\n", ImportMode::Add)
    );
    assert!(state.importing());
    assert_eq!(state.submit_label(), IMPORTING);
    assert_eq!(state.update(Action::Submit), [], "one at a time");
}

#[test]
fn a_replace_asks_first_and_sends_only_on_confirm() {
    let mut state = State::new();
    picked(&mut state);
    let _ = state.update(Action::PickMode(ImportMode::Replace));
    assert_eq!(state.submit_label(), REPLACE);
    assert_eq!(state.explainer(), REPLACE_EXPLAINER);
    assert_eq!(state.update(Action::Submit), [Effect::ConfirmReplace]);
    assert!(!state.importing(), "nothing sent before the confirm");
    let effects = state.update(Action::ConfirmReplace);
    let Some(Effect::ImportCollection { mode, .. }) = effects.first() else {
        panic!("{effects:?}")
    };
    assert_eq!(*mode, ImportMode::Replace);
}

#[test]
fn a_refused_pick_clears_the_last_one_so_import_sends_nothing() {
    let refusals = [
        (
            "read over the cap",
            Action::PickFile {
                name: "huge.csv".into(),
                text: "x".repeat(MAX_BODY_BYTES + 1),
            },
            TOO_BIG,
        ),
        ("too big to read", Action::PickedTooBig, TOO_BIG),
        ("unreadable", Action::Unreadable, UNREADABLE),
    ];
    for (case, refused, line) in refusals {
        let mut state = State::new();
        picked(&mut state);
        assert_eq!(
            state.update(refused),
            [Effect::Toast(Toast::normal(ToastKind::Error, line))],
            "{case}"
        );
        assert_eq!(state.file_name(), None, "{case}: the field empties");
        assert!(!state.can_submit(), "{case}: Import is off");
        assert_eq!(
            state.update(Action::Submit),
            [],
            "{case}: the old file stays unsent"
        );
        picked(&mut state);
        assert!(state.can_submit(), "{case}: a good pick turns it back on");
    }
}

#[test]
fn the_summary_lands_and_a_late_reply_is_ignored() {
    let mut state = State::new();
    picked(&mut state);
    let first = sent(&state.update(Action::Submit));
    let _ = state.update(Action::Failed { ticket: first });
    let second = sent(&state.update(Action::Submit));
    assert_eq!(
        state.update(Action::Imported {
            ticket: first,
            summary: summary(Vec::new(), 0)
        }),
        []
    );
    assert_eq!(state.summary(), None, "the older reply changes nothing");
    let effects = state.update(Action::Imported {
        ticket: second,
        summary: summary(Vec::new(), 0),
    });
    assert_eq!(
        effects,
        [
            Effect::Toast(Toast::quick(ToastKind::Success, "Collection imported")),
            Effect::CollectionChanged,
        ]
    );
    assert_eq!(state.summary().map(|s| s.cards), Some(7));
    assert!(!state.importing());
}

#[test]
fn the_counts_and_unmatched_rows_read_as_text() {
    let rows = vec![
        UnmatchedRow {
            line: 4,
            name: Some("Forest".into()),
            set: Some("zzz".into()),
            reason: UnmatchedReason::AmbiguousName,
        },
        UnmatchedRow {
            line: 9,
            name: None,
            set: None,
            reason: UnmatchedReason::MissingName,
        },
    ];
    let done = summary(rows, 230);
    assert_eq!(
        count_rows(&done).first(),
        Some(&("4".to_owned(), "ManaBox rows read".to_owned()))
    );
    assert_eq!(
        unmatched_rows(&done),
        [
            ("Line 4: Forest (zzz)".to_owned(), "more than one printing"),
            ("Line 9: no name".to_owned(), "no card given")
        ]
    );
    assert_eq!(
        more_unmatched(&done),
        Some("And 228 more not shown".to_owned())
    );
    assert_eq!(more_unmatched(&summary(Vec::new(), 0)), None);
    assert_eq!(
        outcome_toast(&done).text,
        "Imported; 230 rows didn't match a card"
    );
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Collection");
    assert_eq!(REPLACE_TITLE, "Replace your collection?");
    assert_eq!(FILE_PLACEHOLDER, "Choose a .csv export");
}

#[test]
fn a_finished_import_clears_the_file_so_a_second_click_sends_nothing() {
    let mut state = State::new();
    picked(&mut state);
    let ticket = sent(&state.update(Action::Submit));
    let _ = state.update(Action::Imported {
        ticket,
        summary: summary(Vec::new(), 0),
    });
    assert_eq!(state.file_name(), None, "the field reads as empty again");
    assert!(!state.can_submit());
    assert_eq!(
        state.update(Action::Submit),
        [],
        "the same file isn't sent twice"
    );
    picked(&mut state);
    assert!(state.can_submit(), "a new pick can go");
}

#[test]
fn the_size_check_counts_the_escaped_body() {
    let mut state = State::new();
    // Under the cap as a file, over it once every quote and line break is escaped in the body.
    let text = "\"\n".repeat(MAX_BODY_BYTES / 2 - 8);
    assert!(text.len() < MAX_BODY_BYTES);
    let effects = state.update(Action::PickFile {
        name: "quoted.csv".into(),
        text,
    });
    assert_eq!(
        effects,
        [Effect::Toast(Toast::normal(ToastKind::Error, TOO_BIG))]
    );
    assert_eq!(state.file_name(), None);
}

#[test]
fn a_landed_import_says_the_collection_changed_and_a_failed_one_does_not() {
    let mut state = State::new();
    picked(&mut state);
    let ticket = sent(&state.update(Action::Submit));
    let failed = state.update(Action::Failed { ticket });
    assert!(!failed.contains(&Effect::CollectionChanged));
    let ticket = sent(&state.update(Action::Submit));
    let landed = state.update(Action::Imported {
        ticket,
        summary: summary(Vec::new(), 0),
    });
    assert!(
        landed.contains(&Effect::CollectionChanged),
        "the In my collection chip reads the new collection: {landed:?}"
    );
}
