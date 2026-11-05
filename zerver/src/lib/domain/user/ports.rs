//! Port traits for user profile operations.
//!
//! This module defines the interfaces (ports) for user data access.
//! User profiles are read-only in this module - mutations (register, change password,
//! etc.) are handled by the auth module.
//!
//! # Separation of Concerns
//!
//! - **User Module**: Read operations (get user profile)
//! - **Auth Module**: Write operations (register, update profile, delete)
//!
//! This separation keeps authentication logic isolated from user data access.

use std::future::Future;

use uuid::Uuid;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::models::hints::MarkHintShown;
use zwipe_core::domain::user::requests::get_user::GetUser;
use zwipe_core::domain::user::settings::{UpdateSettings, UserSettings};

use crate::domain::user::models::get_user::GetUserError;
use crate::domain::user::models::hints::MarkHintShownError;
use crate::domain::user::models::settings::{FetchSettingsError, UpdateSettingsError};

/// Database port for user profile operations.
///
/// Provides read access to user data and settings. Auth mutations
/// (register, change password, etc.) are in the `auth` module.
pub trait UserRepository: Clone + Send + Sync + 'static {
    /// Retrieves a user profile by ID.
    ///
    /// Returns user data without password hash (use `AuthRepository` for that).
    fn fetch_user(&self, user_id: Uuid) -> impl Future<Output = Result<User, GetUserError>> + Send;

    /// Fetches display settings for a user. Returns defaults if no row exists.
    fn fetch_settings(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<UserSettings, FetchSettingsError>> + Send;

    /// Upserts display settings for a user. Creates the row on first update.
    fn update_settings(
        &self,
        request: &UpdateSettings,
    ) -> impl Future<Output = Result<UserSettings, UpdateSettingsError>> + Send;

    /// Marks a one-time UI hint as shown; returns the updated user.
    fn mark_hint_shown(
        &self,
        request: &MarkHintShown,
    ) -> impl Future<Output = Result<User, MarkHintShownError>> + Send;
}

/// Service port for user profile business logic.
pub trait UserService: Clone + Send + Sync + 'static {
    // == get ==

    /// Retrieves a user profile by ID.
    ///
    /// Returns public user data (username, email, ID).
    fn fetch_user(
        &self,
        request: &GetUser,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;

    // == settings ==

    /// Fetches display settings for a user.
    fn fetch_settings(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<UserSettings, FetchSettingsError>> + Send;

    /// Validates and updates display settings for a user.
    fn update_settings(
        &self,
        request: &UpdateSettings,
    ) -> impl Future<Output = Result<UserSettings, UpdateSettingsError>> + Send;

    // == hints ==

    /// Marks a one-time UI hint as shown; returns the updated user.
    fn mark_hint_shown(
        &self,
        request: &MarkHintShown,
    ) -> impl Future<Output = Result<User, MarkHintShownError>> + Send;
}
