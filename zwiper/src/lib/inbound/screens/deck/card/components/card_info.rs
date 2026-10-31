use components::{BottomSheet, Button, ButtonVariant, CardDetails};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;

use crate::inbound::components::catalog_cache::CatalogCache;

/// Displays card metadata: prices, set, release date, artist. The card's oracle
/// text and stats live in [`CardDetailsSheet`], opened from the header eyeball.
#[component]
pub(crate) fn CardInfoDisplay(card: Card) -> Element {
    let has_prices = card.scryfall_data.prices.usd.is_some()
        || card.scryfall_data.prices.eur.is_some()
        || card.scryfall_data.prices.tix.is_some();

    let price_text = if has_prices {
        let prices = &card.scryfall_data.prices;
        let shown: Vec<String> = [
            prices.usd.as_ref().map(|usd| format!(" ${usd}")),
            prices.eur.as_ref().map(|eur| format!(" €{eur}")),
            prices.tix.as_ref().map(|tix| format!(" {tix} TIX")),
        ]
        .into_iter()
        .flatten()
        .collect();
        let display = format!("Prices:{}", shown.join(" |"));
        display
    } else {
        "\u{00a0}".to_string()
    };

    let artist_text = card
        .scryfall_data
        .artist
        .filter(|a| !a.is_empty())
        .map_or_else(|| "\u{00a0}".to_string(), |a| format!("Artist: {a}"));

    rsx! {
        div { class: "card-info",
            span { class: "card-info-name", "{card.scryfall_data.name}" }
            span { "{price_text}" }
            span { "Set: {card.scryfall_data.set_name}" }
            span { "Released: {card.scryfall_data.released_at}" }
            span { "{artist_text}" }
        }
    }
}

/// Util-bar button (eye icon) that toggles the [`CardDetailsSheet`] for the
/// active swipe card via the shared `open` signal.
#[component]
pub(crate) fn RulesButton(open: Signal<bool>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Action,
            class: "btn-action-eye",
            onclick: move |_| open.set(!open()),
            svg {
                class: "eye-icon",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                path { d: "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" }
                circle { cx: "12", cy: "12", r: "3" }
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
/// Lair, full-art, foreign-language). Opened from the action-bar
/// [`RulesButton`] via the shared `open` signal. The body is the shared
/// [`CardDetails`], opening on the shown face's mana cost since the sheet's
/// title holds only text. The details and the host's `footer` share one
/// panel, so nothing lies straight on the sheet's grid. The sheet's own
/// footer holds Back, then Flip for a multi-faced card (it leaves the sheet
/// open) and an optional Printings.
///
/// Printings leaves this sheet open: the host's printing sheet draws over it,
/// and the host closes this one too when a printing is saved. Mount it at the
/// screen root, beside the printing sheet, so the sheet covers the action bar.
#[component]
pub(crate) fn CardDetailsSheet(
    mut open: Signal<bool>,
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

    rsx! {
        BottomSheet {
            open,
            title: name,
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| open.set(false),
                    "Back"
                }
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
            },
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

/// Skeleton placeholder for when no card is loaded.
#[component]
pub(crate) fn CardSkeleton(#[props(default = false)] is_loading: bool) -> Element {
    rsx! {
        div { class: "skeleton-card",
            // While a search is in flight the image area stays a plain ghost
            // block (no spinner); "No cards" only when a finished search is
            // genuinely empty.
            div { class: "skeleton-image",
                if !is_loading {
                    "No cards"
                }
            }
            div { class: "skeleton-info",
                div { class: "skeleton-bar skeleton-bar-name" }
                div { class: "skeleton-bar skeleton-bar-price" }
                div { class: "skeleton-bar skeleton-bar-set" }
                div { class: "skeleton-bar skeleton-bar-date" }
                div { class: "skeleton-bar skeleton-bar-artist" }
            }
        }
    }
}
