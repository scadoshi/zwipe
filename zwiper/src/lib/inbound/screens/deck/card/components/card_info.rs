use dioxus::prelude::*;
use zwipe_core::domain::card::Card;

/// Displays card metadata: prices, set, release date, artist.
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
            if count > 0 { display.push_str(" |"); }
            display.push_str(format!(" €{eur}").as_str());
            count += 1;
        }
        if let Some(tix) = card.scryfall_data.prices.tix {
            if count > 0 { display.push_str(" |"); }
            display.push_str(format!(" {tix} TIX").as_str());
        }
        display
    } else {
        "\u{00a0}".to_string()
    };

    let artist_text = card.scryfall_data.artist
        .filter(|a| !a.is_empty())
        .map(|a| format!("Artist: {a}"))
        .unwrap_or_else(|| "\u{00a0}".to_string());

    rsx! {
        div { class: "card-info",
            span { "{price_text}" }
            span { "Set: {card.scryfall_data.set_name}" }
            span { "Released: {card.scryfall_data.released_at}" }
            span { "{artist_text}" }
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
            div { class: "skeleton-image",
                if is_loading {
                    div { class: "spinner" }
                } else {
                    "No cards"
                }
            }
            div { class: "skeleton-info",
                div { class: "skeleton-bar skeleton-bar-price" }
                div { class: "skeleton-bar skeleton-bar-set" }
                div { class: "skeleton-bar skeleton-bar-date" }
                div { class: "skeleton-bar skeleton-bar-artist" }
            }
        }
    }
}
