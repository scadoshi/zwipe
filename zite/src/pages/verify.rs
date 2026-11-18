use components::{HintKey, Panel};
use dioxus::prelude::*;
use zwipe_client::ClientError;
use zwipe_ui::write_repeat;

use crate::components::PageMeta;
use crate::{Nav, api};

/// Why a verification didn't go through, and what the page says about it.
#[derive(Clone, PartialEq)]
enum Failure {
    /// Unknown, used or expired.
    DeadLink,
    /// The answer was lost: the verification may have gone through.
    Lost,
    /// The link has no token, or the reply didn't read.
    Other(String),
}

#[component]
pub fn Verify(token: String) -> Element {
    let result: Resource<Result<(), Failure>> = use_resource(move || {
        let token = token.clone();
        async move {
            if token.is_empty() {
                return Err(Failure::Other(
                    "This link is missing its code. Open it from the email again.".to_string(),
                ));
            }

            api::client().verify_email(token).await.map_err(|e| {
                if matches!(e, ClientError::Network(_)) {
                    Failure::Lost
                } else if matches!(e, ClientError::Decode(_)) {
                    Failure::Other(e.to_user_message())
                } else {
                    Failure::DeadLink
                }
            })
        }
    });

    rsx! {
        PageMeta {
            title: "Verify email",
            description: "Verify the email address on your Zwipe account.",
            path: "/verify",
        }
        Nav {}
        main { id: "content", class: "form-page content-enter",
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
                Some(Err(Failure::DeadLink)) => rsx! {
                    div { class: "outcome-error",
                    Panel { title: "Link already used or expired", title_h1: true, centered: true,
                        p { class: "card-summary", "If you already verified, you're all set. If not, tap " HintKey { "Resend" } " on " HintKey { "Account" } " in the app for a new link." }
                    }
                    }
                },
                Some(Err(Failure::Lost)) => rsx! {
                    div { class: "outcome-error",
                    Panel { title: write_repeat::LINK_UNCONFIRMED_TITLE, title_h1: true, centered: true,
                        p { class: "card-summary", {write_repeat::VERIFY_LOST} }
                        p { class: "card-summary", "If not, tap " HintKey { "Resend" } " on " HintKey { "Account" } " in the app for a new link." }
                    }
                    }
                },
                Some(Err(Failure::Other(e))) => rsx! {
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
