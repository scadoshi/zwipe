//! The Change password sheet, a centered modal on a wide window, with its
//! confirm dialog stacked on top.

use components::{
    AlertDialog, BottomSheet, Button, ButtonVariant, DialogAction, TOAST_QUICK, TextInput,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::http::contracts::auth::HttpChangePassword;

use crate::domain::validation;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::hint_topic::{self, HintTopic};
use crate::inbound::chrome::sheet::{SheetForm, SheetFrame};

/// Current, new and confirm. Only the new password is judged, once it or the
/// confirmation has held text or after an attempt; the current one never
/// is, so a password from before the policy can still be changed. `Save
/// changes` (or Enter) opens the confirm dialog, whose `Confirm` checks again
/// and sends. This device stays signed in.
#[component]
pub(super) fn ChangePasswordSheet(mut open: Signal<bool>) -> Element {
    let authed = use_authed();
    let toast = use_toast();

    let mut current = use_signal(String::new);
    let mut new_password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut touched = use_signal(|| false);
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
            touched.set(false);
            submit_attempted.set(false);
        }
    });
    use_effect(move || {
        if !new_password().is_empty() || !confirm().is_empty() {
            touched.set(true);
        }
    });

    let error_now = move |attempted: bool| {
        validation::new_password_error(
            &current.peek(),
            &new_password.peek(),
            &confirm.peek(),
            attempted,
        )
    };
    let attempted = submit_attempted();
    let password_error = (touched() || attempted)
        .then(|| validation::new_password_error(&current(), &new_password(), &confirm(), attempted))
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
                    "Password change successful".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                clear();
                submit_attempted.set(false);
                open.set(false);
            }
            is_loading.set(false);
        });
    });

    rsx! {
        SheetFrame { form: SheetForm::Modal,
            BottomSheet {
                open,
                title: "Change password".to_string(),
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Action,
                        disabled: is_loading(),
                        onclick: move |_| open.set(false),
                        "Back"
                    }
                    Button {
                        variant: ButtonVariant::Action,
                        disabled: is_loading(),
                        onclick: move |_| ask_to_confirm(),
                        if is_loading() { "Saving..." } else { "Save changes" }
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
                        label: "Current password",
                        placeholder: "Current password",
                        input_type: "password",
                    }
                    TextInput {
                        value: new_password,
                        id: "new_password",
                        label: "New password",
                        placeholder: "New password",
                        input_type: "password",
                        error: password_error,
                        on_info: hint_topic::open_topic(HintTopic::Password),
                    }
                    TextInput {
                        value: confirm,
                        id: "confirm_password",
                        label: "Confirm password",
                        placeholder: "Confirm new",
                        input_type: "password",
                    }
                    button { r#type: "submit", hidden: true }
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
