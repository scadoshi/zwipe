//! User domain models: server-side error types only.
//!
//! Domain types (User, Username, UserSettings) live in zwipe-core.

/// One account at a glance, for `zerver-admin show`.
pub mod account_summary;
/// User profile retrieval errors.
pub mod get_user;
/// One-time UI hint marking errors.
pub mod hints;
/// User setting operation errors.
pub mod settings;
