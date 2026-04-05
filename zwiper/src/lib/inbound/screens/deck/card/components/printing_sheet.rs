//! Bottom sheet for browsing and selecting card printings.

use crate::outbound::client::{
    ZwipeClient,
    card::get_printings::ClientGetPrintings,
    deck_card::update_deck_card::ClientUpdateDeckCard,
};
use crate::inbound::components::auth::session_upkeep::Upkeep;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::deck_card::HttpUpdateDeckCard;

/// Bottom sheet for browsing all printings of a card and selecting one.
#[component]
pub(crate) fn PrintingSheet(
    card: Card,
    deck_id: Uuid,
    mut open: Signal<bool>,
    on_printing_changed: EventHandler<Card>,
) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    let mut printings: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| false);
    let mut cached_oracle_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut selected_index: Signal<usize> = use_signal(|| 0);

    let current_scryfall_id = card.scryfall_data.id;
    let oracle_id = card.scryfall_data.oracle_id;

    // Fetch printings when sheet opens (cache by oracle_id)
    use_effect(move || {
        if !open() { return; }
        let Some(oid) = oracle_id else { return; };

        // Skip fetch if already cached for this oracle_id
        if cached_oracle_id() == Some(oid) {
            // Just update selected index for current card
            let idx = printings()
                .iter()
                .position(|p| p.scryfall_data.id == current_scryfall_id)
                .unwrap_or(0);
            selected_index.set(idx);
            return;
        }

        is_loading.set(true);

        spawn(async move {
            let session_val = match session() {
                Some(s) => s,
                None => {
                    is_loading.set(false);
                    return;
                }
            };

            match client().get_printings(oid, &session_val).await {
                Ok(cards) => {
                    let idx = cards
                        .iter()
                        .position(|p| p.scryfall_data.id == current_scryfall_id)
                        .unwrap_or(0);
                    printings.set(cards);
                    selected_index.set(idx);
                    cached_oracle_id.set(Some(oid));
                }
                Err(e) => {
                    toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                }
            }
            is_loading.set(false);
        });
    });

    let selected_card = printings()
        .get(selected_index())
        .cloned();

    rsx! {
        // Modal backdrop
        div {
            class: if open() { "modal-backdrop show" } else { "modal-backdrop" },
            onclick: move |_| open.set(false),
        }

        // Bottom sheet
        div {
            class: if open() { "bottom-sheet show" } else { "bottom-sheet" },

            div { class: "modal-header",
                button {
                    class: "btn btn-sm",
                    onclick: move |_| open.set(false),
                    "close"
                }
            }

            div { class: "modal-content",
                if is_loading() {
                    div { class: "spinner" }
                } else if let Some(card) = &selected_card {
                    // Large image of selected printing
                    if let Some(ref image_url) = card.scryfall_data.image_uris.as_ref().and_then(|iu| iu.large.clone()) {
                        div { style: "display: flex; justify-content: center; margin-bottom: 0.75rem;",
                            img {
                                src: "{image_url}",
                                alt: "{card.scryfall_data.name}",
                                class: "card-image",
                            }
                        }
                    }

                    // Set name, collector number, year
                    div { style: "text-align: center; margin-bottom: 0.5rem;",
                        span { class: "text-muted",
                            "{card.scryfall_data.set_name.to_lowercase()} · #{card.scryfall_data.collector_number} · {card.scryfall_data.released_at.format(\"%Y\")}"
                        }
                    }

                    // Price row
                    div { style: "text-align: center; margin-bottom: 0.75rem;",
                        span { class: "text-muted",
                            {
                                let usd = card.scryfall_data.prices.usd.as_deref().map(|p| format!("${p}")).unwrap_or_default();
                                let eur = card.scryfall_data.prices.eur.as_deref().map(|p| format!("€{p}")).unwrap_or_default();
                                let parts: Vec<&str> = [usd.as_str(), eur.as_str()].into_iter().filter(|s| !s.is_empty()).collect();
                                parts.join(" · ")
                            }
                        }
                    }

                    // Horizontal printing selector
                    if printings().len() > 1 {
                        div { style: "overflow-x: auto; -webkit-overflow-scrolling: touch; padding: 0.5rem 0;",
                            div { style: "display: flex; gap: 0.5rem; min-width: min-content;",
                                for (i, printing) in printings().iter().enumerate() {
                                    {
                                        let printing_id = printing.scryfall_data.id;
                                        let is_selected = i == selected_index();
                                        let thumb_url = printing.scryfall_data.image_uris.as_ref()
                                            .and_then(|iu| iu.small.clone())
                                            .unwrap_or_default();
                                        let set = printing.scryfall_data.set_name.to_lowercase();
                                        rsx! {
                                            div {
                                                key: "{printing_id}",
                                                style: if is_selected {
                                                    "flex-shrink: 0; text-align: center; cursor: pointer; opacity: 1; border-bottom: 2px solid var(--color-text);"
                                                } else {
                                                    "flex-shrink: 0; text-align: center; cursor: pointer; opacity: 0.5;"
                                                },
                                                onclick: move |_| {
                                                    selected_index.set(i);
                                                },
                                                if !thumb_url.is_empty() {
                                                    img {
                                                        src: "{thumb_url}",
                                                        alt: "{set}",
                                                        style: "width: 4rem; border-radius: 0.25rem;",
                                                    }
                                                }
                                                div { style: "font-size: 0.7rem; margin-top: 0.2rem;",
                                                    class: "text-muted",
                                                    "{set}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Select button (inside scrollable content, below thumbnails)
                    if let Some(new_card) = selected_card.as_ref().filter(|c| c.scryfall_data.id != current_scryfall_id) {
                        {
                            let new_card = new_card.clone();
                            rsx! {
                                div { style: "display: flex; justify-content: center; padding: 0.75rem 0;",
                                    button {
                                        class: "btn btn-sm",
                                        onclick: move |_| {
                                            let new_card = new_card.clone();
                                            let new_id = new_card.scryfall_data.id;
                                            let request = HttpUpdateDeckCard::with_printing(&new_id.to_string());

                                            session.upkeep(client);
                                            let Some(session) = session() else { return; };

                                            let on_printing_changed = on_printing_changed;
                                            spawn(async move {
                                                match client().update_deck_card(deck_id, current_scryfall_id, &request, &session).await {
                                                    Ok(_) => {
                                                        toast.info(
                                                            "printing updated".to_string(),
                                                            ToastOptions::default().duration(Duration::from_millis(1500)),
                                                        );
                                                        on_printing_changed(new_card);
                                                        open.set(false);
                                                    }
                                                    Err(e) => {
                                                        toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                                    }
                                                }
                                            });
                                        },
                                        "select printing"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
