use std::future::Future;

use crate::domain::{
    auth::models::{jwt::JwtSecret, UserRegistrationRequest},
    user::models::{
        User, UserAuthenticationError, UserAuthenticationRequest,
        UserAuthenticationSuccessResponse, UserCreationRequest, UserRegistrationError,
        UserWithPasswordHash,
    },
};

pub trait AuthRepository: Send + Sync + 'static {
    fn create_user_with_password_hash(
        &self,
        req: &UserRegistrationRequest,
    ) -> impl Future<Output = Result<UserAuthenticationSuccessResponse, UserRegistrationError>> + Send;

    fn get_user_with_password_hash(
        &self,
        req: &UserAuthenticationRequest,
    ) -> impl Future<Output = Result<UserAuthenticationSuccessResponse, UserAuthenticationError>> + Send;
}

pub trait AuthService: Send + Sync + Clone + 'static {
    fn register_user(
        &self,
        req: &UserCreationRequest,
        jwt_secret: JwtSecret,
    ) -> impl Future<Output = Result<UserAuthenticationSuccessResponse, UserRegistrationError>> + Send;

    fn authenticate_user(
        &self,
        req: &UserAuthenticationRequest,
        jwt_secret: JwtSecret,
    ) -> impl Future<Output = Result<UserAuthenticationSuccessResponse, UserAuthenticationError>> + Send;
}
