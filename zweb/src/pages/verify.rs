use dioxus::prelude::*;
use serde::Serialize;
use crate::{API_BASE, Nav};

#[derive(Serialize)]
struct VerifyEmailRequest {
    token: String,
}

fn read_token_param() -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    params.get("token")
}

#[component]
pub fn Verify() -> Element {
    let result: Resource<Result<(), String>> = use_resource(move || async move {
        let token = match read_token_param() {
            Some(t) if !t.is_empty() => t,
            _ => return Err("no token found in url".to_string()),
        };

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
            Err("token not found or expired".to_string())
        }
    });

    rsx! {
        Nav {}
        div { class: "form-page",
            match &*result.read() {
                None => rsx! {
                    h1 { "verifying..." }
                    p { class: "subtitle", "please wait." }
                },
                Some(Ok(())) => rsx! {
                    h1 { "email verified" }
                    p { class: "subtitle", "your email address has been confirmed. you can close this page and return to the app." }
                    div { class: "status-message success", "verification successful" }
                },
                Some(Err(e)) => rsx! {
                    h1 { "verification failed" }
                    p { class: "subtitle", "this link may have expired. request a new one from the app." }
                    div { class: "status-message error", "{e}" }
                },
            }
        }
    }
}
