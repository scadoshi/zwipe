//! User profile repository implementation.

/// SQLx error-to-domain error mappings and intermediate conversion errors.
pub mod error;
/// Database-to-domain user model conversions.
pub mod models;

use sqlx::{query, query_as};
use uuid::Uuid;
use zwipe_core::domain::Email;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::models::hints::MarkHintShown;
use zwipe_core::domain::user::preferences::{
    ParsePipStyleError, PipStyle, UpdatePreferences, UserPreferences,
};

use crate::domain::user::models::get_user::GetUserError;
use crate::domain::user::models::hints::MarkHintShownError;
use crate::domain::user::models::preferences::{GetPreferencesError, UpdatePreferencesError};
use crate::domain::user::ports::UserRepository;
use crate::outbound::sqlx::postgres::Postgres;
use crate::outbound::sqlx::user::models::DatabaseUser;

/// Raw database preferences record.
#[derive(Debug, Clone, sqlx::FromRow)]
struct DatabaseUserPreferences {
    #[allow(dead_code)]
    user_id: Uuid,
    theme: String,
    dark_mode: bool,
    exclude_universes_beyond: bool,
    universes_beyond_exceptions: Vec<String>,
    pip_style: String,
}

impl TryFrom<DatabaseUserPreferences> for UserPreferences {
    type Error = ParsePipStyleError;

    fn try_from(db: DatabaseUserPreferences) -> Result<Self, Self::Error> {
        Ok(Self {
            theme: db.theme,
            dark_mode: db.dark_mode,
            exclude_universes_beyond: db.exclude_universes_beyond,
            universes_beyond_exceptions: db.universes_beyond_exceptions,
            pip_style: db.pip_style.parse()?,
        })
    }
}

impl UserRepository for Postgres {
    // == get ==

    async fn get_user(&self, user_id: Uuid) -> Result<User, GetUserError> {
        let database_user = query_as!(
            DatabaseUser,
            "SELECT id, username, email, email_verified_at, hints_shown FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        let mut user: User = database_user.try_into()?;

        // Lenient like `hints_shown`: an address that no longer validates
        // reads as no pending change.
        if let Some(row) = query!(
            "SELECT new_email, expires_at FROM email_change_requests WHERE user_id = $1 AND expires_at > NOW()",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?
            && let Ok(email) = Email::new(&row.new_email)
        {
            user.pending_email = Some(email);
            user.pending_email_expires_at = Some(row.expires_at);
        }

        Ok(user)
    }

    // == hints ==

    async fn mark_hint_shown(&self, request: &MarkHintShown) -> Result<User, MarkHintShownError> {
        let database_user = query_as!(
            DatabaseUser,
            r#"UPDATE users
               SET hints_shown = hints_shown || jsonb_build_object($2::text, true)
               WHERE id = $1
               RETURNING id, username, email, email_verified_at, hints_shown"#,
            request.user_id,
            request.hint
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => MarkHintShownError::NotFound,
            other => MarkHintShownError::Database(other.into()),
        })?;

        let user: User = database_user.try_into().map_err(
            |e: crate::outbound::sqlx::user::error::IntoUserError| {
                MarkHintShownError::UserFromDb(anyhow::anyhow!(e))
            },
        )?;

        Ok(user)
    }

    // == preferences ==

    async fn get_preferences(&self, user_id: Uuid) -> Result<UserPreferences, GetPreferencesError> {
        let result = query_as!(
            DatabaseUserPreferences,
            "SELECT user_id, theme, dark_mode, exclude_universes_beyond, universes_beyond_exceptions, \
             pip_style FROM user_preferences WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| GetPreferencesError::Database(e.into()))?;

        result
            .map(UserPreferences::try_from)
            .transpose()
            .map(Option::unwrap_or_default)
            .map_err(|e| GetPreferencesError::Database(e.into()))
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
            r#"INSERT INTO user_preferences
                   (user_id, theme, dark_mode, exclude_universes_beyond,
                    universes_beyond_exceptions, pip_style, updated_at)
               VALUES ($1, COALESCE($2, 'zwipe'), COALESCE($3, true),
                       COALESCE($4, false), COALESCE($5::text[], '{}'::text[]),
                       COALESCE($6, 'theme'), NOW())
               ON CONFLICT (user_id)
               DO UPDATE SET
                   theme = COALESCE($2, user_preferences.theme),
                   dark_mode = COALESCE($3, user_preferences.dark_mode),
                   exclude_universes_beyond =
                       COALESCE($4, user_preferences.exclude_universes_beyond),
                   universes_beyond_exceptions =
                       COALESCE($5::text[], user_preferences.universes_beyond_exceptions),
                   pip_style = COALESCE($6, user_preferences.pip_style),
                   updated_at = NOW()
               RETURNING user_id, theme, dark_mode, exclude_universes_beyond,
                         universes_beyond_exceptions, pip_style"#,
            request.user_id,
            request.theme,
            request.dark_mode,
            request.exclude_universes_beyond,
            request.universes_beyond_exceptions.as_deref(),
            request.pip_style.map(PipStyle::as_str)
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| UpdatePreferencesError::Database(e.into()))?;

        tx.commit()
            .await
            .map_err(|e: sqlx::Error| UpdatePreferencesError::Database(e.into()))?;

        result
            .try_into()
            .map_err(|e: ParsePipStyleError| UpdatePreferencesError::Database(e.into()))
    }
}
