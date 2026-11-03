use crossterm::event::KeyCode;
use zwipe_core::domain::deck::requests::import_deck_cards::{ImportedCard, UnresolvedCard};
use zwipe_ui::screens::deck_import::Board;
use zwipe_ui::toast::{NORMAL, QUICK};

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

/// The ticket the latest import went out with.
fn last_ticket(harness: &Harness) -> Ticket {
    match harness.pending().last() {
        Some(screens::Request::DeckImport(request)) => request.ticket(),
        other => panic!("no import went out: {other:?}"),
    }
}

/// Import with a decklist typed and sent, its ticket, and nothing else queued.
fn importing() -> (Harness, Ticket) {
    let (mut harness, _) = opened();
    harness.type_text("1 Sol Ring");
    harness.press(KeyCode::F(2));
    let ticket = last_ticket(&harness);
    harness.clear();
    (harness, ticket)
}

/// One card that came in and one that did not.
fn mixed() -> ImportDeckCardsResult {
    ImportDeckCardsResult {
        imported: vec![ImportedCard {
            name: "Sol Ring".to_owned(),
            quantity: 1,
        }],
        unresolved: vec![UnresolvedCard {
            name: "Nope".to_owned(),
            reason: "not found".to_owned(),
        }],
    }
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
    let ticket = last_ticket(&harness);
    assert_eq!(
        harness.pending(),
        [screens::Request::DeckImport(Request::Text {
            ticket,
            deck_id,
            text: "1 Sol Ring\nq".to_owned(),
            board: None,
            mode: ImportMode::Add,
        })]
    );
    assert_eq!(view(&harness).model.submit_label(), "Importing...");
    harness.press(KeyCode::F(2));
    assert_eq!(harness.pending().len(), 1, "one import at a time");
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
    let v = &view(&harness).model;
    assert_eq!(
        (v.source(), v.mode(), v.board()),
        (Source::Archidekt, ImportMode::Replace, Board::Side)
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
        [screens::Request::DeckImport(Request::Archidekt { deck_id: id, board: Some("sideboard"), mode: ImportMode::Replace, .. })] if *id == deck_id
    ));
}

#[test]
fn results_toast_and_list_what_came_in() {
    let (mut harness, ticket) = importing();
    harness.apply(Update::Imported {
        ticket,
        result: mixed(),
    });
    let up = &harness.app.shared.toasts.items()[0];
    assert_eq!(up.message, "Imported 1, 1 unresolved");
    assert_eq!(up.expires_at.duration_since(up.shown_at), QUICK);
    let screen = screen_text(&harness.app);
    for expected in ["Imported", "Sol Ring", "x1", "Unresolved", "not found"] {
        assert!(screen.contains(expected), "missing {expected}");
    }

    harness.press(KeyCode::F(2));
    let ticket = last_ticket(&harness);
    harness.apply(Update::Imported {
        ticket,
        result: ImportDeckCardsResult {
            imported: Vec::new(),
            unresolved: vec![UnresolvedCard {
                name: "Nope".to_owned(),
                reason: "not found".to_owned(),
            }],
        },
    });
    let failed = harness.app.shared.toasts.items().last().unwrap();
    assert_eq!(failed.message, "1 card unresolved");
    assert_eq!(failed.expires_at.duration_since(failed.shown_at), NORMAL);
}

#[test]
fn a_failed_import_toasts_the_server_message() {
    let (mut harness, ticket) = importing();
    harness.apply(Update::Failed {
        ticket,
        error: "deck not found".to_owned(),
    });
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "deck not found"
    );
    assert!(!view(&harness).model.importing());
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
            "Too many card lines, the limit is 1000",
        ),
        (
            ClientError::PayloadTooLarge("length limit exceeded".to_owned()),
            "Request too large, try importing fewer cards",
        ),
    ];
    for (error, message) in cases {
        let (mut harness, ticket) = importing();
        harness.apply(outcome(ticket, Err(error)));
        let toast = &harness.app.shared.toasts.items()[0];
        assert_eq!(toast.message, message);
        assert_eq!(toast.kind, ToastKind::Error);
        assert!(!view(&harness).model.importing());
    }
}

#[test]
fn an_import_clears_the_decks_undo_log_and_only_that_deck() {
    let (mut harness, deck_id) = opened();
    harness.type_text("1 Sol Ring");
    harness.press(KeyCode::F(2));
    let ticket = last_ticket(&harness);
    let other = Uuid::new_v4();
    for id in [deck_id, other] {
        harness.app.shared.cards.undo.push(
            id,
            zwipe_ui::stores::undo::UndoAction::Added {
                card_id: Uuid::new_v4(),
                card_name: "Sol Ring".to_owned(),
            },
        );
    }
    harness.apply(Update::Imported {
        ticket,
        result: ImportDeckCardsResult {
            imported: Vec::new(),
            unresolved: Vec::new(),
        },
    });
    assert!(
        !harness.app.shared.cards.undo.is_empty(deck_id),
        "nothing imported keeps the log"
    );
    harness.press(KeyCode::F(2));
    let ticket = last_ticket(&harness);
    harness.apply(Update::Imported {
        ticket,
        result: ImportDeckCardsResult {
            imported: vec![ImportedCard {
                name: "Sol Ring".to_owned(),
                quantity: 1,
            }],
            unresolved: Vec::new(),
        },
    });
    assert!(harness.app.shared.cards.undo.is_empty(deck_id));
    assert!(!harness.app.shared.cards.undo.is_empty(other));
}

#[test]
fn import_keeps_every_key_on_a_narrow_terminal() {
    crate::test_support::assert_key_bar_fits(&opened().0.app, "F2 Import");
}

#[test]
fn the_paste_box_draws_empty_and_the_hint_shows_an_example_list() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    let deck_id = Uuid::new_v4();
    harness.open(|cx| open(cx, deck_id));
    let first = screen_text(&harness.app);
    for line in EXAMPLE {
        assert!(first.contains(line), "the first hint shows {line:?}");
    }
    harness.press(KeyCode::Enter);
    assert!(harness.app.shared.hint.is_none());
    let screen = screen_text(&harness.app);
    for line in EXAMPLE {
        assert!(!screen.contains(line), "the empty box shows no {line:?}");
    }
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Char('?'));
    let again = screen_text(&harness.app);
    assert!(
        again.contains("one card a line"),
        "? reopens it with the example"
    );
    assert!(again.contains(EXAMPLE[2]));
}

#[test]
fn a_stale_or_repeated_reply_changes_nothing() {
    let (mut harness, first) = importing();
    harness.apply(Update::Failed {
        ticket: first,
        error: "offline".to_owned(),
    });
    harness.press(KeyCode::F(2));
    let second = last_ticket(&harness);
    let toasts = harness.app.shared.toasts.items().len();
    harness.apply(Update::Imported {
        ticket: first,
        result: mixed(),
    });
    assert!(
        view(&harness).model.importing() && view(&harness).model.result().is_none(),
        "the first import's late reply leaves the second out"
    );
    assert_eq!(
        harness.app.shared.toasts.items().len(),
        toasts,
        "and raises nothing"
    );
    harness.apply(Update::Imported {
        ticket: second,
        result: mixed(),
    });
    harness.apply(Update::Imported {
        ticket: second,
        result: ImportDeckCardsResult {
            imported: Vec::new(),
            unresolved: Vec::new(),
        },
    });
    assert_eq!(
        view(&harness).model.result().map(|r| r.imported.len()),
        Some(1),
        "a repeated reply keeps the first answer"
    );
}
