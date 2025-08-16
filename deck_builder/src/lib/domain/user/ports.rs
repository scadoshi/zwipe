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

pub trait UserMetrics: Send + Sync + 'static {
    fn record_user_creation_success(&self) -> impl Future<Output = ()> + Send;
    fn record_user_creation_failure(&self) -> impl Future<Output = ()> + Send;
}
