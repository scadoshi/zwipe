use std::future::Future;

use crate::domain::{
    auth::models::jwt::JwtSecret,
    user::models::{
        User, UserAuthenticationError, UserAuthenticationRequest,
        UserAuthenticationSuccessResponse, UserCreationRequest, UserRegistrationError,
        UserWithPasswordHash,
    },
};

pub trait UserRepository: Send + Sync + Clone + 'static {
    fn create_user(
        &self,
        req: &UserCreationRequest,
    ) -> impl Future<Output = Result<User, UserRegistrationError>> + Send;

    fn get_user_with_password_hash(
        &self,
        req: &UserAuthenticationRequest,
    ) -> impl Future<Output = Result<UserWithPasswordHash, UserAuthenticationError>> + Send;
}

pub trait UserService {
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
