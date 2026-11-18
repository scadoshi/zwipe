//! An open card row's Send to sits in the row with Image and Printings, never on a line of its
//! own.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed parse or render fails the test"
)]

use components::{CardRow, SendTo, data_action};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;

/// A real card as the API serves it, shared with zwipe-ui's tests.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

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

/// The card's row, open, with Image, Printings and Send to.
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
            on_qty_change: None,
            on_move_to: None,
            current_board: None,
            on_printing: Some(EventHandler::new(|_: Card| {})),
            mvp: None,
            on_toggle_mvp: None,
            on_hover_enter: None,
            on_hover_leave: None,
            show_art: Some(true),
            send_to: true,
        }
    }
}

#[test]
fn send_to_shares_the_row_with_image_and_printings() {
    let html = render(open_row);
    let has = |row: &str, action: &str| row.contains(&format!(r#"data-action="{action}""#));
    let rows: Vec<&str> = html.split(r#"class="card-action-row""#).skip(1).collect();
    let with_send_to: Vec<&&str> = rows
        .iter()
        .filter(|row| has(row, components::SEND_TO_ACTION))
        .collect();
    assert_eq!(with_send_to.len(), 1, "one Send to: {html}");
    let row = with_send_to.first().unwrap();
    assert!(has(row, data_action::PRINTINGS), "beside Printings: {row}");
    assert!(has(row, "image"), "beside Image: {row}");
}
