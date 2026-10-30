use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, render, screen_text};

fn ask(request: Request) -> screens::Request {
    request.into()
}

fn on_home() -> Harness {
    let mut harness = Harness::new();
    harness.open(open);
    harness
}

fn home(harness: &Harness) -> &HomeView {
    match harness.screen() {
        Screen::Home(view) => view,
        other => panic!("left home: {other:?}"),
    }
}

#[test]
fn home_fetches_its_panels_and_fills_them_as_they_arrive() {
    let mut harness = on_home();
    assert_eq!(
        harness.pending(),
        [
            ask(Request::PublicMetrics),
            ask(Request::PlayStyle),
            ask(Request::Flavor)
        ]
    );
    let figures = Figures {
        decks_created: 1,
        searches: 2,
        cards_swiped: 3,
    };
    harness.apply(Update::PublicMetrics(figures));
    harness.apply(Update::PlayStyleFailed);
    harness.apply(Update::Flavor(Box::new(card("Flavor"))));
    let home = home(&harness);
    assert_eq!(home.figures(), Some(figures));
    assert!(home.play_style_failed);
    assert!(home.flavor.is_some());
}

#[test]
fn s_on_home_switches_to_my_counters_and_fetches_them() {
    let mut harness = on_home();
    harness.clear();
    harness.press(KeyCode::Char('s'));
    let view = home(&harness);
    assert!(view.scope == Scope::OnlyMe && view.figures().is_none());
    assert_eq!(harness.pending(), [ask(Request::MyMetrics)]);

    harness.clear();
    harness.press(KeyCode::Char('s'));
    assert_eq!(home(&harness).scope, Scope::Everybody);
    assert_eq!(harness.pending(), []);
}

#[test]
fn d_on_home_opens_the_decks() {
    let mut harness = on_home();
    harness.clear();
    harness.press(KeyCode::Char('d'));
    assert!(matches!(harness.screen(), Screen::Decks(_)));
    assert_eq!(
        harness.pending(),
        [screens::Request::Decks(screens::decks::Request::Load)]
    );
}

#[test]
fn home_animates_only_while_the_logo_decodes() {
    let mut harness = Harness::new();
    assert!(!harness.app.animating(), "the login screen is still");
    harness.open(open);
    assert!(harness.app.animating(), "a fresh home decodes its logo");
    harness.app.nav.reset(Screen::Home(Box::default()));
    assert!(
        !harness.app.animating(),
        "a home that never started is settled"
    );
}

#[test]
fn coming_back_home_keeps_the_scope_and_fetches_again() {
    let mut harness = on_home();
    harness.press(KeyCode::Char('s'));
    harness.press(KeyCode::Char('c'));
    harness.clear();
    harness.press(KeyCode::Esc);
    assert_eq!(home(&harness).scope, Scope::OnlyMe);
    assert_eq!(
        harness.pending(),
        [
            ask(Request::PublicMetrics),
            ask(Request::PlayStyle),
            ask(Request::Flavor),
            ask(Request::MyMetrics)
        ]
    );
}

fn color_map() -> HttpPlayStyleSummary {
    serde_json::from_value(serde_json::json!({
        "deck_count": 2,
        "color_map": {
            "colors": [{"color": "G", "cards": 30}, {"color": "U", "cards": 12}],
            "colorless_cards": 4,
            "pairs": [{"colors": ["U", "G"], "decks": 2}],
        },
    }))
    .unwrap()
}

#[test]
fn home_shows_the_counters_color_map_and_flavor() {
    let mut flavor = card("Llanowar Elves");
    flavor.scryfall_data.flavor_text = Some("One bone broken for every twig snapped.".to_owned());
    let mut harness = Harness::on(Screen::Home(Box::new(HomeView {
        everybody: Some(Figures {
            decks_created: 1_234,
            searches: 56_789,
            cards_swiped: 3,
        }),
        play_style: Some(color_map()),
        flavor: Some(flavor),
        ..HomeView::default()
    })));
    harness.app.shared.username = Some("scotty".to_owned());
    let screen = screen_text(&harness.app);
    for expected in [
        "Home",
        "1,234 Decks created",
        "56,789 Searches run",
        "3 Cards swiped",
        "Everybody",
        "Color map",
        "Most shared: UG 2",
        "Featured flavor",
        "One bone broken",
        "Llanowar Elves",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn home_without_decks_asks_for_one_and_hides_a_card_without_flavor() {
    let mut empty = color_map();
    empty.deck_count = 0;
    let harness = Harness::on(Screen::Home(Box::new(HomeView {
        play_style: Some(empty),
        flavor: Some(card("No flavor")),
        ..HomeView::default()
    })));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Build a deck to see your playstyle"));
    assert!(screen.contains("Counting..."));
    assert!(!screen.contains("Featured flavor"));
}

#[test]
fn home_fits_a_tiny_terminal() {
    let harness = Harness::on(Screen::Home(Box::new(HomeView {
        play_style: Some(color_map()),
        ..HomeView::default()
    })));
    render(&harness.app, 12, 5);
    render(&harness.app, 40, 12);
}
