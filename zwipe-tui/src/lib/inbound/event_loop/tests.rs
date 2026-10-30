//! Tests for the main loop: starting up, the back stack, and the app's own results, with no network.

use crossterm::event::KeyCode;

use super::Flow;
use super::harness::Harness;
use crate::domain::theme::Palette;
use crate::inbound::screens::decks::DeckList;
use crate::inbound::screens::login::LoginForm;
use crate::inbound::screens::{Request, Screen, Update, deck, decks};
use crate::test_support::{card, render, screen_text};

#[test]
fn without_a_session_it_asks_to_sign_in_and_checks_the_version() {
    let mut harness = Harness::new();
    harness.start();
    assert!(matches!(harness.screen(), Screen::Login(_)));
    assert_eq!(harness.pending(), [Request::CheckVersion]);
}

#[test]
fn esc_steps_back_from_swipe_to_deck_to_decks_to_home() {
    let (mut harness, deck_id) = Harness::swiping(vec![card("A")]);
    harness.press(KeyCode::Esc);
    assert!(
        matches!(harness.screen(), Screen::Deck(view) if view.profile.id == deck_id && view.deck.is_none())
    );
    assert_eq!(
        harness.pending(),
        [Request::Deck(deck::Request::Load(deck_id))],
        "the deck reloads with the new cards"
    );

    harness.clear();
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Decks(list) if list.loading));
    assert_eq!(harness.pending(), [Request::Decks(decks::Request::Load)]);

    harness.clear();
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
    assert_eq!(harness.app.nav.depth(), 1);
    assert_eq!(harness.press(KeyCode::Esc), Flow::Quit);
}

#[test]
fn a_result_reaches_its_screen_under_another() {
    let (mut harness, _) = Harness::swiping(Vec::new());
    harness.apply(decks::Update::Failed("offline".to_owned()));
    harness.press(KeyCode::Esc);
    harness.clear();
    harness.press(KeyCode::Esc);
    assert!(
        matches!(harness.screen(), Screen::Decks(list) if list.loading && list.error.is_none()),
        "going back reloads over the old error"
    );
}

#[test]
fn an_ended_session_returns_to_sign_in_and_clears_the_saved_one() {
    let (mut harness, _) = Harness::swiping(vec![card("A")]);
    harness.app.shared.palette = Palette::for_account("gruvbox", true);
    harness.app.shared.username = Some("scotty".to_owned());
    harness.apply(Update::SessionEnded);
    assert!(matches!(harness.screen(), Screen::Login(form) if form.error.is_some()));
    assert_eq!(harness.app.nav.depth(), 1, "the stack starts over");
    assert_eq!(
        harness.app.shared.palette,
        Palette::for_account("gruvbox", true),
        "the theme stays"
    );
    assert!(harness.app.shared.username.is_none());
    assert_eq!(harness.pending(), [Request::SignOut]);
}

#[test]
fn an_old_build_is_stopped_until_it_updates() {
    let mut harness = Harness::new();
    harness.apply(Update::TooOld);
    assert!(matches!(harness.screen(), Screen::UpdateRequired));
    assert_eq!(harness.press(KeyCode::Char('l')), Flow::Continue);
    assert_eq!(harness.press(KeyCode::Char('q')), Flow::Quit);
}

#[test]
fn the_update_notice_gives_the_install_command() {
    let harness = Harness::on(Screen::UpdateRequired);
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Update required"));
    assert!(screen.contains("zwipe-tui"));
}

#[test]
fn every_screen_fits_a_tiny_terminal() {
    let (harness, _) = Harness::swiping(vec![card("Tiny")]);
    render(&harness.app, 12, 5);
    for screen in [
        Screen::Login(LoginForm::default()),
        Screen::Decks(DeckList::default()),
        Screen::UpdateRequired,
    ] {
        render(&Harness::on(screen).app, 12, 5);
    }
}
