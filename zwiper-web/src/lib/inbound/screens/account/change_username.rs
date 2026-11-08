//! The Change username sheet, a centered modal on a wide window.

use components::icons::{CHECK, DockIcon};
use components::{BottomSheet, SheetFrame, SheetWide, TOAST_QUICK, TextInput};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::username;
use zwipe_core::http::contracts::auth::HttpChangeUsername;
use zwipe_ui::account;
use zwipe_ui::text::sentence_case;

use crate::inbound::chrome::authed::use_authed;

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
    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    use_effect(move || {
        // Opening and closing both start the form over.
        let _open = open();
        new_username.set(String::new());
        password.set(String::new());
        submit_attempted.set(false);
    });

    let attempted = submit_attempted();
    // A field is judged while it holds text, and every field once a submit
    // was tried.
    let username_error = (!new_username().is_empty() || attempted)
        .then(|| account::username_error(&new_username()))
        .flatten();
    let password_error = (!password().is_empty() || attempted)
        .then(|| account::password_required(&password()))
        .flatten();

    let mut attempt_submit = move || {
        if *is_loading.peek() {
            return;
        }
        submit_attempted.set(true);
        let typed = new_username.peek().clone();
        let typed_password = password.peek().clone();
        if account::username_error(&typed).is_some()
            || account::password_required(&typed_password).is_some()
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
        SheetFrame { wide: SheetWide::Drawer,
            BottomSheet {
                open,
                title: "Change username".to_string(),
                closable: true,
                form {
                    class: "flex-col text-center",
                    onsubmit: move |event| {
                        event.prevent_default();
                        attempt_submit();
                    },
                    TextInput {
                        value: new_username,
                        id: "new_username",
                        name: "username",
                        autocomplete: "username",
                        maxlength: username::MAX_LENGTH,
                        label: "New username",
                        placeholder: "New username",
                        error: username_error.as_deref().map(sentence_case),
                    }
                    TextInput {
                        value: password,
                        id: "username_password",
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
                        // An error line showing holds Save; the password's never does.
                        disabled: is_loading() || username_error.is_some(),
                        onclick: move |_| attempt_submit(),
                        DockIcon { path: CHECK }
                    }
                }
            }
        }
    }
}
