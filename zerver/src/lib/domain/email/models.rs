//! Email send request and error types.

/// A request to send a single email.
pub struct SendEmail {
    /// Recipient address.
    pub to: String,
    /// Email subject line.
    pub subject: String,
    /// HTML body of the email.
    pub html_body: String,
    /// Plain-text body, for mail clients that show no HTML; `None` sends the HTML alone.
    pub text_body: Option<String>,
    /// Where a reply goes, in place of the sending address; `None` replies to the sender.
    pub reply_to: Option<String>,
}

/// Redacted: the bodies carry one-time verify and reset links.
impl std::fmt::Debug for SendEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SendEmail")
            .field("to", &self.to)
            .field("subject", &self.subject)
            .field("html_body", &"REDACTED")
            .field("text_body", &self.text_body.as_ref().map(|_| "REDACTED"))
            .field("reply_to", &self.reply_to)
            .finish()
    }
}

/// Errors that can occur when sending an email.
#[derive(Debug, thiserror::Error)]
pub enum SendEmailError {
    /// Network or I/O failure calling the email provider.
    #[error("network error: {0}")]
    Network(#[from] anyhow::Error),
    /// The email provider returned a non-success HTTP status.
    #[error("resend api error {0}: {1}")]
    ApiError(u16, String),
}
