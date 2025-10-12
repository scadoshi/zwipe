pub mod error;
pub mod models;

use crate::domain::user::{
    models::{get_user::GetUserError, User},
    ports::UserRepository,
};
use crate::outbound::sqlx::{postgres::Postgres, user::models::DatabaseUser};
use sqlx::query_as;
use uuid::Uuid;

impl UserRepository for Postgres {
    // =====
    //  get
    // =====
    async fn get_user(&self, user_id: &Uuid) -> Result<User, GetUserError> {
        let database_user = query_as!(
            DatabaseUser,
            "SELECT id, username, email FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        let user: User = database_user.try_into()?;

        Ok(user)
    }
}
