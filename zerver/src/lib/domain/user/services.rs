use std::fmt::Debug;

use uuid::Uuid;

use crate::domain::user::{
    models::{
        get_user::GetUserError,
        hints::MarkHintShownError,
        preferences::{GetPreferencesError, UpdatePreferencesError},
    },
    ports::{UserRepository, UserService},
};
use zwipe_core::domain::user::{
    models::hints::MarkHintShown,
    preferences::{UpdatePreferences, UserPreferences},
    requests::get_user::GetUser,
    User,
};

/// User service implementation handling user data retrieval operations.
///
/// This service provides read-only access to user data. User modifications
/// (username, email, password changes) are handled by the auth service for
/// security reasons (require password re-authentication).
#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: UserRepository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: UserRepository,
{
    /// Creates a new user service with the provided repository.
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: UserRepository> UserService for Service<R> {
    // =====
    //  get
    // =====

    async fn get_user(&self, request: &GetUser) -> Result<User, GetUserError> {
        self.repo.get_user(request.user_id).await
    }

    // ===============
    //  preferences
    // ===============

    async fn get_preferences(&self, user_id: Uuid) -> Result<UserPreferences, GetPreferencesError> {
        self.repo.get_preferences(user_id).await
    }

    async fn update_preferences(
        &self,
        request: &UpdatePreferences,
    ) -> Result<UserPreferences, UpdatePreferencesError> {
        self.repo.update_preferences(request).await
    }

    // =======
    //  hints
    // =======

    async fn mark_hint_shown(&self, request: &MarkHintShown) -> Result<User, MarkHintShownError> {
        self.repo.mark_hint_shown(request).await
    }
}
