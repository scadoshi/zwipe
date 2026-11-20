//! The card search: the magnifying glass is one of the nav's icon buttons just left of the box,
//! a chip-row gap from it, and the box holds only the text. Where the host's header folds, the
//! bar is the header's: the glass alone until a tap opens the box across the header.

#![expect(clippy::panic, reason = "a test: a missing part fails the test")]

use components::{CardSearchBar, HEADER_ALWAYS};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use zwipe_ui::screens::card_search::{Action, State};

const MAGIC: &str = include_str!("../assets/magic.css");

/// Renders `app` once.
fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

/// The declarations of the rule whose selector is exactly `selector`.
fn rule<'css>(selector: &str) -> &'css str {
    let (_, rest) = MAGIC
        .split_once(&format!("\n{selector} {{"))
        .unwrap_or_else(|| panic!("a rule for {selector}"));
    rest.split_once('}').unwrap_or_else(|| panic!("its end")).0
}

#[test]
fn the_glass_is_a_label_chip_for_the_box_left_of_it() {
    let html = render(|| {
        let model = use_signal(State::new);
        rsx! {
            CardSearchBar { model, on_action: move |_: Action| {}, input_id: "nav-card-search".to_owned() }
        }
    });
    let chip = html
        .find(r#"<label class="btn-action nav-icon card-search-label" for="nav-card-search""#)
        .unwrap_or_else(|| panic!("the glass is a nav icon button for the box: {html}"));
    let glass = html
        .find("card-search-icon")
        .unwrap_or_else(|| panic!("the glass: {html}"));
    let field = html
        .find("card-search-field")
        .unwrap_or_else(|| panic!("the field: {html}"));
    assert!(
        chip < glass && glass < field,
        "the glass sits in the chip, left of the box: {html}"
    );
    assert!(
        !html.contains("card-search-header"),
        "a wide nav's bar is not the header's: {html}"
    );
}

#[test]
fn the_chip_sits_a_chip_row_gap_from_the_box() {
    let nav = rule(".card-search-nav");
    assert!(nav.contains("gap: var(--space-2);"), "{nav}");
    assert!(
        nav.contains("flex: 0 1000 calc(2.2rem + var(--space-2) + 120px);"),
        "{nav}"
    );
}

#[test]
fn the_box_keeps_no_room_for_the_glass() {
    assert!(
        !MAGIC.contains(".card-search-nav .card-search-input {\n    padding-left: var(--space-6);"),
        "the box's text starts at its own edge"
    );
}

#[test]
fn the_app_s_bar_is_always_the_header_s() {
    let html = render(|| {
        let model = use_signal(State::new);
        rsx! {
            CardSearchBar {
                model,
                on_action: move |_: Action| {},
                input_id: "header-card-search".to_owned(),
                header_media: HEADER_ALWAYS.to_owned(),
            }
        }
    });
    assert!(
        html.contains(r#"class="card-search card-search-nav card-search-header""#),
        "{html}"
    );
}

#[test]
fn the_header_s_box_waits_unseen_but_focusable_and_opens_across_the_header() {
    let rest = rule(".card-search-header .card-search-field");
    assert!(
        rest.contains("width: 0;") && rest.contains("opacity: 0;"),
        "{rest}"
    );
    assert!(
        !rest.contains("display") && !rest.contains("visibility"),
        "a tap on the chip can focus it: {rest}"
    );
    let open = rule(".card-search-header .card-search-slot:focus-within .card-search-field");
    assert!(
        open.contains("width: 100%;") && open.contains("opacity: 1;"),
        "{open}"
    );
    let others =
        rule("nav:has(> .card-search-header:focus-within) > :not(.card-search-nav, .nav-panel)");
    assert!(
        others.contains("opacity: 0;") && others.contains("pointer-events: none;"),
        "{others}"
    );
    let clear = rule(".card-search-header .card-search-slot:focus-within .input-clear-btn");
    assert!(
        clear.contains("opacity: 1;"),
        "the ✕ shows open, text or not: {clear}"
    );
    let drop = rule(".card-search-header .card-search-drop");
    assert!(
        drop.contains("position: fixed;") && drop.contains("overflow-y: auto;"),
        "{drop}"
    );
}

#[test]
fn an_open_header_search_takes_taps_on_its_chips() {
    assert!(
        rule(".card-search-nav.card-search-header").contains("pointer-events: none;"),
        "the header's bar lets taps through where nothing of it shows"
    );
    let open = rule(".card-search-header .card-search-slot:focus-within .card-search-drop");
    assert!(
        open.contains("pointer-events: auto;"),
        "the chips under an open header search take taps: {open}"
    );
}

#[test]
fn a_result_chip_s_entry_lets_go_of_its_transform_once_it_ends() {
    let chip = rule(".card-search-chip");
    assert!(
        chip.contains("animation: card-search-chip-in 0.25s ease-out backwards;"),
        "a hover's lift can move the chip after it comes in: {chip}"
    );
}

#[test]
fn the_glass_keeps_the_nav_icon_s_own_box() {
    for selector in [
        ".card-search-label",
        ".card-search-header .card-search-label",
    ] {
        let glass = rule(selector);
        for property in ["width:", "height:", "padding:", "border", "background"] {
            assert!(
                !glass.contains(property),
                "{selector} leaves {property} to the nav icon: {glass}"
            );
        }
    }
}
