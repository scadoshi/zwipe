use crossterm::event::KeyCode;
use zwipe_core::domain::deck::Board;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, deck, deck_profile, render, screen_text, user};

fn loaded() -> (Harness, Uuid) {
    let profile = deck_profile("Pile");
    let deck_id = profile.id;
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.open(|cx| open(cx, deck_id));
    harness.app.shared.hint = None;
    let mut built = deck(profile, vec![card("Sol Ring"), card("Opt")]);
    built.entries[1].deck_card.board = Board::Maybeboard;
    harness.apply(Update::Loaded(Box::new(built)));
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
    assert_eq!(
        harness.pending(),
        [
            screens::Request::DeckExport(Request::Load(deck_id)),
            screens::Request::MarkHintShown(HINT_KEY.to_owned())
        ]
    );
    assert!(screen_text(&harness.app).contains("Exporting your deck"));
}

#[test]
fn the_board_chips_change_the_text_and_keep_one_on() {
    let (mut harness, _) = loaded();
    assert_eq!(view(&harness).text().unwrap(), "// Deck\n1 Sol Ring");
    harness.press(KeyCode::Enter);
    assert!(view(&harness).boards.deck, "the only board stays on");
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char(' '));
    assert_eq!(
        view(&harness).text().unwrap(),
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
    let (mut harness, _) = loaded();
    harness.apply(Update::Failed("deck not found".to_owned()));
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
