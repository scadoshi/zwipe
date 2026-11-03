use crossterm::event::KeyCode;
use zwipe_core::domain::deck::{Board, DeckProfile};

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, deck, deck_profile, render, screen_text, user};

/// The ticket the latest deck fetch went out with.
fn last_ticket(harness: &Harness) -> Ticket {
    match harness
        .pending()
        .iter()
        .rev()
        .find(|r| matches!(r, screens::Request::DeckExport(_)))
    {
        Some(screens::Request::DeckExport(Request::Load { ticket, .. })) => *ticket,
        other => panic!("no fetch went out: {other:?}"),
    }
}

/// A deck with Sol Ring in the deck and Opt on the maybeboard.
fn pile(profile: DeckProfile) -> Box<Deck> {
    let mut built = deck(profile, vec![card("Sol Ring"), card("Opt")]);
    built.entries[1].deck_card.board = Board::Maybeboard;
    Box::new(built)
}

/// Fetches the deck again, as a second `Load` would.
fn reload(harness: &mut Harness, deck_id: Uuid) {
    harness.on_top(|screen, cx| {
        let Screen::DeckExport(view) = screen else {
            panic!("not on export: {screen:?}");
        };
        dispatch(view, Action::Load(deck_id), cx);
    });
}

fn loaded() -> (Harness, Uuid) {
    let profile = deck_profile("Pile");
    let deck_id = profile.id;
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.open(|cx| open(cx, deck_id));
    harness.app.shared.hint = None;
    let ticket = last_ticket(&harness);
    harness.apply(Update::Loaded {
        ticket,
        deck: pile(profile),
    });
    (harness, deck_id)
}

fn view(harness: &Harness) -> &ExportView {
    match harness.screen() {
        Screen::DeckExport(view) => view,
        other => panic!("not on export: {other:?}"),
    }
}

#[test]
fn opening_fetches_the_deck_and_shows_the_hint_once() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    let deck_id = Uuid::new_v4();
    harness.open(|cx| open(cx, deck_id));
    let ticket = last_ticket(&harness);
    assert_eq!(
        harness.pending(),
        [
            screens::Request::MarkHintShown(OneTimeHint::Export.key().to_owned()),
            screens::Request::DeckExport(Request::Load { ticket, deck_id }),
        ]
    );
    assert!(screen_text(&harness.app).contains("Exporting your deck"));
}

#[test]
fn the_board_chips_change_the_text_and_keep_one_on() {
    let (mut harness, _) = loaded();
    assert_eq!(view(&harness).model.text().unwrap(), "// Deck\n1 Sol Ring");
    harness.press(KeyCode::Enter);
    assert!(
        view(&harness).model.boards().deck,
        "the only board stays on"
    );
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char(' '));
    assert_eq!(
        view(&harness).model.text().unwrap(),
        "// Deck\n1 Sol Ring\n\n// Maybeboard\n1 Opt"
    );
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Maybeboard"));
    assert!(screen.contains("Decklist"));
}

#[test]
fn copy_toasts_once_loaded_and_esc_goes_back() {
    let (mut harness, _) = loaded();
    harness.press(KeyCode::Char('y'));
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Copied to clipboard"
    );
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
}

#[test]
fn a_failed_load_says_so() {
    let (mut harness, deck_id) = loaded();
    reload(&mut harness, deck_id);
    let ticket = last_ticket(&harness);
    harness.apply(Update::Failed {
        ticket,
        error: "deck not found".to_owned(),
    });
    assert!(screen_text(&harness.app).contains("Could not load deck"));
}

#[test]
fn export_fits_a_tiny_terminal() {
    let (harness, _) = loaded();
    render(&harness.app, 12, 5);
    render(&harness.app, 1, 1);
}

#[test]
fn export_keeps_every_key_on_a_narrow_terminal() {
    crate::test_support::assert_key_bar_fits(&loaded().0.app, "q quit");
}

#[test]
fn a_reply_a_reload_replaced_changes_nothing() {
    let (mut harness, deck_id) = loaded();
    let first = last_ticket(&harness);
    reload(&mut harness, deck_id);
    let second = last_ticket(&harness);
    harness.apply(Update::Loaded {
        ticket: first,
        deck: pile(deck_profile("Stale")),
    });
    harness.apply(Update::Failed {
        ticket: first,
        error: "offline".to_owned(),
    });
    assert_eq!(
        view(&harness).model.status(),
        zwipe_ui::screens::deck_export::Status::Loading,
        "only the reload's reply lands"
    );
    harness.apply(Update::Loaded {
        ticket: second,
        deck: pile(deck_profile("Pile")),
    });
    assert_eq!(view(&harness).model.text().unwrap(), "// Deck\n1 Sol Ring");
}
