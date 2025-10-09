use std::future::Future;
use uuid::Uuid;

use crate::domain::user::models::{GetUser, GetUserError, User};

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn get_user(&self, user_id: &Uuid) -> impl Future<Output = Result<User, GetUserError>> + Send;
}

pub trait UserService: Clone + Send + Sync + 'static {
    fn get_user(
        &self,
        request: &GetUser,
    ) -> impl Future<Output = Result<User, GetUserError>> + Send;
}
