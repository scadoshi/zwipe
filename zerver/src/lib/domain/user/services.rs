use std::fmt::Debug;

use crate::domain::user::{
    models::{
        CreateUser, CreateUserError, DeleteUser, DeleteUserError, GetUser, GetUserError,
        UpdateUser, UpdateUserError, User,
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
    async fn create_user(&self, request: &CreateUser) -> Result<User, CreateUserError> {
        self.repo.create_user(request).await
    }

    async fn get_user(&self, request: &GetUser) -> Result<User, GetUserError> {
        self.repo.get_user(request).await
    }

    async fn update_user(&self, request: &UpdateUser) -> Result<User, UpdateUserError> {
        self.repo.update_user(request).await
    }

    async fn delete_user(&self, request: &DeleteUser) -> Result<(), DeleteUserError> {
        self.repo.delete_user(request).await
    }
}
