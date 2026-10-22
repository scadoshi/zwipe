use components::{HintKey, Panel};
use dioxus::prelude::*;
use zwipe_client::ClientError;

use crate::{Nav, api};

#[component]
pub fn ConfirmEmail(token: String) -> Element {
    let result: Resource<Result<(), String>> = use_resource(move || {
        let token = token.clone();
        async move {
            if token.is_empty() {
                return Err(
                    "This link is missing its code. Open it from the email again.".to_string(),
                );
            }

            api::client()
                .confirm_email_change(token)
                .await
                .map_err(|e| match e {
                    // A request that never landed is not a bad token; say so.
                    ClientError::Network(_) | ClientError::Decode(_) => e.to_user_message(),
                    ClientError::Conflict(_) => {
                        "Another account started using this address after you asked for it."
                            .to_string()
                    }
                    _ => "This link is invalid or has expired.".to_string(),
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
                Some(Err(e)) => rsx! {
                    div { class: "outcome-error",
                    Panel { title: "Email not changed", title_h1: true, centered: true,
                        p { class: "card-summary", "{e}" }
                        p { class: "card-summary", "Your email is the same as before. To try again, tap " HintKey { "Change" } " next to your email on " HintKey { "Profile" } " in the app." }
                    }
                    }
                },
            }
        }
    }
}
