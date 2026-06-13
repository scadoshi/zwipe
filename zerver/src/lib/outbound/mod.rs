//! Outbound adapters: database repositories and external service clients.

/// Archidekt deck import adapter.
#[cfg(feature = "zerver")]
pub mod archidekt;

/// Recommander deck-aware card-recommendation adapter.
#[cfg(feature = "zerver")]
pub mod recommander;

/// Resend email delivery adapter.
#[cfg(feature = "zerver")]
pub mod resend;

/// SQLx-based PostgreSQL repository implementations.
pub mod sqlx;
