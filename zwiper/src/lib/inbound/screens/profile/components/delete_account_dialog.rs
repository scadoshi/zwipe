use crate::inbound::components::alert_dialog::{
    AlertDialogActions, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogRoot, AlertDialogTitle,
};
use crate::inbound::components::auth::{ensure_session::EnsureFresh, signal_logout::SignalLogout};
use crate::inbound::components::fields::text_input::TextInput;
use crate::outbound::client::{user::delete_user::ClientDeleteUser, ZwipeClient};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpDeleteUser;

/// Delete account dialog with 5-second countdown, password confirmation, and deletion logic.
#[component]
pub(crate) fn DeleteAccountDialog(mut open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    let mut delete_countdown = use_signal(|| 5u8);
    let mut delete_password = use_signal(String::new);
    let mut is_deleting = use_signal(|| false);

    // Reset state and start countdown when opened
    use_effect(move || {
        if open() {
            delete_password.set(String::new());
            delete_countdown.set(5);
            spawn(async move {
                for i in (0..5u8).rev() {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    delete_countdown.set(i);
                }
            });
        }
    });

    rsx! {
        AlertDialogRoot {
            open: open(),
            on_open_change: move |v: bool| {
                if !v { open.set(false); }
            },
            AlertDialogContent {
                AlertDialogTitle { "Delete account" }
                AlertDialogDescription {
                    "This will permanently delete your account, all decks, and all card data. This cannot be undone."
                }
                TextInput {
                    value: delete_password,
                    input_type: "password".to_string(),
                    placeholder: "Confirm your password".to_string(),
                }
                AlertDialogActions {
                    AlertDialogCancel {
                        on_click: move |_| open.set(false),
                        "Cancel"
                    }
                    button {
                        class: "alert-dialog-action-danger",
                        disabled: delete_countdown() > 0 || is_deleting(),
                        onclick: move |_| {
                            is_deleting.set(true);
                            let password = delete_password();
                            spawn(async move {
                                let s = match session.ensure_fresh(client).await {
                                    Ok(s) => s,
                                    Err(e) => {
                                        toast.error(e.to_user_message(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                        is_deleting.set(false);
                                        return;
                                    }
                                };
                                match client().delete_user(HttpDeleteUser { password }, &s).await {
                                    Ok(()) => {
                                        open.set(false);
                                        session.logout(client);
                                    }
                                    Err(e) => {
                                        toast.error(e.to_user_message(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                        is_deleting.set(false);
                                    }
                                }
                            });
                        },
                        if delete_countdown() > 0 {
                            "Delete ({delete_countdown()})"
                        } else if is_deleting() {
                            "Deleting..."
                        } else {
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
