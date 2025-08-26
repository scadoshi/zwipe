use std::future::Future;

use crate::domain::{
    auth::models::{
        jwt::JwtSecret, AuthenticateUserError, AuthenticateUserRequest,
        AuthenticateUserSuccessResponse, ChangePasswordError, ChangePasswordRequest,
        RegisterUserError, RegisterUserRequest, UserWithPasswordHash,
    },
    user::models::User,
};

pub trait AuthRepository: Clone + Send + Sync + 'static {
    fn create_user_with_password_hash(
        &self,
        request: &RegisterUserRequest,
    ) -> impl Future<Output = Result<User, RegisterUserError>> + Send;

    fn get_user_with_password_hash(
        &self,
        request: &AuthenticateUserRequest,
    ) -> impl Future<Output = Result<UserWithPasswordHash, AuthenticateUserError>> + Send;

    fn change_password(
        &self,
        request: &ChangePasswordRequest,
    ) -> impl Future<Output = Result<(), ChangePasswordError>> + Send;
}

pub trait AuthService: Clone + Send + Sync + 'static {
    fn jwt_secret(&self) -> &JwtSecret;

    fn register_user(
        &self,
        request: &RegisterUserRequest,
    ) -> impl Future<Output = Result<AuthenticateUserSuccessResponse, RegisterUserError>> + Send;

    fn authenticate_user(
        &self,
        request: &AuthenticateUserRequest,
    ) -> impl Future<Output = Result<AuthenticateUserSuccessResponse, AuthenticateUserError>> + Send;

    fn change_password(
        &self,
        request: &ChangePasswordRequest,
    ) -> impl Future<Output = Result<(), ChangePasswordError>> + Send;
}
