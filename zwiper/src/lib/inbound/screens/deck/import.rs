//! Import cards screen — plain-text decklist or an Archidekt URL, in add or
//! replace mode. Both sources import into the selected board of this deck and
//! share the same result shape.

use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
        router::Router,
    },
    outbound::client::{
        ZwipeClient, deck::import_archidekt_deck::ClientImportArchidektDeck,
        deck_card::import_deck_cards::ClientImportDeckCards,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCardsResult;

/// Which import source is active.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ImportSource {
    /// Paste a plain-text decklist.
    Text,
    /// Paste an Archidekt deck URL.
    Archidekt,
}

#[component]
pub fn ImportDeck(deck_id: Uuid) -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut source = use_signal(|| ImportSource::Text);
    let mut mode = use_signal(|| ImportMode::Add);
    let mut text = use_signal(String::new);
    let mut url = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut result: Signal<Option<ImportDeckCardsResult>> = use_signal(|| None);
    let mut board_selection: Signal<Option<&'static str>> = use_signal(|| None);
    let toast = use_toast();

    let board_word = board_selection.read().unwrap_or("mainboard");

    let mut do_import = move || {
        let board = *board_selection.peek();
        let mode = *mode.peek();
        let source = *source.peek();
        result.set(None);
        loading.set(true);

        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    loading.set(false);
                    return;
                }
            };

            let response = match source {
                ImportSource::Text => {
                    client()
                        .import_deck_cards(deck_id, &text(), board, mode, &session)
                        .await
                }
                ImportSource::Archidekt => {
                    client()
                        .import_archidekt_deck(deck_id, &url(), board, mode, &session)
                        .await
                }
            };

            match response {
                Ok(r) => {
                    result.set(Some(r.clone()));
                    let imported = r.imported.len();
                    let unresolved = r.unresolved.len();
                    let opts = ToastOptions::default().duration(Duration::from_millis(1500));
                    match (imported, unresolved) {
                        (0, 0) => toast.info("No cards found".to_string(), opts),
                        (0, _) => toast.error(
                            format!(
                                "{unresolved} card{} unresolved",
                                if unresolved == 1 { "" } else { "s" }
                            ),
                            ToastOptions::default().duration(Duration::from_millis(3000)),
                        ),
                        (_, 0) => toast.success(
                            format!(
                                "Imported {imported} card{}",
                                if imported == 1 { "" } else { "s" }
                            ),
                            opts,
                        ),
                        _ => toast.info(
                            format!("Imported {imported}, {unresolved} unresolved"),
                            opts,
                        ),
                    }
                    loading.set(false);
                }
                Err(e) => {
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    loading.set(false);
                }
            }
        });
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "Import" }
                }

                div { class: "screen-content content-enter",
                    div { class: "import-controls",
                        div { class: "chip-row",
                            span { class: "chip-row-label", "From:" }
                            for (label, value) in [("Text", ImportSource::Text), ("Archidekt", ImportSource::Archidekt)] {
                                button {
                                    class: if *source.read() == value { "chip selected" } else { "chip" },
                                    onclick: move |_| source.set(value),
                                    "{label}"
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "chip-row-label", "Mode:" }
                            for (label, value) in [("Add", ImportMode::Add), ("Replace", ImportMode::Replace)] {
                                button {
                                    class: if *mode.read() == value { "chip selected" } else { "chip" },
                                    onclick: move |_| mode.set(value),
                                    "{label}"
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "chip-row-label", "Board:" }
                            for (label, value) in [("Main", None), ("Maybe", Some("maybeboard")), ("Side", Some("sideboard"))] {
                                button {
                                    class: if *board_selection.read() == value { "chip selected" } else { "chip" },
                                    onclick: move |_| board_selection.set(value),
                                    "{label}"
                                }
                            }
                        }
                        p { class: "text-muted text-sm import-hint",
                            if mode().is_replace() {
                                "Removes all existing cards in the "
                                span { class: "hl-key", "{board_word}" }
                                ", replacing with imported cards."
                            } else {
                                "Adds imported cards into the "
                                span { class: "hl-key", "{board_word}" }
                                ". Existing cards stay but take new quantities if applicable."
                            }
                        }
                    }

                    div { class: "container-sm",
                        if source() == ImportSource::Text {
                            label { class: "label", r#for: "import-text", "Paste decklist" }
                            textarea {
                                id: "import-text",
                                class: "input",
                                style: "width:100%;min-height:12rem;resize:vertical;",
                                placeholder: "1 Krenko, Mob Boss\n1 Urza, Lord High Artificer\n1 Sol Ring",
                                value: "{text}",
                                oninput: move |e| text.set(e.value()),
                            }
                        } else {
                            label { class: "label", r#for: "import-url", "Paste an Archidekt deck URL" }
                            input {
                                id: "import-url",
                                class: "input",
                                style: "width:100%;",
                                r#type: "url",
                                placeholder: "https://archidekt.com/decks/...",
                                value: "{url}",
                                oninput: move |e| url.set(e.value()),
                            }
                        }

                        if let Some(r) = result() {
                            if !r.imported.is_empty() {
                                label { class: "label mt-2", "Imported" }
                                for card in r.imported.iter() {
                                    div { class: "chip-bubble",
                                        span { class: "font-light", "{card.name}" }
                                        span { class: "font-light opacity-50", "x{card.quantity}" }
                                    }
                                }
                            }
                            if !r.unresolved.is_empty() {
                                label { class: "label mt-2", "Unresolved" }
                                for card in r.unresolved.iter() {
                                    div { class: "chip-bubble-error",
                                        span { class: "font-light", "{card.name}" }
                                        span { class: "font-light opacity-50", "{card.reason}" }
                                    }
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
                        disabled: loading() || match source() {
                            ImportSource::Text => text().trim().is_empty(),
                            ImportSource::Archidekt => url().trim().is_empty(),
                        },
                        onclick: move |_| do_import(),
                        if loading() {
                            "Importing..."
                        } else if mode().is_replace() {
                            "Replace"
                        } else {
                            "Import"
                        }
                    }
                }
            }
        }
    }
}
