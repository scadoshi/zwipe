use crate::domain::user::{
    models::{User, UserCreationError},
    ports::{UserMetrics, UserRepository, UserService},
};

pub struct Service<R, M>
where
    R: UserRepository,
    M: UserMetrics,
{
    repo: R,
    metrics: M,
}

impl<R, M> Service<R, M>
where
    R: UserRepository,
    M: UserMetrics,
{
    pub fn new(repo: R, metrics: M) -> Self {
        Self { repo, metrics }
    }
}

impl<R, M> UserService for Service<R, M>
where
    R: UserRepository,
    M: UserMetrics,
{
    async fn create_user(
        &self,
        req: &super::models::UserCreationRequest,
    ) -> Result<User, UserCreationError> {
        let result = self.repo.create_user(req).await;

        if result.is_err() {
            self.metrics.record_user_creation_failure().await;
        } else {
            self.metrics.record_user_creation_success().await;
        }

        result
    }
}
