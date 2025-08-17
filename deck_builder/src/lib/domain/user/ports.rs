use std::future::Future;

use crate::domain::{
    auth::models::password::HashedPassword,
    user::models::{
        User, UserAuthenticationError, UserAuthenticationRequest,
        UserAuthenticationSuccessResponse, UserCreationError, UserCreationRequest,
    },
};

pub trait UserService {
    fn create_user(
        &self,
        req: &UserCreationRequest,
    ) -> impl Future<Output = Result<User, UserCreationError>> + Send;

    fn authenticate_user(
        &self,
        req: &UserAuthenticationRequest,
    ) -> impl Future<Output = Result<UserAuthenticationSuccessResponse, UserAuthenticationError>> + Send;
}

pub trait UserRepository: Send + Sync + Clone + 'static {
    fn create_user(
        &self,
        req: &UserCreationRequest,
    ) -> impl Future<Output = Result<User, UserCreationError>> + Send;

    fn get_user_password_hash(
        &self,
        req: &UserAuthenticationRequest,
    ) -> impl Future<Output = Result<HashedPassword, UserAuthenticationError>> + Send;
}
