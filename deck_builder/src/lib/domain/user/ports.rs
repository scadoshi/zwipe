use std::fmt::Debug;

use axum::async_trait;

use crate::domain::user::models::{
    CreateUserError, CreateUserRequest, DeleteUserError, DeleteUserRequest, GetUserError,
    GetUserRequest, UpdateUserError, UpdateUserRequest, User,
};

pub trait UserRepository: Send + Sync + 'static {
    async fn create_user(&self, req: &CreateUserRequest) -> Result<User, CreateUserError>;

    async fn get_user(&self, req: &GetUserRequest) -> Result<User, GetUserError>;

    async fn update_user(&self, req: &UpdateUserRequest) -> Result<User, UpdateUserError>;

    async fn delete_user(&self, req: &DeleteUserRequest) -> Result<(), DeleteUserError>;
}

#[async_trait]
pub trait UserService: Debug + Send + Sync + 'static {
    async fn create_user(&self, req: &CreateUserRequest) -> Result<User, CreateUserError>;

    async fn get_user(&self, req: &GetUserRequest) -> Result<User, GetUserError>;

    async fn update_user(&self, req: &UpdateUserRequest) -> Result<User, UpdateUserError>;

    async fn delete_user(&self, req: &DeleteUserRequest) -> Result<(), DeleteUserError>;
}
