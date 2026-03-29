//! User profile repository implementation.

/// SQLx error-to-domain error mappings and intermediate conversion errors.
pub mod error;
/// Database-to-domain user model conversions.
pub mod models;

use crate::domain::user::{
    models::{
        get_user::GetUserError,
        preferences::{
            GetPreferencesError, UpdatePreferences, UpdatePreferencesError, UserPreferences,
        },
        User,
    },
    ports::UserRepository,
};
use crate::outbound::sqlx::{postgres::Postgres, user::models::DatabaseUser};
use sqlx::query_as;
use uuid::Uuid;

/// Raw database preferences record.
#[derive(Debug, Clone, sqlx::FromRow)]
struct DatabaseUserPreferences {
    #[allow(dead_code)]
    user_id: Uuid,
    theme: String,
    dark_mode: bool,
}

impl From<DatabaseUserPreferences> for UserPreferences {
    fn from(db: DatabaseUserPreferences) -> Self {
        Self {
            theme: db.theme,
            dark_mode: db.dark_mode,
        }
    }
}

impl UserRepository for Postgres {
    // =====
    //  get
    // =====

    async fn get_user(&self, user_id: Uuid) -> Result<User, GetUserError> {
        let database_user = query_as!(
            DatabaseUser,
            "SELECT id, username, email, email_verified_at FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        let user: User = database_user.try_into()?;

        Ok(user)
    }

    // ===============
    //  preferences
    // ===============

    async fn get_preferences(&self, user_id: Uuid) -> Result<UserPreferences, GetPreferencesError> {
        let result = query_as!(
            DatabaseUserPreferences,
            "SELECT user_id, theme, dark_mode FROM user_preferences WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| GetPreferencesError::Database(e.into()))?;

        Ok(result.map(UserPreferences::from).unwrap_or_default())
    }

    async fn update_preferences(
        &self,
        request: &UpdatePreferences,
    ) -> Result<UserPreferences, UpdatePreferencesError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e: sqlx::Error| UpdatePreferencesError::Database(e.into()))?;

        let result = query_as!(
            DatabaseUserPreferences,
            r#"INSERT INTO user_preferences (user_id, theme, dark_mode, updated_at)
               VALUES ($1, COALESCE($2, 'zwipe'), COALESCE($3, true), NOW())
               ON CONFLICT (user_id)
               DO UPDATE SET
                   theme = COALESCE($2, user_preferences.theme),
                   dark_mode = COALESCE($3, user_preferences.dark_mode),
                   updated_at = NOW()
               RETURNING user_id, theme, dark_mode"#,
            request.user_id,
            request.theme,
            request.dark_mode
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| UpdatePreferencesError::Database(e.into()))?;

        tx.commit()
            .await
            .map_err(|e: sqlx::Error| UpdatePreferencesError::Database(e.into()))?;

        Ok(result.into())
    }
}
