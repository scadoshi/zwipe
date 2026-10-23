//! Change email bottom sheet.

use components::{BottomSheet, Button, ButtonVariant, TOAST_NORMAL, TextInput};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::Email;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpChangeEmail;

use crate::domain::error::UserFacing;
use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::telemetry::vocabulary::{ProfileScreen, Screen};

/// Bottom sheet for updating the user's email address.
#[component]
pub fn ChangeEmailSheet(mut open: Signal<bool>) -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let authed = use_authed(Screen::Profile(ProfileScreen::ChangeEmail));

    let mut new_email = use_signal(String::new);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut email_touched = use_signal(|| false);
    let mut validate_email = move || {
        if let Err(e) = Email::new(new_email()) {
            email_error.set(Some(e.to_user_facing_string()));
        } else {
            email_error.set(None)
        }
    };

    let mut password = use_signal(String::new);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut password_touched = use_signal(|| false);
    let mut validate_password = move || {
        if password().is_empty() {
            password_error.set(Some("Password is required".to_string()));
        } else {
            password_error.set(None);
        }
    };

    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);
    let toast = use_toast();

    let mut inputs_are_valid = move || {
        validate_email();
        validate_password();
        email_error().is_none() && password_error().is_none()
    };

    let mut clear_inputs = move || {
        new_email.set(String::new());
        password.set(String::new());
    };

    // Reset the form each time the sheet opens so it never shows stale input.
    use_effect(move || {
        if open() {
            clear_inputs();
            email_error.set(None);
            password_error.set(None);
            email_touched.set(false);
            password_touched.set(false);
            submit_attempted.set(false);
        }
    });

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        if inputs_are_valid() {
            tracing::info!("change email");
            let request = HttpChangeEmail::new(&new_email(), &password());
            is_loading.set(true);
            spawn(async move {
                if let Some(updated_user) = authed
                    .run("change_email", |c, s| async move {
                        c.change_email(request, &s).await
                    })
                    .await
                {
                    // The typed address, not the response's: the response
                    // reads the same whether or not the address is free.
                    let requested = new_email.peek().clone();
                    // Take the whole user so `pending_email` shows on
                    // Profile. Re-read rather than mutate the captured
                    // session (the facade consumed it).
                    let current = session.peek().clone();
                    if let Some(mut current) = current {
                        current.user = updated_user;
                        session.set(Some(current));
                    }
                    toast.success(
                        format!("Check {requested} for a link to confirm the change"),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                    clear_inputs();
                    submit_attempted.set(false);
                    open.set(false);
                }
                is_loading.set(false);
            });
        }
    };

    use_effect(move || {
        let value = new_email();
        if !value.is_empty() && !email_touched() {
            email_touched.set(true);
        }
        if email_touched() || submit_attempted() {
            validate_email();
        }
    });

    use_effect(move || {
        let value = password();
        if !value.is_empty() && !password_touched() {
            password_touched.set(true);
        }
        if password_touched() || submit_attempted() {
            validate_password();
        }
    });

    rsx! {
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

            div { class: "flex-col text-center",

                TextInput {
                    value: new_email,
                    id: "new_email",
                    label: "New email",
                    placeholder: "New email",
                    error: email_error(),
                }

                TextInput {
                    value: password,
                    id: "password",
                    label: "Password",
                    placeholder: "Password",
                    input_type: "password",
                    error: password_error(),
                }
            }
        }
    }
}
