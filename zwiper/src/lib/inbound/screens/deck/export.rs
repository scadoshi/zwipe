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
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::Deck;

#[component]
pub fn ExportDeck(deck_id: Uuid) -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    let mut include_maybeboard: Signal<bool> = use_signal(|| false);

    let deck_resource: Resource<Result<Deck, ApiError>> = use_resource(move || async move {
        session.upkeep(client);
        let Some(session) = session() else {
            return Err(ApiError::Unauthorized("session expired".to_string()));
        };
        client().get_deck(deck_id, &session).await
    });

    use_effect(move || {
        if let Some(Err(e)) = &*deck_resource.read() {
            toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
        }
    });

    // Derive export text reactively from deck data + maybeboard toggle
    let export_text: Memo<Option<String>> = use_memo(move || {
        let deck = deck_resource()?.ok()?;
        let mut lines: Vec<String> = Vec::new();

        // Active deck cards
        for entry in deck.entries.iter().filter(|e| !e.deck_card.maybeboard) {
            lines.push(format!("{} {}", *entry.deck_card.quantity, entry.card.scryfall_data.name));
        }

        // Maybeboard section (only if toggled on AND cards exist)
        if include_maybeboard() {
            let mb: Vec<_> = deck.entries.iter().filter(|e| e.deck_card.maybeboard).collect();
            if !mb.is_empty() {
                lines.push(String::new());
                lines.push("// Maybeboard".to_string());
                for entry in mb {
                    lines.push(format!("{} {}", *entry.deck_card.quantity, entry.card.scryfall_data.name));
                }
            }
        }

        Some(lines.join("\n"))
    });

    let has_maybeboard = deck_resource()
        .and_then(|r| r.ok())
        .is_some_and(|d| d.entries.iter().any(|e| e.deck_card.maybeboard));

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "export" }
                }

                div { class: "screen-content centered content-enter",
                    div { class: "container-sm",

                        if has_maybeboard {
                            div { class: "chip-row",
                                span { class: "chip-row-label", "include:" }
                                button {
                                    class: if include_maybeboard() { "chip selected" } else { "chip" },
                                    onclick: move |_| include_maybeboard.set(!include_maybeboard()),
                                    "maybeboard"
                                }
                            }
                        }

                        match export_text() {
                            Some(text) => rsx! {
                                label { class: "label", r#for: "export-text", "decklist" }
                                textarea {
                                    id: "export-text",
                                    class: "input",
                                    style: "width:100%;min-height:16rem;resize:vertical;font-family:monospace;",
                                    readonly: true,
                                    value: "{text}",
                                }
                            },
                            None => {
                                if deck_resource().is_some_and(|r| r.is_err()) {
                                    rsx! { p { class: "text-muted", "could not load deck" } }
                                } else {
                                    rsx! { div { class: "spinner" } }
                                }
                            }
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
                            if let Some(text) = export_text() {
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
