//! Metrics repository implementation.

use sqlx::query;
use uuid::Uuid;

use crate::domain::metrics::{
    models::{
        errors::MetricsError,
        kinds::{AuditAction, EventKind},
        lifetime_counters::LifetimeCounters,
        public_metrics::PublicMetrics,
    },
    ports::MetricsRepository,
};
use crate::outbound::sqlx::postgres::Postgres;
use zwipe_core::http::contracts::metrics::HttpUsageBatch;

fn db(err: sqlx::Error) -> MetricsError {
    MetricsError::Database(err.into())
}

impl MetricsRepository for Postgres {
    async fn apply_usage(
        &self,
        user_id: Uuid,
        batch: &HttpUsageBatch,
    ) -> Result<(), MetricsError> {
        let r = batch.swipes_right as i64;
        let l = batch.swipes_left as i64;
        let u = batch.swipes_up as i64;
        let d = batch.swipes_down as i64;
        let s = batch.searches as i64;

        let mut tx = self.pool.begin().await.map_err(db)?;

        query!(
            r#"UPDATE user_lifetime_counters
               SET swipes_right = swipes_right + $2,
                   swipes_left  = swipes_left  + $3,
                   swipes_up    = swipes_up    + $4,
                   swipes_down  = swipes_down  + $5,
                   searches     = searches     + $6,
                   updated_at   = NOW()
               WHERE user_id = $1"#,
            user_id,
            r,
            l,
            u,
            d,
            s,
        )
        .execute(&mut *tx)
        .await
        .map_err(db)?;

        let r32 = batch.swipes_right as i32;
        let l32 = batch.swipes_left as i32;
        let u32 = batch.swipes_up as i32;
        let d32 = batch.swipes_down as i32;
        let s32 = batch.searches as i32;

        query!(
            r#"INSERT INTO user_daily_activity
                   (user_id, day, swipes_right, swipes_left, swipes_up, swipes_down, searches)
               VALUES ($1, (NOW() AT TIME ZONE 'UTC')::date, $2, $3, $4, $5, $6)
               ON CONFLICT (user_id, day) DO UPDATE SET
                   swipes_right = user_daily_activity.swipes_right + EXCLUDED.swipes_right,
                   swipes_left  = user_daily_activity.swipes_left  + EXCLUDED.swipes_left,
                   swipes_up    = user_daily_activity.swipes_up    + EXCLUDED.swipes_up,
                   swipes_down  = user_daily_activity.swipes_down  + EXCLUDED.swipes_down,
                   searches     = user_daily_activity.searches     + EXCLUDED.searches"#,
            user_id,
            r32,
            l32,
            u32,
            d32,
            s32,
        )
        .execute(&mut *tx)
        .await
        .map_err(db)?;

        tx.commit().await.map_err(db)?;

        Ok(())
    }

    async fn insert_lifetime_row(&self, user_id: Uuid) -> Result<(), MetricsError> {
        query!(
            r#"INSERT INTO user_lifetime_counters (user_id)
               VALUES ($1)
               ON CONFLICT (user_id) DO NOTHING"#,
            user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(db)?;

        Ok(())
    }

    async fn increment_decks_created(&self, user_id: Uuid) -> Result<(), MetricsError> {
        query!(
            r#"UPDATE user_lifetime_counters
               SET decks_created = decks_created + 1,
                   updated_at = NOW()
               WHERE user_id = $1"#,
            user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn increment_decks_completed(&self, user_id: Uuid) -> Result<(), MetricsError> {
        query!(
            r#"UPDATE user_lifetime_counters
               SET decks_completed = decks_completed + 1,
                   updated_at = NOW()
               WHERE user_id = $1"#,
            user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn record_event(
        &self,
        user_id: Uuid,
        kind: EventKind,
        deck_id: Option<Uuid>,
    ) -> Result<(), MetricsError> {
        let kind_str = kind.as_str();
        query!(
            r#"INSERT INTO user_events (user_id, kind, deck_id)
               VALUES ($1, $2, $3)"#,
            user_id,
            kind_str,
            deck_id,
        )
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn record_audit(
        &self,
        user_id: Uuid,
        action: AuditAction,
    ) -> Result<(), MetricsError> {
        let action_str = action.as_str();
        query!(
            r#"INSERT INTO user_audit_log (user_id, action)
               VALUES ($1, $2)"#,
            user_id,
            action_str,
        )
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn lifetime_counters(&self, user_id: Uuid) -> Result<LifetimeCounters, MetricsError> {
        let row = query!(
            r#"SELECT user_id, swipes_right, swipes_left, swipes_up, swipes_down,
                      searches, decks_created, decks_completed, updated_at
               FROM user_lifetime_counters
               WHERE user_id = $1"#,
            user_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db)?;

        let row = row.ok_or(MetricsError::NotFound)?;

        Ok(LifetimeCounters {
            user_id: row.user_id,
            swipes_right: row.swipes_right,
            swipes_left: row.swipes_left,
            swipes_up: row.swipes_up,
            swipes_down: row.swipes_down,
            searches: row.searches,
            decks_created: row.decks_created,
            decks_completed: row.decks_completed,
            updated_at: row.updated_at,
        })
    }

    async fn public_metrics(&self) -> Result<PublicMetrics, MetricsError> {
        let row = sqlx::query!(
            r#"SELECT
                   COALESCE(SUM(swipes_right + swipes_left + swipes_up + swipes_down), 0)::BIGINT AS "cards_swiped!",
                   COALESCE(SUM(searches), 0)::BIGINT AS "searches!",
                   COALESCE(SUM(decks_created), 0)::BIGINT AS "decks_created!"
               FROM user_lifetime_counters"#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;

        Ok(PublicMetrics {
            cards_swiped: row.cards_swiped,
            searches: row.searches,
            decks_created: row.decks_created as i64,
        })
    }

    async fn touch_last_active(&self, user_id: Uuid) -> Result<(), MetricsError> {
        query!(
            r#"UPDATE users
               SET last_active_at = NOW()
               WHERE id = $1"#,
            user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn mark_user_first_swiped(&self, user_id: Uuid) -> Result<bool, MetricsError> {
        let result = query!(
            r#"UPDATE users
               SET first_swiped_at = NOW()
               WHERE id = $1 AND first_swiped_at IS NULL"#,
            user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(db)?;

        Ok(result.rows_affected() > 0)
    }

    async fn mark_deck_first_completed(&self, deck_id: Uuid) -> Result<bool, MetricsError> {
        let result = query!(
            r#"UPDATE decks
               SET first_completed_at = NOW()
               WHERE id = $1 AND first_completed_at IS NULL"#,
            deck_id,
        )
        .execute(&self.pool)
        .await
        .map_err(db)?;

        Ok(result.rows_affected() > 0)
    }
}
