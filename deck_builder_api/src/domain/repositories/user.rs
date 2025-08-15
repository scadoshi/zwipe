use std::future::Future;

use crate::domain::models::user::{NewUser, NewUserError, User};

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn create_user(&self, req: &NewUser)
        -> impl Future<Output = Result<User, NewUserError>> + Send;
}
