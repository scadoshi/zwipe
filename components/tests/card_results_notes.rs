//! The card search's results and the card page's several cards: the heading over the grid and
//! the line when nothing matches sit in the note chip every empty line uses, on the web and in
//! the app, and the cards float under a mouse as the deck page's featured cards do.

use components::CardResultsGrid;
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

const MAGIC: &str = include_str!("../assets/magic.css");
const WEB_RESULTS: &str = include_str!("../../zwiper-web/src/lib/inbound/screens/search.rs");
const WEB_CARD: &str = include_str!("../../zwiper-web/src/lib/inbound/screens/card/mod.rs");
const APP_RESULTS: &str = include_str!("../../zwiper/src/lib/inbound/screens/card/search.rs");
const APP_CARD: &str = include_str!("../../zwiper/src/lib/inbound/screens/card/mod.rs");

/// Renders `app` once.
fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

#[test]
fn the_heading_sits_in_a_note_chip() {
    let html = render(|| {
        rsx! { CardResultsGrid { title: "Cards matching “sonic”".to_owned(), cards: Vec::new(), on_select: |_| {} } }
    });
    assert!(
        html.contains(r#"<span class="chip-note">Cards matching “sonic”</span>"#),
        "the heading is a note chip: {html}"
    );
}

#[test]
fn the_empty_lines_sit_in_a_note_chip_on_the_web_and_in_the_app() {
    for (page, source, message) in [
        ("web results", WEB_RESULTS, "{card_results::NO_MATCH}"),
        ("app results", APP_RESULTS, "{card_results::NO_MATCH}"),
        (
            "web card",
            WEB_CARD,
            "{message.unwrap_or(card_lookup::NOT_FOUND)}",
        ),
        (
            "app card",
            APP_CARD,
            "{message.unwrap_or(card_lookup::NOT_FOUND)}",
        ),
    ] {
        let chip = format!(r#"span {{ class: "chip-note", {message} }}"#);
        assert!(source.contains(&chip), "{page}: the line is a note chip");
    }
}

#[test]
fn the_cards_float_on_hover_as_the_featured_cards_do() {
    let (_, after_hover) = MAGIC
        .split_once("@media (hover: hover) and (pointer: fine) and (prefers-reduced-motion: no-preference) {\n    .floatable.float-on-hover:hover")
        .unwrap_or_else(|| panic!("the featured cards' hover rule"));
    let (selectors, _) = after_hover
        .split_once('{')
        .unwrap_or_else(|| panic!("its block"));
    assert!(
        selectors.contains(".card-list-results .card-stack-card.floatable:hover"),
        "the results' cards share the featured cards' hover rule: {selectors}"
    );
    let (_, after_settings) = MAGIC
        .split_once("\n.card-list-results .card-stack-card {")
        .unwrap_or_else(|| panic!("the results' float settings"));
    let (settings, _) = after_settings
        .split_once('}')
        .unwrap_or_else(|| panic!("their end"));
    for setting in [
        "--float-lift",
        "--float-bob-lift",
        "--float-grow",
        "--float-swing",
        "--float-delay",
    ] {
        assert!(
            settings.contains(&format!("{setting}: initial;")),
            "{setting} falls back to the featured cards' default"
        );
    }
}
