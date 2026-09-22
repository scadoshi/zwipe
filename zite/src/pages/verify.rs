use crate::{Nav, api};
use dioxus::prelude::*;
use zwipe_client::ClientError;

#[component]
pub fn Verify(token: String) -> Element {
    let result: Resource<Result<(), String>> = use_resource(move || {
        let token = token.clone();
        async move {
            if token.is_empty() {
                return Err("No token found in URL".to_string());
            }

            api::client()
                .verify_email(token)
                .await
                .map_err(|e| match e {
                    // A request that never landed is not a bad token; say so.
                    ClientError::Network(_) | ClientError::Decode(_) => e.to_user_message(),
                    _ => "Token not found or expired".to_string(),
                })
        }
    });

    rsx! {
        Nav {}
        div { class: "form-page content-enter",
            match &*result.read() {
                None => rsx! {
                    h1 { "Verifying" }
                    p { class: "subtitle", "Checking your verification link." }
                    div { class: "spinner-row",
                        div { class: "spinner" }
                    }
                },
                Some(Ok(())) => rsx! {
                    h1 { "Email Verified" }
                    p { class: "subtitle", "Your email address has been confirmed. You can close this page and return to the app." }
                    div { class: "status-message success", "Verification successful" }
                },
                Some(Err(e)) => rsx! {
                    h1 { "Verification Failed" }
                    p { class: "subtitle", "This link may have expired or already been used. Request a new one from the app." }
                    div { class: "status-message error", "{e}" }
                },
            }
        }
    }
}
