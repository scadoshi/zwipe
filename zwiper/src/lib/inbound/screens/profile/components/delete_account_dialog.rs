use crate::inbound::components::{
    auth::{authed::use_authed, signal_logout::SignalLogout},
    fields::text_input::TextInput,
    telemetry::vocabulary::{ProfileScreen, Screen, component},
};
use dioxus::prelude::*;
use std::time::Duration;
use zwipe_client::ZwipeClient;
use zwipe_components::{AlertDialog, DialogAction};
use zwipe_core::{
    domain::auth::models::{secret::Secret, session::Session},
    http::contracts::auth::HttpDeleteUser,
};

/// Delete account dialog with 5-second countdown, password confirmation, and deletion logic.
/// The Delete button sits in the body, since it stays disabled through the
/// countdown and keeps the dialog open while the request runs.
#[component]
pub(crate) fn DeleteAccountDialog(mut open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let authed = use_authed(Screen::Profile(ProfileScreen::Main));

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

    let delete_account = use_callback(move |()| {
        is_deleting.set(true);
        let password = delete_password();
        spawn(async move {
            if authed
                .run_at(
                    component::DELETE_ACCOUNT_DIALOG,
                    "delete_account",
                    |c, s| async move {
                        c.delete_user(
                            HttpDeleteUser {
                                password: Secret::new(password),
                            },
                            &s,
                        )
                        .await
                    },
                )
                .await
                .is_some()
            {
                open.set(false);
                session.logout(client);
            }
            is_deleting.set(false);
        });
    });

    rsx! {
        AlertDialog {
            open,
            title: "Delete account",
            actions: vec![DialogAction::Dismiss("Cancel".into())],
            DeleteAccountBody {
                password: delete_password,
                countdown: delete_countdown,
                is_deleting,
                on_delete: delete_account,
            }
        }
    }
}

/// The warning, the password field and the Delete button. Its own component so
/// the countdown and the field update where the dialog host draws them.
#[component]
fn DeleteAccountBody(
    password: Signal<String>,
    countdown: Signal<u8>,
    is_deleting: Signal<bool>,
    on_delete: Callback<()>,
) -> Element {
    rsx! {
        "This will permanently delete your account, all decks, and all card data. This cannot be undone."
        TextInput {
            value: password,
            input_type: "password".to_string(),
            placeholder: "Password".to_string(),
        }
        div { class: "alert-dialog-actions",
            button {
                class: "alert-dialog-action-danger",
                r#type: "button",
                disabled: countdown() > 0 || is_deleting(),
                onclick: move |_| on_delete.call(()),
                if countdown() > 0 {
                    "Delete ({countdown()})"
                } else if is_deleting() {
                    "Deleting..."
                } else {
                    "Delete"
                }
            }
        }
    }
}
