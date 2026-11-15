//! The featured strip at the top of a deck's Cards on a phone: the command zone and the main
//! board's MVPs as card images on one line, each labeled, the deck's identity at a glance. The
//! app draws it, and the web does at phone width, so both are the same strip.
//!
//! Every card takes the same width, two to the strip's width up to 11rem, whatever the count, so
//! the strip's height never changes when a card joins or leaves; a third and later card scrolls
//! in from the faded edge. The strip holds that height from its first frame, each image box
//! ghosted until its art fades in, so nothing under it moves while the deck and its images land.

use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use crate::flippable_card_image::FlippableCardImage;
use crate::screen_return::ScreenReturns;
use crate::{share_face, use_shared_face};

/// The deal's keyframes, as the stylesheet names them (`.deck-featured.dealing`).
const DEAL_ANIMATION: &str = "deck-featured-deal";

/// One labeled featured card: its image (when it has one) over its name and role chips.
/// Tapping it fires `on_tap` with the card, and the tap stops at the card so the screen's tap-off
/// doesn't see it. Nothing covers a featured card, so it never floats: the first tap acts. It
/// carries the card's `data-row` and is a stop of the keyboard cursor, Enter acting as a tap.
#[component]
fn StripCard(
    card: Card,
    role: String,
    on_tap: EventHandler<Card>,
    #[props(default)] faces: Option<Signal<HashMap<Uuid, usize>>>,
) -> Element {
    let name = card.scryfall_data.name.clone();
    let sd = card.scryfall_data.clone();
    let has_image = sd.primary_image_url(ImageSize::Normal).is_some();
    let id = sd.id;
    let keyed = card.clone();
    let shared = use_shared_face(faces, id);
    rsx! {
        div {
            class: "deck-featured-card",
            // Named for the settle, so the strip's cards slide when a star adds or takes one.
            "data-settle-key": "featured-{card.scryfall_data.id}",
            "data-row": "{id}",
            tabindex: "0",
            role: "button",
            "aria-label": "{name}",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Enter {
                    event.prevent_default();
                    on_tap.call(keyed.clone());
                }
            },
            onclick: move |event: MouseEvent| {
                event.stop_propagation();
                on_tap.call(card.clone());
            },
            if has_image {
                FlippableCardImage {
                    sd,
                    size: ImageSize::Normal,
                    class: "deck-featured-image".to_string(),
                    draggable: false,
                    face: shared,
                    on_face_change: share_face(faces, id),
                }
            }
            div { class: "deck-featured-chips",
                span { class: "stat-chip deck-featured-name", "{name}" }
                if !role.is_empty() {
                    span {
                        class: if role == "MVP" { "stat-chip deck-featured-role-mvp" } else { "stat-chip deck-featured-role-zone" },
                        "{role}"
                    }
                }
            }
        }
    }
}

/// The strip. While the deck loads it is a ghost at the strip's height when `ghost` says the deck
/// likely features a card, else nothing; once loaded, nothing when there is nothing to feature.
///
/// Its cards deal into the held space (opacity and transform only) when it mounts with them, each
/// time `deal_key` changes, and each time its screen comes back from under another
/// ([`ScreenReturns`]); the deal ends on its last card's `animationend`, after which a card
/// starred later just joins.
#[component]
pub fn FeaturedStrip(
    /// `(card, role label)` pairs, command zone first then MVPs.
    cards: Vec<(Card, String)>,
    /// Whether the deck's cards and command zone have all arrived.
    loaded: bool,
    /// Whether to hold the strip's place with a ghost while the deck loads.
    #[props(default)]
    ghost: bool,
    /// A host's count of entries: each change deals the cards again.
    #[props(default)]
    deal_key: u32,
    /// Fired with the tapped card: the host opens its details or its image.
    on_tap: EventHandler<Card>,
    /// Every card's shown face where the screen keeps them, so a strip card's flip turns its
    /// details and rows too, and theirs turn it.
    #[props(default)]
    faces: Option<Signal<HashMap<Uuid, usize>>>,
) -> Element {
    let mut dealing = use_signal(|| true);
    let mut landed = use_signal(|| 0usize);
    let returns = try_use_context::<ScreenReturns>().map_or(0, |ScreenReturns(r)| r());
    use_effect(use_reactive!(|deal_key, returns| {
        let _ = (deal_key, returns);
        landed.set(0);
        dealing.set(true);
    }));
    if !loaded {
        return if ghost {
            rsx! { FeaturedGhost {} }
        } else {
            rsx! {}
        };
    }
    if cards.is_empty() {
        return rsx! {};
    }
    let count = cards.len();
    rsx! {
        section {
            class: if dealing() { "deck-featured dealing" } else { "deck-featured" },
            onanimationend: move |event: AnimationEvent| {
                if event.animation_name() == DEAL_ANIMATION {
                    let now = landed.peek().saturating_add(1);
                    landed.set(now);
                    if now >= count {
                        dealing.set(false);
                    }
                }
            },
            div { class: "deck-featured-row scroll-fade-x", "data-settle-scope": "",
                for (card, role) in cards {
                    StripCard { key: "{card.scryfall_data.id}", card, role, on_tap, faces }
                }
            }
        }
    }
}

/// The strip's ghost: one card's image box and its name and role chips, at the strip's height.
#[component]
fn FeaturedGhost() -> Element {
    rsx! {
        section { class: "deck-featured", "aria-hidden": "true",
            div { class: "deck-featured-row",
                div { class: "deck-featured-card",
                    div { class: "deck-featured-image" }
                    // Chip boxes at a loaded card's chips' height, filled as ghosts.
                    div { class: "deck-featured-chips",
                        span { class: "stat-chip deck-featured-ghost-chip", style: "width: 7rem;", "\u{a0}" }
                        span { class: "stat-chip deck-featured-ghost-chip", style: "width: 4.5rem;", "\u{a0}" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// A transform card with an image per face, as Delver of Secrets is served.
    fn two_faced() -> Card {
        let json = include_str!("../../zwipe-ui/src/test_support/card.json");
        let mut card: serde_json::Value =
            serde_json::from_str(json).expect("the fixture card parses");
        let sd = card.get_mut("scryfall_data").expect("its data");
        let front = sd["image_uris"].clone();
        sd["image_uris"] = serde_json::Value::Null;
        sd["oracle_text"] = serde_json::Value::Null;
        sd["card_faces"] = serde_json::json!([
            { "object": "card_face", "name": "Delver of Secrets", "mana_cost": "{U}",
              "type_line": "Creature — Human Wizard", "oracle_text": "Upkeep.", "image_uris": front },
            { "object": "card_face", "name": "Insectile Aberration", "mana_cost": "",
              "type_line": "Creature — Human Insect", "oracle_text": "Flying", "image_uris": front }
        ]);
        serde_json::from_value(card).expect("the two-faced card parses")
    }

    #[test]
    fn a_strip_card_shows_the_face_its_screen_shares() {
        fn app() -> Element {
            let card = two_faced();
            let faces = use_signal(|| HashMap::from([(card.scryfall_data.id, 1_usize)]));
            rsx! {
                StripCard { card, role: "MVP".to_owned(), on_tap: |_: Card| {}, faces }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::dioxus_core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains(r#"alt="Insectile Aberration""#), "{html}");
    }

    #[test]
    fn a_strip_card_is_a_stop_the_keyboard_cursor_can_focus() {
        fn app() -> Element {
            let json = include_str!("../../zwipe-ui/src/test_support/card.json");
            let card: Card = serde_json::from_str(json).expect("the fixture card parses");
            rsx! {
                StripCard { card, role: "MVP".to_owned(), on_tap: |_: Card| {} }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let (_, tag) = html
            .split_once(r#"class="deck-featured-card""#)
            .expect("the strip card");
        let (tag, _) = tag.split_once('>').expect("its tag");
        assert!(tag.contains("data-row="), "{tag}");
        assert!(tag.contains(r#"tabindex="0""#), "{tag}");
        assert!(tag.contains(r#"role="button""#), "{tag}");
    }
}
