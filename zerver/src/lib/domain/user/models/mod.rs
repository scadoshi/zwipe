//! User domain models and value objects.
//!
//! Re-exported from `zwipe_core`. Service-layer error types remain here.

pub mod get_user;
/// User display preferences (theme, dark mode).
pub mod preferences;
pub mod username;

pub use zwipe_core::domain::user::User;
