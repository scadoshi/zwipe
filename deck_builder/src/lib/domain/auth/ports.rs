use std::fmt::Debug;

use axum::async_trait;

use crate::domain::{
    auth::models::{
        jwt::JwtSecret, AuthenticateUserError, AuthenticateUserRequest,
        AuthenticateUserSuccessResponse, ChangePasswordError, ChangePasswordRequest,
        RegisterUserError, RegisterUserRequest, UserWithPasswordHash,
    },
    user::models::User,
};

pub trait AuthRepository: Send + Sync + 'static {
    async fn create_user_with_password_hash(
        &self,
        req: &RegisterUserRequest,
    ) -> Result<User, RegisterUserError>;

    async fn get_user_with_password_hash(
        &self,
        req: &AuthenticateUserRequest,
    ) -> Result<UserWithPasswordHash, AuthenticateUserError>;

    async fn change_password(&self, req: &ChangePasswordRequest)
        -> Result<(), ChangePasswordError>;
}

#[async_trait]
pub trait AuthService: Debug + Send + Sync + 'static {
    async fn register_user(
        &self,
        req: &RegisterUserRequest,
        jwt_secret: JwtSecret,
    ) -> Result<AuthenticateUserSuccessResponse, RegisterUserError>;

    async fn authenticate_user(
        &self,
        req: &AuthenticateUserRequest,
        jwt_secret: JwtSecret,
    ) -> Result<AuthenticateUserSuccessResponse, AuthenticateUserError>;

    async fn change_password(&self, req: &ChangePasswordRequest)
        -> Result<(), ChangePasswordError>;
}
