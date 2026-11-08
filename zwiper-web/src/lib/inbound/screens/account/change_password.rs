//! The Change password sheet, a centered modal on a wide window, with its
//! confirm dialog stacked on top.

use components::icons::{CHECK, DockIcon};
use components::{
    AlertDialog, BottomSheet, Button, ButtonVariant, DialogAction, SheetFrame, SheetWide,
    TOAST_NORMAL, TOAST_QUICK, TextInput,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::auth::password;
use zwipe_core::http::contracts::auth::{HttpChangePassword, HttpRequestPasswordReset};
use zwipe_ui::account;
use zwipe_ui::screens::account::{FORGOT_PASSWORD, RESET_LINK_SENT};
use zwipe_ui::text::sentence_case;

use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::hint_topic::{self, HintTopic};

/// Current, new and confirm. Only the new password is judged, once it or the
/// confirmation has held text or after an attempt; the current one never
/// is, so a password from before the policy can still be changed. `Save
/// changes` (or Enter) opens the confirm dialog, whose `Confirm` checks again
/// and sends. This device stays signed in. `Forgot password` in the header
/// emails the account a reset link, as the signed-out screen does, for a
/// user who no longer knows the current one.
#[component]
pub(super) fn ChangePasswordSheet(mut open: Signal<bool>) -> Element {
    let authed = use_authed();
    let toast = use_toast();
    let client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();

    let mut current = use_signal(String::new);
    let mut new_password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut submit_attempted = use_signal(|| false);
    let mut show_confirm = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    let mut clear = move || {
        current.set(String::new());
        new_password.set(String::new());
        confirm.set(String::new());
    };
    use_effect(move || {
        if open() {
            clear();
            submit_attempted.set(false);
        }
    });

    let error_now = move |attempted: bool| {
        account::change_password_error(
            &current.peek(),
            &new_password.peek(),
            &confirm.peek(),
            attempted,
        )
    };
    let attempted = submit_attempted();
    // The line is judged while either box holds text, and once a submit was
    // tried.
    let password_error = (!new_password().is_empty() || !confirm().is_empty() || attempted)
        .then(|| account::change_password_error(&current(), &new_password(), &confirm(), attempted))
        .flatten();

    let mut ask_to_confirm = move || {
        if *is_loading.peek() {
            return;
        }
        submit_attempted.set(true);
        if error_now(true).is_none() {
            show_confirm.set(true);
        }
    };

    let send = use_callback(move |()| {
        submit_attempted.set(true);
        if error_now(true).is_some() {
            return;
        }
        let request = HttpChangePassword::new(&current.peek(), &new_password.peek());
        is_loading.set(true);
        spawn(async move {
            if authed
                .run("change_password", |c, s| async move {
                    c.change_password(request, &s).await
                })
                .await
                .is_some()
            {
                toast.success(
                    "Password changed".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                clear();
                submit_attempted.set(false);
                open.set(false);
            }
            is_loading.set(false);
        });
    });

    let mut sending_reset = use_signal(|| false);
    let forgot = use_callback(move |()| {
        if *sending_reset.peek() {
            return;
        }
        let Some(email) = session.peek().as_ref().map(|s| s.user.email.to_string()) else {
            return;
        };
        sending_reset.set(true);
        let request = HttpRequestPasswordReset::new(&email);
        spawn(async move {
            let client = client.peek().clone();
            match client.request_password_reset(request).await {
                Ok(()) => {
                    toast.success(
                        format!("{RESET_LINK_SENT} {email}"),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                    open.set(false);
                }
                Err(e) => {
                    tracing::warn!("password reset request failed: {e}");
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                }
            }
            sending_reset.set(false);
        });
    });

    rsx! {
        SheetFrame { wide: SheetWide::Drawer,
            BottomSheet {
                open,
                title: "Change password".to_string(),
                closable: true,
                trailing: rsx! {
                    Button {
                        variant: ButtonVariant::Action,
                        class: "page-header-corner",
                        disabled: sending_reset(),
                        onclick: move |_| forgot.call(()),
                        {FORGOT_PASSWORD}
                    }
                },
                form {
                    class: "flex-col text-center",
                    onsubmit: move |event| {
                        event.prevent_default();
                        ask_to_confirm();
                    },
                    TextInput {
                        value: current,
                        id: "current_password",
                        name: "current-password",
                        autocomplete: "current-password",
                        label: "Current password",
                        placeholder: "Current password",
                        input_type: "password",
                    }
                    TextInput {
                        value: new_password,
                        id: "new_password",
                        name: "new-password",
                        autocomplete: "new-password",
                        maxlength: password::MAX_LENGTH,
                        label: "New password",
                        placeholder: "New password",
                        input_type: "password",
                        error: password_error.as_deref().map(sentence_case),
                        on_info: hint_topic::open_topic(HintTopic::Password),
                    }
                    TextInput {
                        value: confirm,
                        id: "confirm_password",
                        name: "confirm-password",
                        autocomplete: "new-password",
                        maxlength: password::MAX_LENGTH,
                        label: "Confirm password",
                        placeholder: "Confirm new",
                        input_type: "password",
                    }
                    button { r#type: "submit", hidden: true }
                }
                // Save floats at the sheet's bottom right.
                div { class: "action-dock action-dock-right sheet-dock",
                    button {
                        class: "action-dock-btn",
                        "aria-label": if is_loading() { "Saving…" } else { "Save changes" },
                        disabled: is_loading(),
                        onclick: move |_| ask_to_confirm(),
                        DockIcon { path: CHECK }
                    }
                }
            }
        }
        AlertDialog {
            open: show_confirm,
            title: "Change password",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Confirm".into(),
                    danger: false,
                    on_confirm: send,
                },
            ],
            "Changing your password will log you out on all other devices."
        }
    }
}
