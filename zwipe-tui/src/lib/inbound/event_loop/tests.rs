//! Tests for the main loop: every key and update, with no network.

use std::collections::HashSet;

use reqwest::Url;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;

use super::settings::{self, SettingsRequest};
use super::*;
use crate::domain::theme::Palette;
use crate::inbound::app::Section;
use crate::outbound::account::SessionFile;
use crate::outbound::theme_file::ThemeFile;
use crate::test_support::{card, deck, deck_profile};

fn event_loop() -> EventLoop {
    let client = ZwipeClient::new(Url::parse("http://127.0.0.1:9").unwrap());
    let file = SessionFile::at(std::env::temp_dir().join("zwipe-tui-tests-no-session.json"));
    let theme_file = ThemeFile::at(std::env::temp_dir().join("zwipe-tui-tests-theme.json"));
    EventLoop::new(Arc::new(Account::new(client, file)), Arc::new(theme_file))
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
    let mut app = App::new(Palette::terminal());
    let profile = deck_profile("Pile");
    let deck_id = profile.id;
    app.screen = Screen::Decks(DeckList::default());
    event_loop.apply(&mut app, Update::Decks(vec![profile.clone()]));
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    event_loop.apply(
        &mut app,
        Update::DeckLoaded(Box::new(deck(profile, Vec::new()))),
    );
    event_loop.handle_key(&mut app, key(KeyCode::Char('s')));
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
    let mut app = App::new(Palette::terminal());
    event_loop.start(&mut app, None);
    assert!(matches!(app.screen, Screen::Login(_)));
    assert_eq!(event_loop.pending, [Request::CheckVersion]);
}

#[test]
fn the_form_moves_to_the_password_then_signs_in() {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
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
    let mut app = App::new(Palette::terminal());
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
    let mut app = App::new(Palette::terminal());
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
    let mut app = App::new(Palette::terminal());
    assert_eq!(
        event_loop.handle_key(&mut app, key(KeyCode::Char('q'))),
        Flow::Continue
    );
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(event_loop.handle_key(&mut app, ctrl_c), Flow::Quit);
}

#[test]
fn enter_on_a_deck_opens_its_screen() {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
    let (first, second) = (deck_profile("First"), deck_profile("Second"));
    app.screen = Screen::Decks(DeckList::loading());
    event_loop.apply(&mut app, Update::Decks(vec![first, second.clone()]));
    event_loop.handle_key(&mut app, key(KeyCode::Char('j')));
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert!(matches!(&app.screen, Screen::Deck(view) if view.profile.id == second.id));
    assert_eq!(event_loop.pending, [Request::LoadDeck(second.id)]);
}

#[test]
fn s_on_a_loaded_deck_starts_swiping_and_fetches_the_first_page() {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
    let profile = deck_profile("Pile");
    app.screen = Screen::Deck(Box::new(DeckView::opening(profile.clone())));
    event_loop.handle_key(&mut app, key(KeyCode::Char('s')));
    assert!(
        matches!(app.screen, Screen::Deck(_)),
        "nothing to swipe before the deck loads"
    );

    event_loop.apply(
        &mut app,
        Update::DeckLoaded(Box::new(deck(profile.clone(), Vec::new()))),
    );
    event_loop.handle_key(&mut app, key(KeyCode::Char('s')));
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
    let (mut event_loop, mut app, deck_id) = swiping(vec![card("A")]);
    assert_eq!(
        event_loop.pending,
        [],
        "the helper clears the follow-up fetch"
    );

    let page = |cards| HttpDeckCardSearch {
        cards,
        synergy_applied: false,
        related_from: None,
    };
    if let Screen::Swipe(view) = &mut app.screen {
        view.fetching = false;
    }
    event_loop.apply(
        &mut app,
        Update::Cards {
            deck_id,
            page: page(vec![card("B")]),
        },
    );
    assert_eq!(
        event_loop.pending.len(),
        1,
        "two cards still leave the stack low"
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
fn esc_steps_back_from_swipe_to_deck_to_decks_to_home() {
    let (mut event_loop, mut app, deck_id) = swiping(vec![card("A")]);
    event_loop.handle_key(&mut app, key(KeyCode::Esc));
    assert!(
        matches!(&app.screen, Screen::Deck(view) if view.profile.id == deck_id && view.deck.is_none())
    );
    assert_eq!(
        event_loop.pending,
        [Request::LoadDeck(deck_id)],
        "the deck reloads with the new cards"
    );

    event_loop.pending.clear();
    event_loop.handle_key(&mut app, key(KeyCode::Esc));
    assert!(matches!(&app.screen, Screen::Decks(list) if list.loading));
    assert_eq!(event_loop.pending, [Request::LoadDecks]);

    event_loop.pending.clear();
    event_loop.handle_key(&mut app, key(KeyCode::Esc));
    assert!(matches!(app.screen, Screen::Home(_)));
    assert_eq!(
        event_loop.handle_key(&mut app, key(KeyCode::Esc)),
        Flow::Quit
    );
}

#[test]
fn home_fetches_its_panels_and_fills_them_as_they_arrive() {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
    event_loop.open_home(&mut app);
    assert_eq!(
        event_loop.pending,
        [Request::PublicMetrics, Request::PlayStyle, Request::Flavor]
    );
    let figures = Figures {
        decks_created: 1,
        searches: 2,
        cards_swiped: 3,
    };
    event_loop.apply(&mut app, Update::PublicMetrics(figures));
    event_loop.apply(&mut app, Update::PlayStyleFailed);
    event_loop.apply(&mut app, Update::Flavor(Box::new(card("Flavor"))));
    let Screen::Home(home) = &app.screen else {
        panic!("left home");
    };
    assert_eq!(home.figures(), Some(figures));
    assert!(home.play_style_failed);
    assert!(home.flavor.is_some());
}

#[test]
fn s_on_home_switches_to_my_counters_and_fetches_them() {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
    event_loop.open_home(&mut app);
    event_loop.pending.clear();
    event_loop.handle_key(&mut app, key(KeyCode::Char('s')));
    assert!(
        matches!(&app.screen, Screen::Home(home) if home.scope == Scope::OnlyMe && home.figures().is_none())
    );
    assert_eq!(event_loop.pending, [Request::MyMetrics]);

    event_loop.pending.clear();
    event_loop.handle_key(&mut app, key(KeyCode::Char('s')));
    assert!(matches!(&app.screen, Screen::Home(home) if home.scope == Scope::Everybody));
    assert_eq!(event_loop.pending, []);
}

#[test]
fn d_on_home_opens_the_decks() {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
    event_loop.open_home(&mut app);
    event_loop.pending.clear();
    event_loop.handle_key(&mut app, key(KeyCode::Char('d')));
    assert!(matches!(app.screen, Screen::Decks(_)));
    assert_eq!(event_loop.pending, [Request::LoadDecks]);
}

#[test]
fn the_deck_screen_steps_tabs_currency_and_turn() {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
    let profile = deck_profile("Pile");
    app.screen = Screen::Deck(Box::new(DeckView::opening(profile.clone())));
    event_loop.apply(
        &mut app,
        Update::DeckLoaded(Box::new(deck(profile, vec![card("A")]))),
    );

    let deck_view = |app: &App| match &app.screen {
        Screen::Deck(view) => (view.section, view.currency, view.turn, view.sections()),
        other => panic!("left the deck screen: {other:?}"),
    };
    let (section, _, _, sections) = deck_view(&app);
    assert_eq!(section, Section::Budget);
    assert!(sections.contains(&Section::Budget));

    event_loop.handle_key(&mut app, key(KeyCode::Char('c')));
    event_loop.handle_key(&mut app, key(KeyCode::Char('-')));
    for _ in 0..25 {
        event_loop.handle_key(&mut app, key(KeyCode::Char('+')));
    }
    let (_, currency, turn, _) = deck_view(&app);
    assert_eq!(currency, PriceCurrency::Eur);
    assert_eq!(turn, crate::inbound::app::MAX_TURN);

    let count = sections.len();
    for _ in 0..count {
        event_loop.handle_key(&mut app, key(KeyCode::Tab));
    }
    assert_eq!(deck_view(&app).0, Section::Budget, "Tab wraps back around");
    event_loop.handle_key(&mut app, key(KeyCode::BackTab));
    assert_eq!(Some(&deck_view(&app).0), sections.last());
}

#[test]
fn a_deck_result_for_another_deck_is_ignored() {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
    app.screen = Screen::Deck(Box::new(DeckView::opening(deck_profile("Open"))));
    event_loop.apply(
        &mut app,
        Update::DeckLoaded(Box::new(deck(deck_profile("Other"), Vec::new()))),
    );
    assert!(matches!(&app.screen, Screen::Deck(view) if view.deck.is_none()));
}

#[test]
fn an_ended_session_returns_to_sign_in_and_clears_the_saved_one() {
    let (mut event_loop, mut app, _) = swiping(vec![card("A")]);
    app.palette = Palette::for_account("gruvbox", true);
    app.username = Some("scotty".to_owned());
    event_loop.apply(&mut app, Update::SessionEnded);
    assert!(matches!(&app.screen, Screen::Login(form) if form.error.is_some()));
    assert_eq!(
        app.palette,
        Palette::for_account("gruvbox", true),
        "the theme stays"
    );
    assert!(app.username.is_none());
    assert_eq!(event_loop.pending, [Request::SignOut]);
}

#[test]
fn an_old_build_is_stopped_until_it_updates() {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
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

#[test]
fn home_animates_only_while_the_logo_decodes() {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
    assert!(!app.animating(), "the login screen is still");
    event_loop.open_home(&mut app);
    assert!(app.animating(), "a fresh home decodes its logo");
    app.screen = Screen::Home(Box::default());
    assert!(!app.animating(), "a home that never started is settled");
}

#[test]
fn images_load_around_the_current_card_and_drop_behind_it() {
    use ratatui_image::picker::Picker;

    let cards: Vec<Card> = (0..6).map(|i| card(&i.to_string())).collect();
    let ids: Vec<Uuid> = cards.iter().map(|c| c.scryfall_data.id).collect();
    let (mut event_loop, mut app, deck_id) = swiping(Vec::new());
    app.images = Some(Picker::halfblocks());
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
    let requested = |event_loop: &EventLoop| -> Vec<Uuid> {
        event_loop
            .pending
            .iter()
            .filter_map(|r| match r {
                Request::Image { card_id, url } => {
                    assert!(images::allowed(url), "{url}");
                    Some(*card_id)
                }
                _ => None,
            })
            .collect()
    };
    assert_eq!(
        requested(&event_loop),
        ids[..4],
        "the card on screen and three ahead"
    );

    event_loop.pending.clear();
    for _ in 0..3 {
        event_loop.handle_key(&mut app, key(KeyCode::Char('h')));
    }
    assert_eq!(requested(&event_loop), ids[4..6]);
    let held: HashSet<Uuid> = swipe_view(&app).images.keys().copied().collect();
    assert_eq!(
        held,
        ids[2..6].iter().copied().collect(),
        "one behind, the rest dropped"
    );

    let image = DynamicImage::new_rgb8(8, 11);
    event_loop.apply(
        &mut app,
        Update::Image {
            card_id: ids[3],
            image: Some(Box::new(image)),
        },
    );
    event_loop.apply(
        &mut app,
        Update::Image {
            card_id: ids[4],
            image: None,
        },
    );
    let view = swipe_view(&app);
    assert!(matches!(
        view.images.get(&ids[3]),
        Some(CardImage::Ready(_))
    ));
    assert!(matches!(view.images.get(&ids[4]), Some(CardImage::Failed)));
}

#[test]
fn without_images_no_image_is_requested_and_i_still_toggles() {
    let (mut event_loop, mut app, _) = swiping(vec![card("A")]);
    event_loop.handle_key(&mut app, key(KeyCode::Char('h')));
    assert!(
        !event_loop
            .pending
            .iter()
            .any(|r| matches!(r, Request::Image { .. }))
    );
    event_loop.handle_key(&mut app, key(KeyCode::Char('i')));
    assert!(swipe_view(&app).show_image);
}

fn test_user(verified: bool) -> zwipe_core::domain::user::User {
    let mut user: zwipe_core::domain::user::User = serde_json::from_value(serde_json::json!({
        "id": "00000000-0000-0000-0000-000000000001",
        "username": "scotty",
        "email": "scotty@zwipe.net",
        "email_verified_at": null,
        "hints_shown": {},
    }))
    .unwrap();
    if verified {
        user.email_verified_at = Some(chrono::Utc::now());
    }
    user
}

fn on_account(verified: bool) -> (EventLoop, App) {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
    app.user = Some(test_user(verified));
    event_loop.open_home(&mut app);
    event_loop.pending.clear();
    event_loop.handle_key(&mut app, key(KeyCode::Char('a')));
    (event_loop, app)
}

fn account_view(app: &App) -> &crate::inbound::app::AccountView {
    match &app.screen {
        Screen::Account(view) => view,
        other => panic!("not on the account screen: {other:?}"),
    }
}

fn config_view(app: &App) -> &crate::inbound::app::ConfigView {
    match &app.screen {
        Screen::Config(view) => view,
        other => panic!("not on the config screen: {other:?}"),
    }
}

#[test]
fn a_opens_the_account_and_refreshes_the_user() {
    let (event_loop, app) = on_account(true);
    assert_eq!(account_view(&app).user.username.to_string(), "scotty");
    assert_eq!(
        event_loop.pending,
        [Request::Settings(SettingsRequest::LoadUser)]
    );
}

#[test]
fn a_bad_username_is_refused_in_the_form_and_a_good_one_is_sent() {
    use crate::inbound::app::Dialog;

    let (mut event_loop, mut app) = on_account(true);
    event_loop.pending.clear();
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    type_text(&mut event_loop, &mut app, "ab");
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    type_text(&mut event_loop, &mut app, "hunter2");
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert!(matches!(
        &account_view(&app).dialog,
        Some(Dialog::Form(form)) if form.error.as_deref() == Some("must be at least 3 characters long")
    ));
    assert_eq!(event_loop.pending, []);

    event_loop.handle_key(&mut app, key(KeyCode::Tab));
    type_text(&mut event_loop, &mut app, "cdef");
    event_loop.handle_key(&mut app, key(KeyCode::Tab));
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert_eq!(
        event_loop.pending,
        [Request::Settings(SettingsRequest::ChangeAccount(
            settings::AccountChange::Username {
                username: "abcdef".to_owned(),
                password: Password("hunter2".to_owned()),
            }
        ))]
    );
    assert!(!format!("{:?}", event_loop.pending).contains("hunter2"));

    event_loop.apply(
        &mut app,
        Update::AccountChangeFailed("invalid password".to_owned()),
    );
    assert!(matches!(
        &account_view(&app).dialog,
        Some(Dialog::Form(form)) if !form.busy && form.error.as_deref() == Some("invalid password")
    ));

    let mut renamed = test_user(true);
    renamed.username = zwipe_core::domain::user::Username::new("abcdef").unwrap();
    event_loop.apply(
        &mut app,
        Update::AccountChanged {
            user: Some(Box::new(renamed)),
            message: "Username changed to abcdef".to_owned(),
        },
    );
    assert!(account_view(&app).dialog.is_none());
    assert_eq!(app.username.as_deref(), Some("abcdef"));
    assert_eq!(app.status, "Username changed to abcdef");
}

#[test]
fn a_password_change_asks_before_it_signs_out_other_devices() {
    use crate::inbound::app::Dialog;

    let (mut event_loop, mut app) = on_account(true);
    event_loop.pending.clear();
    for _ in 0..2 {
        event_loop.handle_key(&mut app, key(KeyCode::Char('j')));
    }
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    for value in ["Old!pass1", "New!pass22", "New!pass22"] {
        type_text(&mut event_loop, &mut app, value);
        event_loop.handle_key(&mut app, key(KeyCode::Enter));
    }
    assert!(matches!(
        account_view(&app).dialog,
        Some(Dialog::ConfirmPassword(_))
    ));
    assert_eq!(event_loop.pending, []);
    event_loop.handle_key(&mut app, key(KeyCode::Char('y')));
    assert!(matches!(
        event_loop.pending.as_slice(),
        [Request::Settings(SettingsRequest::ChangeAccount(
            settings::AccountChange::Password { .. }
        ))]
    ));
}

#[test]
fn delete_waits_five_seconds_and_log_out_asks_first() {
    use crate::inbound::app::Dialog;

    let (mut event_loop, mut app) = on_account(true);
    event_loop.pending.clear();
    for _ in 0..10 {
        event_loop.handle_key(&mut app, key(KeyCode::Char('j')));
    }
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    type_text(&mut event_loop, &mut app, "pw");
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert!(matches!(&account_view(&app).dialog, Some(Dialog::Form(form)) if !form.busy));
    assert_eq!(
        event_loop.pending,
        [],
        "the delete button is still counting down"
    );
    assert!(app.animating(), "the countdown redraws");

    event_loop.handle_key(&mut app, key(KeyCode::Esc));
    event_loop.handle_key(&mut app, key(KeyCode::Char('k')));
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert!(matches!(account_view(&app).dialog, Some(Dialog::LogOut)));
    event_loop.handle_key(&mut app, key(KeyCode::Char('y')));
    assert_eq!(event_loop.pending, [Request::SignOut]);
}

#[test]
fn resend_waits_a_minute_between_emails_unless_it_failed() {
    let (mut event_loop, mut app) = on_account(false);
    event_loop.pending.clear();
    for _ in 0..2 {
        event_loop.handle_key(&mut app, key(KeyCode::Char('j')));
    }
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert_eq!(
        event_loop.pending,
        [Request::Settings(SettingsRequest::Resend)]
    );
    assert!(account_view(&app).resend_wait().is_some());

    event_loop.apply(&mut app, Update::ResendFailed("offline".to_owned()));
    assert!(account_view(&app).resend_wait().is_none());
    assert_eq!(app.status, "offline");
}

fn on_config() -> (EventLoop, App) {
    let mut event_loop = event_loop();
    let mut app = App::new(Palette::terminal());
    event_loop.open_home(&mut app);
    event_loop.handle_key(&mut app, key(KeyCode::Char('c')));
    assert_eq!(
        event_loop.pending.last(),
        Some(&Request::Settings(SettingsRequest::LoadPreferences))
    );
    event_loop.apply(&mut app, Update::Preferences(Box::default()));
    event_loop.pending.clear();
    (event_loop, app)
}

#[test]
fn loaded_preferences_set_the_theme_and_fill_the_screen() {
    let (_, app) = on_config();
    assert!(config_view(&app).shown.is_some());
    assert_eq!(app.palette, Palette::for_account("gruvbox", true));
}

#[test]
fn a_toggle_shows_at_once_and_a_failure_puts_it_back() {
    use zwipe_core::domain::user::preferences::PipStyle;

    let (mut event_loop, mut app) = on_config();
    for _ in 0..2 {
        event_loop.handle_key(&mut app, key(KeyCode::Char('j')));
    }
    let before = app.palette;
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert_eq!(
        config_view(&app).shown.as_ref().map(|p| p.pip_style),
        Some(PipStyle::Natural)
    );
    assert_ne!(
        app.palette, before,
        "natural pips change the mana colors at once"
    );
    assert!(matches!(
        event_loop.pending.as_slice(),
        [Request::Settings(SettingsRequest::SavePreferences {
            seq: 1,
            ..
        })]
    ));

    event_loop.apply(
        &mut app,
        Update::PreferencesFailed {
            seq: 1,
            message: "offline".to_owned(),
        },
    );
    assert_eq!(app.palette, before);
    assert_eq!(app.status, "offline");
}

#[test]
fn the_theme_list_previews_and_esc_puts_the_theme_back() {
    let (mut event_loop, mut app) = on_config();
    let before = app.palette;
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert!(config_view(&app).picker.is_some());
    event_loop.handle_key(&mut app, key(KeyCode::Char('j')));
    assert_ne!(app.palette, before, "the next theme previews");
    event_loop.handle_key(&mut app, key(KeyCode::Esc));
    assert_eq!(app.palette, before);
    assert_eq!(event_loop.pending, []);
    assert_eq!(app.status, "Theme unchanged");

    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    event_loop.handle_key(&mut app, key(KeyCode::Char('j')));
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert!(config_view(&app).picker.is_none());
    assert!(matches!(
        event_loop.pending.as_slice(),
        [Request::Settings(SettingsRequest::SavePreferences {
            change: crate::inbound::app::PreferenceChange::Theme { .. },
            ..
        })]
    ));
}

#[test]
fn exceptions_stay_shut_while_universes_beyond_is_shown() {
    let (mut event_loop, mut app) = on_config();
    for _ in 0..4 {
        event_loop.handle_key(&mut app, key(KeyCode::Char('j')));
    }
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert!(config_view(&app).exceptions.is_none());
    assert_eq!(event_loop.pending, []);

    event_loop.handle_key(&mut app, key(KeyCode::Char('k')));
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    event_loop.handle_key(&mut app, key(KeyCode::Char('j')));
    event_loop.pending.clear();
    event_loop.handle_key(&mut app, key(KeyCode::Enter));
    assert!(config_view(&app).exceptions.is_some());
    assert_eq!(
        event_loop.pending,
        [Request::Settings(SettingsRequest::LoadFranchises)]
    );
}
