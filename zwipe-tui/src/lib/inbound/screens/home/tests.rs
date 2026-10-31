use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use zwipe_core::domain::user::models::hints::HINT_FIRST_LOGIN;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::image_preview::ImagePreview;
use crate::test_support::{card, render, screen_text, user};

fn ask(request: Request) -> screens::Request {
    request.into()
}

/// Home just after sign-in, with a user who has seen the welcome hint.
fn on_home() -> Harness {
    let mut harness = Harness::new();
    harness.app.shared.user = Some(user(true));
    harness.open(open);
    harness
}

fn home(harness: &Harness) -> &HomeView {
    match harness.screen() {
        Screen::Home(view) => view,
        other => panic!("left home: {other:?}"),
    }
}

fn me(harness: &Harness) -> Uuid {
    harness.app.shared.user.as_ref().unwrap().id
}

fn figures(n: u64) -> Figures {
    Figures {
        decks_created: n,
        searches: n,
        cards_swiped: n,
    }
}

fn summary(deck_count: u32) -> HttpPlayStyleSummary {
    serde_json::from_value(serde_json::json!({
        "deck_count": deck_count,
        "color_map": {
            "colors": [{"color": "G", "cards": 30}, {"color": "U", "cards": 12}],
            "colorless_cards": 4,
            "pairs": [],
        },
    }))
    .unwrap()
}

fn flavored(name: &str) -> Card {
    let mut card = card(name);
    card.scryfall_data.flavor_text = Some("One bone broken for every twig snapped.".to_owned());
    card
}

#[test]
fn the_first_home_restores_then_fetches_every_panel() {
    let harness = on_home();
    assert_eq!(
        harness.pending(),
        [
            ask(Request::Restore),
            ask(Request::User),
            ask(Request::Preferences),
            ask(Request::PublicMetrics),
            ask(Request::PlayStyle),
            ask(Request::Flavor),
            screens::oracle_tags::Request::Catalog.into(),
        ]
    );
}

#[test]
fn a_return_home_skips_fresh_counters_and_an_unexpired_card() {
    let mut harness = on_home();
    harness.apply(Update::PublicMetrics(figures(5)));
    harness.apply(Update::Flavor(Box::new(FlavorCard {
        card: flavored("Elves"),
        expires_at: Utc::now() + chrono::Duration::minutes(30),
    })));
    harness.press(KeyCode::Char('c'));
    harness.clear();
    harness.press(KeyCode::Esc);
    assert_eq!(
        harness.pending(),
        [
            ask(Request::User),
            ask(Request::Preferences),
            ask(Request::PlayStyle)
        ]
    );
}

#[test]
fn a_restored_copy_shows_but_is_not_live() {
    let mut harness = on_home();
    harness.apply(Update::Restored {
        counts: Some(HttpPublicMetrics {
            cards_swiped: 3,
            searches: 2,
            decks_created: 1,
        }),
        scope: Scope::Everybody,
    });
    let restored = Figures {
        decks_created: 1,
        searches: 2,
        cards_swiped: 3,
    };
    assert_eq!(figures_for(&harness.app.shared), Some(restored));
    assert!(!live(&harness.app.shared));
    harness.apply(Update::PublicMetrics(figures(9)));
    assert!(live(&harness.app.shared));
    assert!(home(&harness).figures_arrived.is_some());
}

#[test]
fn s_switches_to_my_counters_saves_the_scope_and_fetches_them() {
    let mut harness = on_home();
    harness.clear();
    harness.press(KeyCode::Char('s'));
    assert_eq!(harness.app.shared.caches.scope, Scope::OnlyMe);
    assert!(figures_for(&harness.app.shared).is_none());
    assert_eq!(
        harness.pending(),
        [
            ask(Request::SaveScope(Scope::OnlyMe)),
            ask(Request::MyMetrics)
        ]
    );
    let user_id = me(&harness);
    harness.apply(Update::MyMetrics {
        user_id,
        figures: figures(7),
    });
    assert_eq!(figures_for(&harness.app.shared), Some(figures(7)));
    assert!(live(&harness.app.shared));

    harness.clear();
    harness.press(KeyCode::Char('s'));
    assert_eq!(
        harness.pending(),
        [ask(Request::SaveScope(Scope::Everybody))]
    );
}

#[test]
fn another_users_counts_and_map_never_show() {
    let mut harness = on_home();
    harness.press(KeyCode::Char('s'));
    harness.apply(Update::MyMetrics {
        user_id: Uuid::new_v4(),
        figures: figures(7),
    });
    harness.apply(Update::PlayStyle {
        user_id: Uuid::new_v4(),
        summary: Box::new(summary(2)),
    });
    assert!(figures_for(&harness.app.shared).is_none());
    assert!(play_style_for(&harness.app.shared).is_none());
}

#[test]
fn an_unverified_user_is_told_to_verify_and_not_greeted() {
    let mut harness = on_home();
    harness.apply(Update::User(Box::new(user(false))));
    let toasts: Vec<&str> = harness
        .app
        .shared
        .toasts
        .items()
        .iter()
        .map(|t| t.message.as_str())
        .collect();
    assert_eq!(toasts, ["Verify your email!"]);
    assert!(!harness.app.shared.caches.greeted);
}

#[test]
fn a_verified_user_is_greeted_once_per_launch() {
    let mut harness = on_home();
    harness.apply(Update::User(Box::new(user(true))));
    harness.apply(Update::User(Box::new(user(true))));
    harness.apply(Update::UserFailed);
    let toasts: Vec<&str> = harness
        .app
        .shared
        .toasts
        .items()
        .iter()
        .map(|t| t.message.as_str())
        .collect();
    assert_eq!(toasts, ["Hello, scotty!"]);
}

#[test]
fn a_failed_refresh_greets_from_the_cached_user() {
    let mut harness = on_home();
    harness.apply(Update::UserFailed);
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "Hello, scotty!"
    );
}

#[test]
fn the_welcome_hint_shows_on_the_first_home() {
    let mut harness = Harness::new();
    let mut newcomer = user(true);
    newcomer.hints_shown.clear();
    harness.app.shared.user = Some(newcomer);
    harness.open(open);
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(hint.title, "Welcome to Zwipe");
    assert!(hint.body.contains("Verify your email in Account"));
    assert!(harness.pending().contains(&screens::Request::MarkHintShown(
        HINT_FIRST_LOGIN.to_owned()
    )));
}

#[test]
fn d_a_and_c_open_their_screens() {
    let mut harness = on_home();
    harness.press(KeyCode::Char('d'));
    assert!(matches!(harness.screen(), Screen::Decks(_)));
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('a'));
    assert!(matches!(harness.screen(), Screen::Account(_)));
}

#[test]
fn question_mark_explains_the_color_map() {
    let mut harness = on_home();
    harness.press(KeyCode::Char('?'));
    assert!(screen_text(&harness.app).contains("The shape is generated per player"));
    harness.press(KeyCode::Esc);
    assert!(home(&harness).overlay.is_none());
    assert!(
        matches!(harness.screen(), Screen::Home(_)),
        "Esc closed only the popup"
    );
}

#[test]
fn buy_opens_only_for_a_priced_card_and_close_dismisses() {
    let mut harness = on_home();
    let mut priced = flavored("Llanowar Elves");
    harness.app.shared.caches.flavor = Some(FlavorCard {
        card: priced.clone(),
        expires_at: Utc::now(),
    });
    priced.scryfall_data.prices.usd = None;
    priced.scryfall_data.prices.usd_foil = None;
    priced.scryfall_data.prices.usd_etched = None;
    let has_price = zwipe_core::domain::deck::deck_metrics::card_price(
        &harness
            .app
            .shared
            .caches
            .flavor
            .as_ref()
            .unwrap()
            .card
            .scryfall_data,
        zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency::Usd,
    )
    .is_some();
    harness.press(KeyCode::Char('b'));
    assert_eq!(home(&harness).overlay.is_some(), has_price);
    if has_price {
        let screen = screen_text(&harness.app);
        assert!(screen.contains("Buy card"));
        assert!(screen.contains("TCGplayer \u{2197}"));
        harness.press(KeyCode::Enter);
        assert!(home(&harness).overlay.is_none());
    }
    harness.app.shared.caches.flavor = Some(FlavorCard {
        card: priced,
        expires_at: Utc::now(),
    });
    harness.press(KeyCode::Char('b'));
    assert!(home(&harness).overlay.is_none(), "no price, no Buy");
}

#[test]
fn i_opens_the_featured_card_full_screen() {
    let mut harness = on_home();
    harness.app.shared.caches.flavor = Some(FlavorCard {
        card: flavored("Llanowar Elves"),
        expires_at: Utc::now(),
    });
    harness.press(KeyCode::Char('i'));
    let Screen::ImagePreview(view) = harness.screen() else {
        panic!("no preview");
    };
    let view: &ImagePreview = view;
    assert_eq!(view.card.scryfall_data.name, "Llanowar Elves");
}

#[test]
fn home_draws_the_strip_map_key_and_flavor() {
    let mut harness = on_home();
    let user_id = me(&harness);
    harness.apply(Update::PublicMetrics(Figures {
        decks_created: 28_049,
        searches: 12,
        cards_swiped: 1_234,
    }));
    harness.apply(Update::PlayStyle {
        user_id,
        summary: Box::new(summary(2)),
    });
    harness.app.shared.caches.flavor = Some(FlavorCard {
        card: flavored("Llanowar Elves"),
        expires_at: Utc::now(),
    });
    if let Screen::Home(view) = harness.screen_mut() {
        let long_ago = Instant::now().checked_sub(Duration::from_secs(5)).unwrap();
        view.figures_arrived = Some(long_ago);
        view.map_started = Some(long_ago);
        view.logo_started = None;
        view.opened = long_ago;
    }
    let screen = screen_text(&harness.app);
    for expected in [
        "28.0k",
        "Decks created",
        "1,234",
        "Live",
        "Everybody",
        "v Activity",
        "Color map",
        "██",
        "W 0  U 12  B 0  R 0  G 30  Colorless 4  p Playstyle",
        "Featured flavor",
        "One bone broken for every twig snapped.",
        "i Image",
        "d Decks",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
    assert!(!home(&harness).animating(), "everything has landed");
}

#[test]
fn a_user_without_decks_is_asked_to_build_one() {
    let mut harness = on_home();
    settle_entrance(&mut harness);
    let user_id = me(&harness);
    harness.apply(Update::PlayStyle {
        user_id,
        summary: Box::new(summary(0)),
    });
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Build a deck to see your playstyle"));
    assert!(screen.contains("p Playstyle"));
}

#[test]
fn a_failed_map_says_so_and_loading_panels_ghost() {
    let mut harness = on_home();
    settle_entrance(&mut harness);
    let loading = screen_text(&harness.app);
    assert!(loading.contains("▒▒") || loading.contains("░"));
    assert!(!loading.contains("Featured flavor"));
    assert!(home(&harness).animating());
    harness.apply(Update::PlayStyleFailed);
    assert!(screen_text(&harness.app).contains("Could not load your playstyle"));
}

#[test]
fn home_fits_a_tiny_terminal() {
    let mut harness = on_home();
    let user_id = me(&harness);
    harness.apply(Update::PlayStyle {
        user_id,
        summary: Box::new(summary(2)),
    });
    harness.app.shared.caches.flavor = Some(FlavorCard {
        card: flavored("Elves"),
        expires_at: Utc::now(),
    });
    render(&harness.app, 12, 5);
    render(&harness.app, 40, 12);
    render(&harness.app, 1, 1);
}

/// Moves home's entrance past its end, keeping the logo and figures as they are.
fn settle_entrance(harness: &mut Harness) {
    if let Screen::Home(view) = harness.screen_mut() {
        view.opened = Instant::now().checked_sub(Duration::from_secs(5)).unwrap();
    }
}

#[test]
fn the_panels_enter_after_the_hero() {
    let harness = on_home();
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Decks created"), "the hero enters at once");
    assert!(!screen.contains("░░░░"), "the panels wait 0.15 s");
}

#[test]
fn a_return_through_the_back_stack_replays_the_entrance() {
    let mut harness = on_home();
    harness.press(KeyCode::Char('c'));
    assert!(matches!(harness.screen(), Screen::Config(_)));
    let _ = harness.app.nav.find_mut(|s| match s {
        Screen::Home(view) => {
            **view = HomeView::default();
            Some(view)
        }
        _ => None,
    });
    harness.press(KeyCode::Esc);
    let view = home(&harness);
    assert!(view.logo_started.is_some(), "the logo decodes again");
    assert!(view.opened.elapsed() < Duration::from_secs(1));
    assert!(view.animating());
}

#[test]
fn switching_scope_plays_the_scope_in() {
    let mut harness = on_home();
    harness.press(KeyCode::Char('s'));
    assert!(home(&harness).scope_at.is_some());
    assert!(home(&harness).animating());
}

#[test]
fn the_logo_draws_in_the_palette_logo_color() {
    let mut harness = on_home();
    harness.app.shared.palette = crate::domain::theme::Palette::for_account("gruvbox", true);
    settle_entrance(&mut harness);
    if let Screen::Home(view) = harness.screen_mut() {
        view.logo_started = None;
    }
    let terminal = render(&harness.app, 100, 45);
    let logo = harness.app.shared.palette.logo.fg.unwrap();
    let drawn = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .any(|c| c.symbol() == "▉" && c.fg == logo);
    assert!(drawn);
}
