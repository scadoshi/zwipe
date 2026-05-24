use dioxus::prelude::*;
use serde::Serialize;
use crate::{API_BASE, Nav};

#[derive(Serialize)]
struct VerifyEmailRequest {
    token: String,
}

#[component]
pub fn Verify(token: String) -> Element {
    let result: Resource<Result<(), String>> = use_resource(move || {
        let token = token.clone();
        async move {
        if token.is_empty() {
            return Err("No token found in URL".to_string());
        }

        let client = reqwest::Client::new();
        let res = client
            .post(format!("{API_BASE}/api/auth/verify-email"))
            .json(&VerifyEmailRequest { token })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err("Token not found or expired".to_string())
        }
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
