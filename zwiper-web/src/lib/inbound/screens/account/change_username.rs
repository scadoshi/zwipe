//! The Change username sheet, a centered modal on a wide window.

use components::{BottomSheet, Button, ButtonVariant, TOAST_QUICK, TextInput};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpChangeUsername;

use crate::domain::validation;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::sheet::{SheetForm, SheetFrame};

/// New username and password. Every open starts empty; each field shows its
/// error line once it has held text or after a submit attempt. Enter runs
/// `Save changes`.
#[component]
pub(super) fn ChangeUsernameSheet(mut open: Signal<bool>) -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let authed = use_authed();
    let toast = use_toast();

    let mut new_username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut username_touched = use_signal(|| false);
    let mut password_touched = use_signal(|| false);
    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    use_effect(move || {
        if open() {
            new_username.set(String::new());
            password.set(String::new());
            username_touched.set(false);
            password_touched.set(false);
            submit_attempted.set(false);
        }
    });
    use_effect(move || {
        if !new_username().is_empty() {
            username_touched.set(true);
        }
    });
    use_effect(move || {
        if !password().is_empty() {
            password_touched.set(true);
        }
    });

    let attempted = submit_attempted();
    let username_error = (username_touched() || attempted)
        .then(|| validation::username_error(&new_username()))
        .flatten();
    let password_error = (password_touched() || attempted)
        .then(|| validation::required_password_error(&password()))
        .flatten();

    let mut attempt_submit = move || {
        if *is_loading.peek() {
            return;
        }
        submit_attempted.set(true);
        let typed = new_username.peek().clone();
        let typed_password = password.peek().clone();
        if validation::username_error(&typed).is_some()
            || validation::required_password_error(&typed_password).is_some()
        {
            return;
        }
        let request = HttpChangeUsername::new(&typed, &typed_password);
        is_loading.set(true);
        spawn(async move {
            if let Some(user) = authed
                .run("change_username", |c, s| async move {
                    c.change_username(request, &s).await
                })
                .await
            {
                let name = user.username.clone();
                let latest = session.peek().clone();
                if let Some(mut latest) = latest {
                    latest.user.username = user.username;
                    session.set(Some(latest));
                }
                toast.success(
                    format!("Username changed to {name}"),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                new_username.set(String::new());
                password.set(String::new());
                submit_attempted.set(false);
                open.set(false);
            }
            is_loading.set(false);
        });
    };

    rsx! {
        SheetFrame { form: SheetForm::Modal,
            BottomSheet {
                open,
                title: "Change username".to_string(),
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
                        onclick: move |_| attempt_submit(),
                        if is_loading() { "Saving..." } else { "Save changes" }
                    }
                },
                form {
                    class: "flex-col text-center",
                    onsubmit: move |event| {
                        event.prevent_default();
                        attempt_submit();
                    },
                    TextInput {
                        value: new_username,
                        id: "new_username",
                        label: "New username",
                        placeholder: "New username",
                        error: username_error,
                    }
                    TextInput {
                        value: password,
                        id: "password",
                        label: "Password",
                        placeholder: "Password",
                        input_type: "password",
                        error: password_error,
                    }
                    button { r#type: "submit", hidden: true }
                }
            }
        }
    }
}
