use crate::inbound::components::catalog_cache::CatalogCache;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_components::{AlertDialog, Button, ButtonVariant, CardDetails, DialogAction};
use zwipe_core::domain::card::Card;

/// Displays card metadata: prices, set, release date, artist. The card's oracle
/// text and stats live in [`CardDetailsDialog`], opened from the header eyeball.
#[component]
pub(crate) fn CardInfoDisplay(card: Card) -> Element {
    let has_prices = card.scryfall_data.prices.usd.is_some()
        || card.scryfall_data.prices.eur.is_some()
        || card.scryfall_data.prices.tix.is_some();

    let price_text = if has_prices {
        let mut display = String::from("Prices:");
        let mut count = 0;
        if let Some(usd) = card.scryfall_data.prices.usd {
            display.push_str(format!(" ${usd}").as_str());
            count += 1;
        }
        if let Some(eur) = card.scryfall_data.prices.eur {
            if count > 0 {
                display.push_str(" |");
            }
            display.push_str(format!(" €{eur}").as_str());
            count += 1;
        }
        if let Some(tix) = card.scryfall_data.prices.tix {
            if count > 0 {
                display.push_str(" |");
            }
            display.push_str(format!(" {tix} TIX").as_str());
        }
        display
    } else {
        "\u{00a0}".to_string()
    };

    let artist_text = card
        .scryfall_data
        .artist
        .filter(|a| !a.is_empty())
        .map(|a| format!("Artist: {a}"))
        .unwrap_or_else(|| "\u{00a0}".to_string());

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

/// Util-bar button (eye icon) that toggles the [`CardDetailsDialog`] for the
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

/// Dialog showing a card's full details (type, oracle text, stats, keywords,
/// card roles), for printings whose image is text-light (Secret Lair, full-art,
/// foreign-language). Opened from the action-bar [`RulesButton`] via the shared
/// `open` signal. The body is the shared [`CardDetails`] with the mana cost and
/// the Flip control inline, since every footer action closes the dialog. The
/// footer holds Close and an optional view-only Printings.
#[component]
pub(crate) fn CardDetailsDialog(
    open: Signal<bool>,
    card: Card,
    /// Opens the printings sheet. When set, a "Printings" action shows in the
    /// dialog footer (mirrors the deck-view expand-row → Printing button).
    #[props(default)]
    on_printings: Option<EventHandler<()>>,
) -> Element {
    let name = card.scryfall_data.name.clone();

    // Tag definitions for the card-role cluster: tapping a tag reveals its
    // description inline (same reveal as the deck view's expanded rows).
    // Resolved from the app-wide catalog cache; warmed on mount so a cold cache
    // still fills while the dialog is open.
    let cache: CatalogCache = use_context();
    let client: Signal<ZwipeClient> = use_context();
    use_effect(move || {
        cache.ensure_oracle_tags(client);
    });
    let describe_tag = use_callback(move |slug: String| {
        cache.oracle_tags.cell().read().loaded().and_then(|tags| {
            tags.iter()
                .find(|t| t.slug == slug)
                .and_then(|t| t.description.clone())
        })
    });

    let mut actions = vec![DialogAction::Dismiss("Close".into())];
    if let Some(handler) = on_printings {
        actions.push(DialogAction::Confirm {
            label: "Printings".into(),
            danger: false,
            on_confirm: handler,
        });
    }

    rsx! {
        AlertDialog {
            open,
            title: name,
            actions,
            // The description is the sole scroll container; `card-rules` only
            // styles the text.
            div { class: "card-rules alert-dialog-long",
                CardDetails {
                    card,
                    show_name: false,
                    show_cost: true,
                    show_classification: true,
                    show_flip: true,
                    describe_tag: Some(describe_tag),
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
