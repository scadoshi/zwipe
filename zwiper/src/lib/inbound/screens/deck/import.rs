//! Import cards from plain-text decklist screen.

use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        router::Router,
    },
    outbound::client::{
        ZwipeClient, deck_card::import_deck_cards::ClientImportDeckCards,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use uuid::Uuid;
use zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCardsResult;
use zwipe_core::domain::auth::models::session::Session;

#[component]
pub fn ImportDeck(deck_id: Uuid) -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut text = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut result: Signal<Option<ImportDeckCardsResult>> = use_signal(|| None);
    let mut board_selection: Signal<Option<&'static str>> = use_signal(|| None);
    let toast = use_toast();

    let mut do_import = move || {
        let board = *board_selection.peek();
        result.set(None);
        loading.set(true);

        spawn(async move {
            session.upkeep(client);
            let Some(session) = session() else {
                toast.error("session expired".to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                loading.set(false);
                return;
            };

            match client().import_deck_cards(deck_id, &text(), board, &session).await {
                Ok(r) => {
                    result.set(Some(r.clone()));
                    let imported = r.imported.len();
                    let unresolved = r.unresolved.len();
                    let opts = ToastOptions::default().duration(Duration::from_millis(1500));
                    match (imported, unresolved) {
                        (0, 0) => toast.info("no cards found".to_string(), opts),
                        (0, _) => toast.error(
                            format!("{unresolved} card{} unresolved", if unresolved == 1 { "" } else { "s" }),
                            ToastOptions::default().duration(Duration::from_millis(3000)),
                        ),
                        (_, 0) => toast.success(
                            format!("imported {imported} card{}", if imported == 1 { "" } else { "s" }),
                            opts,
                        ),
                        _ => toast.info(
                            format!("imported {imported}, {unresolved} unresolved"),
                            opts,
                        ),
                    }
                    loading.set(false);
                }
                Err(e) => {
                    toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                    loading.set(false);
                }
            }
        });
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "import" }
                }

                div { class: "screen-content centered content-enter",
                    div { class: "container-sm",
                        div { class: "chip-row",
                            span { class: "chip-row-label", "import:" }
                            for (label, value) in [("deck", None), ("maybe", Some("maybeboard")), ("side", Some("sideboard"))] {
                                button {
                                    class: if *board_selection.read() == value { "chip selected" } else { "chip" },
                                    onclick: move |_| board_selection.set(value),
                                    "{label}"
                                }
                            }
                        }
                        label { class: "label", r#for: "import-text", "paste decklist" }
                        textarea {
                            id: "import-text",
                            class: "input",
                            style: "width:100%;min-height:12rem;resize:vertical;font-family:monospace;",
                            placeholder: "5 island\n4 mountain\n1 guide of souls\n1 gonti's aether heart\n1 decoction module\n1 whirler virtuoso",
                            value: "{text}",
                            oninput: move |e| text.set(e.value()),
                        }

                        if let Some(r) = result() {
                            if !r.imported.is_empty() {
                                label { class: "label mt-2", "imported" }
                                for card in r.imported.iter() {
                                    div { class: "chip-bubble",
                                        span { class: "font-light", "{card.name.to_lowercase()}" }
                                        span { class: "font-light opacity-50", "x{card.quantity}" }
                                    }
                                }
                            }
                            if !r.unresolved.is_empty() {
                                label { class: "label mt-2", "unresolved" }
                                for card in r.unresolved.iter() {
                                    div { class: "chip-bubble-error",
                                        span { class: "font-light", "{card.name.to_lowercase()}" }
                                        span { class: "font-light opacity-50", "{card.reason.to_lowercase()}" }
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
                        "back"
                    }
                    button {
                        class: "util-btn",
                        disabled: loading(),
                        onclick: move |_| do_import(),
                        if loading() { "importing..." } else { "import" }
                    }
                }
            }
        }
    }
}
