use std::future::Future;

use crate::domain::{
    auth::models::{
        jwt::JwtSecret, AuthenticateUserError, AuthenticateUserRequest,
        AuthenticateUserSuccessResponse, ChangePasswordError, ChangePasswordRequest,
        RegisterUserError, RegisterUserRequest, UserWithPasswordHash,
    },
    user::models::User,
};

pub trait AuthRepository: Send + Sync + 'static {
    fn create_user_with_password_hash(
        &self,
        req: &RegisterUserRequest,
    ) -> impl Future<Output = Result<User, RegisterUserError>> + Send;

    fn get_user_with_password_hash(
        &self,
        req: &AuthenticateUserRequest,
    ) -> impl Future<Output = Result<UserWithPasswordHash, AuthenticateUserError>> + Send;

    fn change_password(
        &self,
        req: &ChangePasswordRequest,
    ) -> impl Future<Output = Result<(), ChangePasswordError>> + Send;
}

pub trait AuthService: Send + Sync + Clone + 'static {
    fn register_user(
        &self,
        req: &RegisterUserRequest,
        jwt_secret: JwtSecret,
    ) -> impl Future<Output = Result<AuthenticateUserSuccessResponse, RegisterUserError>> + Send;

    fn authenticate_user(
        &self,
        req: &AuthenticateUserRequest,
        jwt_secret: JwtSecret,
    ) -> impl Future<Output = Result<AuthenticateUserSuccessResponse, AuthenticateUserError>> + Send;

    fn change_password(
        &self,
        req: &ChangePasswordRequest,
    ) -> impl Future<Output = Result<(), ChangePasswordError>> + Send;
}
