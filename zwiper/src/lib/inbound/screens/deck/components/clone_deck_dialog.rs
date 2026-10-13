use crate::inbound::{
    components::{
        auth::authed::use_authed,
        telemetry::vocabulary::{DeckScreen, Screen, component},
    },
    router::Router,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_components::{ActionButton, AlertDialog, DialogAction, TOAST_QUICK};
use zwipe_core::http::contracts::deck::HttpCloneDeck;

/// Clone deck dialog: prompts for a new name, calls the clone endpoint,
/// and navigates to the new deck on success.
#[component]
pub(crate) fn CloneDeckDialog(
    source_deck_id: Uuid,
    default_name: String,
    open: Signal<bool>,
) -> Element {
    let navigator = use_navigator();
    let toast = use_toast();
    let authed = use_authed(Screen::Deck(DeckScreen::View));

    let mut new_name = use_signal(String::new);

    // Reseed the prefill with the latest source deck name whenever the dialog
    // reopens.
    use_effect(move || {
        if open() {
            new_name.set(default_name.clone());
        }
    });

    let name_empty = use_memo(move || new_name().trim().is_empty());

    let save = ActionButton::new("Save")
        .disabled(name_empty)
        .busy("Cloning...", move || {
            let name = new_name.peek().trim().to_string();
            async move {
                let body = HttpCloneDeck {
                    new_name: name.clone(),
                };
                let Some(cloned) = authed
                    .run_at(
                        component::CLONE_DECK_DIALOG,
                        "clone_deck",
                        |c, s| async move { c.clone_deck(source_deck_id, &body, &s).await },
                    )
                    .await
                else {
                    return false;
                };
                toast.info(
                    format!("Cloned as \"{name}\""),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                navigator.push(Router::ViewDeck {
                    deck_id: cloned.deck_id,
                });
                true
            }
        });

    rsx! {
        AlertDialog {
            open,
            title: "Clone deck",
            actions: vec![DialogAction::Dismiss("Cancel".into()), save.into()],
            "Make an exact copy of your deck under a new name."
            CloneNameInput { name: new_name }
        }
    }
}

/// The new name field. Its own component so it reads the name where the
/// dialog host draws it.
#[component]
fn CloneNameInput(mut name: Signal<String>) -> Element {
    rsx! {
        input {
            r#type: "text",
            class: "input",
            placeholder: "New deck name",
            value: "{name}",
            autocapitalize: "none",
            autocorrect: "off",
            spellcheck: "false",
            oninput: move |evt| name.set(evt.value()),
        }
    }
}
