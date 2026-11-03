//! [`EmailSender`](crate::domain::email::ports::EmailSender) outbound port for email delivery.

use std::future::Future;
use std::pin::Pin;

use crate::domain::email::models::{SendEmail, SendEmailError};

/// Future returned by [`EmailSender::send_email`], boxed so the port is object safe.
pub type SendEmailFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), SendEmailError>> + Send + 'a>>;

/// Outbound port for sending transactional email.
///
/// The auth service holds it as `Arc<dyn EmailSender>`: the Resend adapter in `outbound/resend` sends for real, and the integration tests swap in a fake that captures each message.
pub trait EmailSender: std::fmt::Debug + Send + Sync + 'static {
    /// Send a single email.
    fn send_email(&self, email: SendEmail) -> SendEmailFuture<'_>;
}
