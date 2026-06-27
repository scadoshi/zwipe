use crate::inbound::components::alert_dialog::{
    AlertDialogActions, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogRoot, AlertDialogTitle,
};
use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::router::Router;
use crate::outbound::client::{ZwipeClient, deck::clone_deck::ClientCloneDeck};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::deck::HttpCloneDeck;

/// Clone deck dialog — prompts for a new name, calls the clone endpoint,
/// and navigates to the new deck on success.
#[component]
pub(crate) fn CloneDeckDialog(
    source_deck_id: Uuid,
    default_name: String,
    mut open: Signal<bool>,
) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let navigator = use_navigator();
    let toast = use_toast();

    let mut new_name = use_signal(String::new);
    let mut is_cloning = use_signal(|| false);

    // Reset state whenever the dialog reopens, reseeding the prefill with the
    // latest source deck name.
    use_effect(move || {
        if open() {
            new_name.set(default_name.clone());
            is_cloning.set(false);
        }
    });

    rsx! {
        AlertDialogRoot {
            open: open(),
            on_open_change: move |v: bool| {
                if !v { open.set(false); }
            },
            AlertDialogContent {
                AlertDialogTitle { "Clone deck" }
                hr { class: "dialog-rule" }
                AlertDialogDescription {
                    "Give your new deck a name. All cards, commander, and format will be copied over."
                }
                input {
                    r#type: "text",
                    class: "input",
                    placeholder: "New deck name",
                    value: "{new_name}",
                    oninput: move |evt| new_name.set(evt.value()),
                }
                hr { class: "dialog-rule" }
                AlertDialogActions {
                    AlertDialogCancel {
                        on_click: move |_| open.set(false),
                        "Cancel"
                    }
                    button {
                        class: "alert-dialog-action",
                        disabled: is_cloning() || new_name().trim().is_empty(),
                        onclick: move |_| {
                            is_cloning.set(true);
                            let name = new_name().trim().to_string();
                            spawn(async move {
                                let s = match session.ensure_fresh(client).await {
                                    Ok(s) => s,
                                    Err(e) => {
                                        toast.error(
                                            e.to_user_message(),
                                            ToastOptions::default().duration(Duration::from_millis(3000)),
                                        );
                                        is_cloning.set(false);
                                        return;
                                    }
                                };
                                let body = HttpCloneDeck { new_name: name.clone() };
                                match client().clone_deck(source_deck_id, &body, &s).await {
                                    Ok(cloned) => {
                                        open.set(false);
                                        toast.info(
                                            format!("Cloned as '{name}'"),
                                            ToastOptions::default().duration(Duration::from_millis(2000)),
                                        );
                                        navigator.push(Router::ViewDeck { deck_id: cloned.deck_id });
                                    }
                                    Err(e) => {
                                        toast.error(
                                            e.to_user_message(),
                                            ToastOptions::default().duration(Duration::from_millis(3000)),
                                        );
                                        is_cloning.set(false);
                                    }
                                }
                            });
                        },
                        if is_cloning() { "Cloning..." } else { "Save" }
                    }
                }
            }
        }
    }
}
