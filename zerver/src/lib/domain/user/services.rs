use std::fmt::Debug;

use uuid::Uuid;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::models::hints::MarkHintShown;
use zwipe_core::domain::user::requests::get_user::GetUser;
use zwipe_core::domain::user::settings::{UpdateSettings, UserSettings};

use crate::domain::user::models::account_summary::AccountSummary;
use crate::domain::user::models::get_user::GetUserError;
use crate::domain::user::models::hints::MarkHintShownError;
use crate::domain::user::models::settings::{FetchSettingsError, UpdateSettingsError};
use crate::domain::user::ports::{UserRepository, UserService};

/// User service implementation handling user data retrieval operations.
///
/// This service provides read-only access to user data. User modifications
/// (username, email, password changes) are handled by the auth service for
/// security reasons (require password re-authentication).
#[derive(Debug, Clone)]
pub struct Service<R> {
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
    // == get ==

    async fn fetch_user(&self, request: &GetUser) -> Result<User, GetUserError> {
        self.repo.fetch_user(request.user_id).await
    }

    async fn fetch_account_summary(&self, user_id: Uuid) -> Result<AccountSummary, GetUserError> {
        self.repo.fetch_account_summary(user_id).await
    }

    // == settings ==

    async fn fetch_settings(&self, user_id: Uuid) -> Result<UserSettings, FetchSettingsError> {
        self.repo.fetch_settings(user_id).await
    }

    async fn update_settings(
        &self,
        request: &UpdateSettings,
    ) -> Result<UserSettings, UpdateSettingsError> {
        self.repo.update_settings(request).await
    }

    // == hints ==

    async fn mark_hint_shown(&self, request: &MarkHintShown) -> Result<User, MarkHintShownError> {
        self.repo.mark_hint_shown(request).await
    }
}
