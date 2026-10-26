use components::{HintKey, Panel};
use dioxus::prelude::*;
use zwipe_client::ClientError;

use crate::{Nav, api};

#[component]
pub fn Verify(token: String) -> Element {
    // `Err(None)` is a link that is unknown, used or expired; `Err(Some(_))`
    // is anything else, with what to say about it.
    let result: Resource<Result<(), Option<String>>> = use_resource(move || {
        let token = token.clone();
        async move {
            if token.is_empty() {
                return Err(Some(
                    "This link is missing its code. Open it from the email again.".to_string(),
                ));
            }

            api::client()
                .verify_email(token)
                .await
                .map_err(|e| match e {
                    // A request that never landed is not a bad token; say so.
                    ClientError::Network(_) | ClientError::Decode(_) => Some(e.to_user_message()),
                    _ => None,
                })
        }
    });

    rsx! {
        Nav {}
        div { class: "form-page content-enter",
            match &*result.read() {
                None => rsx! {
                    Panel { title: "Verifying", title_h1: true, centered: true,
                        div { class: "spinner-row",
                            div { class: "spinner" }
                        }
                    }
                },
                Some(Ok(())) => rsx! {
                    div { class: "outcome-success",
                    Panel { title: "Email verified", title_h1: true, centered: true,
                        p { class: "card-summary", "You can close this page and go back to the app." }
                    }
                    }
                },
                // A mail scanner opening the link first spends it and does the
                // verifying, so a dead link can't claim it failed.
                Some(Err(None)) => rsx! {
                    div { class: "outcome-error",
                    Panel { title: "Link already used or expired", title_h1: true, centered: true,
                        p { class: "card-summary", "If you already verified, you're all set. If not, tap " HintKey { "Resend" } " on " HintKey { "Account" } " in the app for a new link." }
                    }
                    }
                },
                Some(Err(Some(e))) => rsx! {
                    div { class: "outcome-error",
                    Panel { title: "Verification failed", title_h1: true, centered: true,
                        p { class: "card-summary", "{e}" }
                        p { class: "card-summary", "Tap " HintKey { "Resend" } " on " HintKey { "Account" } " in the app for a new link." }
                    }
                    }
                },
            }
        }
    }
}
