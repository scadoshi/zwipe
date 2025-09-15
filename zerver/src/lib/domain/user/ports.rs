use std::future::Future;

use crate::domain::user::models::{GetUser, GetUserError, User};

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn get_user(
        &self,
        request: &GetUser,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;
}

pub trait UserService: Clone + Send + Sync + 'static {
    fn get_user(
        &self,
        request: &GetUser,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;
}
