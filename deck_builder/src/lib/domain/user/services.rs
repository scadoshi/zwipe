use anyhow::anyhow;

use crate::domain::user::ports::{UserRepository, UserService};

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: UserRepository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: UserRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R> UserService for Service<R> {
    // service layer impls go here
    // create user
    // read user
    // update user
    // delete user
}
