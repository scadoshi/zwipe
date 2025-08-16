use std::future::Future;

use crate::domain::user::models::{User, UserCreationError, UserCreationRequest};

pub trait UserService {
    fn create_user(
        &self,
        req: &UserCreationRequest,
    ) -> impl Future<Output = Result<User, UserCreationError>> + Send;
}

pub trait UserRepository: Send + Sync + Clone + 'static {
    fn create_user(
        &self,
        req: &UserCreationRequest,
    ) -> impl Future<Output = Result<User, UserCreationError>> + Send;
}
