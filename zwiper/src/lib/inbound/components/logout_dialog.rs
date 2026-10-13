use crate::inbound::components::auth::signal_logout::SignalLogout;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_components::{AlertDialog, DialogAction};
use zwipe_core::domain::auth::models::session::Session;

/// Simple logout confirmation dialog.
#[component]
pub(crate) fn LogoutDialog(open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let log_out = use_callback(move |()| session.logout(client));

    rsx! {
        AlertDialog {
            open,
            title: "Log out",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Log out".into(),
                    danger: true,
                    on_confirm: log_out,
                },
            ],
            "Are you sure you want to log out?"
        }
    }
}
