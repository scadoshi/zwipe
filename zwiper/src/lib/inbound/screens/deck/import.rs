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
use uuid::Uuid;
use zwipe::domain::{
    auth::models::session::Session,
    deck::models::deck_card::import_deck_cards::ImportDeckCardsResult,
};

#[component]
pub fn ImportDeck(deck_id: Uuid) -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut text = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut result: Signal<Option<ImportDeckCardsResult>> = use_signal(|| None);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let mut attempt_import = move || {
        error.set(None);
        result.set(None);
        loading.set(true);

        spawn(async move {
            session.upkeep(client);
            let Some(session) = session() else {
                error.set(Some("session expired".to_string()));
                loading.set(false);
                return;
            };

            match client().import_deck_cards(deck_id, &text(), &session).await {
                Ok(r) => {
                    result.set(Some(r));
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
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

                div { class: "screen-content centered",
                    div { class: "container-sm",
                        label { class: "label", r#for: "import-text", "paste decklist" }
                        textarea {
                            id: "import-text",
                            class: "input",
                            style: "width:100%;min-height:12rem;resize:vertical;font-family:monospace;",
                            placeholder: "5 island\n4 mountain\n1 guide of souls\n1 gonti's aether heart\n1 decoction module\n1 whirler virtuoso",
                            value: "{text}",
                            oninput: move |e| text.set(e.value()),
                        }

                        if let Some(err) = error() {
                            div { class: "message-error", "{err}" }
                        }

                        if let Some(r) = result() {
                            if !r.imported.is_empty() {
                                label { class: "label mt-2", "imported" }
                                for card in r.imported.iter() {
                                    div { class: "flex items-center flex-between mb-1",
                                        span { class: "text-sm font-light", "{card.name.to_lowercase()}" }
                                        span { class: "text-sm font-light opacity-50", "x{card.quantity}" }
                                    }
                                }
                            }
                            if !r.unresolved.is_empty() {
                                label { class: "label mt-2", "unresolved" }
                                for card in r.unresolved.iter() {
                                    div { class: "flex items-center flex-between mb-1",
                                        span { class: "text-sm font-light", "{card.name.to_lowercase()}" }
                                        span { class: "text-sm font-light opacity-50", "{card.reason.to_lowercase()}" }
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
                        onclick: move |_| attempt_import(),
                        if loading() { "importing..." } else { "import" }
                    }
                }
            }
        }
    }
}
