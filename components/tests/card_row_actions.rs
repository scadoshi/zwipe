//! An open deck card row's actions, and the deck page's card drawer's: the icons in one group with
//! the stepper at the end of their line, then a block of the word buttons, Printings and the board
//! moves, under them.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "test helpers: a failed parse, render or lookup fails the test"
)]

use components::{CardDetailsPanel, CardEdit, CardRow, SendTo, data_action};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;

/// A real card as the API serves it, shared with zwipe-ui's tests.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

/// The opening tag of the icons' line.
const SPLIT_ROW: &str = r#"<div class="card-action-row card-action-split">"#;

/// The opening tag of the word buttons' block.
const WORDS: &str = r#"<div class="card-action-row card-action-words">"#;

fn card() -> Card {
    serde_json::from_str(CARD_JSON).unwrap()
}

/// Renders `app` once its effects have run.
fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

/// The card's row on a deck's Cards list, open, with every action.
fn open_row() -> Element {
    use_context_provider(|| SendTo {
        open: Signal::new(false),
        card: Signal::new(None),
    });
    let card = card();
    let expanded: Signal<Option<Uuid>> = use_signal(|| Some(card.scryfall_data.id));
    rsx! {
        CardRow {
            card,
            qty: 1,
            expanded_card: expanded,
            on_image: Some(EventHandler::new(|(): ()| {})),
            on_face_change: None,
            on_qty_change: Some(EventHandler::new(|_: i32| {})),
            on_move_to: Some(EventHandler::new(|_: Board| {})),
            current_board: None,
            on_printing: Some(EventHandler::new(|_: Card| {})),
            mvp: Some(false),
            on_toggle_mvp: Some(EventHandler::new(|(): ()| {})),
            flagged: Some(false),
            on_toggle_flag: Some(EventHandler::new(|(): ()| {})),
            on_hover_enter: None,
            on_hover_leave: None,
            show_art: Some(true),
            send_to: true,
        }
    }
}

/// The card's details as the deck page's drawer shows them, with every action.
fn drawer_panel() -> Element {
    use_context_provider(|| SendTo {
        open: Signal::new(false),
        card: Signal::new(None),
    });
    rsx! {
        CardDetailsPanel {
            card: card(),
            on_image: Some(EventHandler::new(|(): ()| {})),
            on_printings: Some(EventHandler::new(|(): ()| {})),
            edit: CardEdit {
                qty: 1,
                on_qty_change: Some(EventHandler::new(|_: i32| {})),
                mvp: Some(false),
                on_toggle_mvp: Some(EventHandler::new(|(): ()| {})),
                flagged: Some(false),
                on_toggle_flag: Some(EventHandler::new(|(): ()| {})),
                current_board: None,
                on_move_to: EventHandler::new(|_: Board| {}),
            },
        }
    }
}

/// The part of `html` from `open` to the next `<div class="card-action-row` or the end.
fn section<'html>(html: &'html str, open: &str) -> &'html str {
    let (_, rest) = html
        .split_once(open)
        .unwrap_or_else(|| panic!("{open} in {html}"));
    rest.split_once(r#"<div class="card-action-row"#)
        .map_or(rest, |(section, _)| section)
}

fn has(section: &str, action: &str) -> bool {
    section.contains(&format!(r#"data-action="{action}""#))
}

#[test]
fn the_icons_share_one_group_with_the_stepper_after_it() {
    for html in [render(open_row), render(drawer_panel)] {
        icons_then_stepper(&html);
    }
}

fn icons_then_stepper(html: &str) {
    let line = section(html, SPLIT_ROW);
    let (group, after) = line
        .split_once(r#"<span class="card-action-group">"#)
        .and_then(|(_, rest)| rest.split_once("</span><span class=\"card-action-stepper\">"))
        .unwrap_or_else(|| panic!("the group, then the stepper: {line}"));
    for action in ["image", components::SEND_TO_ACTION, "star", "flag"] {
        assert!(has(group, action), "{action} in the group: {group}");
    }
    assert!(
        after.contains(">-</button>") && after.contains(">+</button>"),
        "{after}"
    );
    assert!(
        !has(line, data_action::PRINTINGS),
        "Printings is a word: {line}"
    );
    assert_eq!(
        html.matches(components::SEND_TO_ACTION).count(),
        1,
        "one Send to: {html}"
    );
}

#[test]
fn printings_and_the_moves_sit_in_the_block_under_the_icons() {
    for html in [render(open_row), render(drawer_panel)] {
        words_under_icons(&html);
    }
}

fn words_under_icons(html: &str) {
    let words = section(html, WORDS);
    assert!(has(words, data_action::PRINTINGS), "{words}");
    for label in [">To maybeboard<", ">To sideboard<"] {
        assert!(words.contains(label), "{label} in {words}");
    }
    let (icons, _) = html.split_once(WORDS).unwrap();
    assert!(icons.contains(SPLIT_ROW), "the icons come first: {html}");
    assert_eq!(
        html.matches(r#"data-action="printings""#).count(),
        1,
        "one Printings: {html}"
    );
}

/// The declarations of the first rule whose selector is exactly `selector` in `css`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let (_, rest) = css
        .split_once(&format!("{selector} {{"))
        .unwrap_or_else(|| panic!("a {selector} rule"));
    rest.split_once('}').unwrap().0
}

#[test]
fn on_a_phone_the_stepper_ends_the_line_and_an_odd_block_s_first_word_spans_both_columns() {
    let (_, phone) = components::MAGIC_CSS
        .split_once("@media (max-width: 899px) {\n    /* A card's actions on a phone")
        .unwrap();
    let phone = phone.split_once("\n}\n").unwrap().0;
    assert!(rule(phone, ".card-action-split").contains("justify-content: space-between;"));
    let words = rule(phone, ".card-action-words");
    assert!(words.contains("grid-template-columns: 1fr 1fr;"), "{words}");
    let first = rule(
        phone,
        ".card-action-words > :first-child:nth-last-child(odd)",
    );
    assert!(first.contains("grid-column: 1 / -1;"), "{first}");
}

#[test]
fn from_900px_the_stepper_stands_as_far_from_the_icons_as_they_stand_from_each_other() {
    let split = rule(components::MAGIC_CSS, "\n.card-action-split");
    assert!(split.contains("column-gap: var(--space-2);"), "{split}");
    let group = rule(components::MAGIC_CSS, "\n.card-action-group");
    assert!(group.contains("gap: var(--space-2);"), "{group}");
}

#[test]
fn on_a_touch_screen_the_icons_stand_far_enough_apart_that_their_reaches_never_meet() {
    let (_, coarse) = components::MAGIC_CSS
        .split_once("@media (pointer: coarse) {")
        .unwrap();
    let coarse = coarse.split_once("\n}\n").unwrap().0;
    let group = rule(coarse, ".card-action-group");
    assert!(group.contains("gap: var(--space-3);"), "{group}");
    let split = rule(coarse, ".card-action-split");
    assert!(
        split.contains("column-gap: var(--space-3);"),
        "the stepper too: {split}"
    );
}

#[test]
fn the_narrow_drawer_lays_its_bar_out_as_a_phone_does_at_every_width() {
    let css = components::MAGIC_CSS;
    let split = rule(css, "\n.card-details-drawer .card-action-split");
    assert!(split.contains("justify-content: space-between;"), "{split}");
    let words = rule(css, "\n.card-details-drawer .card-action-words");
    assert!(words.contains("grid-template-columns: 1fr 1fr;"), "{words}");
    let first = rule(
        css,
        "\n.card-details-drawer .card-action-words > :first-child:nth-last-child(odd)",
    );
    assert!(first.contains("grid-column: 1 / -1;"), "{first}");
}

/// A suggested commander's row: no stepper and no moves, its host's words after Printings.
fn suggestion_row() -> Element {
    let card = card();
    let expanded: Signal<Option<Uuid>> = use_signal(|| Some(card.scryfall_data.id));
    rsx! {
        CardRow {
            card,
            qty: 1,
            expanded_card: expanded,
            on_image: Some(EventHandler::new(|(): ()| {})),
            on_face_change: None,
            on_qty_change: None,
            on_move_to: None,
            current_board: None,
            on_printing: Some(EventHandler::new(|_: Card| {})),
            mvp: None,
            on_toggle_mvp: None,
            on_hover_enter: None,
            on_hover_leave: None,
            show_art: Some(true),
            extra_actions: rsx! {
                button { class: "card-action-btn", "Create deck" }
                button { class: "card-action-btn", "Maybeboard" }
                button { class: "card-action-btn", "Remove" }
            },
        }
    }
}

#[test]
fn a_row_without_moves_lays_its_words_in_the_same_block() {
    let html = render(suggestion_row);
    let words = section(&html, WORDS);
    for label in [
        r#"data-action="printings""#,
        ">Create deck<",
        ">Maybeboard<",
        ">Remove<",
    ] {
        assert!(words.contains(label), "{label} in {words}");
    }
}
