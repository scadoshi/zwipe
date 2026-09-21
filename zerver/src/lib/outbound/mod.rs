//! Outbound adapters: database repositories and external service clients.

/// Archidekt deck import adapter.
pub mod archidekt;

/// Resend email delivery adapter.
pub mod resend;

/// SQLx-based PostgreSQL repository implementations.
pub mod sqlx;
