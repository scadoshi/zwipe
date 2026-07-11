//! Card data and Scryfall sync repository implementation.

/// Database-to-domain card profile conversion.
pub mod card_profile;
/// SQLx error-to-domain error mappings.
pub mod error;
/// SQL generation helpers for bulk upserts and field binding.
pub mod helpers;
/// Database-to-domain Scryfall data conversion.
pub mod models;
/// Sync metrics JSONB codecs and database model.
pub mod zervice_metrics;

use crate::{
    domain::card::{
        models::{
            helpers::SleeveCardProfile, search_card::error::SearchCardsError,
            zervice_metrics::ZerviceMetrics,
        },
        ports::CardRepository,
        requests::{
            create_card::CreateCardError,
            get_artists::GetArtistsError,
            get_card::GetCardError,
            get_card_profile::{CardProfileIds, GetCardProfile, GetCardProfileError},
            get_card_types::GetCardTypesError,
            get_keywords::GetKeywordsError,
            get_languages::GetLanguagesError,
            get_oracle_words::GetOracleWordsError,
            get_scryfall_data::{
                GetScryfallData, GetScryfallDataError, ScryfallDataIds, SearchScryfallDataError,
            },
            get_sets::GetSetsError,
        },
    },
    outbound::sqlx::{
        card::{
            card_profile::DatabaseCardProfile,
            helpers::upsert_card::{
                BatchDeltaUpsertWithTx, BatchUpsertWithTx, BulkUpsertWithTx, SingleUpsertWithTx,
            },
            models::DatabaseScryfallData,
            zervice_metrics::DatabaseZerviceMetrics,
        },
        postgres::Postgres as MyPostgres,
    },
};
use zwipe_core::domain::{
    card::{
        Card,
        card_profile::CardProfile,
        scryfall_data::ScryfallData,
        search_card::card_filter::{
            CardQuery, card_sort_key::CardSortKey, criteria::PLAYABLE_LAYOUTS,
        },
    },
    deck::Format,
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder, query_as, query_builder::Separated, query_scalar};

/// Hard ceiling on rows returned by a single card search.
///
/// `CardQuery::limit` arrives from untrusted request JSON; without a cap a
/// client could ask for an arbitrarily large page and force the DB to
/// materialize and serialize it. The frontend default is 25; 250 is generous
/// headroom. Note `CardQuery` is also used for *client-side* in-memory
/// filtering with much larger limits, but that path never reaches this query.
const MAX_SEARCH_LIMIT: u32 = 250;

// Default-synergy-ordering dials (context/plans/suggestion_signal.md,
// Phase 3a+3b band revision). `W_SIGNAL = 0.0` and `BAND_SIZE = 1` together
// reproduce the pre-signal score ordering exactly — that's the revert lever.

/// Weight of the pooled add-rate term (centered on the global rate, so a
/// card with no data contributes exactly zero). Units are synergy-score
/// points (scores span roughly -0.6..1.0, measured 2026-07-06); enough to
/// migrate a proven card across a band boundary over time.
const W_SIGNAL: f64 = 0.15;

/// Band size for the default-ordering shuffle: cards ranked by
/// (base + signal) score are cut into hands of this many, bands served in
/// strict order, position within a band purely the (card, deck, day) hash.
/// 25 matches the client page size, so each page is exactly one shuffled
/// hand — cast-level variety per deck per day without a band-2 card ever
/// leading band 1. 1 = pure score ordering (no shuffle).
const BAND_SIZE: i64 = 25;

/// Shrinkage pseudo-count: impressions a card needs before its own add-rate
/// outweighs the global prior.
const SHRINK_K: f64 = 10.0;

/// Base score for cards absent from the commander's synergy map. Sits below
/// the scoreless-list floor (-10, see `SynergyPayload::into_scores`) so the
/// unscored tail stays below every scored card — at zero dials this exactly
/// reproduces the old `NULLS LAST` ordering. Signal and jitter still shuffle
/// the tail internally; letting strong signal lift tail cards into the scored
/// region is a deliberate future retune (raise this anchor), not v1.
const UNSCORED_ANCHOR: f64 = -10.5;

// Wildcard slot dials (context/archive/wildcard_slot/).

/// Wildcard slots per page: positions reserved for cards drawn from beyond
/// the reachable horizon (rank > DEEP_POOL_FLOOR), least-shown first, so the
/// deep pool accrues impressions at all. 0 reverts to pure band serving.
const WILDCARD_SLOTS: i64 = 1;

/// The reachable horizon: mirrors the client add-stack hard cap (zwiper
/// `MAX_CARDS_IN_STACK`, action_history.rs — zerver can't import it). Cards
/// ranked past this are structurally unreachable by band pagination within a
/// session, so no signal can ever accrue for them without the wildcard probe.
const DEEP_POOL_FLOOR: i64 = 500;

/// 0-based in-page index where a wildcard lands: deep enough not to lead the
/// hand, early enough to be reliably seen before the page is exhausted.
const WILDCARD_POSITION: usize = 17;

/// Base ranking for the commander-select ordering: decks-helmed popularity
/// first (`commander_popularity`, written by the synergy worker), then
/// `edhrec_rank` as the fallback for commanders the table doesn't cover, then
/// name, then id for a stable tiebreak. Shared by the wildcard CTE's
/// `row_number()` and the non-wildcard banded ORDER BY. Absent popularity rows
/// sort last, so an empty table degrades to pure `edhrec_rank` — the revert
/// lever. (context/archive/commander_select_ordering.md §2.)
const POPULARITY_RANK: &str = "pop.pop_decks DESC NULLS LAST, latest_cards.edhrec_rank ASC NULLS LAST, latest_cards.name ASC, latest_cards.id";

/// The popularity join, aliased so it exposes only `pop_decks` — never a bare
/// `name`/`oracle_id`/`decks`, which would collide with `latest_cards` and the
/// shared WHERE filters (name search, exclude-oracle_ids) and make those
/// columns ambiguous. Keyed on the real card's oracle_id.
const POPULARITY_JOIN: &str = "LEFT JOIN (SELECT oracle_id AS pop_oracle_id, decks AS pop_decks FROM commander_popularity) pop ON pop.pop_oracle_id = latest_cards.oracle_id";

/// First-party select-signal join for the commander-select wildcard's
/// least-shown deep slice, aliased for the same ambiguity reason as
/// [`POPULARITY_JOIN`]. An absent row COALESCEs to 0 impressions, so an empty
/// table leaves the deep slice on the daily shuffle alone — the dormant-until-
/// data-accrues behavior (context/archive/commander_select_signal.md §3).
const SELECT_SIGNAL_JOIN: &str = "LEFT JOIN (SELECT commander_oracle_id AS sel_oid, shown AS sel_shown FROM commander_select_signal) sel ON sel.sel_oid = latest_cards.oracle_id";

impl CardRepository for MyPostgres {
    // ========
    //  create
    // ========
    async fn upsert(&self, scryfall_data: &ScryfallData) -> Result<Card, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let card = scryfall_data.single_upsert_with_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(card)
    }

    async fn bulk_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let cards = multiple_scryfall_data.bulk_upsert_with_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(cards)
    }

    async fn batch_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        zervice_metrics: &mut ZerviceMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let cards = multiple_scryfall_data
            .batch_upsert_with_tx(&mut tx, batch_size, zervice_metrics)
            .await?;
        tx.commit().await?;
        Ok(cards)
    }

    async fn batch_delta_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        zervice_metrics: &mut ZerviceMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let cards = multiple_scryfall_data
            .batch_delta_upsert_with_tx(&mut tx, batch_size, zervice_metrics)
            .await?;
        tx.commit().await?;
        Ok(cards)
    }

    /// Persists a completed sync run to `zervice_metrics`.
    async fn record_zervice_metrics(
        &self,
        zervice_metrics: &ZerviceMetrics,
    ) -> Result<ZerviceMetrics, anyhow::Error> {
        let mut tx = self.pool.begin().await?;
        let query_sql = "INSERT INTO zervice_metrics".to_string()
            + " (started_at, ended_at, duration_in_seconds, status, received_count, upserted_count, skipped_count, error_count, errors)"
            + " VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *";
        let database_zervice_metrics: DatabaseZerviceMetrics = query_as(&query_sql)
            .bind(zervice_metrics.started_at())
            .bind(zervice_metrics.ended_at())
            .bind(zervice_metrics.duration_in_seconds())
            .bind(zervice_metrics.status().to_string())
            .bind(zervice_metrics.received_count())
            .bind(zervice_metrics.upserted_count())
            .bind(zervice_metrics.skipped_count())
            .bind(zervice_metrics.error_count())
            .bind(zervice_metrics.errors())
            .fetch_one(&mut *tx)
            .await?;
        let zervice_metrics: ZerviceMetrics = database_zervice_metrics.try_into()?;
        tx.commit().await?;
        Ok(zervice_metrics)
    }

    async fn refresh_latest_cards(&self) -> anyhow::Result<()> {
        sqlx::query("REFRESH MATERIALIZED VIEW latest_cards")
            .execute(&self.pool)
            .await
            .context("failed to refresh latest_cards materialized view")?;
        Ok(())
    }

    async fn refresh_card_signal_rollup(&self) -> anyhow::Result<()> {
        sqlx::query("REFRESH MATERIALIZED VIEW card_signal_rollup")
            .execute(&self.pool)
            .await
            .context("failed to refresh card_signal_rollup materialized view")?;
        Ok(())
    }

    async fn sync_oracle_tags(
        &self,
        tags: &[crate::inbound::external::scryfall::oracle_tag::OracleTag],
    ) -> anyhow::Result<(u32, u32)> {
        crate::outbound::sqlx::card::helpers::oracle_tags::sync_oracle_tags(&self.pool, tags).await
    }

    // =====
    //  get
    // =====
    async fn get_scryfall_data(
        &self,
        request: &GetScryfallData,
    ) -> Result<ScryfallData, GetScryfallDataError> {
        let db: DatabaseScryfallData = query_as("SELECT * FROM scryfall_data WHERE id = $1")
            .bind(**request)
            .fetch_one(&self.pool)
            .await?;
        let scryfall_data: ScryfallData = db.try_into().map_err(GetScryfallDataError::Database)?;

        Ok(scryfall_data)
    }

    async fn get_multiple_scryfall_data(
        &self,
        request: &ScryfallDataIds,
    ) -> Result<Vec<ScryfallData>, GetScryfallDataError> {
        let db_rows: Vec<DatabaseScryfallData> =
            query_as("SELECT * FROM scryfall_data WHERE id = ANY($1)")
                .bind(&**request)
                .fetch_all(&self.pool)
                .await?;
        let scryfall_data: Vec<ScryfallData> = db_rows
            .into_iter()
            .map(ScryfallData::try_from)
            .collect::<Result<_, _>>()
            .map_err(GetScryfallDataError::Database)?;

        Ok(scryfall_data)
    }

    /// Searches the `latest_cards` materialized view (pre-deduplicated to one row per
    /// oracle_id). Joins `card_profiles` for is_token / mechanical_categories filters.
    /// Filter clauses are composed with `AND` via `QueryBuilder::separated`.
    async fn search_scryfall_data(
        &self,
        request: &CardQuery,
    ) -> Result<Vec<ScryfallData>, SearchScryfallDataError> {
        self.search_scryfall_data_deck_aware(request, None, &[], None, false, None)
            .await
    }

    /// `search_scryfall_data` plus the deck-aware extras: oracle_id exclusion,
    /// suppression filtering for `deck_id` (skipped/removed cards), synergy-score
    /// default ordering, and (when `synergy_only`) a membership constraint to
    /// the commander's synergy pool. The plain search is this with no extras.
    ///
    /// `commander_seed` switches the engine into commander-select mode
    /// (context/archive/commander_select_ordering.md): decks-helmed popularity
    /// ordering, banded + wildcarded by that caller-supplied seed (typically
    /// `{user_id}:{date}` — no deck required), and token/emblem printings
    /// excluded from the candidate pool. Exposed as `search_commanders`; every
    /// other caller passes `None` and is unaffected.
    async fn search_scryfall_data_deck_aware(
        &self,
        request: &CardQuery,
        deck_id: Option<uuid::Uuid>,
        exclude_oracle_ids: &[uuid::Uuid],
        synergy_scores: Option<&serde_json::Value>,
        synergy_only: bool,
        commander_seed: Option<String>,
    ) -> Result<Vec<ScryfallData>, SearchScryfallDataError> {
        // WHERE clauses read the predicate fields; LIMIT/OFFSET/ORDER BY read
        // the query config — the CardCriteria/CardQuery split, mirrored here.
        let criteria = request.criteria();
        // Default synergy ordering (no explicit sort, score map present) gets
        // the signal + jitter terms, which need the pooled rollup and the
        // global rate in scope; every other path keeps the plain FROM.
        let signal_ordering = request.sort().is_none() && synergy_scores.is_some();
        // Commander-select mode: caller (search_commanders) supplied a shuffle
        // seed. Token/emblem exclusion applies whenever this is set; popularity
        // ordering + banding + wildcard apply only when the user hasn't pinned
        // an explicit sort (an explicit sort still wins, as everywhere).
        let commander_select = commander_seed.is_some();
        let popularity_ordering = commander_select && request.sort().is_none();
        // Wildcard serving (context/archive/wildcard_slot/): the banded serve
        // reserves WILDCARD_SLOTS per page for deep-pool probes. It needs the
        // ranked pool twice (band + deep slice), so the query becomes a CTE.
        // Synergy seeds by deck; commander-select seeds by `commander_seed`.
        let wildcard_serving =
            WILDCARD_SLOTS > 0 && ((signal_ordering && deck_id.is_some()) || popularity_ordering);
        // The (base + signal) score expression, shared by the wildcard CTE
        // header and the plain signal ORDER BY below:
        //   base: the commander's synergy score (unscored cards anchor below
        //         the scored floor, see UNSCORED_ANCHOR)
        //   signal: the pooled net-rate, shrunk toward and centered on the
        //         global rate — a card with no impressions adds zero.
        let push_score = |qb: &mut QueryBuilder<'_, Postgres>, scores: &serde_json::Value| {
            qb.push("COALESCE((");
            qb.push_bind(scores.clone());
            qb.push(format!(" ->> LOWER(name))::float8, {UNSCORED_ANCHOR})"));
            qb.push(format!(
                " + {W_SIGNAL} * ((COALESCE(sig.net, 0) + {SHRINK_K} * g.rate) \
                   / (COALESCE(sig.shown, 0) + {SHRINK_K}) - g.rate)"
            ));
        };
        // Banding/wildcard shuffle seed: synergy serves seed by deck+day,
        // commander-select by the caller's `commander_seed` (user+day, no deck).
        let band_seed: Option<String> = if popularity_ordering {
            commander_seed.clone()
        } else if signal_ordering {
            deck_id.map(|d| format!("{d}:{}", Utc::now().date_naive()))
        } else {
            None
        };
        let mut qb: QueryBuilder<'_, Postgres> = if wildcard_serving
            && signal_ordering
            && let (Some(scores), Some(seed)) = (synergy_scores, band_seed.clone())
        {
            // Synergy wildcard CTE: rank + shuffle + exposure computed once; the
            // two slices at the end page it two ways. The predicate pushes below
            // land inside the CTE, so filters, legality, and suppressions bound
            // both slices.
            let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new(
                "WITH pool AS (SELECT latest_cards.*, row_number() OVER (ORDER BY ",
            );
            push_score(&mut qb, scores);
            qb.push(
                " DESC, name ASC, latest_cards.id) AS rn, \
                     hashtext(COALESCE(latest_cards.oracle_id::text, '') || ",
            );
            qb.push_bind(seed);
            qb.push(
                ") AS shuffle, COALESCE(sig.shown, 0) AS pool_shown
             FROM latest_cards
             JOIN card_profiles ON latest_cards.id = card_profiles.scryfall_data_id
             LEFT JOIN card_signal_rollup sig ON sig.card_oracle_id = latest_cards.oracle_id
             CROSS JOIN (SELECT COALESCE(SUM(net) / NULLIF(SUM(shown), 0), 0) AS rate
                         FROM card_signal_rollup) g
             WHERE ",
            );
            qb
        } else if wildcard_serving
            && popularity_ordering
            && let Some(seed) = band_seed.clone()
        {
            // Commander-select wildcard CTE: same banded-pool + deep-probe
            // machinery as the synergy arm, ranked by decks-helmed popularity
            // instead of synergy score. The deep slice orders least-shown first
            // using the first-party select signal; with no signal rows yet the
            // COALESCE floors every pool_shown to 0 and the ordering collapses
            // to the daily shuffle alone — byte-identical to the pre-signal
            // behavior (context/archive/commander_select_signal.md §3). Both
            // joins are 1:1 (PK on oracle_id) and aliased so no bare
            // `name`/`oracle_id` collides with the shared WHERE filters.
            let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new(
                "WITH pool AS (SELECT latest_cards.*, row_number() OVER (ORDER BY ",
            );
            qb.push(POPULARITY_RANK);
            qb.push(
                ") AS rn, \
                     hashtext(COALESCE(latest_cards.oracle_id::text, '') || ",
            );
            qb.push_bind(seed);
            qb.push(format!(
                ") AS shuffle, COALESCE(sel.sel_shown, 0) AS pool_shown
             FROM latest_cards
             JOIN card_profiles ON latest_cards.id = card_profiles.scryfall_data_id
             {POPULARITY_JOIN}
             {SELECT_SIGNAL_JOIN}
             WHERE "
            ));
            qb
        } else if signal_ordering {
            QueryBuilder::new(
                "SELECT latest_cards.* FROM latest_cards
             JOIN card_profiles ON latest_cards.id = card_profiles.scryfall_data_id
             LEFT JOIN card_signal_rollup sig ON sig.card_oracle_id = latest_cards.oracle_id
             CROSS JOIN (SELECT COALESCE(SUM(net) / NULLIF(SUM(shown), 0), 0) AS rate
                         FROM card_signal_rollup) g
             WHERE ",
            )
        } else if popularity_ordering {
            // Non-wildcard commander-select (WILDCARD_SLOTS = 0 revert lever):
            // popularity join for the banded ORDER BY below, no deep-probe CTE.
            QueryBuilder::new(format!(
                "SELECT latest_cards.* FROM latest_cards
             JOIN card_profiles ON latest_cards.id = card_profiles.scryfall_data_id
             {POPULARITY_JOIN}
             WHERE "
            ))
        } else {
            QueryBuilder::new(
                "SELECT latest_cards.* FROM latest_cards
             JOIN card_profiles ON latest_cards.id = card_profiles.scryfall_data_id
             WHERE ",
            )
        };
        let mut sep: Separated<Postgres, &'static str> = qb.separated(" AND ");
        // Seed an always-true clause so the baked `WHERE` is valid even when no
        // filter conditions are pushed (e.g. an empty filter). It also lets every
        // real condition below rely on the `AND` separator: the first real push
        // becomes the second element, so it is correctly prefixed with ` AND `.
        sep.push("TRUE");

        // Commander-select candidate pool must exclude token/emblem printings.
        // The popularity table (written by the synergy worker) keys on the real
        // card's oracle_id and excludes these layouts, so serving must agree —
        // otherwise a same-named token could be offered as a commander and would
        // never join a real deck. Applies to every commander search, even one
        // with an explicit sort. (context/archive/commander_select_ordering.md §1.)
        if commander_select {
            sep.push("latest_cards.layout NOT IN ('token', 'double_faced_token', 'emblem')");
        }

        // Strip punctuation from DB columns for punctuation-insensitive text search.
        // The query values are already stripped by CardQueryBuilder setters.
        const STRIP_NAME: &str = "regexp_replace(name, '[^a-zA-Z0-9 ]', '', 'g') ILIKE ";
        const STRIP_TYPE: &str = "regexp_replace(type_line, '[^a-zA-Z0-9 ]', '', 'g') ILIKE ";

        if let Some(query_string) = &criteria.name_contains() {
            sep.push(STRIP_NAME);
            sep.push_bind_unseparated(format!("%{}%", query_string));
        }

        if let Some(query_string) = &criteria.name_not_contains() {
            sep.push("NOT (");
            sep.push_unseparated(STRIP_NAME);
            sep.push_bind_unseparated(format!("%{}%", query_string));
            sep.push_unseparated(")");
        }

        if let Some(query_string) = &criteria.type_line_contains() {
            sep.push(STRIP_TYPE);
            sep.push_bind_unseparated(format!("%{}%", query_string));
        }

        if let Some(query_string_array) = &criteria.type_line_contains_any() {
            sep.push(" (");
            query_string_array
                .iter()
                .enumerate()
                .for_each(|(i, query_string)| {
                    if i > 0 {
                        sep.push_unseparated(" OR ");
                    }
                    sep.push_unseparated(STRIP_TYPE);
                    sep.push_bind_unseparated(format!("%{}%", query_string));
                });
            sep.push_unseparated(") ");
        }

        if let Some(card_types) = &criteria.card_type_contains_any() {
            sep.push(" (");
            card_types.iter().enumerate().for_each(|(i, query_string)| {
                if i > 0 {
                    sep.push_unseparated(" OR ");
                }
                sep.push_unseparated(STRIP_TYPE);
                sep.push_bind_unseparated(format!("%{}%", query_string));
            });
            sep.push_unseparated(") ");
        }

        if let Some(query_string_array) = &criteria.type_line_contains_all() {
            for query_string in query_string_array.iter() {
                sep.push(STRIP_TYPE);
                sep.push_bind_unseparated(format!("%{}%", query_string));
            }
        }

        if let Some(card_types) = &criteria.card_type_contains_all() {
            for card_type in card_types.iter() {
                sep.push(STRIP_TYPE);
                sep.push_bind_unseparated(format!("%{}%", card_type));
            }
        }

        if let Some(query_string) = &criteria.type_line_not_contains() {
            sep.push("(type_line IS NULL OR NOT (");
            sep.push_unseparated(STRIP_TYPE);
            sep.push_bind_unseparated(format!("%{}%", query_string));
            sep.push_unseparated("))");
        }

        if let Some(query_string_array) = &criteria.type_line_excludes_any() {
            sep.push("(type_line IS NULL OR NOT (");
            query_string_array
                .iter()
                .enumerate()
                .for_each(|(i, query_string)| {
                    if i > 0 {
                        sep.push_unseparated(" OR ");
                    }
                    sep.push_unseparated(STRIP_TYPE);
                    sep.push_bind_unseparated(format!("%{}%", query_string));
                });
            sep.push_unseparated(")) ");
        }

        if let Some(card_types) = &criteria.card_type_excludes_any() {
            sep.push("(type_line IS NULL OR NOT (");
            card_types.iter().enumerate().for_each(|(i, query_string)| {
                if i > 0 {
                    sep.push_unseparated(" OR ");
                }
                sep.push_unseparated(STRIP_TYPE);
                sep.push_bind_unseparated(format!("%{}%", query_string));
            });
            sep.push_unseparated(")) ");
        }

        if let Some(sets) = criteria.set_equals_any() {
            sep.push("set_name = ANY(");
            sep.push_bind_unseparated(sets);
            sep.push_unseparated(")");
        }

        if let Some(artists) = criteria.artist_equals_any() {
            sep.push("artist = ANY(");
            sep.push_bind_unseparated(artists);
            sep.push_unseparated(")");
        }

        if let Some(rarities) = criteria.rarity_equals_any() {
            sep.push("rarity = ANY(");
            sep.push_bind_unseparated(rarities.to_short_names());
            sep.push_unseparated(")");
        }

        if let Some(sets) = criteria.set_excludes_any() {
            sep.push("NOT (set_name = ANY(");
            sep.push_bind_unseparated(sets);
            sep.push_unseparated("))");
        }

        if let Some(artists) = criteria.artist_excludes_any() {
            sep.push("(artist IS NULL OR NOT (artist = ANY(");
            sep.push_bind_unseparated(artists);
            sep.push_unseparated(")))");
        }

        if let Some(rarities) = criteria.rarity_excludes_any() {
            sep.push("NOT (rarity = ANY(");
            sep.push_bind_unseparated(rarities.to_short_names());
            sep.push_unseparated("))");
        }

        if let Some(query_string) = criteria.cmc_equals() {
            sep.push("cmc = ");
            sep.push_bind_unseparated(query_string);
        }

        if let Some(cmc_range) = criteria.cmc_range() {
            let lower = cmc_range.0.min(cmc_range.1);
            let higher = cmc_range.0.max(cmc_range.1);
            sep.push("cmc between ");
            sep.push_bind_unseparated(lower);
            sep.push("");
            sep.push_bind_unseparated(higher);
        }

        // Price range against the selected currency's JSONB price. NULLIF turns
        // empty/missing prices into NULL (excluded — no cast error), matching the
        // client predicate. json_key() is a fixed enum literal, not user input.
        if criteria.price_min().is_some() || criteria.price_max().is_some() {
            let col = format!(
                "NULLIF(prices->>'{}', '')::FLOAT8",
                criteria.price_currency().unwrap_or_default().json_key()
            );
            if let Some(min) = criteria.price_min() {
                sep.push(format!("{col} >= "));
                sep.push_bind_unseparated(min);
            }
            if let Some(max) = criteria.price_max() {
                sep.push(format!("{col} <= "));
                sep.push_bind_unseparated(max);
            }
        }

        if let Some(query_string) = criteria.power_equals() {
            sep.push("power ~ '^\\d+$' AND CAST(power AS INT) = ");
            sep.push_bind_unseparated(query_string);
        }

        if let Some(power_range) = criteria.power_range() {
            let lower = power_range.0.min(power_range.1);
            let higher = power_range.0.max(power_range.1);
            sep.push("power ~ '^\\d+$' AND CAST(power AS INT) between ");
            sep.push_bind_unseparated(lower);
            sep.push_unseparated(" AND ");
            sep.push_bind_unseparated(higher);
        }

        if let Some(query_string) = criteria.toughness_equals() {
            sep.push("toughness ~ '^\\d+$' AND CAST(toughness AS INT) = ");
            sep.push_bind_unseparated(query_string);
        }

        if let Some(toughness_range) = criteria.toughness_range() {
            let lower = toughness_range.0.min(toughness_range.1);
            let higher = toughness_range.0.max(toughness_range.1);
            sep.push("toughness ~ '^\\d+$' AND CAST(toughness AS INT) between ");
            sep.push_bind_unseparated(lower);
            sep.push_unseparated(" AND ");
            sep.push_bind_unseparated(higher);
        }

        if let Some(colors) = criteria.color_identity_equals() {
            sep.push("color_identity @> ");
            sep.push_bind_unseparated(colors.to_short_names());
            sep.push("color_identity <@ ");
            sep.push_bind_unseparated(colors.to_short_names());
        }

        if let Some(colors) = criteria.color_identity_within() {
            sep.push("color_identity <@ ");
            sep.push_bind_unseparated(colors.to_short_names());
        }

        const STRIP_ORACLE: &str = "regexp_replace(oracle_text, '[^a-zA-Z0-9 ]', '', 'g') ILIKE ";

        if let Some(query_string) = &criteria.oracle_text_contains() {
            sep.push(STRIP_ORACLE);
            sep.push_bind_unseparated(format!("%{}%", query_string));
        }

        if let Some(query_string_array) = &criteria.oracle_text_contains_any() {
            sep.push(" (");
            query_string_array
                .iter()
                .enumerate()
                .for_each(|(i, query_string)| {
                    if i > 0 {
                        sep.push_unseparated(" OR ");
                    }
                    sep.push_unseparated(STRIP_ORACLE);
                    sep.push_bind_unseparated(format!("%{}%", query_string));
                });
            sep.push_unseparated(") ");
        }

        if let Some(query_string_array) = &criteria.oracle_text_contains_all() {
            for query_string in query_string_array.iter() {
                sep.push(STRIP_ORACLE);
                sep.push_bind_unseparated(format!("%{}%", query_string));
            }
        }

        if let Some(query_string) = &criteria.oracle_text_not_contains() {
            sep.push("(oracle_text IS NULL OR NOT (");
            sep.push_unseparated(STRIP_ORACLE);
            sep.push_bind_unseparated(format!("%{}%", query_string));
            sep.push_unseparated("))");
        }

        if let Some(query_string_array) = &criteria.oracle_text_excludes_any() {
            sep.push("(oracle_text IS NULL OR NOT (");
            query_string_array
                .iter()
                .enumerate()
                .for_each(|(i, query_string)| {
                    if i > 0 {
                        sep.push_unseparated(" OR ");
                    }
                    sep.push_unseparated(STRIP_ORACLE);
                    sep.push_bind_unseparated(format!("%{}%", query_string));
                });
            sep.push_unseparated(")) ");
        }

        // Keywords are stored capitalized (e.g. "Flying") but UI lowercases them.
        // Use array_lowercase() via subquery to compare case-insensitively.
        if let Some(keywords) = &criteria.keywords_contains_any() {
            sep.push("(SELECT array_agg(lower(k)) FROM unnest(keywords) k) && ARRAY[");
            keywords.iter().enumerate().for_each(|(i, kw)| {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(kw.to_lowercase());
            });
            sep.push_unseparated("]::text[]");
        }

        if let Some(keywords) = &criteria.keywords_contains_all() {
            sep.push("(SELECT array_agg(lower(k)) FROM unnest(keywords) k) @> ARRAY[");
            keywords.iter().enumerate().for_each(|(i, kw)| {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(kw.to_lowercase());
            });
            sep.push_unseparated("]::text[]");
        }

        if let Some(keywords) = &criteria.keywords_excludes() {
            sep.push("(keywords IS NULL OR NOT ((SELECT array_agg(lower(k)) FROM unnest(keywords) k) && ARRAY[");
            keywords.iter().enumerate().for_each(|(i, kw)| {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(kw.to_lowercase());
            });
            sep.push_unseparated("]::text[]))");
        }

        if let Some(colors) = &criteria.produced_mana_contains_any() {
            sep.push("produced_mana && ARRAY[");
            colors.iter().enumerate().for_each(|(i, c)| {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(c.to_uppercase());
            });
            sep.push_unseparated("]::text[]");
        }

        if let Some(colors) = &criteria.produced_mana_contains_all() {
            sep.push("produced_mana @> ARRAY[");
            colors.iter().enumerate().for_each(|(i, c)| {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(c.to_uppercase());
            });
            sep.push_unseparated("]::text[]");
        }

        if let Some(colors) = &criteria.produced_mana_excludes() {
            sep.push("(produced_mana IS NULL OR NOT (produced_mana && ARRAY[");
            colors.iter().enumerate().for_each(|(i, c)| {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(c.to_uppercase());
            });
            sep.push_unseparated("]::text[]))");
        }

        if let Some(query_string) = &criteria.flavor_text_contains() {
            sep.push("regexp_replace(flavor_text, '[^a-zA-Z0-9 ]', '', 'g') ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", query_string));
        }

        if let Some(query_string) = &criteria.flavor_text_not_contains() {
            sep.push("(flavor_text IS NULL OR NOT (regexp_replace(flavor_text, '[^a-zA-Z0-9 ]', '', 'g') ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", query_string));
            sep.push_unseparated("))");
        }

        if let Some(has_flavor_text) = criteria.has_flavor_text() {
            if has_flavor_text {
                sep.push("flavor_text IS NOT NULL AND flavor_text != ''");
            } else {
                sep.push("(flavor_text IS NULL OR flavor_text = '')");
            }
        }

        // flag filters
        if let Some(is_tok) = criteria.is_token() {
            sep.push(" card_profiles.is_token = ");
            sep.push_bind_unseparated(is_tok);
        }

        if let Some(is_playable) = criteria.is_playable()
            && is_playable
        {
            // Only playable layouts
            sep.push("latest_cards.layout = ANY(");
            sep.push_bind_unseparated(PLAYABLE_LAYOUTS);
            sep.push_unseparated(")");
        } else if let Some(is_playable) = criteria.is_playable()
            && !is_playable
        {
            // Only non-playable layouts
            sep.push("latest_cards.layout != ALL(");
            sep.push_bind_unseparated(PLAYABLE_LAYOUTS);
            sep.push_unseparated(")");
        }

        if let Some(is_digital) = criteria.digital() {
            sep.push("latest_cards.digital = ");
            sep.push_bind_unseparated(is_digital);
        }

        if let Some(is_oversized) = criteria.oversized() {
            sep.push("latest_cards.oversized = ");
            sep.push_bind_unseparated(is_oversized);
        }

        if let Some(is_promo) = criteria.promo() {
            sep.push("latest_cards.promo = ");
            sep.push_bind_unseparated(is_promo);
        }

        if let Some(has_warning) = criteria.content_warning() {
            if has_warning {
                sep.push("latest_cards.content_warning = true");
            } else {
                // Hide cards with warnings (include false OR null)
                sep.push("(latest_cards.content_warning = false OR latest_cards.content_warning IS NULL)");
            }
        }

        if let Some(language) = criteria.language() {
            sep.push("latest_cards.lang = ");
            sep.push_bind_unseparated(language);
        }

        if let Some(formats) = criteria.legalities_contains_any() {
            sep.push("(");
            for (i, format_key) in formats.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(" OR ");
                }
                sep.push_unseparated("legalities->>");
                sep.push_bind_unseparated(format_key.clone());
                sep.push_unseparated(" IN ('legal', 'restricted')");
            }
            sep.push_unseparated(")");
        }

        if let Some(format) = criteria.is_commander_in_format() {
            match format {
                // Legendary creature, legendary vehicle with P/T, or "can be your commander"
                Format::Commander | Format::Duel | Format::Predh => {
                    sep.push(
                        "((type_line ILIKE '%Legendary%' AND type_line ILIKE '%Creature%') \
                         OR (type_line ILIKE '%Legendary%' AND power IS NOT NULL AND toughness IS NOT NULL) \
                         OR oracle_text ILIKE '%can be your commander%')",
                    );
                }
                // Legendary creature or legendary planeswalker
                Format::Brawl | Format::StandardBrawl | Format::HistoricBrawl => {
                    sep.push(
                        "(type_line ILIKE '%Legendary%' AND \
                         (type_line ILIKE '%Creature%' OR type_line ILIKE '%Planeswalker%'))",
                    );
                }
                // Uncommon creature — legendary or not. Two fixes here:
                //   1. Rarity is stored as the short code ('U'), not the word
                //      'uncommon' — the old literal matched nothing.
                //   2. PDH eligibility is "has appeared at uncommon in ANY
                //      printing", not "this cached printing is uncommon", so we
                //      check all printings via scryfall_data (catches cards whose
                //      preferred printing is common but were printed uncommon).
                Format::PauperCommander => {
                    sep.push(
                        "(type_line ILIKE '%Creature%' AND EXISTS (\
                         SELECT 1 FROM scryfall_data sd2 \
                         WHERE sd2.oracle_id = latest_cards.oracle_id \
                         AND sd2.rarity = 'U'))",
                    );
                }
                // Any planeswalker
                Format::Oathbreaker => {
                    sep.push("type_line ILIKE '%Planeswalker%'");
                }
                // Non-commander formats: no filter (should not happen, but safe)
                _ => {}
            }
        }

        // partner/background/spell filters
        if let Some(true) = criteria.is_partner() {
            sep.push(
                "(type_line ILIKE '%Legendary%' AND type_line ILIKE '%Creature%' AND (\
                 keywords @> ARRAY['Partner']::text[] \
                 OR keywords @> ARRAY['Friends forever']::text[] \
                 OR keywords @> ARRAY['Doctor''s companion']::text[] \
                 OR oracle_text ILIKE '%partner with%'))",
            );
        }

        if let Some(true) = criteria.is_background() {
            sep.push(
                "(type_line ILIKE '%Legendary%' AND type_line ILIKE '%Enchantment%' \
                 AND type_line ILIKE '%Background%')",
            );
        }

        if let Some(true) = criteria.is_signature_spell() {
            sep.push("(type_line ILIKE '%Instant%' OR type_line ILIKE '%Sorcery%')");
        }

        // mechanical category filters
        if let Some(categories) = criteria.mechanical_categories_contains_any() {
            sep.push("(card_profiles.mechanical_categories ?| ");
            sep.push_bind_unseparated(categories.to_vec());
            sep.push_unseparated(")");
        }

        if let Some(categories) = criteria.mechanical_categories_contains_all() {
            let json = serde_json::to_value(categories).unwrap_or_default();
            sep.push("(card_profiles.mechanical_categories @> ");
            sep.push_bind_unseparated(json);
            sep.push_unseparated(")");
        }

        if let Some(categories) = criteria.mechanical_categories_excludes() {
            sep.push("NOT (card_profiles.mechanical_categories ?| ");
            sep.push_bind_unseparated(categories.to_vec());
            sep.push_unseparated(")");
        }

        // Deck-aware exclusion: omit cards already in the deck. Null-oracle
        // printings are kept — they can't match a deck's oracle_ids anyway,
        // and a bare NOT(= ANY) would NULL them out of the results.
        if !exclude_oracle_ids.is_empty() {
            sep.push("(oracle_id IS NULL OR NOT (oracle_id = ANY(");
            sep.push_bind_unseparated(exclude_oracle_ids.to_vec());
            sep.push_unseparated(")))");
        }

        // Suppression filtering: the deck's skipped/removed cards never come
        // back through the deck-aware search (Clear skips is the escape
        // hatch). NOT EXISTS rather than a bind array — the set can be
        // thousands of rows. Null-oracle printings pass, matching the
        // exclusion clause above.
        if let Some(deck_id) = deck_id {
            sep.push("NOT EXISTS (SELECT 1 FROM deck_card_suppressions dcs WHERE dcs.deck_id = ");
            sep.push_bind_unseparated(deck_id);
            sep.push_unseparated(" AND dcs.oracle_id = latest_cards.oracle_id)");
        }

        // Synergy ON: constrain to the commander's synergy pool (membership).
        // Same per-row jsonb probe the synergy ORDER BY uses; the user's sort, if
        // any, then applies within this set. Skipped when no score map (cold
        // cache / no commander) so it gracefully falls back to the full pool.
        if synergy_only && let Some(scores) = synergy_scores {
            sep.push("(");
            sep.push_bind_unseparated(scores.clone());
            sep.push_unseparated(" ->> LOWER(name)) IS NOT NULL");
        }

        // Filter out NULLs for sorted field
        if let Some(order_by) = request.sort() {
            let null_filter = match order_by {
                CardSortKey::Power => Some("power IS NOT NULL AND power ~ '^\\d+$'"),
                CardSortKey::Toughness => Some("toughness IS NOT NULL AND toughness ~ '^\\d+$'"),
                CardSortKey::PriceUsd => {
                    Some("prices->>'usd' IS NOT NULL AND prices->>'usd' != ''")
                }
                CardSortKey::PriceEur => {
                    Some("prices->>'eur' IS NOT NULL AND prices->>'eur' != ''")
                }
                CardSortKey::PriceTix => {
                    Some("prices->>'tix' IS NOT NULL AND prices->>'tix' != ''")
                }
                _ => None,
            };
            if let Some(filter) = null_filter {
                sep.push(filter);
            }
        }

        // ORDER BY
        if let Some(order_by) = request.sort() {
            qb.push(" ORDER BY ");
            let col = match order_by {
                CardSortKey::Name => "name",
                CardSortKey::Cmc => "cmc",
                CardSortKey::Power => "CAST(NULLIF(power, '') AS INT)",
                CardSortKey::Toughness => "CAST(NULLIF(toughness, '') AS INT)",
                CardSortKey::Rarity => "rarity",
                CardSortKey::ReleasedAt => "released_at",
                CardSortKey::PriceUsd => "(prices->>'usd')::NUMERIC",
                CardSortKey::PriceEur => "(prices->>'eur')::NUMERIC",
                CardSortKey::PriceTix => "(prices->>'tix')::NUMERIC",
                CardSortKey::EdhrecRank => "edhrec_rank",
                CardSortKey::Random => "RANDOM()",
            };
            qb.push(col);
            if order_by != CardSortKey::Random {
                qb.push(if request.ascending() { " ASC" } else { " DESC" });
            }
            // edhrec_rank is nullable (obscure/new cards lack a rank): keep them
            // but sort last in either direction, with a name tiebreak so paging
            // through the unranked tail stays stable.
            if order_by == CardSortKey::EdhrecRank {
                qb.push(" NULLS LAST, name ASC");
            }
        } else if wildcard_serving {
            // Close the pool CTE and take two slices of it
            // (context/archive/wildcard_slot/server.md):
            //   band slice: the normal banded page, WILDCARD_SLOTS narrower.
            //         Offsets are consumption-aligned (page_index * band
            //         width), so no ranked card is ever skipped between
            //         pages — the client advances by its page size but
            //         dedups by id, so the server owns the math here.
            //   deep slice: the probe — cards beyond the reachable horizon,
            //         least-shown first, then the daily shuffle; pages walk
            //         the deep list so a probe never repeats within a day.
            //         A pool that never reaches the horizon (tight filters,
            //         synergy ON) yields an empty slice and pure band
            //         serving, automatically.
            // The outer ORDER BY re-sorts deterministically (UNION ALL order
            // is not guaranteed): band cards in band order, probes last —
            // the Rust splice below lifts them to WILDCARD_POSITION.
            let limit = i64::from(request.limit().min(MAX_SEARCH_LIMIT));
            let band_limit = (limit - WILDCARD_SLOTS).max(1);
            let page_index = i64::from(request.offset().min(i32::MAX as u32)) / limit.max(1);
            qb.push(format!(
                ") SELECT * FROM ((SELECT *, 0 AS slice FROM pool \
                 ORDER BY (rn - 1) / {BAND_SIZE}, shuffle, name ASC LIMIT "
            ));
            qb.push_bind(band_limit);
            qb.push(" OFFSET ");
            qb.push_bind(page_index * band_limit);
            qb.push(format!(
                ") UNION ALL (SELECT *, 1 AS slice FROM pool WHERE rn > {DEEP_POOL_FLOOR} \
                 ORDER BY pool_shown ASC, shuffle LIMIT {WILDCARD_SLOTS} OFFSET "
            ));
            qb.push_bind(page_index * WILDCARD_SLOTS);
            qb.push(format!(
                ")) AS combined ORDER BY slice ASC, (rn - 1) / {BAND_SIZE}, shuffle, name ASC"
            ));
        } else if let Some(scores) = synergy_scores {
            // Default synergy ordering: band shuffle over (base + signal)
            // (context/plans/suggestion_signal.md, Phase 3a+3b band revision).
            //   score: the commander's synergy score (unscored cards anchor
            //          below the scored floor, see UNSCORED_ANCHOR) + the
            //          pooled net-rate, shrunk toward and centered on the
            //          global rate — a card with no impressions adds zero.
            //   bands: cards ranked by score are cut into BAND_SIZE hands.
            //          Bands stay in strict order; position *within* a band
            //          is purely the (card, deck, day) hash — a different
            //          opening hand per deck per day, while a band-2 card
            //          can never lead band 1. Score-jitter was tried first
            //          and replaced: it permutes positions but never rotates
            //          the visible cast, which reads as "same order"
            //          (2026-07-06 live Krenko tests at 0.01/0.04/0.08).
            qb.push(" ORDER BY ");
            if let Some(deck_id) = deck_id {
                let seed = format!("{deck_id}:{}", Utc::now().date_naive());
                qb.push("(row_number() OVER (ORDER BY ");
                push_score(&mut qb, scores);
                qb.push(format!(
                    " DESC, name ASC, latest_cards.id) - 1) / {BAND_SIZE}, "
                ));
                // COALESCE the oracle_id: it is nullable, and NULL || seed
                // would NULL the shuffle key (caught by the dev harness,
                // 2026-07-06).
                qb.push("hashtext(COALESCE(latest_cards.oracle_id::text, '') || ");
                qb.push_bind(seed);
                qb.push("), name ASC");
            } else {
                // No deck to seed by (plain search): pure score ordering.
                push_score(&mut qb, scores);
                qb.push(" DESC, name ASC");
            }
        } else if popularity_ordering {
            // Commander-select without a wildcard (WILDCARD_SLOTS = 0): band
            // shuffle over the decks-helmed popularity base, same machinery as
            // the synergy branch above with POPULARITY_RANK as the score.
            // band_seed is guaranteed present (popularity_ordering requires it).
            if let Some(seed) = band_seed.clone() {
                qb.push(" ORDER BY (row_number() OVER (ORDER BY ");
                qb.push(POPULARITY_RANK);
                qb.push(format!(") - 1) / {BAND_SIZE}, "));
                qb.push("hashtext(COALESCE(latest_cards.oracle_id::text, '') || ");
                qb.push_bind(seed);
                qb.push("), latest_cards.name ASC");
            }
        }

        // The wildcard CTE carries per-slice LIMIT/OFFSET above.
        if !wildcard_serving {
            qb.push(" LIMIT ");
            qb.push_bind(request.limit().min(MAX_SEARCH_LIMIT) as i32);

            // Guard the u32->i32 cast: a value above i32::MAX wraps negative, and
            // Postgres rejects a negative OFFSET (errors the whole query).
            qb.push(" OFFSET ");
            qb.push_bind(request.offset().min(i32::MAX as u32) as i32);
        }

        let db_rows: Vec<DatabaseScryfallData> = qb.build_query_as().fetch_all(&self.pool).await?;
        let mut scryfall_data: Vec<ScryfallData> = db_rows
            .into_iter()
            .map(ScryfallData::try_from)
            .collect::<Result<_, _>>()
            .map_err(SearchScryfallDataError::Database)?;

        // Wildcard splice: the outer ORDER BY sorts probes after the band
        // page, so any rows past the band width are wildcards — lift them to
        // a fixed mid-hand position. A short band page (pool exhausted) can
        // hide a probe inside the band width; it then simply serves at the
        // tail, which is fine.
        if wildcard_serving {
            let band_limit = (request.limit().min(MAX_SEARCH_LIMIT) as usize)
                .saturating_sub(WILDCARD_SLOTS as usize)
                .max(1);
            if scryfall_data.len() > band_limit {
                let wildcards = scryfall_data.split_off(band_limit);
                for wildcard in wildcards {
                    let position = WILDCARD_POSITION.min(scryfall_data.len());
                    scryfall_data.insert(position, wildcard);
                }
            }
        }
        Ok(scryfall_data)
    }

    async fn get_card(&self, request: &GetScryfallData) -> Result<Card, GetCardError> {
        let scryfall_data = self.get_scryfall_data(request).await?;
        let card_profile = self.get_card_profile_with_scryfall_data_id(request).await?;
        let card = Card::new(card_profile, scryfall_data);
        Ok(card)
    }

    async fn get_cards(&self, request: &ScryfallDataIds) -> Result<Vec<Card>, GetCardError> {
        let scryfall_data = self.get_multiple_scryfall_data(request).await?;
        let scryfall_data_ids: ScryfallDataIds = scryfall_data.as_slice().into();
        let card_profiles = self
            .get_card_profiles_with_scryfall_data_ids(&scryfall_data_ids)
            .await?;
        let cards = card_profiles.sleeve(scryfall_data);
        Ok(cards)
    }

    /// Returns all printings of a card by oracle_id, ordered by release date (newest first).
    async fn get_printings(&self, oracle_id: uuid::Uuid) -> Result<Vec<Card>, GetCardError> {
        let db_rows: Vec<DatabaseScryfallData> =
            query_as("SELECT sd.* FROM scryfall_data sd JOIN card_profiles cp ON sd.id = cp.scryfall_data_id WHERE sd.oracle_id = $1 ORDER BY sd.released_at ASC")
                .bind(oracle_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| GetScryfallDataError::Database(e.into()))?;
        let scryfall_data: Vec<ScryfallData> = db_rows
            .into_iter()
            .map(ScryfallData::try_from)
            .collect::<Result<_, _>>()
            .map_err(GetScryfallDataError::Database)?;
        if scryfall_data.is_empty() {
            return Ok(vec![]);
        }
        let scryfall_data_ids: ScryfallDataIds = scryfall_data.as_slice().into();
        let card_profiles = self
            .get_card_profiles_with_scryfall_data_ids(&scryfall_data_ids)
            .await?;
        let cards = card_profiles.sleeve(scryfall_data);
        Ok(cards)
    }

    /// Composes `search_scryfall_data` results with card profiles into `Card` values.
    async fn search_cards(&self, request: &CardQuery) -> Result<Vec<Card>, SearchCardsError> {
        let scryfall_data = self.search_scryfall_data(request).await?;
        if scryfall_data.is_empty() {
            return Ok(vec![]);
        }
        let scryfall_data_ids: ScryfallDataIds = scryfall_data.as_slice().into();
        let card_profiles = self
            .get_card_profiles_with_scryfall_data_ids(&scryfall_data_ids)
            .await?;
        let cards = card_profiles.sleeve(scryfall_data);
        Ok(cards)
    }

    /// Extracts distinct card types by tokenizing `type_line` with `STRING_TO_ARRAY`.
    async fn get_card_types(&self) -> Result<Vec<String>, GetCardTypesError> {
        // Stop words: see domain::card::models::search_card::stop_words::TYPE_STOP_WORDS
        let card_types: Vec<String> = query_scalar!(
            "SELECT DISTINCT subtype FROM (
                SELECT TRIM(BOTH ':-?, ' FROM UNNEST(STRING_TO_ARRAY(type_line, ' '))) subtype
                FROM latest_cards
            ) subtypes
            WHERE subtype NOT IN ('//', '-', 'the', 'of', 'and/or', 'you', 'you''ll')
            ORDER BY subtype ASC"
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .flatten()
        .collect();
        Ok(card_types)
    }

    /// Returns distinct keyword abilities from the `keywords` array column.
    async fn get_keywords(&self) -> Result<Vec<String>, GetKeywordsError> {
        let keywords: Vec<String> = query_scalar!(
            "SELECT DISTINCT LOWER(TRIM(keyword)) AS keyword
             FROM latest_cards, UNNEST(keywords) AS keyword
             WHERE keywords IS NOT NULL
             ORDER BY keyword ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GetKeywordsError::Database(e.into()))?
        .into_iter()
        .flatten()
        .collect();
        Ok(keywords)
    }

    /// Returns distinct normalized words extracted from oracle text, noise-filtered.
    async fn get_oracle_words(&self) -> Result<Vec<String>, GetOracleWordsError> {
        // Stop words: see domain::card::models::search_card::stop_words::ORACLE_STOP_WORDS
        let words: Vec<String> = query_scalar!(
            "SELECT DISTINCT LOWER(REGEXP_REPLACE(word, '[^a-zA-Z]', '', 'g')) AS word
             FROM latest_cards, REGEXP_SPLIT_TO_TABLE(oracle_text, '\\s+') AS word
             WHERE oracle_text IS NOT NULL
               AND LOWER(REGEXP_REPLACE(word, '[^a-zA-Z]', '', 'g')) NOT IN (
                 'a', 'an', 'the', 'of', 'to', 'in', 'on', 'at', 'by', 'for', 'with',
                 'from', 'into', 'as', 'and', 'or', 'but', 'that', 'which', 'who',
                 'it', 'its', 'you', 'your', 'this', 'those', 'these', 'they', 'their',
                 'is', 'are', 'was', 'be', 'has', 'have', 'do', 'does', 'been', ''
               )
             ORDER BY word ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GetOracleWordsError::Database(e.into()))?
        .into_iter()
        .flatten()
        .collect();
        Ok(words)
    }

    async fn get_artists(&self) -> Result<Vec<String>, GetArtistsError> {
        let artists: Vec<String> = query_scalar!(
            "SELECT DISTINCT artist FROM latest_cards
             WHERE artist IS NOT NULL
             ORDER BY artist"
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .flatten()
        .collect();
        Ok(artists)
    }

    async fn get_sets(&self) -> Result<Vec<String>, GetSetsError> {
        let sets: Vec<String> =
            query_scalar!("SELECT DISTINCT set_name FROM latest_cards ORDER BY set_name")
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .flatten()
                .collect();
        Ok(sets)
    }

    async fn get_languages(&self) -> Result<Vec<String>, GetLanguagesError> {
        let languages: Vec<String> = query_scalar!(
            "SELECT DISTINCT lang FROM latest_cards
             WHERE lang IS NOT NULL
             ORDER BY lang"
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .flatten()
        .collect();
        Ok(languages)
    }

    async fn get_card_profile_with_id(
        &self,
        request: &GetCardProfile,
    ) -> Result<CardProfile, GetCardProfileError> {
        let card_profile: CardProfile = query_as!(
            DatabaseCardProfile,
            "SELECT scryfall_data_id, is_token, mechanical_categories, created_at, updated_at FROM card_profiles WHERE scryfall_data_id = $1",
            **request
        )
        .fetch_one(&self.pool)
        .await?
        .into();
        Ok(card_profile)
    }

    async fn get_card_profile_with_scryfall_data_id(
        &self,
        request: &GetScryfallData,
    ) -> Result<CardProfile, GetCardProfileError> {
        let card_profile: CardProfile = query_as!(
            DatabaseCardProfile,
            "SELECT scryfall_data_id, is_token, mechanical_categories, created_at, updated_at
            FROM card_profiles WHERE scryfall_data_id = $1",
            **request
        )
        .fetch_one(&self.pool)
        .await?
        .into();
        Ok(card_profile)
    }

    async fn get_card_profiles_with_ids(
        &self,
        request: &CardProfileIds,
    ) -> Result<Vec<CardProfile>, GetCardProfileError> {
        let card_profiles: Vec<CardProfile> = query_as!(
            DatabaseCardProfile,
            "SELECT scryfall_data_id, is_token, mechanical_categories, created_at, updated_at
            FROM card_profiles WHERE scryfall_data_id = ANY($1)",
            &**request
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|dcp| dcp.into())
        .collect();
        Ok(card_profiles)
    }

    async fn get_card_profiles_with_scryfall_data_ids(
        &self,
        request: &ScryfallDataIds,
    ) -> Result<Vec<CardProfile>, GetCardProfileError> {
        let card_profiles: Vec<CardProfile> = query_as!(
            DatabaseCardProfile,
            "SELECT scryfall_data_id, is_token, mechanical_categories, created_at, updated_at
            FROM card_profiles WHERE scryfall_data_id = ANY($1)",
            &**request
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|dcp| dcp.into())
        .collect();
        Ok(card_profiles)
    }

    async fn get_last_sync_date(&self) -> anyhow::Result<Option<DateTime<Utc>>> {
        let last_sync_date: Option<DateTime<Utc>> = query_scalar(
            "SELECT started_at FROM zervice_metrics
            ORDER BY started_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to get last sync date")?;
        Ok(last_sync_date)
    }

    async fn find_cards_by_exact_names(
        &self,
        names: &[String],
    ) -> Result<Vec<Card>, SearchCardsError> {
        if names.is_empty() {
            return Ok(vec![]);
        }
        let lowered: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
        let db_rows: Vec<DatabaseScryfallData> =
            query_as("SELECT * FROM latest_cards WHERE LOWER(name) = ANY($1)")
                .bind(&lowered)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SearchScryfallDataError::Database(e.into()))?;
        let scryfall_data: Vec<ScryfallData> = db_rows
            .into_iter()
            .map(ScryfallData::try_from)
            .collect::<Result<_, _>>()
            .map_err(SearchScryfallDataError::Database)?;
        if scryfall_data.is_empty() {
            return Ok(vec![]);
        }
        let scryfall_data_ids: ScryfallDataIds = scryfall_data.as_slice().into();
        let card_profiles = self
            .get_card_profiles_with_scryfall_data_ids(&scryfall_data_ids)
            .await?;
        let cards = card_profiles.sleeve(scryfall_data);
        Ok(cards)
    }

    async fn search_cards_deck_aware(
        &self,
        request: &CardQuery,
        deck_id: Option<uuid::Uuid>,
        exclude_oracle_ids: &[uuid::Uuid],
        synergy_scores: Option<&serde_json::Value>,
        synergy_only: bool,
    ) -> Result<Vec<Card>, SearchCardsError> {
        let scryfall_data = self
            .search_scryfall_data_deck_aware(
                request,
                deck_id,
                exclude_oracle_ids,
                synergy_scores,
                synergy_only,
                None,
            )
            .await?;
        if scryfall_data.is_empty() {
            return Ok(vec![]);
        }
        let scryfall_data_ids: ScryfallDataIds = scryfall_data.as_slice().into();
        let card_profiles = self
            .get_card_profiles_with_scryfall_data_ids(&scryfall_data_ids)
            .await?;
        let cards = card_profiles.sleeve(scryfall_data);
        Ok(cards)
    }

    /// First-class commander search (context/archive/commander_select_ordering.md):
    /// decks-helmed popularity ordering, banded + wildcarded per user per day
    /// (no deck required), token/emblem printings excluded. An explicit sort in
    /// `request` still wins; the pool is always token-free. The shuffle seed is
    /// `{user_id}:{date}`, mirroring the synergy serve's `{deck_id}:{date}`.
    async fn search_commanders(
        &self,
        request: &CardQuery,
        user_id: uuid::Uuid,
    ) -> Result<Vec<Card>, SearchCardsError> {
        let seed = format!("{user_id}:{}", Utc::now().date_naive());
        let scryfall_data = self
            .search_scryfall_data_deck_aware(request, None, &[], None, false, Some(seed))
            .await?;
        if scryfall_data.is_empty() {
            return Ok(vec![]);
        }
        let scryfall_data_ids: ScryfallDataIds = scryfall_data.as_slice().into();
        let card_profiles = self
            .get_card_profiles_with_scryfall_data_ids(&scryfall_data_ids)
            .await?;
        let cards = card_profiles.sleeve(scryfall_data);
        Ok(cards)
    }

    async fn commander_synergy_payload(
        &self,
        commander_printing_id: uuid::Uuid,
    ) -> Result<Option<serde_json::Value>, SearchCardsError> {
        let payload: Option<serde_json::Value> = sqlx::query_scalar(
            "SELECT cs.payload FROM scryfall_data s
             JOIN commander_synergy cs ON cs.oracle_id = s.oracle_id
             WHERE s.id = $1",
        )
        .bind(commander_printing_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SearchScryfallDataError::Database(e.into()))?;
        Ok(payload)
    }

    // ============
    //  classify
    // ============

    async fn get_unclassified_card_ids(&self) -> Result<Vec<uuid::Uuid>, anyhow::Error> {
        let ids = sqlx::query_scalar!(
            "SELECT cp.scryfall_data_id FROM card_profiles cp
             JOIN scryfall_data sd ON cp.scryfall_data_id = sd.id
             WHERE (cp.mechanical_categories = '[]'::jsonb OR cp.mechanical_categories IS NULL)
               AND sd.oracle_text IS NOT NULL
               AND sd.oracle_id IS NOT NULL"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(ids)
    }

    async fn get_cards_batch(&self, ids: &[uuid::Uuid]) -> Result<Vec<Card>, anyhow::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let scryfall_data_ids: ScryfallDataIds = ids.iter().copied().collect();
        self.get_cards(&scryfall_data_ids)
            .await
            .map_err(|e| anyhow::anyhow!("failed to fetch card batch: {e}"))
    }

    async fn update_mechanical_categories(
        &self,
        updates: &[(
            uuid::Uuid,
            Vec<zwipe_core::domain::card::mechanical_category::MechanicalCategory>,
        )],
    ) -> Result<(), anyhow::Error> {
        if updates.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for (id, categories) in updates {
            let cat_strings: Vec<String> = categories.iter().map(|c| c.to_string()).collect();
            let json = serde_json::to_value(&cat_strings)?;
            sqlx::query!(
                "UPDATE card_profiles SET mechanical_categories = $1, updated_at = NOW() WHERE scryfall_data_id = $2",
                json,
                id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn clear_all_categories(&self) -> Result<(), anyhow::Error> {
        // Batch clear to avoid a single slow UPDATE on 100k+ rows
        loop {
            let result = sqlx::query!(
                "UPDATE card_profiles SET mechanical_categories = '[]'::jsonb
                 WHERE scryfall_data_id IN (
                     SELECT scryfall_data_id FROM card_profiles
                     WHERE mechanical_categories != '[]'::jsonb
                     LIMIT 5000
                 )"
            )
            .execute(&self.pool)
            .await?;

            if result.rows_affected() == 0 {
                break;
            }
        }
        Ok(())
    }
}
