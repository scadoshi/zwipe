//! User profile handlers.

/// Returns the authenticated user's display preferences.
pub mod get_preferences;
/// Returns the authenticated user's profile.
pub mod get_user;
/// Marks a one-time UI hint as shown for the authenticated user.
pub mod mark_hint_shown;
/// Updates the authenticated user's display preferences.
pub mod update_preferences;
