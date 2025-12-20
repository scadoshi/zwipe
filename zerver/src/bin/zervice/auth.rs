use chrono::{Duration, NaiveDateTime, Utc};
use std::future::Future;
use zwipe::domain::{
    auth::{
        self,
        ports::{AuthRepository, AuthService},
    },
    user::ports::UserRepository,
};

pub trait CheckSessions {
    fn check_sessions(
        &self,
        latest: &mut Option<NaiveDateTime>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

impl<AR, UR> CheckSessions for auth::services::Service<AR, UR>
where
    AR: AuthRepository,
    UR: UserRepository,
{
    async fn check_sessions(&self, latest: &mut Option<NaiveDateTime>) -> anyhow::Result<()> {
        if latest.is_none_or(|d| d < Utc::now().naive_utc() - Duration::days(7)) {
            tracing::info!(
                "session clean up: attempting to delete expired refresh tokens from database"
            );
            self.delete_expired_sessions().await?;
            tracing::info!("session clean up completed");
            *latest = Some(Utc::now().naive_utc());
        }
        Ok(())
    }
}
