//! The Change email sheet, a centered modal on a wide window.

use components::icons::{CHECK, DockIcon};
use components::{BottomSheet, SheetFrame, SheetWide, TOAST_NORMAL, TextInput};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::email;
use zwipe_core::http::contracts::auth::HttpChangeEmail;
use zwipe_ui::account;
use zwipe_ui::text::sentence_case;

use crate::inbound::chrome::authed::use_authed;

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
    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    use_effect(move || {
        if open() {
            new_email.set(String::new());
            password.set(String::new());
            submit_attempted.set(false);
        }
    });

    let attempted = submit_attempted();
    // A field is judged while it holds text, and every field once a submit
    // was tried.
    let email_error = (!new_email().is_empty() || attempted)
        .then(|| account::email_error(&new_email()))
        .flatten();
    let password_error = (!password().is_empty() || attempted)
        .then(|| account::password_required(&password()))
        .flatten();

    let mut attempt_submit = move || {
        if *is_loading.peek() {
            return;
        }
        submit_attempted.set(true);
        let typed = new_email.peek().clone();
        let typed_password = password.peek().clone();
        if account::email_error(&typed).is_some()
            || account::password_required(&typed_password).is_some()
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
        SheetFrame { wide: SheetWide::Drawer,
            BottomSheet {
                open,
                title: "Change email".to_string(),
                closable: true,
                form {
                    class: "flex-col text-center",
                    onsubmit: move |event| {
                        event.prevent_default();
                        attempt_submit();
                    },
                    TextInput {
                        value: new_email,
                        id: "new_email",
                        input_type: "email",
                        name: "email",
                        autocomplete: "email",
                        maxlength: email::MAX_LENGTH,
                        label: "New email",
                        placeholder: "New email",
                        error: email_error.as_deref().map(sentence_case),
                    }
                    TextInput {
                        value: password,
                        id: "email_password",
                        name: "password",
                        autocomplete: "current-password",
                        label: "Password",
                        placeholder: "Password",
                        input_type: "password",
                        error: password_error.as_deref().map(sentence_case),
                    }
                    button { r#type: "submit", hidden: true }
                }
                // Save floats at the sheet's bottom right.
                div { class: "action-dock action-dock-right sheet-dock",
                    button {
                        class: "action-dock-btn",
                        "aria-label": if is_loading() { "Saving…" } else { "Save changes" },
                        disabled: is_loading(),
                        onclick: move |_| attempt_submit(),
                        DockIcon { path: CHECK }
                    }
                }
            }
        }
    }
}
