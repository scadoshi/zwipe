use components::{HintKey, InfoButton, Panel, PasswordInput};
use dioxus::prelude::*;
use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::secret::Secret;
use zwipe_ui::account;
use zwipe_ui::text::sentence_case;
use zwipe_ui::write_repeat::{self, FormAnswer, FormNote, FormRepeat};

use crate::components::PageMeta;
use crate::{Nav, api};

#[derive(Clone, PartialEq)]
enum ResetState {
    Form,
    /// The check on load found the token used or expired.
    DeadLink,
    Loading,
    Success,
    Error(String),
}

/// What a submit does.
#[derive(Debug, PartialEq)]
enum SubmitCheck {
    /// A reset is already out: ignore the submit, the token is single use.
    Ignore,
    /// Show this error instead of sending.
    Reject(String),
    /// Send the reset.
    Send,
}

/// The line under the fields while typing: the first password rule the new
/// password breaks, or a confirmation as long as it that doesn't match. The
/// app's check, from zwipe-ui, so the website and the app judge alike.
fn live_error(pw: &str, cf: &str) -> Option<String> {
    account::password_error(pw, cf, false)
}

/// Decides a submit from the page state, the two fields and the link's token.
/// An empty password, one that breaks a rule of the policy, or a confirmation
/// that doesn't match is held with its line.
fn check_submit(state: &ResetState, pw: &str, cf: &str, token: &str) -> SubmitCheck {
    if *state == ResetState::Loading {
        return SubmitCheck::Ignore;
    }
    if let Some(line) = account::password_error(pw, cf, true) {
        return SubmitCheck::Reject(line);
    }
    if token.is_empty() {
        return SubmitCheck::Reject(
            "This link is missing its code. Open it from the email again.".to_string(),
        );
    }
    SubmitCheck::Send
}

/// How a reset failed: a dead link (401) is the refusal a repeat gets once an earlier, lost try
/// used the link.
fn reset_answer(error: &ClientError) -> FormAnswer {
    if matches!(error, ClientError::Network(_)) {
        FormAnswer::Network
    } else if matches!(error, ClientError::Unauthorized(_)) {
        FormAnswer::RepeatRefused
    } else {
        FormAnswer::Other
    }
}

/// The line a failed reset shows: after a lost answer, or for the dead link its retry then finds
/// (`note`), to try the new password; else the server's reason for a password it turns down
/// (422), the connection message for a request that never landed, and a dead link for anything
/// else.
fn failure_message(error: &ClientError, note: Option<FormNote>) -> String {
    if note.is_some() {
        return write_repeat::RESET_UNCONFIRMED.to_owned();
    }
    match error {
        ClientError::UnprocessableEntity(reason) => format!("Password {reason}"),
        ClientError::Network(_) | ClientError::Decode(_) => error.to_user_message(),
        ClientError::BadRequest(_)
        | ClientError::Unauthorized(_)
        | ClientError::Forbidden(_)
        | ClientError::NotFound(_)
        | ClientError::Conflict(_)
        | ClientError::PayloadTooLarge(_)
        | ClientError::TooManyRequests(_)
        | ClientError::InternalServerError(_) => {
            "This link is invalid or has expired. Request a new one from the app.".to_string()
        }
    }
}

/// Whether the check on load says the link is dead: only a 400, the
/// server's answer for a used or expired token. Anything else says nothing
/// about the token, so the form stays.
fn link_is_dead(answer: &Result<(), ClientError>) -> bool {
    matches!(answer, Err(ClientError::BadRequest(_)))
}

#[component]
pub fn Reset(token: String) -> Element {
    let password = use_signal(String::new);
    let confirm = use_signal(String::new);
    let mut state = use_signal(|| ResetState::Form);
    let mut rules_open = use_signal(|| false);
    // Whether an earlier submit's answer was lost, so its retry's dead link may be that try.
    let mut repeat = use_signal(FormRepeat::default);

    // The link's token is checked once on load. Only the server's word that
    // it is dead (a 400) replaces the form; any other answer, a rate limit or
    // a failed request leaves the form up, and the submit says what it finds.
    let check_token = token.clone();
    use_future(move || {
        let token = check_token.clone();
        async move {
            if token.is_empty() {
                return;
            }
            let answer = api::client().check_reset_token(token).await;
            if link_is_dead(&answer) && *state.peek() == ResetState::Form {
                state.set(ResetState::DeadLink);
            }
        }
    });

    let on_submit = move |e: FormEvent| {
        e.prevent_default();

        let pw = password.read().clone();
        let cf = confirm.read().clone();
        let token = token.clone();

        let check = check_submit(&state.peek(), &pw, &cf, &token);
        match check {
            SubmitCheck::Ignore => return,
            SubmitCheck::Reject(err) => {
                state.set(ResetState::Error(err));
                return;
            }
            SubmitCheck::Send => {}
        }

        state.set(ResetState::Loading);

        spawn(async move {
            match api::client().reset_password(token, Secret::new(pw)).await {
                Ok(()) => state.set(ResetState::Success),
                Err(e) => {
                    let note = repeat.write().settle(reset_answer(&e));
                    state.set(ResetState::Error(failure_message(&e, note)));
                }
            }
        });
    };

    // Editing either field clears a submit error, so the live check below
    // isn't hidden behind a stale one.
    use_effect(move || {
        let _ = (password.read().len(), confirm.read().len());
        if matches!(*state.peek(), ResetState::Error(_)) {
            state.set(ResetState::Form);
        }
    });

    let current_state = state.read().clone();

    let live = live_error(&password.read(), &confirm.read());

    rsx! {
        PageMeta {
            title: "Reset password",
            description: "Set a new password for your Zwipe account.",
            path: "/reset",
        }
        Nav {}
        // The password policy, from the same list the app's hint shows. A tap
        // on the backdrop or Got it closes it.
        if rules_open() {
            div { class: "sd-image-overlay-backdrop" }
            div { class: "hint-overlay", onclick: move |_| rules_open.set(false),
                div { class: "hint-dialog", onclick: move |e| e.stop_propagation(),
                    Panel {
                        title: "Password rules",
                        centered: true,
                        actions: rsx! {
                            button {
                                class: "panel-action",
                                r#type: "button",
                                onclick: move |_| rules_open.set(false),
                                "Got it"
                            }
                        },
                        ul { class: "card-bullets",
                            for line in zwipe_core::domain::auth::password::requirements() {
                                li { "{line}" }
                            }
                        }
                    }
                }
            }
        }
        main { id: "content", class: "form-page content-enter",
            match current_state {
                ResetState::Success => rsx! {
                    div { class: "outcome-success",
                    Panel { title: "Password reset", title_h1: true, centered: true,
                        p { class: "card-summary", "Your password is updated and every session is signed out." }
                    }
                    }
                },
                ResetState::DeadLink => rsx! {
                    div { class: "outcome-error",
                    Panel { title: "Link already used or expired", title_h1: true, centered: true,
                        p { class: "card-summary", "If you already reset your password, log in with the new one. If not, tap " HintKey { "Forgot password" } " on " HintKey { "Log in" } " in the app for a new link." }
                    }
                    }
                },
                ResetState::Form | ResetState::Loading | ResetState::Error(_) => rsx! {
                    Panel { title: "Reset password", title_h1: true, centered: true,
                        actions: rsx! {
                            button {
                                r#type: "submit",
                                form: "reset-form",
                                class: "panel-action",
                                disabled: current_state == ResetState::Loading || live.is_some(),
                                if current_state == ResetState::Loading { "Updating…" } else { "Set new password" }
                            }
                        },

                    form { id: "reset-form", class: "form-centered", onsubmit: on_submit,
                        div { class: "form-group",
                            div { class: "label-row",
                                span { class: "label-hint-anchor",
                                    label { r#for: "reset-password", "New password" }
                                    InfoButton { onclick: move |_| rules_open.set(true) }
                                }
                            }
                            PasswordInput {
                                value: password,
                                id: "reset-password",
                                name: "new-password",
                                autocomplete: "new-password",
                                placeholder: "New password",
                                disabled: current_state == ResetState::Loading,
                            }
                        }
                        div { class: "form-group",
                            label { r#for: "reset-confirm", "Confirm password" }
                            PasswordInput {
                                value: confirm,
                                id: "reset-confirm",
                                name: "confirm-password",
                                autocomplete: "new-password",
                                placeholder: "Confirm password",
                                disabled: current_state == ResetState::Loading,
                            }
                        }
                    }

                    if let ResetState::Error(msg) = &*state.read() {
                        div { class: "status-message error", {sentence_case(msg)} }
                    } else if let Some(line) = &live {
                        div { class: "status-message error", {line.clone()} }
                    }
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "Correct!Horse!Battery9";

    #[test]
    fn only_a_400_marks_the_link_dead() {
        assert!(link_is_dead(&Err(ClientError::BadRequest(String::new()))));
        for kept in [
            Ok(()),
            Err(ClientError::NotFound(String::new())),
            Err(ClientError::TooManyRequests(String::new())),
            Err(ClientError::InternalServerError(String::new())),
            Err(ClientError::Unauthorized(String::new())),
        ] {
            assert!(!link_is_dead(&kept), "{kept:?} replaced the form");
        }
    }

    #[test]
    fn a_submit_while_loading_is_ignored() {
        assert_eq!(
            check_submit(&ResetState::Loading, GOOD, GOOD, "token"),
            SubmitCheck::Ignore,
            "a second send would burn the single-use token"
        );
    }

    #[test]
    fn an_idle_submit_checks_the_form() {
        assert_eq!(
            check_submit(&ResetState::Form, GOOD, GOOD, "token"),
            SubmitCheck::Send
        );
        assert_eq!(
            check_submit(&ResetState::Form, GOOD, "other", "token"),
            SubmitCheck::Reject("Passwords do not match".to_string())
        );
        assert!(matches!(
            check_submit(&ResetState::Form, GOOD, GOOD, ""),
            SubmitCheck::Reject(_)
        ));
        assert_eq!(
            check_submit(&ResetState::Form, "", "", "token"),
            SubmitCheck::Reject("Password is required".to_string())
        );
    }

    #[test]
    fn a_password_that_breaks_a_rule_is_held_with_the_rule() {
        for (weak, rule) in [
            ("short", "Must be at least 8 characters long"),
            ("alllowercase1!", "Must have at least one uppercase letter"),
        ] {
            assert_eq!(
                check_submit(&ResetState::Form, weak, weak, "token"),
                SubmitCheck::Reject(rule.to_string()),
                "{weak:?}"
            );
            assert_eq!(
                live_error(weak, "").as_deref(),
                Some(rule),
                "{weak:?} while typing"
            );
        }
        assert_eq!(
            live_error(GOOD, ""),
            None,
            "a good password waits for its confirmation"
        );
        assert_eq!(live_error(GOOD, GOOD), None);
    }

    #[test]
    fn a_password_the_server_turns_down_shows_its_reason() {
        assert_eq!(
            failure_message(
                &ClientError::UnprocessableEntity("must be at least 8 characters long".to_string()),
                None
            ),
            "Password must be at least 8 characters long"
        );
        assert_eq!(
            failure_message(&ClientError::Unauthorized(String::new()), None),
            "This link is invalid or has expired. Request a new one from the app."
        );
    }

    /// A reset whose answer was lost may have used the link, so its retry's dead link says to try
    /// the new password too.
    #[test]
    fn a_lost_reset_s_retry_finding_the_link_dead_says_to_try_the_new_password() {
        let dead = ClientError::Unauthorized("token not found or expired".to_owned());
        assert_eq!(reset_answer(&dead), FormAnswer::RepeatRefused);
        let mut repeat = FormRepeat::default();
        let lost = repeat.settle(FormAnswer::Network);
        assert_eq!(
            failure_message(&dead, lost),
            write_repeat::RESET_UNCONFIRMED,
            "the lost answer's own line"
        );
        let retry = repeat.settle(reset_answer(&dead));
        assert_eq!(
            failure_message(&dead, retry),
            write_repeat::RESET_UNCONFIRMED
        );
        let again = repeat.settle(reset_answer(&dead));
        assert_eq!(
            failure_message(&dead, again),
            "This link is invalid or has expired. Request a new one from the app.",
            "once said, a later refusal is the server's own"
        );
    }
}
