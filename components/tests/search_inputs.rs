//! A picker's search is the filter's compact input, the same height, padding, text size, radius
//! and border as Name contains.

use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

#[test]
fn a_picker_s_search_is_the_filter_s_compact_input() {
    fn picker() -> Element {
        let query = use_signal(String::new);
        rsx! { components::PickerSearch { id: "send-to-search", placeholder: "Search decks", query } }
    }
    let html = render(picker);
    assert!(html.contains(r#"class="input input-compact""#), "{html}");
}

#[test]
fn a_picker_s_search_has_its_label_chip_on_its_left_on_one_line() {
    fn picker() -> Element {
        let query = use_signal(String::new);
        rsx! { components::PickerSearch { id: "send-to-search", placeholder: "Search decks", query } }
    }
    let html = render(picker);
    let row = html.find(r#"<div class="chip-row">"#);
    let label =
        html.find(r#"<label class="stat-chip stat-chip-p1 chip-row-label" for="send-to-search">Search</label>"#);
    let input = html.find(r#"class="input input-compact""#);
    assert!(
        row.is_some_and(|r| label.is_some_and(|l| input.is_some_and(|i| r < l && l < i))),
        "{html}"
    );
}

#[test]
fn send_to_labels_its_boards_and_draws_no_search_heading() {
    let src = include_str!("../src/send_to.rs");
    let boards = src
        .split(r#"class: "chip-row send-to-boards""#)
        .nth(1)
        .unwrap_or_default();
    let label = boards.find(r#"span { class: "stat-chip stat-chip-p1 chip-row-label", {TITLE} }"#);
    let chips = boards.find("for choice in BOARDS");
    assert!(
        label.is_some_and(|l| chips.is_some_and(|c| l < c)),
        "the Send to chip leads the boards row"
    );
    assert!(
        !src.contains("tag-search-label"),
        "the field's own row carries its label"
    );
}
