//! Accounts, emails, usernames and preferences.

pub mod models;
pub mod requests;

pub use models::{
    ALLOWED_THEMES, Email, InvalidEmail, InvalidHintKey, InvalidUpdatePreferences, InvalidUsername,
    MarkHintShown, ParsePipStyleError, PipStyle, UpdatePreferences, User, UserPreferences,
    Username, email, hints, preferences, theme, username,
};
pub use requests::{GetUser, get_user};
