//! User profile repository implementation.

pub mod error;
pub mod models;

use chrono::{DateTime, Utc};
use sqlx::{query, query_as};
use uuid::Uuid;
use zwipe_core::domain::Email;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::models::hints::MarkHintShown;
use zwipe_core::domain::user::settings::{
    ParsePipStyleError, ParseSwipeConfirmationsError, PipStyle, SwipeConfirmations, UpdateSettings,
    UserSettings,
};

use crate::domain::user::models::account_summary::{
    AccountSummary, AuditEntry, RECENT_AUDIT_EVENTS,
};
use crate::domain::user::models::get_user::GetUserError;
use crate::domain::user::models::hints::MarkHintShownError;
use crate::domain::user::models::settings::{FetchSettingsError, UpdateSettingsError};
use crate::domain::user::ports::UserRepository;
use crate::outbound::sqlx::postgres::Postgres;
use crate::outbound::sqlx::user::models::DatabaseUser;

/// Raw database settings record.
#[derive(Debug, Clone, sqlx::FromRow)]
struct DatabaseUserSettings {
    #[expect(
        dead_code,
        reason = "the query selects it, but UserSettings does not carry it"
    )]
    user_id: Uuid,
    theme: String,
    dark_mode: bool,
    exclude_universes_beyond: bool,
    universes_beyond_exceptions: Vec<String>,
    pip_style: String,
    swipe_confirmations: String,
}

/// A settings row holding a value the domain does not know.
#[derive(Debug, thiserror::Error)]
enum SettingsRowError {
    /// The pip style.
    #[error(transparent)]
    PipStyle(#[from] ParsePipStyleError),
    /// The swipe confirmations.
    #[error(transparent)]
    SwipeConfirmations(#[from] ParseSwipeConfirmationsError),
}

impl TryFrom<DatabaseUserSettings> for UserSettings {
    type Error = SettingsRowError;

    fn try_from(db: DatabaseUserSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            theme: db.theme,
            dark_mode: db.dark_mode,
            exclude_universes_beyond: db.exclude_universes_beyond,
            universes_beyond_exceptions: db.universes_beyond_exceptions,
            pip_style: db.pip_style.parse()?,
            swipe_confirmations: db.swipe_confirmations.parse()?,
        })
    }
}

impl Postgres {
    /// The user's live email change, if one is waiting: the new address and
    /// when its link expires. The one place this lookup lives, so every
    /// response that returns a [`User`] says the same thing about it.
    pub(crate) async fn fetch_pending_email(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(Email, DateTime<Utc>)>, sqlx::Error> {
        let row = query!(
            "SELECT new_email, expires_at FROM email_change_requests WHERE user_id = $1 AND expires_at > NOW()",
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|row| (Email::from_stored(row.new_email), row.expires_at)))
    }

    /// `user` with its pending email change filled in, as
    /// [`Self::fetch_pending_email`] finds it.
    pub(crate) async fn with_pending_email(&self, mut user: User) -> Result<User, sqlx::Error> {
        if let Some((email, expires_at)) = self.fetch_pending_email(user.id).await? {
            user.pending_email = Some(email);
            user.pending_email_expires_at = Some(expires_at);
        }
        Ok(user)
    }
}

impl UserRepository for Postgres {
    // == get ==

    async fn fetch_user(&self, user_id: Uuid) -> Result<User, GetUserError> {
        let database_user = query_as!(
            DatabaseUser,
            "SELECT id, username, email, email_verified_at, hints_shown FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(self.with_pending_email(User::from(database_user)).await?)
    }

    async fn fetch_account_summary(&self, user_id: Uuid) -> Result<AccountSummary, GetUserError> {
        let user = self.fetch_user(user_id).await?;
        let row = query!(
            r#"SELECT u.created_at, u.last_active_at,
                      (SELECT count(*) FROM decks d WHERE d.user_id = u.id) AS "deck_count!"
               FROM users u WHERE u.id = $1"#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        let recent_audit = query_as!(
            AuditEntry,
            "SELECT action, occurred_at FROM user_audit_log WHERE user_id = $1 ORDER BY occurred_at DESC, id DESC LIMIT $2",
            user_id,
            RECENT_AUDIT_EVENTS
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(AccountSummary {
            user,
            created_at: row.created_at,
            last_active_at: row.last_active_at,
            deck_count: row.deck_count,
            recent_audit,
        })
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

        self.with_pending_email(User::from(database_user))
            .await
            .map_err(|e| MarkHintShownError::Database(e.into()))
    }

    // == settings ==

    async fn fetch_settings(&self, user_id: Uuid) -> Result<UserSettings, FetchSettingsError> {
        let result = query_as!(
            DatabaseUserSettings,
            "SELECT user_id, theme, dark_mode, exclude_universes_beyond, universes_beyond_exceptions, \
             pip_style, swipe_confirmations FROM user_settings WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| FetchSettingsError::Database(e.into()))?;

        result
            .map(UserSettings::try_from)
            .transpose()
            .map(Option::unwrap_or_default)
            .map_err(|e| FetchSettingsError::Database(e.into()))
    }

    async fn update_settings(
        &self,
        request: &UpdateSettings,
    ) -> Result<UserSettings, UpdateSettingsError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e: sqlx::Error| UpdateSettingsError::Database(e.into()))?;

        let result = query_as!(
            DatabaseUserSettings,
            r#"INSERT INTO user_settings
                   (user_id, theme, dark_mode, exclude_universes_beyond,
                    universes_beyond_exceptions, pip_style, swipe_confirmations,
                    updated_at)
               VALUES ($1, COALESCE($2, 'zwipe'), COALESCE($3, true),
                       COALESCE($4, false), COALESCE($5::text[], '{}'::text[]),
                       COALESCE($6, 'themed'), COALESCE($7, 'chip'), NOW())
               ON CONFLICT (user_id)
               DO UPDATE SET
                   theme = COALESCE($2, user_settings.theme),
                   dark_mode = COALESCE($3, user_settings.dark_mode),
                   exclude_universes_beyond =
                       COALESCE($4, user_settings.exclude_universes_beyond),
                   universes_beyond_exceptions =
                       COALESCE($5::text[], user_settings.universes_beyond_exceptions),
                   pip_style = COALESCE($6, user_settings.pip_style),
                   swipe_confirmations =
                       COALESCE($7, user_settings.swipe_confirmations),
                   updated_at = NOW()
               RETURNING user_id, theme, dark_mode, exclude_universes_beyond,
                         universes_beyond_exceptions, pip_style, swipe_confirmations"#,
            request.user_id,
            request.theme,
            request.dark_mode,
            request.exclude_universes_beyond,
            request.universes_beyond_exceptions.as_deref(),
            request.pip_style.map(PipStyle::as_str),
            request.swipe_confirmations.map(SwipeConfirmations::as_str)
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| UpdateSettingsError::Database(e.into()))?;

        tx.commit()
            .await
            .map_err(|e: sqlx::Error| UpdateSettingsError::Database(e.into()))?;

        result
            .try_into()
            .map_err(|e: SettingsRowError| UpdateSettingsError::Database(e.into()))
    }
}
