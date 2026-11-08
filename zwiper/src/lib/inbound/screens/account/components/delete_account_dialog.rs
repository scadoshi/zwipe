//! The delete account dialog: a countdown, the password, then the delete.

use components::{ActionButton, AlertDialog, DialogAction, TextInput};
use dioxus::prelude::*;
use zwipe_ui::screens::account::{Action, CANCEL, DELETE_PASSWORD, DELETING, Dialog};

use crate::inbound::screens::account::driver::Driver;

/// Delete account dialog with 5-second countdown, password confirmation, and deletion logic.
#[component]
pub(crate) fn DeleteAccountDialog(open: Signal<bool>) -> Element {
    let driver: Driver = use_context();
    let mut delete_password = use_signal(String::new);

    // Clear the password each time the dialog opens; stop the countdown each
    // time it closes.
    use_effect(move || {
        if open() {
            delete_password.set(String::new());
        } else {
            driver.dispatch(Action::CancelDelete);
        }
    });

    let delete_label = use_memo(move || driver.model.read().delete_label());
    let counting = use_memo(move || driver.model.read().delete_secs() > 0);

    let dialog = Dialog::DeleteAccount;
    let delete = ActionButton::new(dialog.confirm())
        .danger()
        .live_label(delete_label)
        .disabled(counting)
        .busy(DELETING, move || {
            let password = delete_password();
            async move { driver.delete(password).await }
        });

    rsx! {
        AlertDialog {
            open,
            title: dialog.title(),
            actions: vec![DialogAction::Dismiss(CANCEL.into()), delete.into()],
            {dialog.body()}
            TextInput {
                value: delete_password,
                autocomplete: "current-password",
                input_type: "password".to_string(),
                placeholder: DELETE_PASSWORD.to_string(),
            }
        }
    }
}
