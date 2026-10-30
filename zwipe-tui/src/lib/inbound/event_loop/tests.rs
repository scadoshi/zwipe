//! Tests for the main loop: every key and update, with no network.

use reqwest::Url;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;

use super::*;
use crate::outbound::account::SessionFile;
use crate::test_support::{card, deck, deck_profile};

fn event_loop() -> EventLoop {
    let client = ZwipeClient::new(Url::parse("http://127.0.0.1:9").unwrap());
    let file = SessionFile::at(std::env::temp_dir().join("zwipe-tui-tests-no-session.json"));
    EventLoop::new(Arc::new(Account::new(client, file)))
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn type_text(event_loop: &mut EventLoop, app: &mut App, text: &str) {
    for c in text.chars() {
        event_loop.handle_key(app, key(KeyCode::Char(c)));
    }
}

fn swipe_view(app: &App) -> &SwipeView {
    match &app.screen {
        Screen::Swipe(view) => view,
        other => panic!("not on the swipe screen: {other:?}"),
    }
}

/// A loop on the swipe screen for a fresh deck, with `cards` served as the first page.
fn swiping(cards: Vec<Card>) -> (EventLoop, App, Uuid) {
    let mut event_loop = event_loop();
    let mut app = App::new();
    let profile = deck_profile("Pile");
    let deck_id = profile.id;
    app.screen = Screen::Decks(DeckList::default());
    event_loop.apply(&mut app, Update::Decks(vec![profile.clone()]));
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    event_loop.apply(
        &mut app,
        Update::DeckOpened(Box::new(deck(profile, Vec::new()))),
    );
    event_loop.apply(
        &mut app,
        Update::Cards {
            deck_id,
            page: HttpDeckCardSearch {
                cards,
                synergy_applied: false,
                related_from: None,
            },
        },
    );
    event_loop.pending.clear();
    (event_loop, app, deck_id)
}

#[test]
fn without_a_session_it_asks_to_sign_in_and_checks_the_version() {
    let mut event_loop = event_loop();
    let mut app = App::new();
    event_loop.start(&mut app, None);
    assert!(matches!(app.screen, Screen::Login(_)));
    assert_eq!(event_loop.pending, [Request::CheckVersion]);
}

#[test]
fn the_form_moves_to_the_password_then_signs_in() {
    let mut event_loop = event_loop();
    let mut app = App::new();
    type_text(&mut event_loop, &mut app, " scotty ");
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    type_text(&mut event_loop, &mut app, "hunter2");
    event_loop.handle_key(&mut app, key(KeyCode::Enter));

    let Screen::Login(form) = &app.screen else {
        panic!("left the login screen");
    };
    assert!(form.busy);
    assert_eq!(
        event_loop.pending,
        [Request::SignIn {
            identifier: "scotty".to_owned(),
            password: Password("hunter2".to_owned()),
        }]
    );
}

#[test]
fn an_empty_form_is_refused_without_a_request() {
    let mut event_loop = event_loop();
    let mut app = App::new();
    event_loop.handle_key(&mut app, key(KeyCode::Tab));
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    let Screen::Login(form) = &app.screen else {
        panic!("left the login screen");
    };
    assert!(form.error.is_some());
    assert_eq!(event_loop.pending, []);
}

#[test]
fn a_failed_sign_in_shows_why_and_allows_another_try() {
    let mut event_loop = event_loop();
    let mut app = App::new();
    if let Screen::Login(form) = &mut app.screen {
        form.busy = true;
    }
    event_loop.apply(
        &mut app,
        Update::SignInFailed("invalid credentials".to_owned()),
    );
    let Screen::Login(form) = &app.screen else {
        panic!("left the login screen");
    };
    assert!(!form.busy);
    assert_eq!(form.error.as_deref(), Some("invalid credentials"));
}

#[test]
fn the_password_never_shows_in_debug_output() {
    let request = Request::SignIn {
        identifier: "scotty".to_owned(),
        password: Password("hunter2".to_owned()),
    };
    assert!(!format!("{request:?}").contains("hunter2"));
}

#[test]
fn q_types_on_the_login_screen_and_ctrl_c_quits() {
    let mut event_loop = event_loop();
    let mut app = App::new();
    assert_eq!(
        event_loop.handle_key(&mut app, key(KeyCode::Char('q'))),
        Flow::Continue
    );
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(event_loop.handle_key(&mut app, ctrl_c), Flow::Quit);
}

#[test]
fn enter_on_a_deck_opens_it_for_swiping() {
    let mut event_loop = event_loop();
    let mut app = App::new();
    let (first, second) = (deck_profile("First"), deck_profile("Second"));
    app.screen = Screen::Decks(DeckList::loading());
    event_loop.apply(&mut app, Update::Decks(vec![first, second.clone()]));
    event_loop.handle_key(&mut app, key(KeyCode::Char('j')));
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert_eq!(swipe_view(&app).deck.id, second.id);
    assert_eq!(event_loop.pending, [Request::OpenDeck(second.id)]);
}

#[test]
fn an_opened_deck_fetches_its_first_page() {
    let mut event_loop = event_loop();
    let mut app = App::new();
    let profile = deck_profile("Pile");
    app.screen = Screen::Swipe(Box::new(SwipeView::opening(profile.clone())));
    event_loop.apply(
        &mut app,
        Update::DeckOpened(Box::new(deck(profile.clone(), Vec::new()))),
    );
    let view = swipe_view(&app);
    assert!(view.stack.is_some() && view.fetching);
    assert!(matches!(
        event_loop.pending.as_slice(),
        [Request::FetchCards { deck_id, .. }] if *deck_id == profile.id
    ));
}

#[test]
fn swipes_send_their_effects_and_undo_reverses_the_count() {
    let (a, b) = (card("A"), card("B"));
    let (mut event_loop, mut app, deck_id) = swiping(vec![a.clone(), b.clone()]);

    event_loop.handle_key(&mut app, key(KeyCode::Char('l')));
    event_loop.handle_key(&mut app, key(KeyCode::Char('h')));
    assert_eq!(swipe_view(&app).added, 1);
    assert!(matches!(
        event_loop.pending.as_slice(),
        [
            Request::Send {
                effect: Effect::Add {
                    maybeboard: false,
                    ..
                },
                ..
            },
            Request::Send {
                effect: Effect::Skip { .. },
                ..
            },
        ]
    ));

    event_loop.pending.clear();
    event_loop.handle_key(&mut app, key(KeyCode::Char('u')));
    event_loop.handle_key(&mut app, key(KeyCode::Char('u')));
    assert_eq!(swipe_view(&app).added, 0);
    assert_eq!(
        event_loop.pending,
        [
            Request::Send {
                deck_id,
                effect: Effect::Unskip {
                    oracle_id: b.scryfall_data.oracle_id.unwrap()
                }
            },
            Request::Send {
                deck_id,
                effect: Effect::Remove {
                    scryfall_data_id: a.scryfall_data.id
                }
            },
        ]
    );
}

#[test]
fn a_short_page_asks_for_the_next_one_once() {
    let mut event_loop = event_loop();
    let mut app = App::new();
    let profile = deck_profile("Pile");
    let deck_id = profile.id;
    app.screen = Screen::Swipe(Box::new(SwipeView::opening(profile.clone())));
    event_loop.apply(
        &mut app,
        Update::DeckOpened(Box::new(deck(profile, Vec::new()))),
    );
    event_loop.pending.clear();

    let page = |cards| HttpDeckCardSearch {
        cards,
        synergy_applied: false,
        related_from: None,
    };
    event_loop.apply(
        &mut app,
        Update::Cards {
            deck_id,
            page: page(vec![card("A")]),
        },
    );
    assert_eq!(
        event_loop.pending.len(),
        1,
        "a page of one leaves the stack low"
    );

    event_loop.pending.clear();
    event_loop.apply(
        &mut app,
        Update::Cards {
            deck_id,
            page: page(Vec::new()),
        },
    );
    assert_eq!(event_loop.pending, [], "an empty page ends the stack");
    assert!(swipe_view(&app).stack.as_ref().unwrap().is_exhausted());
}

#[test]
fn swiping_with_no_card_does_nothing() {
    let (mut event_loop, mut app, _) = swiping(Vec::new());
    event_loop.handle_key(&mut app, key(KeyCode::Char('l')));
    assert_eq!(swipe_view(&app).added, 0);
    assert!(
        !event_loop
            .pending
            .iter()
            .any(|r| matches!(r, Request::Send { .. }))
    );
}

#[test]
fn esc_goes_back_to_a_fresh_deck_list() {
    let (mut event_loop, mut app, _) = swiping(vec![card("A")]);
    event_loop.handle_key(&mut app, key(KeyCode::Esc));
    assert!(matches!(&app.screen, Screen::Decks(list) if list.loading));
    assert_eq!(event_loop.pending, [Request::LoadDecks]);
}

#[test]
fn an_ended_session_returns_to_sign_in_and_clears_the_saved_one() {
    let (mut event_loop, mut app, _) = swiping(vec![card("A")]);
    app.palette = Palette::for_account("gruvbox", true);
    app.username = Some("scotty".to_owned());
    event_loop.apply(&mut app, Update::SessionEnded);
    assert!(matches!(&app.screen, Screen::Login(form) if form.error.is_some()));
    assert_eq!(app.palette, Palette::terminal());
    assert!(app.username.is_none());
    assert_eq!(event_loop.pending, [Request::SignOut]);
}

#[test]
fn an_old_build_is_stopped_until_it_updates() {
    let mut event_loop = event_loop();
    let mut app = App::new();
    event_loop.apply(&mut app, Update::TooOld);
    assert!(matches!(app.screen, Screen::UpdateRequired));
    assert_eq!(
        event_loop.handle_key(&mut app, key(KeyCode::Char('l'))),
        Flow::Continue
    );
    assert_eq!(
        event_loop.handle_key(&mut app, key(KeyCode::Char('q'))),
        Flow::Quit
    );
}

#[test]
fn a_failed_request_ends_the_session_only_on_unauthorized() {
    assert!(matches!(
        failure(&ClientError::Unauthorized("x".to_owned()), Update::Status),
        Update::SessionEnded
    ));
    assert!(matches!(
        failure(&ClientError::NotFound("gone".to_owned()), Update::Status),
        Update::Status(m) if m == "gone"
    ));
}
