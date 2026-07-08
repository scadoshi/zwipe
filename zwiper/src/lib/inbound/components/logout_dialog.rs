use crate::{
    inbound::components::{
        alert_dialog::{
            AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
            AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
        },
        auth::signal_logout::SignalLogout,
    },
    outbound::client::ZwipeClient,
};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

/// Simple logout confirmation dialog.
#[component]
pub(crate) fn LogoutDialog(mut open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    rsx! {
        AlertDialogRoot {
            open: open(),
            on_open_change: move |v| open.set(v),
            AlertDialogContent {
                AlertDialogTitle { "Log out" }
                hr { class: "dialog-rule" }
                AlertDialogDescription { "Are you sure you want to log out?" }
                hr { class: "dialog-rule" }
                AlertDialogActions {
                    AlertDialogCancel {
                        on_click: move |_| open.set(false),
                        "Cancel"
                    }
                    AlertDialogAction {
                        danger: true,
                        on_click: move |_| session.logout(client),
                        "Log out"
                    }
                }
            }
        }
    }
}
