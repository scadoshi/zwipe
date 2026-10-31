//! Tests for the main loop: starting up, the back stack, and the app's own results, with no network.

use std::time::Instant;

use crossterm::event::KeyCode;
use zwipe_core::http::contracts::changelog::HttpChangelog;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;

use super::Flow;
use super::harness::Harness;
use crate::domain::changelog::ChangelogCache;
use crate::domain::theme::Palette;
use crate::inbound::screens::{Request, Screen, Update, deck, decks};
use crate::inbound::widgets::hint;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::test_support::{card, render, screen_text, user};

#[test]
fn without_a_session_it_asks_to_sign_in_and_fetches_the_changelog_and_sends_a_crash() {
    let mut harness = Harness::new();
    harness.start();
    assert!(matches!(harness.screen(), Screen::Login(_)));
    assert_eq!(
        harness.pending(),
        [
            Request::FetchChangelog,
            Request::SendCrashReport,
            Request::Record(AnonymousEventKind::AppOpened),
            Request::FetchCounts,
        ]
    );
}

#[test]
fn the_changelog_lands_in_the_cache_or_marks_it_failed() {
    let mut harness = Harness::new();
    assert_eq!(harness.app.shared.changelog, ChangelogCache::Loading);
    harness.apply(Update::Changelog(None));
    assert_eq!(harness.app.shared.changelog, ChangelogCache::Failed);
    let changelog = HttpChangelog::current();
    harness.apply(Update::Changelog(Some(Box::new(changelog.clone()))));
    assert_eq!(
        harness.app.shared.changelog,
        ChangelogCache::Loaded(changelog)
    );
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
    harness.app.shared.user = Some(user(true));
    harness.apply(Update::SessionEnded);
    assert!(
        matches!(harness.screen(), Screen::Login(_)),
        "silently, as the app's auth gate"
    );
    assert_eq!(harness.app.nav.depth(), 1, "the stack starts over");
    assert_eq!(
        harness.app.shared.palette,
        Palette::for_account("gruvbox", true),
        "the theme stays"
    );
    assert!(harness.app.shared.username.is_none());
    assert_eq!(harness.pending(), [Request::FetchCounts, Request::SignOut]);
}

#[test]
fn a_toast_shows_over_the_screen_and_redraws_until_it_leaves() {
    let mut harness = Harness::on(Screen::Decks(Box::default()));
    assert!(!harness.app.animating());
    harness.apply(Update::Toast(ToastKind::Success, "Saved".to_owned()));
    assert!(
        harness.app.animating(),
        "the loop ticks while a toast is up"
    );
    assert!(screen_text(&harness.app).contains("Saved"));
    harness.app.tick(Instant::now() + toast::LIFETIME);
    assert!(!harness.app.animating());
    assert!(!screen_text(&harness.app).contains("Saved"));
}

#[test]
fn an_open_hint_takes_every_key_until_it_closes() {
    let mut harness = Harness::on(Screen::Decks(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.open(|cx| {
        hint::show(cx, "swipe", "Swiping", "Right adds the card.");
    });
    assert_eq!(
        harness.pending(),
        [Request::MarkHintShown("swipe".to_owned())]
    );
    assert_eq!(
        harness.press(KeyCode::Char('q')),
        Flow::Continue,
        "q does not reach the screen"
    );
    assert!(harness.app.shared.hint.is_some());
    harness.press(KeyCode::Enter);
    assert!(harness.app.shared.hint.is_none());

    let mut marked = user(true);
    marked.hints_shown.insert("swipe".to_owned(), true);
    harness.apply(Update::HintMarked(Box::new(marked)));
    assert!(
        harness
            .app
            .shared
            .user
            .as_ref()
            .unwrap()
            .has_seen_hint("swipe")
    );
    assert_eq!(harness.press(KeyCode::Char('q')), Flow::Quit);
}

#[test]
fn an_old_build_is_stopped_until_it_updates() {
    let mut harness = Harness::new();
    harness.apply(Update::Upkept {
        ended: false,
        too_old: Some(true),
    });
    assert!(harness.app.shared.update_required);
    assert!(screen_text(&harness.app).contains("Update required"));
    assert_eq!(harness.press(KeyCode::Char('l')), Flow::Continue);
    assert!(
        matches!(harness.screen(), Screen::Login(form) if form.identifier.is_empty()),
        "keys do not reach the screen under it"
    );
    assert_eq!(harness.press(KeyCode::Char('q')), Flow::Quit);
}

#[test]
fn the_update_notice_gives_the_install_command() {
    let mut harness = Harness::new();
    harness.app.shared.update_required = true;
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Update required"));
    assert!(screen.contains("zwiper-tui"));
}

#[test]
fn every_screen_fits_a_tiny_terminal() {
    let (harness, _) = Harness::swiping(vec![card("Tiny")]);
    render(&harness.app, 12, 5);
    for screen in [Screen::Login(Box::default()), Screen::Decks(Box::default())] {
        render(&Harness::on(screen).app, 12, 5);
    }
    let mut notice = Harness::new();
    notice.app.shared.update_required = true;
    render(&notice.app, 12, 5);
}
