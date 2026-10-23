use components::{HintKey, Panel};
use dioxus::prelude::*;
use zwipe_client::ClientError;

use crate::{Nav, api};

/// Why a confirmation didn't go through, and what the page says about it.
#[derive(Clone, PartialEq)]
enum Failure {
    /// Unknown, used or expired. A mail scanner opening the link first spends
    /// it and makes the change, so this can't claim nothing changed.
    DeadLink,
    /// Another account took the address after it was asked for.
    Taken,
    /// The link has no token, or the request never landed.
    Other(String),
}

#[component]
pub fn ConfirmEmail(token: String) -> Element {
    let result: Resource<Result<(), Failure>> = use_resource(move || {
        let token = token.clone();
        async move {
            if token.is_empty() {
                return Err(Failure::Other(
                    "This link is missing its code. Open it from the email again.".to_string(),
                ));
            }

            api::client()
                .confirm_email_change(token)
                .await
                .map_err(|e| match e {
                    // A request that never landed is not a bad token; say so.
                    ClientError::Network(_) | ClientError::Decode(_) => {
                        Failure::Other(e.to_user_message())
                    }
                    ClientError::Conflict(_) => Failure::Taken,
                    _ => Failure::DeadLink,
                })
        }
    });

    rsx! {
        Nav {}
        div { class: "form-page content-enter",
            match &*result.read() {
                None => rsx! {
                    Panel { title: "Confirming", title_h1: true, centered: true,
                        div { class: "spinner-row",
                            div { class: "spinner" }
                        }
                    }
                },
                Some(Ok(())) => rsx! {
                    div { class: "outcome-success",
                    Panel { title: "Email changed", title_h1: true, centered: true,
                        p { class: "card-summary", "Your account uses this address now. You can close this page and go back to the app." }
                    }
                    }
                },
                Some(Err(Failure::DeadLink)) => rsx! {
                    div { class: "outcome-error",
                    Panel { title: "Link already used or expired", title_h1: true, centered: true,
                        p { class: "card-summary", "If you already confirmed, your new email is active. " HintKey { "Profile" } " in the app shows which email your account uses." }
                    }
                    }
                },
                Some(Err(Failure::Taken)) => rsx! {
                    div { class: "outcome-error",
                    Panel { title: "Email not changed", title_h1: true, centered: true,
                        p { class: "card-summary", "Another account started using this address after you asked for it. Your email is the same as before." }
                    }
                    }
                },
                Some(Err(Failure::Other(e))) => rsx! {
                    div { class: "outcome-error",
                    Panel { title: "Email not changed", title_h1: true, centered: true,
                        p { class: "card-summary", "{e}" }
                        p { class: "card-summary", "To try again, tap " HintKey { "Change" } " next to your email on " HintKey { "Profile" } " in the app." }
                    }
                    }
                },
            }
        }
    }
}
