//! [`EmailSender`] outbound port for email delivery.

use crate::domain::email::models::{SendEmail, SendEmailError};
use std::future::Future;

/// Outbound port for sending transactional email.
///
/// Implemented by the Resend adapter in `outbound/resend`. Callers (e.g. `AuthService`)
/// depend on this trait, not the concrete adapter — keeping email delivery swappable.
pub trait EmailSender: Clone + Send + Sync + 'static {
    /// Send a single email.
    fn send_email(
        &self,
        email: SendEmail,
    ) -> impl Future<Output = Result<(), SendEmailError>> + Send;
}
