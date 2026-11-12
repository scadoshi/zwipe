//! A meld half's expanded list row offers the drawer's Meld toggle.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "test helpers: a failed read or render fails the test"
)]

use std::collections::HashMap;

use components::{CardDetails, MeldFetch, MeldSource};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;

/// A real card as the API serves it, shared with zwipe-ui's tests.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

/// The melded card's Scryfall id.
const RESULT_ID: &str = "00000000-0000-0000-0000-00000000beef";

/// The shared card, made a meld half whose result is [`RESULT_ID`].
fn meld_half() -> Card {
    let mut card: serde_json::Value = serde_json::from_str(CARD_JSON).unwrap();
    let sd = &mut card["scryfall_data"];
    let own_id = sd["id"].clone();
    sd["layout"] = "meld".into();
    sd["all_parts"] = serde_json::json!([
        { "object": "related_card", "id": own_id, "component": "meld_part",
          "name": "This Half", "type_line": "Creature", "uri": "https://api.scryfall.com/cards/x" },
        { "object": "related_card", "id": RESULT_ID, "component": "meld_result",
          "name": "The Melded", "type_line": "Creature", "uri": "https://api.scryfall.com/cards/y" },
    ]);
    serde_json::from_value(card).unwrap()
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

#[test]
fn a_meld_half_s_row_shows_the_meld_toggle() {
    fn app() -> Element {
        let results: Signal<HashMap<Uuid, MeldFetch>> = use_signal(HashMap::new);
        use_context_provider(|| MeldSource {
            results,
            request: Callback::new(|_: Uuid| {}),
        });
        rsx! { CardDetails { card: meld_half(), on_image: None, on_face_change: None } }
    }
    let html = render(app);
    assert!(html.contains(r#"data-action="meld""#), "{html}");
}

#[test]
fn a_row_without_a_meld_source_shows_no_toggle() {
    fn app() -> Element {
        rsx! { CardDetails { card: meld_half(), on_image: None, on_face_change: None } }
    }
    let html = render(app);
    assert!(!html.contains(r#"data-action="meld""#), "{html}");
}
