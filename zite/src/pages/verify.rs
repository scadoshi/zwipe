use crate::{Nav, api};
use dioxus::prelude::*;
use zwipe_client::ClientError;
use zwipe_components::{HintKey, Panel};

#[component]
pub fn Verify(token: String) -> Element {
    let result: Resource<Result<(), String>> = use_resource(move || {
        let token = token.clone();
        async move {
            if token.is_empty() {
                return Err(
                    "This link is missing its code. Open it from the email again.".to_string(),
                );
            }

            api::client()
                .verify_email(token)
                .await
                .map_err(|e| match e {
                    // A request that never landed is not a bad token; say so.
                    ClientError::Network(_) | ClientError::Decode(_) => e.to_user_message(),
                    _ => "This link is invalid or has expired.".to_string(),
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
                Some(Err(e)) => rsx! {
                    div { class: "outcome-error",
                    Panel { title: "Verification failed", title_h1: true, centered: true,
                        p { class: "card-summary", "{e}" }
                        p { class: "card-summary", "Tap " HintKey { "Resend" } " on " HintKey { "Profile" } " in the app for a new link." }
                    }
                    }
                },
            }
        }
    }
}
