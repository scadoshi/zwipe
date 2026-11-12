//! Resend email adapter: implements [`EmailSender`]
//! via the [Resend](https://resend.com) HTTP API.

use crate::domain::email::models::{SendEmail, SendEmailError};
use crate::domain::email::ports::{EmailSender, SendEmailFuture};

/// The body `POST /emails` takes; the plain text and reply-to go only when set.
#[derive(Debug, serde::Serialize)]
struct ResendEmail<'email> {
    /// The sending address.
    from: &'email str,
    /// The one recipient.
    to: [&'email str; 1],
    /// The subject line.
    subject: &'email str,
    /// The HTML body.
    html: &'email str,
    /// The plain-text body.
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'email str>,
    /// Where a reply goes.
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<&'email str>,
}

/// Resend HTTP API adapter.
///
/// Calls `POST https://api.resend.com/emails` using `reqwest` (already a workspace dep).
/// Construct once and clone into services: the inner [`reqwest::Client`] is a connection pool.
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
    fn send_email(&self, email: SendEmail) -> SendEmailFuture<'_> {
        Box::pin(self.send(email))
    }
}

impl Resend {
    /// Posts one email to the Resend API.
    async fn send(&self, email: SendEmail) -> Result<(), SendEmailError> {
        let body = ResendEmail {
            from: &self.from_email,
            to: [email.to.as_str()],
            subject: &email.subject,
            html: &email.html_body,
            text: email.text_body.as_deref(),
            reply_to: email.reply_to.as_deref(),
        };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_body_sends_text_and_reply_to_only_when_set() {
        let plain = serde_json::to_value(ResendEmail {
            from: "Zwipe <noreply@zwipe.net>",
            to: ["a@test.local"],
            subject: "Hi",
            html: "<p>Hi</p>",
            text: None,
            reply_to: None,
        })
        .unwrap();
        assert_eq!(
            plain,
            serde_json::json!({
                "from": "Zwipe <noreply@zwipe.net>",
                "to": ["a@test.local"],
                "subject": "Hi",
                "html": "<p>Hi</p>",
            })
        );
        let full = serde_json::to_value(ResendEmail {
            from: "Zwipe <support@zwipe.net>",
            to: ["a@test.local"],
            subject: "Hi",
            html: "<p>Hi</p>",
            text: Some("Hi"),
            reply_to: Some("scotty@zwipe.net"),
        })
        .unwrap();
        assert_eq!(full["text"], "Hi");
        assert_eq!(full["reply_to"], "scotty@zwipe.net");
    }
}
