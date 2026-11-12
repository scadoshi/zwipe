//! The Featured flavor panel: the hour's card's flavor text under a centered
//! title, with its name, price, Image and the cart on the row beneath. Home draws
//! it from its model; the public screens draw it over the cached card.

use components::{CartButton, LoadFailed, StoreDialog};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_ui::screens::home::{self, Flavor};

use crate::inbound::components::auth::session_upkeep::FlavorCard;
use crate::inbound::screens::deck::card::components::image_preview::ImagePreview;

/// The panel for `flavor`, `card` being the one quoted: a card without
/// flavor text leaves the panel empty, before any card it ghosts, and after a
/// failed fetch it says so with Retry when the host passes `on_retry`. Image
/// opens the card's preview and the cart the store dialog, both owned here.
#[component]
pub fn FlavorPanel(
    flavor: Flavor,
    card: Option<Card>,
    #[props(default)] on_retry: Option<EventHandler>,
) -> Element {
    let mut preview_card: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    let preview_dismissing: Signal<bool> = use_signal(|| false);
    let mut show_buy_sheet = use_signal(|| false);

    let flavor_data = card.as_ref().map(|c| c.scryfall_data.clone());
    let buy_links = card.as_ref().map(home::buy_links).unwrap_or_default();

    rsx! {
        div { class: "container-sm text-center flex-col home-flavor content-enter-delayed",
            match flavor {
                Flavor::Quote(quote) => {
                    rsx! {
                        div { class: "settings-list-header", style: "justify-content: center;",
                            span { class: "settings-list-title", {home::FEATURED_FLAVOR} }
                        }
                        div { class: "flavor-quote", style: "padding: 1rem;",
                            "{quote.text} "
                        }
                        div { class: "flavor-buy-row",
                            span { class: "flavor-name", "{quote.name}" }
                            if let Some(price) = quote.price.as_ref() {
                                span { class: "flavor-price", "{price}" }
                            }
                            button {
                                class: "card-action-btn",
                                onclick: move |_| {
                                    if let Some(sd) = flavor_data.clone() {
                                        preview_card.set(Some((sd, 0)));
                                    }
                                },
                                {home::IMAGE}
                            }
                            if quote.price.is_some() {
                                CartButton { onclick: move |_| show_buy_sheet.set(true) }
                            }
                        }
                    }
                }
                Flavor::Blank => rsx! {},
                Flavor::Failed => rsx! {
                    div { class: "settings-list-header", style: "justify-content: center;",
                        span { class: "settings-list-title", {home::FEATURED_FLAVOR} }
                    }
                    if let Some(on_retry) = on_retry {
                        LoadFailed { message: home::FLAVOR_FAILED, on_retry, class: "home-flavor-line" }
                    }
                },
                Flavor::Loading => rsx! {
                    // Mirror the loaded card: a "Featured flavor" title
                    // bar + divider, the quote lines, then a chip
                    // ghost for each tag and button on the row under
                    // them.
                    div { class: "skeleton-card-header", style: "display: flex; justify-content: center;",
                        div { class: "skeleton-bar skeleton-section-label", style: "width: 45%; margin-bottom: 0;" }
                    }
                    div { class: "skeleton-flavor", style: "padding: 1rem;",
                        div { class: "skeleton-bar skeleton-flavor-line skeleton-flavor-line-1" }
                        div { class: "skeleton-bar skeleton-flavor-line skeleton-flavor-line-2" }
                        div { class: "skeleton-bar skeleton-flavor-line skeleton-flavor-line-3" }
                    }
                    div { class: "flavor-buy-row",
                        div { class: "skeleton-bar skeleton-chip skeleton-chip-lg" }
                        for i in 0..3 {
                            div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                        }
                    }
                },
            }
        }

        ImagePreview { card: preview_card, dismissing: preview_dismissing }

        StoreDialog {
            open: show_buy_sheet,
            title: home::BUY_TITLE.to_owned(),
            body: home::BUY_BODY.to_owned(),
            links: buy_links,
        }
    }
}

/// The public screens' featured flavor panel over the cache above the
/// router, fetched on mount when none is held or the hour's pick expired.
/// The endpoint needs no session. A failed fetch keeps the card held and is
/// only logged; with none the panel keeps ghosting.
#[component]
pub fn PublicFlavor() -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let mut cache: Signal<Option<FlavorCard>> = use_context();
    use_effect(move || {
        if cache.peek().as_ref().is_some_and(|f| !f.is_expired()) {
            return;
        }
        spawn(async move {
            match client().featured_flavor().await {
                Ok(card) => cache.set(Some(FlavorCard::new(card))),
                Err(e) => tracing::warn!("featured flavor fetch failed: {e}"),
            }
        });
    });
    let card = cache.read().as_ref().map(|f| f.card.clone());
    let flavor = card.as_ref().map_or(Flavor::Loading, home::quote);
    rsx! {
        FlavorPanel { flavor, card }
    }
}
