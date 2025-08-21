use std::fmt::Debug;

use crate::domain::user::{
    models::{
        CreateUserError, CreateUserRequest, DeleteUserError, GetUserError, UpdateUserError, User,
    },
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
    async fn create_user(&self, req: &CreateUserRequest) -> Result<User, CreateUserError> {
        self.repo.create_user(req).await
    }

    async fn get_user(&self, req: &super::models::GetUserRequest) -> Result<User, GetUserError> {
        self.repo.get_user(req).await
    }

    async fn update_user(
        &self,
        req: &super::models::UpdateUserRequest,
    ) -> Result<User, UpdateUserError> {
        self.repo.update_user(req).await
    }

    async fn delete_user(
        &self,
        req: &super::models::DeleteUserRequest,
    ) -> Result<(), DeleteUserError> {
        self.repo.delete_user(req).await
    }
}
