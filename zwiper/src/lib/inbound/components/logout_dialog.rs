//! Logout confirmation dialog.

use components::{AlertDialog, DialogAction};
use dioxus::prelude::*;
use zwipe_ui::screens::account::{CANCEL, Dialog};

/// Simple logout confirmation dialog. `on_confirm` logs out.
#[component]
pub(crate) fn LogoutDialog(open: Signal<bool>, on_confirm: Callback) -> Element {
    let dialog = Dialog::LogOut;
    rsx! {
        AlertDialog {
            open,
            title: dialog.title(),
            actions: vec![
                DialogAction::Dismiss(CANCEL.into()),
                DialogAction::Confirm {
                    label: dialog.confirm().into(),
                    danger: true,
                    on_confirm,
                },
            ],
            {dialog.body()}
        }
    }
}
