use std::collections::HashMap;

use components::{
    CardDetailsCost, CardDetailsPanel, CardStatsFolds, DrawerSheet, GhostCard, OpenFocus,
    share_face, use_meld, use_overlay_back, use_shared_face,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;

use crate::inbound::components::catalog_cache::CatalogCache;

/// Resolves an oracle tag's description from the app-wide catalog cache, for
/// the tappable tags in a card-role cluster. Warms the cache on mount so a cold
/// cache still fills while the caller shows.
pub(crate) fn use_describe_tag() -> Callback<String, Option<String>> {
    let cache: CatalogCache = use_context();
    let client: Signal<ZwipeClient> = use_context();
    use_effect(move || {
        cache.ensure_oracle_tags(client);
    });
    use_callback(move |slug: String| {
        cache.oracle_tags.cell().read().loaded().and_then(|tags| {
            tags.iter()
                .find(|t| t.slug == slug)
                .and_then(|t| t.description.clone())
        })
    })
}

/// A card's full details as a drawer from the right edge, behind the swipe
/// dock's eye: the card's name and mana cost in the drawer's title bar
/// beside the ×, then the card as a headless [`CardDetailsPanel`] with the
/// host's `footer` in its foot, its body scrolling under the title bar. The host opens it through the shared
/// `open` signal. It closes from its title bar's ×, a backdrop tap or the
/// back gesture. The panel's foot has Flip for a multi-faced card and, with
/// `on_printings`, a Printings pill, as the web's pane does. The body ends
/// with the card's stats, every fold shut at first, fetched as the drawer
/// opens.
///
/// Printings leaves this drawer open: the host's printing sheet draws over
/// it, and the host closes this one too when a printing is saved. Mount it
/// at the screen root, beside the printing sheet, so the drawer covers the
/// screen's buttons.
#[component]
pub(crate) fn CardDetailsSheet(
    open: Signal<bool>,
    card: Card,
    /// Opens the printings sheet. When set, a "Printings" pill shows in the
    /// panel's foot.
    #[props(default)]
    on_printings: Option<EventHandler<()>>,
    /// Host content shown in the panel's foot.
    #[props(default)]
    footer: Option<Element>,
    /// Host content at the body's end, scrolling with the body, such as a commander's reasons.
    #[props(default)]
    body_end: Option<Element>,
    /// Every card's shown face where the screen keeps them, so the drawer's Flip turns the
    /// card's image too, and the image's flip turns the drawer.
    #[props(default)]
    faces: Option<Signal<HashMap<Uuid, usize>>>,
) -> Element {
    let describe_tag = use_describe_tag();
    let layer = use_overlay_back(open);
    // The shown face, shared by the title bar's cost and the panel's Flip: the screen's for the
    // card where it keeps them.
    let id = card.scryfall_data.id;
    let own_face = use_signal(|| 0_usize);
    let face = use_shared_face(faces, id).unwrap_or(own_face);
    // The meld toggle, shared by the title bar and the panel, so the title
    // shows the melded card's name and cost (none) while it is on.
    let meld = use_meld(&card.scryfall_data);
    let title_card = meld.apply(&card);
    let name = title_card.scryfall_data.name.clone();
    let stats_card = card.clone();
    rsx! {
        DrawerSheet {
            layer: layer(),
            open,
            title: name,
            title_trailing: rsx! { CardDetailsCost { card: title_card, face } },
            on_close: move |()| {
                let mut open = open;
                open.set(false);
            },
            every_width: true,
            focus: OpenFocus::Kept,
            class: "card-details-drawer",
            CardDetailsPanel {
                card,
                headless: true,
                face,
                meld: Some(meld),
                describe_tag: Some(describe_tag),
                on_printings,
                footer,
                body_end: rsx! {
                    {body_end}
                    div { class: "card-details-stats",
                        CardStatsFolds { card: stats_card, active: open() }
                    }
                },
                on_face_change: share_face(faces, id),
            }
        }
    }
}

/// The swipe card's ghost where no card shows: the stack's ghost card in a stack of its own, so
/// it takes the top card's box exactly, with `note`, why a filter left nothing to deal, and
/// `detail` under it, whether Synergy may be the reason, inside the card's box. A pile that ran
/// out is said with a toast, so the ghost carries text only as `note`.
#[component]
pub(crate) fn CardSkeleton(
    #[props(default)] note: Option<String>,
    #[props(default)] detail: Option<String>,
) -> Element {
    rsx! {
        div { class: "skeleton-card",
            div { class: "swipe-stack",
                GhostCard { note, detail }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::inbound::components::catalog_cache::use_catalog_cache;

    /// A transform card: the fixture with Delver of Secrets' two faces.
    fn delver() -> Card {
        let json = include_str!("../../../../../../../../zwipe-ui/src/test_support/card.json");
        let mut card: Value = serde_json::from_str(json).expect("the fixture parses");
        let sd = card.get_mut("scryfall_data").expect("its data");
        sd["oracle_text"] = Value::Null;
        sd["card_faces"] = json!([
            { "object": "card_face", "name": "Delver of Secrets", "mana_cost": "{U}",
              "type_line": "Creature — Human Wizard", "oracle_text": "Upkeep." },
            { "object": "card_face", "name": "Insectile Aberration", "mana_cost": "",
              "type_line": "Creature — Human Insect", "oracle_text": "Flying" }
        ]);
        serde_json::from_value(card).expect("the two-faced card parses")
    }

    #[test]
    fn the_drawer_shows_the_face_its_screen_shares() {
        fn app() -> Element {
            let cache = use_catalog_cache();
            use_context_provider(|| cache);
            use_context_provider(|| {
                Signal::new(ZwipeClient::new(
                    "http://localhost".parse().expect("a base url"),
                ))
            });
            let card = delver();
            let faces: Signal<HashMap<Uuid, usize>> =
                use_signal(|| HashMap::from([(card.scryfall_data.id, 1)]));
            let open = use_signal(|| true);
            rsx! {
                CardDetailsSheet { open, card, faces }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(
            html.contains("Creature — Human Insect"),
            "the shared back: {html}"
        );
    }

    /// The host's `body_end` scrolls at the body's end, so a long block such as a commander's
    /// reasons never squeezes the body; the foot keeps only its buttons.
    #[test]
    fn body_end_sits_in_the_body_not_the_foot() {
        fn app() -> Element {
            let cache = use_catalog_cache();
            use_context_provider(|| cache);
            use_context_provider(|| {
                Signal::new(ZwipeClient::new(
                    "http://localhost".parse().expect("a base url"),
                ))
            });
            let open = use_signal(|| true);
            rsx! {
                CardDetailsSheet {
                    open,
                    card: delver(),
                    on_printings: move |()| {},
                    body_end: rsx! { span { class: "why-marker", "Why this commander?" } },
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let marker = html.find("why-marker").expect("the host's block renders");
        let foot = html.find("card-details-foot").expect("the foot");
        let body_end = html.find("card-details-body-end").expect("the body's end");
        assert!(body_end < marker, "inside the body's end: {html}");
        assert!(marker < foot, "before the foot, not in it: {html}");
    }
}
