use components::{CardDetailsCost, CardDetailsPanel, DrawerSheet, use_overlay_back};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_ui::card::info;

use crate::inbound::components::catalog_cache::CatalogCache;

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
    use_overlay_back(open);
    // The shown face, shared by the title bar's cost and the panel's Flip.
    let face = use_signal(|| 0_usize);
    let name = card.scryfall_data.name.clone();
    let title_card = card.clone();
    rsx! {
        DrawerSheet {
            open,
            title: name,
            title_trailing: rsx! { CardDetailsCost { card: title_card, face } },
            on_close: move |()| {
                let mut open = open;
                open.set(false);
            },
            every_width: true,
            class: "card-details-drawer",
            CardDetailsPanel {
                card,
                headless: true,
                face,
                describe_tag: Some(describe_tag),
                on_printings,
                footer,
            }
        }
    }
}

/// Skeleton placeholder for the printing sheet (carousel + info rows).
#[component]
pub(crate) fn PrintingSheetSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-printing",
            div { class: "skeleton-printing-image" }
            div { class: "skeleton-printing-dots",
                for i in 0..5 {
                    div { key: "{i}", class: "skeleton-printing-dot" }
                }
            }
            div { class: "skeleton-printing-info",
                div { class: "skeleton-bar skeleton-printing-info-price" }
                div { class: "skeleton-bar skeleton-printing-info-set" }
                div { class: "skeleton-bar skeleton-printing-info-released" }
                div { class: "skeleton-bar skeleton-printing-info-artist" }
            }
        }
    }
}

/// Skeleton placeholder for when no card is loaded. It carries no text: an empty pile is said with a toast.
#[component]
pub(crate) fn CardSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-card",
            // The stack's slot, the card's ghost centered in it as the real
            // card is.
            div { class: "skeleton-stack",
                div { class: "skeleton-image" }
            }
        }
    }
}
