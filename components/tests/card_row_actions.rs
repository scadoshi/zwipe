//! An open card row's actions, and the deck page's card drawer's: one wrapping line of the icons'
//! group, the stepper after it, then the word buttons' group, Printings, the host's words and the
//! board moves. Each button wraps on its own, so the words share the icons' line while it has room.

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

/// The opening tag of the word buttons' group.
const WORDS: &str = r#"<span class="card-action-group card-action-words">"#;

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
        picked: Signal::new(false),
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

/// An open maybeboard row, whose moves go to the main board and the sideboard.
fn open_maybeboard_row() -> Element {
    use_context_provider(|| SendTo {
        open: Signal::new(false),
        card: Signal::new(None),
        picked: Signal::new(false),
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
            current_board: Some(Board::Maybeboard),
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

#[test]
fn the_moves_name_the_board_they_go_to() {
    let words = render(open_maybeboard_row);
    let words = section(&words, WORDS);
    for label in [">Mainboard<", ">Sideboard<"] {
        assert!(words.contains(label), "{label} in {words}");
    }
    assert!(!words.contains(">Maybeboard<"), "{words}");
}

/// The card's details as the deck page's drawer shows them, with every action.
fn drawer_panel() -> Element {
    use_context_provider(|| SendTo {
        open: Signal::new(false),
        card: Signal::new(None),
        picked: Signal::new(false),
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
        !has(group, data_action::PRINTINGS),
        "Printings is a word, not an icon: {group}"
    );
    assert_eq!(
        html.matches(components::SEND_TO_ACTION).count(),
        1,
        "one Send to: {html}"
    );
}

#[test]
fn printings_and_the_moves_follow_the_stepper_on_the_icons_line() {
    for html in [render(open_row), render(drawer_panel)] {
        words_on_the_icons_line(&html);
    }
}

fn words_on_the_icons_line(html: &str) {
    assert_eq!(
        html.matches(r#"<div class="card-action-row"#).count(),
        1,
        "the bar is one line: {html}"
    );
    let line = section(html, SPLIT_ROW);
    let (before, words) = line
        .split_once(WORDS)
        .unwrap_or_else(|| panic!("the words in the icons' line: {line}"));
    assert!(
        before.contains(r#"<span class="card-action-stepper">"#),
        "the words follow the stepper: {line}"
    );
    assert!(has(words, data_action::PRINTINGS), "{words}");
    for label in [">Maybeboard<", ">Sideboard<"] {
        assert!(words.contains(label), "{label} in {words}");
    }
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
fn on_a_phone_the_stepper_follows_the_icons_and_the_words_keep_their_width() {
    let phone = phone_block();
    assert!(
        !phone.contains(".card-action-split"),
        "the stepper stays beside the icons, the words after it, as on a wide screen"
    );
    assert!(
        !phone.contains(".card-action-words"),
        "the words lay out as on a wide screen: {phone}"
    );
}

#[test]
fn the_words_stand_at_their_own_width_from_the_left_and_wrap() {
    let row = rule(components::MAGIC_CSS, "\n.card-action-row");
    assert!(row.contains("display: flex;"), "{row}");
    assert!(row.contains("flex-wrap: wrap;"), "{row}");
    assert!(!row.contains("justify-content"), "{row}");
    assert!(
        !components::MAGIC_CSS.contains("grid-template-columns: 1fr 1fr;"),
        "no word block stands in equal columns"
    );
}

#[test]
fn from_900px_the_stepper_stands_as_far_from_the_icons_as_they_stand_from_each_other() {
    let split = rule(components::MAGIC_CSS, "\n.card-action-split");
    assert!(split.contains("column-gap: var(--space-2);"), "{split}");
    let group = rule(components::MAGIC_CSS, "\n.card-action-group");
    assert!(group.contains("gap: var(--space-2);"), "{group}");
}

/// The phone block of the card actions in `magic.css`.
fn phone_block() -> &'static str {
    let (_, phone) = components::MAGIC_CSS
        .split_once("@media (max-width: 899px) {\n    /* A card's actions on a phone")
        .unwrap();
    phone.split_once("\n}\n").unwrap().0
}

#[test]
fn a_touch_screen_spaces_the_buttons_8px_apart_as_a_mouse_does() {
    for block in components::MAGIC_CSS
        .split("@media (pointer: coarse) {")
        .skip(1)
    {
        let block = block.split_once("\n}\n").unwrap().0;
        for selector in [
            ".card-action-group {",
            ".card-action-split {",
            ".card-action-row {",
        ] {
            assert!(
                !block.contains(selector),
                "no touch rule sets a gap in {selector}...: {block}"
            );
        }
    }
    let phone = phone_block();
    let row = rule(phone, ".card-action-row");
    assert!(row.contains("gap: var(--space-2);"), "{row}");
    assert!(row.contains("margin-top: var(--space-2);"), "{row}");
}

#[test]
fn on_a_touch_screen_the_kit_s_44px_tap_box_shrinks_to_half_the_gap_past_each_edge() {
    let coarse = components::MAGIC_CSS
        .split("@media (pointer: coarse) {")
        .skip(1)
        .map(|block| block.split_once("\n}\n").unwrap().0)
        .find(|block| block.contains(".card-action-row .card-action-btn::after"))
        .expect("a touch rule for the card actions' tap box");
    let tap = rule(coarse, ".card-action-row .card-action-btn::after");
    for want in [
        "inset: calc(-1 * var(--space-1) - 1px);",
        "width: auto;",
        "height: auto;",
        "transform: none;",
    ] {
        assert!(tap.contains(want), "{want} in {tap}");
    }
}

#[test]
fn on_a_phone_each_button_reaches_half_the_gap_so_neighbors_meet_at_its_middle() {
    let phone = phone_block();
    let reach = rule(phone, ".card-action-btn::before");
    assert!(
        reach.contains("inset: calc(-1 * var(--space-1) - 1px);"),
        "4px past the border, half the 8px gap: {reach}"
    );
    let last = rule(phone, ".card-action-row:last-child");
    assert!(
        last.contains("margin-bottom: var(--space-1);"),
        "the bottom row's reach stays inside the row: {last}"
    );
}

#[test]
fn the_narrow_drawer_lays_its_bar_out_as_the_row_does() {
    let css = components::MAGIC_CSS;
    assert!(
        !css.contains(".card-details-drawer .card-action-split"),
        "the drawer's line lays out as the row's"
    );
    assert!(
        !css.contains(".card-details-drawer .card-action-words"),
        "the drawer's words lay out as the row's do"
    );
}

#[test]
fn each_button_in_the_line_wraps_on_its_own() {
    let groups = rule(
        components::MAGIC_CSS,
        "\n.card-action-split > .card-action-group",
    );
    assert!(groups.contains("display: contents;"), "{groups}");
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
fn a_row_without_moves_lays_its_words_after_image_on_one_line() {
    let html = render(suggestion_row);
    let line = section(&html, SPLIT_ROW);
    let (icons, words) = line
        .split_once(WORDS)
        .unwrap_or_else(|| panic!("the words in Image's line: {line}"));
    assert!(has(icons, "image"), "{icons}");
    for label in [
        r#"data-action="printings""#,
        ">Create deck<",
        ">Maybeboard<",
        ">Remove<",
    ] {
        assert!(words.contains(label), "{label} in {words}");
    }
}
