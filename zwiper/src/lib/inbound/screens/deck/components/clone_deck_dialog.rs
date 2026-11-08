//! The clone deck dialog: a new name, then the clone.

use components::{ActionButton, AlertDialog, DialogAction};
use dioxus::prelude::*;
use zwipe_core::domain::deck::deck_name;
use zwipe_ui::screens::deck_view::{self, Dialog};

use crate::inbound::screens::deck::view::Driver;

/// Clone deck dialog: prompts for a new name, clones through the deck view's
/// model, and the model opens the new deck on success.
#[component]
pub(crate) fn CloneDeckDialog(driver: Driver, default_name: String, open: Signal<bool>) -> Element {
    let mut new_name = use_signal(String::new);

    // Reseed the prefill with the latest source deck name whenever the dialog
    // reopens.
    use_effect(move || {
        // Opening and closing both start the form over.
        let _open = open();
        new_name.set(default_name.clone());
    });

    let name_empty = use_memo(move || !deck_view::can_clone(&new_name()));

    let save = ActionButton::new(deck_view::CLONE_SAVE)
        .disabled(name_empty)
        .busy(deck_view::CLONE_BUSY, move || {
            let name = new_name.peek().clone();
            driver.clone_deck(name)
        });

    rsx! {
        AlertDialog {
            open,
            title: Dialog::Clone.title().to_string(),
            actions: vec![DialogAction::Dismiss(deck_view::CANCEL.into()), save.into()],
            {deck_view::CLONE_BODY}
            CloneNameInput { name: new_name }
        }
    }
}

/// The new name field and its line. Its own component so it reads the name
/// where the dialog host draws it.
#[component]
fn CloneNameInput(mut name: Signal<String>) -> Element {
    let error = deck_view::clone_name_error(&name());
    let error_shown = error.is_some();
    rsx! {
        input {
            r#type: "text",
            class: if error_shown { "input input-error" } else { "input" },
            placeholder: deck_view::CLONE_PLACEHOLDER,
            maxlength: deck_name::MAX_LENGTH.to_string(),
            value: "{name}",
            autocapitalize: "none",
            autocorrect: "off",
            spellcheck: "false",
            oninput: move |evt| name.set(evt.value()),
        }
        if let Some(error) = error {
            div { class: "message-error", "{error}" }
        }
    }
}
