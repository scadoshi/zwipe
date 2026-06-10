//! Deck warnings section with action buttons for card-specific warnings.

use crate::outbound::client::{deck_card::delete_deck_card::ClientDeleteDeckCard, ZwipeClient};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use uuid::Uuid;
use zwipe_core::domain::deck::deck_warning::{DeckWarning, WarningAction};
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;

#[component]
pub(crate) fn DeckWarnings(
    warnings: Vec<DeckWarning>,
    deck_id: Uuid,
    on_remove: EventHandler<()>,
    on_fix_quantity: EventHandler<(Uuid, i32)>,
    on_clear_commander: EventHandler<()>,
) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    rsx! {
        label { class: "label", "Warnings" }
        div { class: "info-list",
            style: "border-color: var(--border-warning);",
            for warning in warnings.iter() {
                div { class: "info-row",
                    style: "justify-content: space-between; gap: 0.5rem;",
                    span { class: "text-muted", "{warning}" }
                    if let Some(card_id) = warning.scryfall_data_id() {
                        {
                            let on_remove = on_remove;
                            let on_fix_quantity = on_fix_quantity;
                            let on_clear_commander = on_clear_commander;
                            match warning.action() {
                                Some(WarningAction::FixQuantity(n)) => {
                                    let target_qty = *n;
                                    rsx! {
                                        button {
                                            class: "btn-xs",
                                            style: "color: var(--color-warning); border-color: var(--border-warning); margin-bottom: 0;",
                                            onclick: move |_| {
                                                on_fix_quantity((card_id, target_qty));
                                            },
                                            "Fix to {target_qty}"
                                        }
                                    }
                                }
                                Some(WarningAction::ClearCommander) => {
                                    rsx! {
                                        button {
                                            class: "btn-xs",
                                            style: "color: var(--color-warning); border-color: var(--border-warning); margin-bottom: 0;",
                                            onclick: move |_| {
                                                on_clear_commander(());
                                            },
                                            "Clear"
                                        }
                                    }
                                }
                                _ => {
                                    rsx! {
                                        button {
                                            class: "btn-xs",
                                            style: "color: var(--color-warning); border-color: var(--border-warning); margin-bottom: 0;",
                                            onclick: move |_| {
                                                let on_remove = on_remove;
                                                spawn(async move {
                                                    let result: Result<(), ApiError> = async {
                                                        let session = session()
                                                            .ok_or_else(|| ApiError::Unauthorized("Session expired".to_string()))?;
                                                        client().delete_deck_card(deck_id, card_id, &session).await
                                                    }.await;
                                                    match result {
                                                        Ok(()) => {
                                                            toast.info(
                                                                "Card removed".to_string(),
                                                                ToastOptions::default().duration(Duration::from_millis(1500)),
                                                            );
                                                            on_remove(());
                                                        }
                                                        Err(e) => {
                                                            toast.error(e.to_user_message(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                                        }
                                                    }
                                                });
                                            },
                                            "Remove"
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
}
