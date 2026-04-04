use dioxus::prelude::*;
use zwipe_core::domain::card::Card;

/// Displays card metadata: prices, set, release date, artist.
#[component]
pub(crate) fn CardInfoDisplay(card: Card) -> Element {
    rsx! {
        div { class: "card-info",
            if card.scryfall_data.prices.usd.is_some()
                || card.scryfall_data.prices.eur.is_some()
                || card.scryfall_data.prices.tix.is_some()
            {
                {
                    let mut display = String::from("prices:");
                    let mut prices_count = 0;
                    if let Some(usd) = card.scryfall_data.prices.usd {
                        display.push_str(format!(" ${usd}").as_str());
                        prices_count += 1;
                    }
                    if let Some(eur) = card.scryfall_data.prices.eur {
                        if prices_count > 0 {
                            display.push_str(" |");
                        }
                        display.push_str(format!(" €{eur}").as_str());
                        prices_count += 1;
                    }
                    if let Some(tix) = card.scryfall_data.prices.tix {
                        if prices_count > 0 {
                            display.push_str(" |");
                        }
                        display.push_str(format!(" {tix} tix").as_str());
                    }
                    rsx! { span { "{display}" } }
                }
            }
            span { "set: {card.scryfall_data.set_name.to_lowercase()}" }
            span { "released: {card.scryfall_data.released_at}" }
            if let Some(artist) = card.scryfall_data.artist && !artist.is_empty() {
                span { "artist: {artist.to_lowercase()}" }
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
                    "no cards"
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
