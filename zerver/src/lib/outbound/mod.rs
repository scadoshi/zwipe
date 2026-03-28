//! Outbound adapters: database repositories and external service clients.

/// Resend email delivery adapter.
#[cfg(feature = "zerver")]
pub mod resend;

/// SQLx-based PostgreSQL repository implementations.
pub mod sqlx;
