//! A meld half's expanded list row offers the drawer's Meld toggle.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "test helpers: a failed read or render fails the test"
)]

use std::collections::{HashMap, HashSet};

use components::{
    CardDetails, CardRow, CardStack, FlippableCardImage, MeldFetch, MeldSource, StackCard,
};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};

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
        let on: Signal<HashSet<Uuid>> = use_signal(HashSet::new);
        use_context_provider(|| MeldSource {
            results,
            on,
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

/// The melded card: the shared card under [`RESULT_ID`], with its own name,
/// words and images.
fn melded() -> ScryfallData {
    let mut card: serde_json::Value = serde_json::from_str(CARD_JSON).unwrap();
    let sd = &mut card["scryfall_data"];
    sd["id"] = RESULT_ID.into();
    sd["name"] = "The Melded".into();
    sd["oracle_text"] = "Melded rules.".into();
    for size in ["small", "normal", "large", "png", "art_crop", "border_crop"] {
        sd["image_uris"][size] = format!("https://cards.example/melded-{size}.jpg").into();
    }
    serde_json::from_value(sd.clone()).unwrap()
}

#[test]
fn a_half_s_meld_is_one_state_for_its_details_image_and_stack_card() {
    fn app() -> Element {
        let half = meld_half();
        let result = Uuid::parse_str(RESULT_ID).unwrap();
        let results: Signal<HashMap<Uuid, MeldFetch>> =
            use_signal(|| HashMap::from([(result, MeldFetch::Ready(Box::new(melded())))]));
        let on: Signal<HashSet<Uuid>> = use_signal(|| HashSet::from([half.scryfall_data.id]));
        use_context_provider(|| MeldSource {
            results,
            on,
            request: Callback::new(|_: Uuid| {}),
        });
        rsx! {
            CardDetails { card: half.clone(), on_image: None, on_face_change: None }
            div { class: "the-image",
                FlippableCardImage { sd: half.scryfall_data.clone(), size: ImageSize::Normal }
            }
            div { class: "the-stack",
                CardStack {
                    cards: vec![StackCard { card: half, qty: 1, face: 0, board: None }],
                    on_hover: |_| {},
                    on_leave: |_| {},
                    on_select: |_| {},
                }
            }
        }
    }
    let html = render(app);
    assert!(
        html.contains("Melded rules."),
        "the details show the melded card: {html}"
    );
    assert!(
        html.contains(r#"aria-pressed="true""#),
        "its Meld reads on: {html}"
    );
    let image = html.split(r#"class="the-image""#).nth(1).unwrap();
    let image = image.split(r#"class="the-stack""#).next().unwrap();
    assert!(
        image.contains("melded-"),
        "the card image shows the melded card: {image}"
    );
    let stack = html.split(r#"class="the-stack""#).nth(1).unwrap();
    assert!(
        stack.contains("melded-"),
        "the stack card shows the melded card: {stack}"
    );
}

#[test]
fn a_melded_half_s_list_row_shows_the_melded_card_s_art() {
    fn app() -> Element {
        let half = meld_half();
        let result = Uuid::parse_str(RESULT_ID).unwrap();
        let results: Signal<HashMap<Uuid, MeldFetch>> =
            use_signal(|| HashMap::from([(result, MeldFetch::Ready(Box::new(melded())))]));
        let on: Signal<HashSet<Uuid>> = use_signal(|| HashSet::from([half.scryfall_data.id]));
        use_context_provider(|| MeldSource {
            results,
            on,
            request: Callback::new(|_: Uuid| {}),
        });
        let expanded: Signal<Option<Uuid>> = use_signal(|| None);
        rsx! {
            CardRow {
                card: half,
                qty: 1,
                expanded_card: expanded,
                on_image: None,
                on_face_change: None,
                on_qty_change: None,
                on_move_to: None,
                current_board: None,
                on_printing: None,
                mvp: None,
                on_toggle_mvp: None,
                on_hover_enter: None,
                on_hover_leave: None,
                show_art: Some(true),
            }
        }
    }
    let html = render(app);
    assert!(
        html.contains("melded-art_crop"),
        "the row's thumbnail shows the melded card: {html}"
    );
}
