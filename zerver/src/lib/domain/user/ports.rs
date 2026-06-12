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

use crate::domain::user::models::{
    get_user::GetUserError,
    hints::MarkHintShownError,
    preferences::{GetPreferencesError, UpdatePreferencesError},
};
use zwipe_core::domain::user::{
    models::hints::MarkHintShown,
    preferences::{UpdatePreferences, UserPreferences},
    requests::get_user::GetUser,
    User,
};

/// Database port for user profile operations.
///
/// Provides read access to user data and preferences. Auth mutations
/// (register, change password, etc.) are in the `auth` module.
pub trait UserRepository: Clone + Send + Sync + 'static {
    /// Retrieves a user profile by ID.
    ///
    /// Returns user data without password hash (use AuthRepository for that).
    fn get_user(&self, user_id: Uuid) -> impl Future<Output = Result<User, GetUserError>> + Send;

    /// Fetches display preferences for a user. Returns defaults if no row exists.
    fn get_preferences(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<UserPreferences, GetPreferencesError>> + Send;

    /// Upserts display preferences for a user. Creates the row on first update.
    fn update_preferences(
        &self,
        request: &UpdatePreferences,
    ) -> impl Future<Output = Result<UserPreferences, UpdatePreferencesError>> + Send;

    /// Marks a one-time UI hint as shown; returns the updated user.
    fn mark_hint_shown(
        &self,
        request: &MarkHintShown,
    ) -> impl Future<Output = Result<User, MarkHintShownError>> + Send;
}

/// Service port for user profile business logic.
pub trait UserService: Clone + Send + Sync + 'static {
    // =====
    //  get
    // =====

    /// Retrieves a user profile by ID.
    ///
    /// Returns public user data (username, email, ID).
    fn get_user(
        &self,
        request: &GetUser,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;

    // ===============
    //  preferences
    // ===============

    /// Fetches display preferences for a user.
    fn get_preferences(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<UserPreferences, GetPreferencesError>> + Send;

    /// Validates and updates display preferences for a user.
    fn update_preferences(
        &self,
        request: &UpdatePreferences,
    ) -> impl Future<Output = Result<UserPreferences, UpdatePreferencesError>> + Send;

    // =======
    //  hints
    // =======

    /// Marks a one-time UI hint as shown; returns the updated user.
    fn mark_hint_shown(
        &self,
        request: &MarkHintShown,
    ) -> impl Future<Output = Result<User, MarkHintShownError>> + Send;
}
