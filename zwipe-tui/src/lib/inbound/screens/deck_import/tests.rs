use crossterm::event::KeyCode;
use zwipe_core::domain::deck::requests::import_deck_cards::{ImportedCard, UnresolvedCard};

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{render, screen_text, user};

fn opened() -> (Harness, Uuid) {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    let deck_id = Uuid::new_v4();
    harness.open(|cx| open(cx, deck_id));
    harness.app.shared.hint = None;
    harness.clear();
    (harness, deck_id)
}

fn view(harness: &Harness) -> &ImportView {
    match harness.screen() {
        Screen::DeckImport(view) => view,
        other => panic!("not on import: {other:?}"),
    }
}

#[test]
fn a_blank_list_does_not_import_and_typing_one_does() {
    let (mut harness, deck_id) = opened();
    harness.press(KeyCode::F(2));
    assert!(harness.pending().is_empty(), "blank input sends nothing");
    harness.type_text("1 Sol Ring");
    harness.press(KeyCode::Enter);
    harness.type_text("q");
    harness.press(KeyCode::F(2));
    assert_eq!(
        harness.pending(),
        [screens::Request::DeckImport(Request::Text {
            deck_id,
            text: "1 Sol Ring\nq".to_owned(),
            target: Target::Main,
            mode: ImportMode::Add,
        })]
    );
    assert_eq!(view(&harness).submit_label(), "Importing...");
}

#[test]
fn the_rows_switch_source_mode_and_board() {
    let (mut harness, deck_id) = opened();
    harness.press(KeyCode::Tab);
    assert_eq!(view(&harness).focus, Focus::From);
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char('l'));
    let v = view(&harness);
    assert_eq!(
        (v.source, v.mode, v.target),
        (Source::Archidekt, ImportMode::Replace, Target::Side)
    );
    assert_eq!(v.submit_label(), "Replace");
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Removes all existing cards in the sideboard"));
    assert!(screen.contains("Paste an Archidekt deck URL"));
    harness.press(KeyCode::Tab);
    harness.type_text("https://archidekt.com/decks/1");
    harness.press(KeyCode::Enter);
    assert!(matches!(
        harness.pending(),
        [screens::Request::DeckImport(Request::Archidekt { deck_id: id, target: Target::Side, mode: ImportMode::Replace, .. })] if *id == deck_id
    ));
}

#[test]
fn results_toast_and_list_what_came_in() {
    let (mut harness, _) = opened();
    harness.apply(Update::Imported(ImportDeckCardsResult {
        imported: vec![ImportedCard {
            name: "Sol Ring".to_owned(),
            quantity: 1,
        }],
        unresolved: vec![UnresolvedCard {
            name: "Nope".to_owned(),
            reason: "not found".to_owned(),
        }],
    }));
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Imported 1, 1 unresolved"
    );
    let screen = screen_text(&harness.app);
    for expected in ["Imported", "Sol Ring", "x1", "Unresolved", "not found"] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn a_failed_import_toasts_the_server_message() {
    let (mut harness, _) = opened();
    harness.apply(Update::Failed("deck not found".to_owned()));
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "deck not found"
    );
    assert!(!view(&harness).loading);
}

#[test]
fn import_fits_a_tiny_terminal() {
    let (harness, _) = opened();
    render(&harness.app, 12, 5);
    render(&harness.app, 1, 1);
}

#[test]
fn the_servers_line_cap_and_body_limit_toast_as_the_app_words_them() {
    let cases = [
        (
            ClientError::UnprocessableEntity("too many card lines, the limit is 1000".to_owned()),
            "too many card lines, the limit is 1000",
        ),
        (
            ClientError::PayloadTooLarge("length limit exceeded".to_owned()),
            "request too large, try importing fewer cards",
        ),
    ];
    for (error, message) in cases {
        let (mut harness, _) = opened();
        harness.apply(outcome(Err(error)));
        let toast = &harness.app.shared.toasts.items()[0];
        assert_eq!(toast.message, message);
        assert_eq!(toast.kind, ToastKind::Error);
        assert!(!view(&harness).loading);
    }
}

#[test]
fn an_import_clears_the_decks_undo_log_and_only_that_deck() {
    let (mut harness, deck_id) = opened();
    let other = Uuid::new_v4();
    for id in [deck_id, other] {
        harness.app.shared.cards.undo.push(
            id,
            crate::domain::stores::undo::UndoAction::Added {
                card_id: Uuid::new_v4(),
                card_name: "Sol Ring".to_owned(),
            },
        );
    }
    harness.apply(Update::Imported(ImportDeckCardsResult {
        imported: Vec::new(),
        unresolved: Vec::new(),
    }));
    assert!(
        !harness.app.shared.cards.undo.is_empty(deck_id),
        "nothing imported keeps the log"
    );
    harness.apply(Update::Imported(ImportDeckCardsResult {
        imported: vec![ImportedCard {
            name: "Sol Ring".to_owned(),
            quantity: 1,
        }],
        unresolved: Vec::new(),
    }));
    assert!(harness.app.shared.cards.undo.is_empty(deck_id));
    assert!(!harness.app.shared.cards.undo.is_empty(other));
}
