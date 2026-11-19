//! An open deck card row's actions: the icons in one group with the stepper at the end of their
//! line, then a block of the word buttons, Printings and the board moves, under them.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "test helpers: a failed parse, render or lookup fails the test"
)]

use components::{CardRow, SendTo, data_action};
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
    let html = render(open_row);
    let line = section(&html, SPLIT_ROW);
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
    let html = render(open_row);
    let words = section(&html, WORDS);
    assert!(has(words, data_action::PRINTINGS), "{words}");
    for label in [">To maybeboard<", ">To sideboard<"] {
        assert!(words.contains(label), "{label} in {words}");
    }
    let (icons, _) = html.split_once(WORDS).unwrap();
    assert!(icons.contains(SPLIT_ROW), "the icons come first: {html}");
}

/// The declarations of the first rule whose selector is exactly `selector` in `css`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let (_, rest) = css
        .split_once(&format!("{selector} {{"))
        .unwrap_or_else(|| panic!("a {selector} rule"));
    rest.split_once('}').unwrap().0
}

#[test]
fn on_a_phone_the_stepper_ends_the_line_and_printings_spans_the_moves() {
    let (_, phone) = components::MAGIC_CSS
        .split_once("@media (max-width: 899px) {\n    /* A card's actions on a phone")
        .unwrap();
    let phone = phone.split_once("\n}\n").unwrap().0;
    assert!(rule(phone, ".card-action-split").contains("justify-content: space-between;"));
    let words = rule(phone, ".card-action-words");
    assert!(words.contains("grid-template-columns: 1fr 1fr;"), "{words}");
    let printings = rule(phone, r#".card-action-words > [data-action="printings"]"#);
    assert!(printings.contains("grid-column: 1 / -1;"), "{printings}");
}

#[test]
fn from_900px_the_stepper_stands_24px_after_the_icons() {
    let split = rule(components::MAGIC_CSS, "\n.card-action-split");
    assert!(split.contains("column-gap: var(--space-5);"), "{split}");
    let group = rule(components::MAGIC_CSS, "\n.card-action-group");
    assert!(group.contains("gap: var(--space-2);"), "{group}");
}

#[test]
fn on_a_touch_screen_the_icons_stand_far_enough_apart_that_their_reaches_never_meet() {
    let (_, coarse) = components::MAGIC_CSS
        .split_once("@media (pointer: coarse) {")
        .unwrap();
    let group = rule(coarse, ".card-action-group");
    assert!(group.contains("gap: var(--space-3);"), "{group}");
}
