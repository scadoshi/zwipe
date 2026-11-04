use components::{BottomSheet, Button, ButtonVariant, CardDetails};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_ui::card::info;

use crate::inbound::components::catalog_cache::CatalogCache;

/// The swipe card's prices as money chips on one line under the card: USD,
/// EUR then TIX, whichever it has. The line keeps its height with none, so the
/// card never jumps. Everything else (set, release date, artist, rules) is in
/// [`CardDetailsSheet`], opened from the dock's eye.
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

/// Bottom sheet showing a card's full details (type, oracle text, stats,
/// keywords, card roles), for printings whose image is text-light (Secret
/// Lair, full-art, foreign-language). The host opens it through the shared
/// `open` signal. The body is the shared
/// [`CardDetails`], opening on the shown face's mana cost since the sheet's
/// title holds only text. The details and the host's `footer` share one
/// panel, so nothing lies straight on the sheet's grid. The sheet closes
/// from its title bar's × or a drag down; its own footer, present only when
/// there is something to hold, has Flip for a multi-faced card (it leaves
/// the sheet open) and an optional Printings.
///
/// Printings leaves this sheet open: the host's printing sheet draws over it,
/// and the host closes this one too when a printing is saved. Mount it at the
/// screen root, beside the printing sheet, so the sheet covers the screen's
/// buttons.
#[component]
pub(crate) fn CardDetailsSheet(
    open: Signal<bool>,
    card: Card,
    /// Opens the printings sheet. When set, a "Printings" button shows in the
    /// sheet footer (mirrors the deck-view expand-row → Printing button).
    #[props(default)]
    on_printings: Option<EventHandler<()>>,
    /// Host content shown under the details, in the same panel.
    #[props(default)]
    footer: Option<Element>,
) -> Element {
    let name = card.scryfall_data.name.clone();
    // Face state lives here so Flip can sit in the footer; the body reads it
    // through the controlled `face` prop.
    let face_count = components::card_face_count(&card);
    let mut face = use_signal(|| 0usize);
    let describe_tag = use_describe_tag();
    let actions = (face_count > 1 || on_printings.is_some()).then(|| {
        rsx! {
            if face_count > 1 {
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| face.set((face() + 1) % face_count),
                        "Flip"
                    }
                }
            if let Some(handler) = on_printings {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| handler.call(()),
                    "Printings"
                }
            }
        }
    });

    rsx! {
        BottomSheet {
            open,
            title: name,
            closable: true,
            footer: actions,
            // The sheet's content is the scroll container; `card-rules` only
            // styles the text. Each panel is the card-group surface, so the
            // sheet reads like the list behind it.
            div { class: "card-rules",
                div { class: "sheet-panel",
                    CardDetails {
                        card,
                        show_name: false,
                        show_cost: true,
                        show_classification: true,
                        show_printing: true,
                        show_flip: false,
                        face: Some(face),
                        describe_tag: Some(describe_tag),
                    }
                    if let Some(footer) = footer {
                        {footer}
                    }
                }
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
            // The price line's ghost: two chip-sized pills, centered.
            div { class: "card-prices",
                div { class: "skeleton-bar skeleton-price-chip" }
                div { class: "skeleton-bar skeleton-price-chip" }
            }
        }
    }
}
