//! Accounts, emails, usernames and settings.

pub mod models;
pub mod requests;

pub use models::{
    ALLOWED_THEMES, Email, InvalidEmail, InvalidHintKey, InvalidUpdateSettings, InvalidUsername,
    MarkHintShown, ParsePipColorError, PipColor, UpdateSettings, User, UserSettings, Username,
    email, hints, settings, theme, username,
};
pub use requests::{GetUser, get_user};
