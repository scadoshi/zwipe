use crate::domain::user::{
    models::{User, UserCreationError},
    ports::{UserRepository, UserService},
};

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

impl<R> UserService for Service<R>
where
    R: UserRepository,
{
    async fn create_user(
        &self,
        req: &super::models::UserCreationRequest,
    ) -> Result<User, UserCreationError> {
        let result = self.repo.create_user(req).await;

        result
    }
}
