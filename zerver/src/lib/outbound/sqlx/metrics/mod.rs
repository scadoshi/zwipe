//! Metrics repository implementation.

use std::collections::HashMap;

use sqlx::query;
use uuid::Uuid;

use crate::{
    domain::metrics::{
        models::{
            errors::MetricsError,
            kinds::{AuditAction, EventKind},
            lifetime_counters::LifetimeCounters,
            public_metrics::PublicMetrics,
        },
        ports::MetricsRepository,
    },
    outbound::sqlx::postgres::Postgres,
};
use zwipe_core::http::contracts::metrics::{AnonymousEventKind, HttpUsageBatch};

fn db(err: sqlx::Error) -> MetricsError {
    MetricsError::Database(err.into())
}

use crate::outbound::sqlx::deck::MAX_SUPPRESSIONS_PER_DECK;

impl MetricsRepository for Postgres {
    async fn apply_usage(&self, user_id: Uuid, batch: &HttpUsageBatch) -> Result<(), MetricsError> {
        // Clamp untrusted counts before they touch any counter: stops a client
        // inflating lifetime / marketing totals, and bounds the i32 casts for
        // the weekly INTEGER columns. Lifetime and daily counters are BIGINT.
        let batch = batch.clamped();

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

        // First-party suggestion signal: aggregate per-(commander, card) tallies.
        // Pure aggregate — no user_id. Clamped above, so the list is bounded and
        // each tally fits the BIGINT accumulation.
        for sig in &batch.signals {
            query!(
                r#"INSERT INTO commander_card_signal
                       (commander_oracle_id, card_oracle_id, shown, added, skipped, maybed, removed, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                   ON CONFLICT (commander_oracle_id, card_oracle_id) DO UPDATE SET
                       shown      = commander_card_signal.shown    + EXCLUDED.shown,
                       added      = commander_card_signal.added    + EXCLUDED.added,
                       skipped    = commander_card_signal.skipped  + EXCLUDED.skipped,
                       maybed     = commander_card_signal.maybed   + EXCLUDED.maybed,
                       removed    = commander_card_signal.removed  + EXCLUDED.removed,
                       updated_at = NOW()"#,
                sig.commander_oracle_id,
                sig.card_oracle_id,
                sig.shown as i64,
                sig.added as i64,
                sig.skipped as i64,
                sig.maybed as i64,
                sig.removed as i64,
            )
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        }

        // Per-user mirror of the aggregate signal: same deltas, user-keyed.
        // Feeds future personalization; nothing consumes it yet.
        for sig in &batch.signals {
            query!(
                r#"INSERT INTO user_card_signal
                       (user_id, commander_oracle_id, card_oracle_id, shown, added, skipped, maybed, removed, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
                   ON CONFLICT (user_id, commander_oracle_id, card_oracle_id) DO UPDATE SET
                       shown      = user_card_signal.shown    + EXCLUDED.shown,
                       added      = user_card_signal.added    + EXCLUDED.added,
                       skipped    = user_card_signal.skipped  + EXCLUDED.skipped,
                       maybed     = user_card_signal.maybed   + EXCLUDED.maybed,
                       removed    = user_card_signal.removed  + EXCLUDED.removed,
                       updated_at = NOW()"#,
                user_id,
                sig.commander_oracle_id,
                sig.card_oracle_id,
                sig.shown as i64,
                sig.added as i64,
                sig.skipped as i64,
                sig.maybed as i64,
                sig.removed as i64,
            )
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        }

        // Commander-select signal: pooled shown/selected/skipped per candidate.
        // Pure aggregate — no user_id, no per-user mirror (deliberately the
        // lighter posture; see context/archive/commander_select_signal.md).
        for sig in &batch.select_signals {
            query!(
                r#"INSERT INTO commander_select_signal
                       (commander_oracle_id, shown, selected, skipped, updated_at)
                   VALUES ($1, $2, $3, $4, NOW())
                   ON CONFLICT (commander_oracle_id) DO UPDATE SET
                       shown      = commander_select_signal.shown    + EXCLUDED.shown,
                       selected   = commander_select_signal.selected + EXCLUDED.selected,
                       skipped    = commander_select_signal.skipped  + EXCLUDED.skipped,
                       updated_at = NOW()"#,
                sig.commander_oracle_id,
                sig.shown as i64,
                sig.selected as i64,
                sig.skipped as i64,
            )
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        }

        // Weekly scalar counters (ISO week, Monday UTC). Clamped inputs keep
        // the per-flush sums (≤1,000 signals × ≤10,000 each) well inside i32.
        let added_sum: i64 = batch.signals.iter().map(|s| s.added as i64).sum();
        let skipped_sum: i64 = batch.signals.iter().map(|s| s.skipped as i64).sum();
        let maybed_sum: i64 = batch.signals.iter().map(|s| s.maybed as i64).sum();
        let removed_sum: i64 = batch.signals.iter().map(|s| s.removed as i64).sum();
        query!(
            r#"INSERT INTO user_week_signal
                   (user_id, week_start, swipes_right, swipes_left, swipes_up, swipes_down, searches, added, skipped, maybed, removed)
               VALUES ($1, (date_trunc('week', now() AT TIME ZONE 'utc'))::date, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               ON CONFLICT (user_id, week_start) DO UPDATE SET
                   swipes_right = user_week_signal.swipes_right + EXCLUDED.swipes_right,
                   swipes_left  = user_week_signal.swipes_left  + EXCLUDED.swipes_left,
                   swipes_up    = user_week_signal.swipes_up    + EXCLUDED.swipes_up,
                   swipes_down  = user_week_signal.swipes_down  + EXCLUDED.swipes_down,
                   searches     = user_week_signal.searches     + EXCLUDED.searches,
                   added        = user_week_signal.added        + EXCLUDED.added,
                   skipped      = user_week_signal.skipped      + EXCLUDED.skipped,
                   maybed       = user_week_signal.maybed       + EXCLUDED.maybed,
                   removed      = user_week_signal.removed      + EXCLUDED.removed"#,
            user_id,
            r32,
            l32,
            u32,
            d32,
            s32,
            added_sum as i32,
            skipped_sum as i32,
            maybed_sum as i32,
            removed_sum as i32,
        )
        .execute(&mut *tx)
        .await
        .map_err(db)?;

        // Weekly facet counters: what kinds of cards this user accepted, by
        // mechanical category and color identity (colorless → 'C').
        let added_oracle_ids: Vec<Uuid> = batch
            .signals
            .iter()
            .filter(|s| s.added > 0)
            .map(|s| s.card_oracle_id)
            .collect();
        if !added_oracle_ids.is_empty() {
            let rows = query!(
                r#"SELECT lc.oracle_id, lc.color_identity, cp.mechanical_categories
                   FROM latest_cards lc
                   JOIN card_profiles cp ON cp.scryfall_data_id = lc.id
                   WHERE lc.oracle_id = ANY($1)"#,
                &added_oracle_ids[..],
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(db)?;

            let mut card_facets: HashMap<Uuid, (Vec<String>, Vec<String>)> = HashMap::new();
            for row in rows {
                let Some(oracle_id) = row.oracle_id else {
                    continue;
                };
                let colors = row.color_identity.unwrap_or_default();
                let categories = row
                    .mechanical_categories
                    .as_ref()
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|c| c.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                card_facets.insert(oracle_id, (colors, categories));
            }

            let mut tallies: HashMap<(&str, String), i64> = HashMap::new();
            for sig in batch.signals.iter().filter(|s| s.added > 0) {
                let Some((colors, categories)) = card_facets.get(&sig.card_oracle_id) else {
                    continue;
                };
                let added = sig.added as i64;
                for category in categories {
                    *tallies.entry(("category", category.clone())).or_default() += added;
                }
                if colors.is_empty() {
                    *tallies.entry(("color", "C".to_string())).or_default() += added;
                } else {
                    for color in colors {
                        *tallies.entry(("color", color.clone())).or_default() += added;
                    }
                }
            }

            for ((facet, key), added) in &tallies {
                query!(
                    r#"INSERT INTO user_week_facet_signal (user_id, week_start, facet, key, added)
                       VALUES ($1, (date_trunc('week', now() AT TIME ZONE 'utc'))::date, $2, $3, $4)
                       ON CONFLICT (user_id, week_start, facet, key) DO UPDATE SET
                           added = user_week_facet_signal.added + EXCLUDED.added"#,
                    user_id,
                    facet,
                    key,
                    *added as i32,
                )
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            }
        }

        // Deck suppression deltas (skips). Ownership is checked per delta; a
        // delta for someone else's deck is dropped silently rather than
        // failing the whole batch. Unskips only clear skip-sourced rows so an
        // undo can't erase a removal suppression.
        for delta in &batch.deck_skips {
            let owner = query!(r#"SELECT user_id FROM decks WHERE id = $1"#, delta.deck_id,)
                .fetch_optional(&mut *tx)
                .await
                .map_err(db)?;
            if owner.map(|row| row.user_id) != Some(user_id) {
                continue;
            }

            if !delta.skipped.is_empty() {
                query!(
                    r#"INSERT INTO deck_card_suppressions (deck_id, oracle_id, source)
                       SELECT $1, unnest($2::uuid[]), 'skip'
                       ON CONFLICT (deck_id, oracle_id) DO UPDATE SET
                           source = EXCLUDED.source,
                           suppressed_at = now()"#,
                    delta.deck_id,
                    &delta.skipped[..],
                )
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            }

            if !delta.unskipped.is_empty() {
                query!(
                    r#"DELETE FROM deck_card_suppressions
                       WHERE deck_id = $1 AND oracle_id = ANY($2) AND source = 'skip'"#,
                    delta.deck_id,
                    &delta.unskipped[..],
                )
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            }

            query!(
                r#"DELETE FROM deck_card_suppressions
                   WHERE deck_id = $1 AND oracle_id IN (
                       SELECT oracle_id FROM deck_card_suppressions
                       WHERE deck_id = $1
                       ORDER BY suppressed_at DESC
                       OFFSET $2
                   )"#,
                delta.deck_id,
                MAX_SUPPRESSIONS_PER_DECK,
            )
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        }

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

    async fn record_audit(&self, user_id: Uuid, action: AuditAction) -> Result<(), MetricsError> {
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

    async fn record_anonymous_event(
        &self,
        session_id: Uuid,
        kind: AnonymousEventKind,
    ) -> Result<(), MetricsError> {
        let kind_str = kind.as_str();
        query!(
            r#"INSERT INTO anonymous_events (session_id, kind)
               VALUES ($1, $2)"#,
            session_id,
            kind_str,
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
