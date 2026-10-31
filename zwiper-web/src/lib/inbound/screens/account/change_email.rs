//! The Change email sheet, a centered modal on a wide window.

use components::{BottomSheet, Button, ButtonVariant, TOAST_NORMAL, TextInput};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpChangeEmail;

use crate::domain::validation;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::sheet::{SheetForm, SheetFrame};

/// New email and password. The answer is the same whether or not the
/// address is free; a success puts it under `Pending` until its link is
/// followed. Every open starts empty; each field shows its
/// error line once it has held text or after a submit attempt. Enter runs
/// `Save changes`.
#[component]
pub(super) fn ChangeEmailSheet(mut open: Signal<bool>) -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let authed = use_authed();
    let toast = use_toast();

    let mut new_email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut email_touched = use_signal(|| false);
    let mut password_touched = use_signal(|| false);
    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    use_effect(move || {
        if open() {
            new_email.set(String::new());
            password.set(String::new());
            email_touched.set(false);
            password_touched.set(false);
            submit_attempted.set(false);
        }
    });
    use_effect(move || {
        if !new_email().is_empty() {
            email_touched.set(true);
        }
    });
    use_effect(move || {
        if !password().is_empty() {
            password_touched.set(true);
        }
    });

    let attempted = submit_attempted();
    let email_error = (email_touched() || attempted)
        .then(|| validation::email_error(&new_email()))
        .flatten();
    let password_error = (password_touched() || attempted)
        .then(|| validation::required_password_error(&password()))
        .flatten();

    let mut attempt_submit = move || {
        if *is_loading.peek() {
            return;
        }
        submit_attempted.set(true);
        let typed = new_email.peek().clone();
        let typed_password = password.peek().clone();
        if validation::email_error(&typed).is_some()
            || validation::required_password_error(&typed_password).is_some()
        {
            return;
        }
        let request = HttpChangeEmail::new(&typed, &typed_password);
        is_loading.set(true);
        spawn(async move {
            if let Some(user) = authed
                .run("change_email", |c, s| async move {
                    c.change_email(request, &s).await
                })
                .await
            {
                let latest = session.peek().clone();
                if let Some(mut latest) = latest {
                    latest.user = user;
                    session.set(Some(latest));
                }
                toast.success(
                    format!("Check {typed} for a link to confirm the change"),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
                new_email.set(String::new());
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
                title: "Change email".to_string(),
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
                        value: new_email,
                        id: "new_email",
                        label: "New email",
                        placeholder: "New email",
                        error: email_error,
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
