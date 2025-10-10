use chrono::{NaiveDateTime, Utc};
use std::future::Future;
use zwipe::domain::{
    auth::{
        self,
        ports::{AuthRepository, AuthService},
    },
    user::ports::UserRepository,
};

use crate::was_ago::WasAgo;

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
        if latest.map_or(true, |d| d.was_a_week_ago()) {
            self.delete_expired_sessions().await?;
            *latest = Some(Utc::now().naive_utc());
        }
        Ok(())
    }
}
