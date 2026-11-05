//! User domain models: server-side error types only.
//!
//! Domain types (User, Username, UserSettings) live in zwipe-core.

/// User profile retrieval errors.
pub mod get_user;
/// One-time UI hint marking errors.
pub mod hints;
/// User setting operation errors.
pub mod settings;
