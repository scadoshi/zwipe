use std::fmt::Debug;

use crate::domain::user::{
    models::{
        get_user::{GetUser, GetUserError},
        User,
    },
    ports::{UserRepository, UserService},
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
    async fn get_user(&self, request: &GetUser) -> Result<User, GetUserError> {
        // =====
        //  get
        // =====
        self.repo.get_user(request.user_id).await
    }
}
