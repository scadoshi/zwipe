//! Export deck as plain-text decklist screen.

use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        router::Router,
    },
    outbound::client::{ZwipeClient, deck::get_deck::ClientGetDeck},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::ApiError,
};

#[component]
pub fn ExportDeck(deck_id: Uuid) -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    let deck_text: Resource<Result<String, ApiError>> = use_resource(move || async move {
            session.upkeep(client);
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };
            let deck = client().get_deck(deck_id, &session).await?;
            let text = deck
                .entries
                .iter()
                .map(|e| format!("{} {}", *e.deck_card.quantity, e.card.scryfall_data.name))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(text)
        });

    use_effect(move || {
        if let Some(Err(e)) = &*deck_text.read() {
            toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
        }
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "export" }
                }

                div { class: "screen-content centered content-enter",
                    div { class: "container-sm",
                        match &*deck_text.read() {
                            Some(Ok(text)) => rsx! {
                                label { class: "label", r#for: "export-text", "decklist" }
                                textarea {
                                    id: "export-text",
                                    class: "input",
                                    style: "width:100%;min-height:16rem;resize:vertical;font-family:monospace;",
                                    readonly: true,
                                    value: "{text}",
                                }
                            },
                            Some(Err(_)) => rsx! { p { class: "text-muted", "could not load deck" } },
                            None => rsx! { div { class: "spinner" } }
                        }
                    }
                }

                div { class: "util-bar",
                    button {
                        class: "util-btn",
                        onclick: move |_| {
                            navigator.push(Router::ViewDeck { deck_id });
                        },
                        "back"
                    }
                    button {
                        class: "util-btn",
                        onclick: move |_| {
                            if let Some(Ok(text)) = deck_text() {
                                let js = format!(
                                    "navigator.clipboard.writeText({})",
                                    serde_json::to_string(&text).unwrap_or_default()
                                );
                                document::eval(&js);
                                toast.info(
                                    "copied to clipboard".to_string(),
                                    ToastOptions::default().duration(Duration::from_millis(2000)),
                                );
                            }
                        },
                        "copy"
                    }
                }
            }
        }
    }
}
