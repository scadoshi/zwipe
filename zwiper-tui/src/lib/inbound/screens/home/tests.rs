use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use zwipe_core::domain::user::models::hints::HINT_FIRST_LOGIN;
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;
use zwipe_ui::color_map::{ColorMapCounts, ColorMapModel};

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::image_preview::ImagePreview;
use crate::inbound::widgets::hero::Counts;
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
        decks_created: Some(n),
        searches: Some(n),
        cards_swiped: Some(n),
    }
}

/// Everybody's counts arriving, each `n`.
fn counts(n: i64) -> screens::Update {
    screens::Update::Counts(Box::new(HttpPublicMetrics {
        cards_swiped: n,
        searches: n,
        decks_created: n,
    }))
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
            screens::Request::FetchCounts,
            ask(Request::PlayStyle),
            ask(Request::Flavor),
            screens::oracle_tags::Request::Catalog.into(),
        ]
    );
}

#[test]
fn a_return_home_skips_fresh_counters_and_an_unexpired_card() {
    let mut harness = on_home();
    harness.apply(counts(5));
    harness.apply(Update::Flavor(Box::new(FlavorCard {
        card: flavored("Elves"),
        expires_at: Utc::now() + chrono::Duration::minutes(30),
    })));
    harness.press(KeyCode::Char('c'));
    // Config's own one-time hint opens over it; close it so Esc goes back.
    harness.app.shared.hint = None;
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
    harness.app.shared.counts = Counts::restored(Some(HttpPublicMetrics {
        cards_swiped: 3,
        searches: 2,
        decks_created: 1,
    }));
    let restored = Figures {
        decks_created: Some(1),
        searches: Some(2),
        cards_swiped: Some(3),
    };
    assert_eq!(figures_for(&harness.app.shared), Some(restored));
    assert!(!live(&harness.app.shared));
    harness.apply(counts(9));
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
    assert_eq!(hint.content.title, "Welcome to Zwipe");
    assert!(hint::text(&hint.content).contains("Verify your email in Account"));
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
    harness.apply(screens::Update::Counts(Box::new(HttpPublicMetrics {
        decks_created: 28_049,
        searches: 12,
        cards_swiped: 1_234,
    })));
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
        "W 0  U 12  B 0  R 0  G 30",
        "C Colorless 4",
        "p Playstyle",
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
    // Config's own one-time hint opens over it; close it so Esc goes back.
    harness.app.shared.hint = None;
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
        .any(|c| c.symbol() == "█" && c.fg == logo);
    assert!(drawn);
}

#[test]
fn home_and_login_share_one_copy_of_everybodys_counts() {
    let mut harness = Harness::new();
    harness.app.shared.user = Some(user(true));
    harness.apply(counts(7));
    harness.open(open);
    assert!(
        !harness.pending().contains(&screens::Request::FetchCounts),
        "the copy the sign-in hero fetched is still fresh"
    );
    assert_eq!(figures_for(&harness.app.shared), Some(figures(7)));
    assert!(live(&harness.app.shared));
    harness.apply(counts(8));
    assert_eq!(
        harness
            .app
            .shared
            .counts
            .metrics
            .as_ref()
            .map(|m| m.searches),
        Some(8)
    );
    assert_eq!(figures_for(&harness.app.shared), Some(figures(8)));
}

#[test]
fn the_color_maps_playstyle_and_the_heros_activity_open_their_screens() {
    let mut harness = on_home();
    harness.apply(Update::PlayStyle {
        user_id: me(&harness),
        summary: Box::new(summary(2)),
    });
    harness.press(KeyCode::Char('p'));
    assert!(matches!(harness.screen(), Screen::Playstyle(_)));
    harness.app.shared.hint = None;
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
    harness.press(KeyCode::Char('v'));
    assert!(matches!(harness.screen(), Screen::Activity(_)));
}

#[test]
fn home_keeps_every_key_on_a_narrow_terminal() {
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
    if let Some(flavor) = &mut harness.app.shared.caches.flavor {
        flavor.card.scryfall_data.flavor_text = Some(
            "\"That drill shall not reach Ba Sing Se! This is what we trained for!\"".to_owned(),
        );
    }
    settle_entrance(&mut harness);
    crate::test_support::assert_key_bar_fits(&harness.app, "q quit");
    for (width, height) in crate::test_support::NARROW {
        let screen = crate::test_support::rows(&harness.app, width, height).join("\n");
        assert!(
            screen.contains("i Image  b Buy"),
            "a long quote keeps the flavor keys in the panel at {width}x{height}:\n{screen}"
        );
    }
    harness.press(KeyCode::Char('!'));
    for (width, height) in crate::test_support::NARROW {
        let screen = crate::test_support::rows(&harness.app, width, height).join("\n");
        assert!(
            screen.contains("Join the Discord"),
            "help fits at {width}x{height}:\n{screen}"
        );
    }
}

/// Home with everybody's counts, a five-color map as big as the live account's, and a featured card, all landed.
fn settled_home() -> Harness {
    let mut harness = on_home();
    let user_id = me(&harness);
    harness.apply(counts(261_100));
    let big: HttpPlayStyleSummary = serde_json::from_value(serde_json::json!({
        "deck_count": 10,
        "color_map": {
            "colors": [
                {"color": "W", "cards": 108},
                {"color": "U", "cards": 76},
                {"color": "B", "cards": 43},
                {"color": "R", "cards": 195},
            ],
            "colorless_cards": 140,
            "pairs": [],
        },
    }))
    .unwrap();
    harness.apply(Update::PlayStyle {
        user_id,
        summary: Box::new(big),
    });
    harness.app.shared.caches.flavor = Some(FlavorCard {
        card: flavored("Fast Forward"),
        expires_at: Utc::now(),
    });
    if let Screen::Home(view) = harness.screen_mut() {
        let long_ago = Instant::now().checked_sub(Duration::from_secs(5)).unwrap();
        view.figures_arrived = Some(long_ago);
        view.map_started = Some(long_ago);
        view.logo_started = None;
        view.opened = long_ago;
    }
    harness.app.shared.hint = None;
    harness
}

#[test]
fn each_figure_sits_in_its_own_tile_until_the_terminal_is_short() {
    let harness = settled_home();
    for (width, height) in [(200, 50), (120, 40), (80, 24), (60, 20)] {
        let rows = crate::test_support::rows(&harness.app, width, height);
        let text = rows.join("\n");
        for label in ["Decks created", "Searches run", "Cards swiped"] {
            let row = rows.iter().position(|r| r.contains(label)).unwrap();
            let above = &rows[row - 1];
            assert!(
                above.contains("261.1k"),
                "the number over {label} at {width}x{height}:\n{text}"
            );
        }
        let tiles = rows
            .iter()
            .find(|r| r.matches('╭').count() >= 3)
            .map(|r| r.matches('╭').count());
        let want = if height >= 22 { Some(3) } else { None };
        assert_eq!(tiles, want, "tiles at {width}x{height}:\n{text}");
        assert!(text.contains("Everybody") && text.contains("v Activity"));
    }
}

#[test]
fn a_short_panel_draws_the_whole_map_at_half_scale_with_its_key() {
    let harness = settled_home();
    let model = ColorMapModel::new(
        &ColorMapCounts {
            cards: [108, 76, 43, 195, 0],
            colorless: 140,
        },
        super::seed(&harness.app.shared),
    );
    for (width, height) in [(200, 50), (120, 40), (80, 24), (60, 20)] {
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let (mut full, mut half) = (0, 0);
        let logo = harness
            .app
            .shared
            .palette
            .logo
            .fg
            .unwrap_or(ratatui::style::Color::Reset);
        for cell in buffer.content() {
            match cell.symbol() {
                "█" if cell.fg != logo => full += 1,
                "▀" if cell.bg != ratatui::style::Color::Reset => half += 2,
                "▀" | "▄" => half += 1,
                _ => {}
            }
        }
        let text = crate::test_support::rows(&harness.app, width, height).join("\n");
        assert!(
            text.contains("C Colorless 140"),
            "the key at {width}x{height}:\n{text}"
        );
        if height >= 40 {
            assert_eq!(
                (full / 2, half),
                (model.tiles.len(), 0),
                "full at {width}x{height}:\n{text}"
            );
        } else if height >= 24 {
            assert_eq!(
                (full, half),
                (0, model.tiles.len()),
                "whole at {width}x{height}:\n{text}"
            );
        } else {
            assert!(full == 0 && half > 0, "half at {width}x{height}:\n{text}");
        }
    }
}

/// A picker that draws with `protocol`.
fn picker(protocol: ratatui_image::picker::ProtocolType) -> ratatui_image::picker::Picker {
    let mut picker = ratatui_image::picker::Picker::halfblocks();
    picker.set_protocol_type(protocol);
    picker
}

/// The featured card's normal image.
fn flavor_url(card: &Card) -> String {
    card.scryfall_data
        .primary_image_url(zwipe_core::domain::card::scryfall_data::ImageSize::Normal)
        .unwrap()
        .to_owned()
}

/// Home opened over a cached featured card, drawing images with `images`.
fn home_with_flavor(images: Option<ratatui_image::picker::Picker>, card: Card) -> Harness {
    let mut harness = Harness::new();
    harness.app.shared.user = Some(user(true));
    harness.app.shared.images = images;
    harness.app.shared.caches.flavor = Some(FlavorCard {
        card,
        expires_at: Utc::now()
            .checked_add_signed(chrono::Duration::hours(1))
            .unwrap(),
    });
    harness.open(open);
    harness
}

#[test]
fn the_featured_image_loads_only_where_the_terminal_draws_real_images() {
    use ratatui_image::picker::ProtocolType;

    let elves = flavored("Llanowar Elves");
    let url = flavor_url(&elves);
    let wants = |harness: &Harness| {
        harness
            .pending()
            .contains(&ask(Request::FlavorImage { url: url.clone() }))
    };
    assert!(!wants(&home_with_flavor(None, elves.clone())), "images off");
    let halfblocks = home_with_flavor(Some(picker(ProtocolType::Halfblocks)), elves.clone());
    assert!(!wants(&halfblocks), "half blocks are too coarse");
    assert!(halfblocks.app.shared.caches.flavor_image.is_none());

    for protocol in [
        ProtocolType::Kitty,
        ProtocolType::Sixel,
        ProtocolType::Iterm2,
    ] {
        assert!(
            wants(&home_with_flavor(Some(picker(protocol)), elves.clone())),
            "{protocol:?}"
        );
    }
    let mut harness = home_with_flavor(Some(picker(ProtocolType::Kitty)), elves);
    harness.apply(Update::FlavorImage {
        url: "https://cards.scryfall.io/normal/front/other.jpg".to_owned(),
        image: Some(Box::new(image::DynamicImage::new_rgb8(8, 11))),
    });
    assert!(
        matches!(
            harness
                .app
                .shared
                .caches
                .flavor_image
                .as_ref()
                .unwrap()
                .image,
            CardImage::Loading
        ),
        "another card's image is dropped"
    );
    harness.apply(Update::FlavorImage {
        url: url.clone(),
        image: Some(Box::new(image::DynamicImage::new_rgb8(8, 11))),
    });
    assert!(matches!(
        harness
            .app
            .shared
            .caches
            .flavor_image
            .as_ref()
            .unwrap()
            .image,
        CardImage::Ready(_)
    ));
    harness.clear();
    harness.open(open);
    assert!(!wants(&harness), "a held image is not fetched again");

    let mut bare = flavored("Blank Card");
    bare.scryfall_data.image_uris = None;
    harness.apply(Update::Flavor(Box::new(FlavorCard {
        card: bare,
        expires_at: Utc::now(),
    })));
    assert!(
        harness.app.shared.caches.flavor_image.is_none(),
        "a card with no image drops the old one and asks for none"
    );
    harness.apply(Update::FlavorImage {
        url: url.clone(),
        image: None,
    });
    assert!(harness.app.shared.caches.flavor_image.is_none());
}

/// Whether an image protocol drew into the screen: kitty's placeholder cells or a sixel or iTerm2 escape.
fn image_drawn(harness: &Harness, width: u16, height: u16) -> bool {
    render(&harness.app, width, height)
        .backend()
        .buffer()
        .content()
        .iter()
        .any(|c| c.symbol().contains('\u{10eeee}') || c.symbol().contains('\u{1b}'))
}

#[test]
fn the_featured_card_shows_its_image_beside_or_above_the_quote_when_there_is_room() {
    use std::cell::RefCell;

    use ratatui_image::picker::ProtocolType;

    let mut harness = settled_home();
    let kitty = picker(ProtocolType::Kitty);
    let card = harness
        .app
        .shared
        .caches
        .flavor
        .as_ref()
        .unwrap()
        .card
        .clone();
    let red = image::RgbImage::from_pixel(40, 56, image::Rgb([255, 0, 0]));
    harness.app.shared.caches.flavor_image = Some(FlavorImage {
        url: flavor_url(&card),
        image: CardImage::Ready(Box::new(RefCell::new(
            kitty.new_resize_protocol(image::DynamicImage::ImageRgb8(red)),
        ))),
    });
    harness.app.shared.images = Some(kitty);
    let quote = "One bone broken for every twig snapped.";
    let quote_at = |harness: &Harness, width: u16, height: u16| {
        crate::test_support::rows(&harness.app, width, height)
            .iter()
            .enumerate()
            .find_map(|(y, row)| {
                row.find(quote)
                    .map(|x| (row.get(..x).unwrap().chars().count(), y))
            })
    };
    let panel_left = |harness: &Harness, width: u16, height: u16| {
        let rows = crate::test_support::rows(&harness.app, width, height);
        rows.iter()
            .find_map(|row| row.find("Featured flavor").map(|_| row))
            .and_then(|row| {
                row.rfind('╭')
                    .map(|x| row.get(..x).unwrap().chars().count())
            })
            .unwrap()
    };

    // Wide and tall enough: the image on the left, the quote beside it.
    assert!(image_drawn(&harness, 200, 50));
    let (x, _) = quote_at(&harness, 200, 50).expect("the quote stays");
    assert!(
        x > panel_left(&harness, 200, 50) + 30,
        "the quote sits right of the image, at {x}"
    );
    assert!(
        crate::test_support::rows(&harness.app, 200, 50)
            .join("\n")
            .contains("b Buy")
    );

    // Too small: today's text alone.
    for (width, height) in [(120, 40), (80, 24), (60, 20)] {
        assert!(
            !image_drawn(&harness, width, height),
            "no room at {width}x{height}"
        );
        let text = crate::test_support::rows(&harness.app, width, height).join("\n");
        assert!(
            text.contains("i Image"),
            "the text at {width}x{height}:\n{text}"
        );
    }

    // Tall and narrow: the image on top, the quote under it.
    assert!(image_drawn(&harness, 60, 100));
    let (_, y) = quote_at(&harness, 60, 100).expect("the quote stays");
    let top = crate::test_support::rows(&harness.app, 60, 100)
        .iter()
        .position(|r| r.contains("Featured flavor"))
        .unwrap();
    assert!(y > top + 18, "the quote sits under the image, on row {y}");

    // Half blocks keep the text alone, and a sixel image waits under a popup.
    harness.app.shared.images = Some(picker(ProtocolType::Halfblocks));
    assert!(
        quote_at(&harness, 200, 50).is_some_and(|(x, _)| x < panel_left(&harness, 200, 50) + 30)
    );
    harness.app.shared.images = Some(picker(ProtocolType::Sixel));
    harness.press(KeyCode::Char('?'));
    assert!(
        !image_drawn(&harness, 200, 50),
        "the sixel waits under the popup"
    );
}

#[test]
fn the_color_maps_about_marker_is_a_button_in_its_corner() {
    let mut harness = on_home();
    let palette = crate::domain::theme::Palette::for_account("gruvbox", true);
    harness.app.shared.palette = palette;
    let user_id = me(&harness);
    harness.apply(Update::PlayStyle {
        user_id,
        summary: Box::new(summary(2)),
    });
    settle_entrance(&mut harness);
    for (width, height) in crate::test_support::BUTTON_SIZES {
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let found = crate::test_support::find_all(buffer, " [?] ");
        let &(x, y) = found
            .first()
            .unwrap_or_else(|| panic!("no corner [?] at {width}x{height}"));
        let context = format!("{width}x{height}");
        assert!(
            matches!(buffer[(x + 5, y)].symbol(), "╮" | "─"),
            "{context}: on the panel's top border"
        );
        crate::test_support::assert_about_marker(buffer, &palette, (x + 1, y), false, &context);
    }
}
