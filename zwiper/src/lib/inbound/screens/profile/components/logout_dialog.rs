use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogRoot, AlertDialogTitle, AlertDialogDescription,
};
use crate::inbound::components::auth::signal_logout::SignalLogout;
use crate::outbound::client::ZwipeClient;
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

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
                AlertDialogTitle { "logout" }
                AlertDialogDescription { "are you sure you want to logout?" }
                AlertDialogActions {
                    AlertDialogCancel {
                        on_click: move |_| open.set(false),
                        "cancel"
                    }
                    AlertDialogAction {
                        on_click: move |_| session.logout(client),
                        "logout"
                    }
                }
            }
        }
    }
}
