use components::{
    CardDetailsCost, CardDetailsPanel, DrawerSheet, OpenFocus, use_meld, use_overlay_back,
};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_ui::card::info;

use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::interactions::swipe::stack::GhostCard;

/// A card's prices as money chips on one line: USD, EUR then TIX, whichever
/// it has, under a printing in the printing sheet. A swipe card's prices are
/// in [`CardDetailsSheet`], with its other details.
#[component]
pub(crate) fn CardInfoDisplay(card: Card) -> Element {
    let prices = info::prices(&card.scryfall_data);
    rsx! {
        div { class: "card-prices",
            for price in prices {
                span { key: "{price}", class: "card-row-price card-price-chip", "{price}" }
            }
        }
    }
}

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
/// `on_printings`, a Printings pill, as the web's pane does.
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
) -> Element {
    let describe_tag = use_describe_tag();
    let layer = use_overlay_back(open);
    // The shown face, shared by the title bar's cost and the panel's Flip.
    let face = use_signal(|| 0_usize);
    // The meld toggle, shared by the title bar and the panel, so the title
    // shows the melded card's name and cost (none) while it is on.
    let meld = use_meld(&card.scryfall_data);
    let title_card = meld.apply(&card);
    let name = title_card.scryfall_data.name.clone();
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
            }
        }
    }
}

/// Skeleton placeholder for when no card is loaded. A pile that ran out is said with a toast, so
/// the ghost carries text only as `note`, why a filter left nothing to deal, and `detail` under
/// it, whether Synergy may be the reason.
#[component]
pub(crate) fn CardSkeleton(
    #[props(default)] note: Option<&'static str>,
    #[props(default)] detail: Option<&'static str>,
) -> Element {
    // With nothing to say, the ghost is the stack's own, in the top card's
    // box; a note needs the text card.
    if note.is_none() {
        return rsx! { StackGhost {} };
    }
    rsx! {
        div { class: "skeleton-card",
            // The stack's slot, the card's ghost centered in it as the real
            // card is.
            div { class: "skeleton-stack",
                if let Some(note) = note {
                    div { class: "skeleton-image skeleton-note",
                        span { {note} }
                        if let Some(detail) = detail {
                            span { class: "skeleton-note-detail", {detail} }
                        }
                    }
                } else {
                    div { class: "skeleton-image" }
                }
            }
        }
    }
}

/// The ghost of a swipe stack's top card, while a pile is on its way: the
/// stack's ghost card in a stack of its own, so it takes the top card's
/// box exactly, as [`SwipeStack`](crate::inbound::components::interactions::swipe::stack::SwipeStack)'s
/// does while it holds its ghost.
#[component]
pub(crate) fn StackGhost() -> Element {
    rsx! {
        div { class: "skeleton-card",
            div { class: "swipe-stack",
                GhostCard {}
            }
        }
    }
}
