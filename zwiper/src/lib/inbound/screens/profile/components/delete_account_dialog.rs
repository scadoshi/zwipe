use crate::inbound::components::{
    auth::{authed::use_authed, signal_logout::SignalLogout},
    fields::text_input::TextInput,
    telemetry::vocabulary::{ProfileScreen, Screen, component},
};
use dioxus::prelude::*;
use std::time::Duration;
use zwipe_client::ZwipeClient;
use zwipe_components::{ActionButton, AlertDialog, DialogAction};
use zwipe_core::{
    domain::auth::models::{secret::Secret, session::Session},
    http::contracts::auth::HttpDeleteUser,
};

/// Delete account dialog with 5-second countdown, password confirmation, and deletion logic.
#[component]
pub(crate) fn DeleteAccountDialog(open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let authed = use_authed(Screen::Profile(ProfileScreen::Main));

    let mut delete_countdown = use_signal(|| 5u8);
    let mut delete_password = use_signal(String::new);

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

    let delete_label = use_memo(move || match delete_countdown() {
        0 => "Delete".to_string(),
        n => format!("Delete ({n})"),
    });
    let counting = use_memo(move || delete_countdown() > 0);

    let delete = ActionButton::new("Delete")
        .danger()
        .live_label(delete_label)
        .disabled(counting)
        .busy("Deleting...", move || {
            let password = delete_password();
            async move {
                let deleted = authed
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
                    .is_some();
                if deleted {
                    session.logout(client);
                }
                deleted
            }
        });

    rsx! {
        AlertDialog {
            open,
            title: "Delete account",
            actions: vec![DialogAction::Dismiss("Cancel".into()), delete.into()],
            "This will permanently delete your account, all decks, and all card data. This cannot be undone."
            TextInput {
                value: delete_password,
                input_type: "password".to_string(),
                placeholder: "Password".to_string(),
            }
        }
    }
}
