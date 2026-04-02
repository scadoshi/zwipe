//! Deck warnings section with optional remove buttons for card-specific warnings.

use crate::outbound::client::{deck_card::delete_deck_card::ClientDeleteDeckCard, ZwipeClient};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::deck_warning::DeckWarning},
    inbound::http::ApiError,
};

#[component]
pub(crate) fn DeckWarnings(
    warnings: Vec<DeckWarning>,
    deck_id: Uuid,
    on_remove: EventHandler<()>,
) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    rsx! {
        label { class: "label", "warnings" }
        div { class: "info-list",
            style: "border-color: var(--border-warning);",
            for warning in warnings.iter() {
                div { class: "info-row",
                    style: "justify-content: space-between; gap: 0.5rem;",
                    span { class: "text-muted", "{warning.to_lowercase()}" }
                    if let Some(card_id) = warning.scryfall_data_id() {
                        {
                            let on_remove = on_remove;
                            rsx! {
                                button {
                                    class: "btn-xs",
                                    style: "color: var(--color-warning); border-color: var(--border-warning); margin-bottom: 0;",
                                    onclick: move |_| {
                                        let on_remove = on_remove;
                                        spawn(async move {
                                            let result: Result<(), ApiError> = async {
                                                let session = session()
                                                    .ok_or_else(|| ApiError::Unauthorized("session expired".to_string()))?;
                                                client().delete_deck_card(deck_id, card_id, &session).await
                                            }.await;
                                            match result {
                                                Ok(()) => on_remove(()),
                                                Err(e) => {
                                                    toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                                }
                                            }
                                        });
                                    },
                                    "remove"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
