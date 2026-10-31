//! The Delete account dialog: a five second countdown, the password, then
//! the delete.

use std::time::Duration;

use components::{ActionButton, AlertDialog, DialogAction, TextInput};
use dioxus::core::Task;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::secret::Secret;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpDeleteUser;

use crate::domain::account::{self, DELETE_COUNTDOWN_SECS};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::session::logout;
use crate::outbound::timer;

/// Counts down from `from` to 0, a step a second, each time `open` turns
/// true. Closing or reopening cancels the running count, so an older one
/// cannot reach 0 early.
fn use_open_countdown(open: Signal<bool>, from: u8) -> Signal<u8> {
    let mut remaining = use_signal(|| from);
    let mut running: Signal<Option<Task>> = use_signal(|| None);
    use_effect(move || {
        let is_open = open();
        if let Some(task) = running.write().take() {
            task.cancel();
        }
        if is_open {
            remaining.set(from);
            let task = spawn(async move {
                for left in (0..from).rev() {
                    timer::sleep(Duration::from_secs(1)).await;
                    remaining.set(left);
                }
            });
            running.set(Some(task));
        }
    });
    remaining
}

/// The dialog. Delete stays disabled through the countdown but not for an
/// empty password, which the server rejects. A delete that goes through logs
/// out, and the auth gate shows Login; one that fails toasts and leaves the
/// dialog open.
#[component]
pub(super) fn DeleteAccountDialog(open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let authed = use_authed();
    let countdown = use_open_countdown(open, DELETE_COUNTDOWN_SECS);
    let mut password = use_signal(String::new);

    use_effect(move || {
        if open() {
            password.set(String::new());
        }
    });

    let label = use_memo(move || account::delete_label(countdown()));
    let counting = use_memo(move || countdown() > 0);
    let delete = ActionButton::new("Delete")
        .danger()
        .live_label(label)
        .disabled(counting)
        .busy("Deleting...", move || {
            let password = password();
            async move {
                let deleted = authed
                    .run("delete_account", |c, s| async move {
                        c.delete_user(
                            HttpDeleteUser {
                                password: Secret::new(password),
                            },
                            &s,
                        )
                        .await
                    })
                    .await
                    .is_some();
                if deleted {
                    logout::logout(session, client);
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
                value: password,
                input_type: "password".to_string(),
                placeholder: "Password".to_string(),
            }
        }
    }
}
