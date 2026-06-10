//! Export deck as plain-text decklist screen.

use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
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

    let mut include_deck: Signal<bool> = use_signal(|| true);
    let mut include_sideboard: Signal<bool> = use_signal(|| false);
    let mut include_maybeboard: Signal<bool> = use_signal(|| false);

    let deck_resource: Resource<Result<Deck, ApiError>> = use_resource(move || async move {
        let session = session.ensure_fresh(client).await?;
        client().get_deck(deck_id, &session).await
    });

    use_effect(move || {
        if let Some(Err(e)) = &*deck_resource.read() {
            toast.error(e.to_user_message(), ToastOptions::default().duration(Duration::from_millis(3000)));
        }
    });

    // Derive export text reactively from deck data + maybeboard toggle
    let export_text: Memo<Option<String>> = use_memo(move || {
        let deck = deck_resource()?.ok()?;
        let mut lines: Vec<String> = Vec::new();

        // Command zone cards (stored on profile, not in entries)
        let cmd_names: Vec<&str> = [
            deck.deck_profile.commander_name.as_deref(),
            deck.deck_profile.partner_commander_name.as_deref(),
            deck.deck_profile.background_name.as_deref(),
            deck.deck_profile.signature_spell_name.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();
        if !cmd_names.is_empty() {
            lines.push("// Commander".to_string());
            for name in cmd_names {
                lines.push(format!("1 {name}"));
            }
            lines.push(String::new());
        }

        // Deck section
        if include_deck() {
            let deck_cards: Vec<_> = deck.entries.iter().filter(|e| e.deck_card.board.is_active()).collect();
            if !deck_cards.is_empty() {
                lines.push("// Deck".to_string());
                for entry in deck_cards {
                    lines.push(format!("{} {}", *entry.deck_card.quantity, entry.card.scryfall_data.name));
                }
            }
        }

        // Maybeboard section (only if toggled on AND cards exist)
        if include_maybeboard() {
            let mb: Vec<_> = deck.entries.iter().filter(|e| e.deck_card.board.is_maybeboard()).collect();
            if !mb.is_empty() {
                lines.push(String::new());
                lines.push("// Maybeboard".to_string());
                for entry in mb {
                    lines.push(format!("{} {}", *entry.deck_card.quantity, entry.card.scryfall_data.name));
                }
            }
        }

        // Sideboard section (only if toggled on AND cards exist)
        if include_sideboard() {
            let sb: Vec<_> = deck.entries.iter().filter(|e| e.deck_card.board.is_sideboard()).collect();
            if !sb.is_empty() {
                lines.push(String::new());
                lines.push("// Sideboard".to_string());
                for entry in sb {
                    lines.push(format!("{} {}", *entry.deck_card.quantity, entry.card.scryfall_data.name));
                }
            }
        }

        Some(lines.join("\n"))
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "Export" }
                }

                div { class: "screen-content centered content-enter",
                    div { class: "container-sm",

                        div { class: "chip-row",
                            span { class: "chip-row-label", "Export:" }
                            button {
                                class: if include_deck() { "chip selected" } else { "chip" },
                                onclick: move |_| {
                                    let new_val = !include_deck();
                                    if !new_val && !include_maybeboard() && !include_sideboard() {
                                        return;
                                    }
                                    include_deck.set(new_val);
                                },
                                "Deck"
                            }
                            button {
                                class: if include_maybeboard() { "chip selected" } else { "chip" },
                                onclick: move |_| {
                                    let new_val = !include_maybeboard();
                                    include_maybeboard.set(new_val);
                                    if new_val && !include_deck() && !include_sideboard() {
                                        // at least one is on, fine
                                    } else if !new_val && !include_deck() && !include_sideboard() {
                                        include_deck.set(true);
                                    }
                                },
                                "Maybe"
                            }
                            button {
                                class: if include_sideboard() { "chip selected" } else { "chip" },
                                onclick: move |_| {
                                    let new_val = !include_sideboard();
                                    include_sideboard.set(new_val);
                                    if !new_val && !include_deck() && !include_maybeboard() {
                                        include_deck.set(true);
                                    }
                                },
                                "Side"
                            }
                        }

                        match export_text() {
                            Some(text) => rsx! {
                                label { class: "label", r#for: "export-text", "Decklist" }
                                textarea {
                                    id: "export-text",
                                    class: "input",
                                    style: "width:100%;min-height:16rem;resize:vertical;",
                                    readonly: true,
                                    value: "{text}",
                                }
                            },
                            None => {
                                if deck_resource().is_some_and(|r| r.is_err()) {
                                    rsx! { p { class: "text-muted", "Could not load deck" } }
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
                        "Back"
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
                                    "Copied to clipboard".to_string(),
                                    ToastOptions::default().duration(Duration::from_millis(2000)),
                                );
                            }
                        },
                        "Copy"
                    }
                }
            }
        }
    }
}
