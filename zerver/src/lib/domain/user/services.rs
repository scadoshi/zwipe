use std::fmt::Debug;

use crate::domain::user::{
    models::{GetUser, GetUserError, User},
    ports::{UserRepository, UserService},
};

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
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: UserRepository> UserService for Service<R> {
    async fn get_user(&self, request: &GetUser) -> Result<User, GetUserError> {
        self.repo.get_user(&request.user_id).await
    }
}
