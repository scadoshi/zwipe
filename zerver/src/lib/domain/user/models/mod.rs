//! User domain models — server-side error types only.
//!
//! Domain types (User, Username, UserPreferences) live in zwipe-core.

/// User profile retrieval errors.
pub mod get_user;
/// One-time UI hint marking errors.
pub mod hints;
/// User preference operation errors.
pub mod preferences;
