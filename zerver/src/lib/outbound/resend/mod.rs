//! Resend email adapter: implements [`EmailSender`](crate::domain::email::ports::EmailSender)
//! via the [Resend](https://resend.com) HTTP API.

use crate::domain::email::{
    models::{SendEmail, SendEmailError},
    ports::EmailSender,
};

/// Resend HTTP API adapter.
///
/// Calls `POST https://api.resend.com/emails` using `reqwest` (already a workspace dep).
/// Construct once and clone into services — the inner [`reqwest::Client`] is a connection pool.
#[derive(Debug, Clone)]
pub struct Resend {
    client: reqwest::Client,
    api_key: String,
    from_email: String,
}

impl Resend {
    /// Creates a new Resend adapter.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Resend API key (from `RESEND_API_KEY` env var)
    /// * `from_email` - Sender address (e.g. `"noreply@zwipe.net"`)
    pub fn new(api_key: String, from_email: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            from_email,
        }
    }
}

impl EmailSender for Resend {
    async fn send_email(&self, email: SendEmail) -> Result<(), SendEmailError> {
        let body = serde_json::json!({
            "from": self.from_email,
            "to": [email.to],
            "subject": email.subject,
            "html": email.html_body,
        });

        let response = self
            .client
            .post("https://api.resend.com/emails")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| SendEmailError::Network(e.into()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(event = "email_send_failure", status = status, body = %body);
            return Err(SendEmailError::ApiError(status, body));
        }

        Ok(())
    }
}
