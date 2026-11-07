use components::icons::{FLIP, LAYERS};
use components::{CardDetailsPanel, DockAction, DrawerSheet, use_overlay_back};
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
/// dock's eye: the card as a [`CardDetailsPanel`] with its release date and
/// the host's `footer` in its foot. The host opens it through the shared
/// `open` signal. It closes from its title bar's ×, a backdrop tap or the
/// back gesture. Its dock, pinned at its bottom right and present only when
/// there is something to hold, has Flip for a multi-faced card (it leaves
/// the drawer open) and an optional Printings.
///
/// Printings leaves this drawer open: the host's printing sheet draws over
/// it, and the host closes this one too when a printing is saved. Mount it
/// at the screen root, beside the printing sheet, so the drawer covers the
/// screen's buttons.
#[component]
pub(crate) fn CardDetailsSheet(
    open: Signal<bool>,
    card: Card,
    /// Opens the printings sheet. When set, a "Printings" button shows in the
    /// drawer's dock (mirrors the deck-view expand-row → Printing button).
    #[props(default)]
    on_printings: Option<EventHandler<()>>,
    /// Host content shown in the panel's foot.
    #[props(default)]
    footer: Option<Element>,
) -> Element {
    // Face state lives here so Flip can sit in the dock; the panel reads it
    // through the controlled `face` prop.
    let face_count = components::card_face_count(&card);
    let mut face = use_signal(|| 0usize);
    let describe_tag = use_describe_tag();
    use_overlay_back(open);
    // Flip and Printings as the drawer's dock, pinned at its bottom right.
    let dock = (face_count > 1 || on_printings.is_some()).then(|| {
        rsx! {
            if face_count > 1 {
                DockAction {
                    label: "Flip",
                    icon: FLIP,
                    onclick: move |()| face.set((face() + 1) % face_count),
                }
            }
            if let Some(handler) = on_printings {
                DockAction { label: "Printings", icon: LAYERS, onclick: move |()| handler.call(()) }
            }
        }
    });

    rsx! {
        DrawerSheet {
            open,
            title: DETAILS_TITLE,
            on_close: move |()| {
                let mut open = open;
                open.set(false);
            },
            every_width: true,
            class: "card-details-drawer",
            dock,
            CardDetailsPanel {
                card,
                show_printing: true,
                show_flip: false,
                face: Some(face),
                describe_tag: Some(describe_tag),
                footer,
            }
        }
    }
}

/// The details drawer's title; the panel inside it carries the card's name.
const DETAILS_TITLE: &str = "Card details";

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
