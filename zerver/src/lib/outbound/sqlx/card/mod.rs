//! Card data and Scryfall sync repository implementation.

pub mod card_profile;
pub mod error;
pub mod helpers;
pub mod models;
pub mod oracle_tag;
pub mod zervice_metrics;

use std::collections::HashMap;

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::query_builder::Separated;
use sqlx::types::Json;
use sqlx::{Postgres, QueryBuilder, Row, query_as, query_scalar};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_profile::CardProfile;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::criteria::PLAYABLE_LAYOUTS;
use zwipe_core::domain::card::search_card::card_filter::sort_direction::SortDirection;
use zwipe_core::domain::deck::Format;
use zwipe_core::domain::deck::requests::import_deck_cards;
use zwipe_core::domain::deck::tag_profile::{PROFILE_EXCLUDED_TAG_PREFIX, PROFILE_TAG_LIMIT};

use crate::domain::card::models::helpers::SleeveCardProfile;
use crate::domain::card::models::search_card::error::SearchCardsError;
use crate::domain::card::models::zervice_metrics::ZerviceMetrics;
use crate::domain::card::ports::{CardRepository, DeckServeContext};
use crate::domain::card::requests::create_card::CreateCardError;
use crate::domain::card::requests::get_artists::GetArtistsError;
use crate::domain::card::requests::get_card::GetCardError;
use crate::domain::card::requests::get_card_profile::{
    CardProfileIds, GetCardProfile, GetCardProfileError,
};
use crate::domain::card::requests::get_card_types::GetCardTypesError;
use crate::domain::card::requests::get_keywords::GetKeywordsError;
use crate::domain::card::requests::get_languages::GetLanguagesError;
use crate::domain::card::requests::get_oracle_tags::GetOracleTagsError;
use crate::domain::card::requests::get_oracle_words::GetOracleWordsError;
use crate::domain::card::requests::get_scryfall_data::{
    GetScryfallData, GetScryfallDataError, ScryfallDataIds, SearchScryfallDataError,
};
use crate::domain::card::requests::get_sets::GetSetsError;
use crate::domain::deck::models::deck::commander_suggestions::{
    CommanderSuggestionsError, DeckTagProfile, ScoredCommander,
};
use crate::outbound::sqlx::card::card_profile::DatabaseCardProfile;
use crate::outbound::sqlx::card::helpers::commander_scoring::SCORE_COMMANDER_SUGGESTIONS_SQL;
use crate::outbound::sqlx::card::helpers::oracle_tag_groups::{self, RoleMembershipBinds};
use crate::outbound::sqlx::card::helpers::upsert_card::{
    BatchDeltaUpsertWithTx, BatchUpsertWithTx, BulkUpsertWithTx, SingleUpsertWithTx,
};
use crate::outbound::sqlx::card::models::DatabaseScryfallData;
use crate::outbound::sqlx::card::oracle_tag::DatabaseOracleTag;
use crate::outbound::sqlx::card::zervice_metrics::DatabaseZerviceMetrics;
use crate::outbound::sqlx::postgres::Postgres as MyPostgres;

/// Hard ceiling on rows returned by a single card search.
///
/// `CardQuery::limit` arrives from untrusted request JSON; without a cap a
/// client could ask for an arbitrarily large page and force the DB to
/// materialize and serialize it. The frontend default is 25; 250 is generous
/// headroom. Note `CardQuery` is also used for *client-side* in-memory
/// filtering with much larger limits, but that path never reaches this query.
const MAX_SEARCH_LIMIT: u32 = 250;

// Default-synergy-ordering dials (context/plans/suggestion_signal.md).
// `W_SIGNAL = 0.0` and `BAND_SIZE = 1` together give plain score ordering.

/// Weight of the pooled add-rate term (centered on the global rate, so a
/// card with no data contributes exactly zero). Units are synergy-score
/// points (scores span roughly -0.6..1.0); enough to
/// migrate a proven card across a band boundary over time.
const W_SIGNAL: f64 = 0.15;

/// Band size for the default-ordering shuffle: cards ranked by
/// (base + signal) score are cut into hands of this many, bands served in
/// strict order, position within a band purely the (card, deck, day) hash.
/// 25 matches the client page size, so each page is exactly one shuffled
/// hand; cast-level variety per deck per day without a band-2 card ever
/// leading band 1. 1 = pure score ordering (no shuffle).
const BAND_SIZE: i64 = 25;

/// Shrinkage pseudo-count: impressions a card needs before its own add-rate
/// outweighs the global prior.
const SHRINK_K: f64 = 10.0;

/// Weight of the oracle-tag correlation term: a fixed lift for cards
/// whose `card_profiles.oracle_tags` overlap the deck's selected otags. Units
/// are synergy-score points (scores span roughly -0.6..1.0), sized to float a
/// strategy-relevant card up a band or so without swamping commander synergy.
/// The term only applies when the deck has selected otags; `0.0` disables it
/// exactly (the revert lever), and an empty otag set is a no-op regardless.
const W_ORACLE_TAG: f64 = 0.20;

/// Weight of the MVP steering term (deck-MVPs phase 3): a flat lift for cards
/// whose `card_profiles.card_roles` overlap the roles of the deck's mainboard
/// MVPs. Units are synergy-score points (scores span roughly -0.6..1.0). Sized
/// below `W_ORACLE_TAG` because a deck's three MVPs can share one role and
/// would otherwise lift that whole role above commander synergy. Flat, not a
/// per-role tally: matching two MVP roles is not twice the endorsement, and a
/// tally spikes hard when all three MVPs are the same kind of card. The term
/// is dormant when the deck has no MVPs, and `0.0` disables it exactly (the
/// revert lever).
const W_STEER: f64 = 0.12;

/// Base score for cards absent from the commander's synergy map. Sits below
/// the scoreless-list floor (-10, see `SynergyPayload::into_scores`) so the
/// unscored tail stays below every scored card; at zero dials this exactly
/// matches plain `NULLS LAST` ordering. Signal and jitter still shuffle
/// the tail internally; letting strong signal lift tail cards into the scored
/// region is a deliberate future retune (raise this anchor), not v1.
const UNSCORED_ANCHOR: f64 = -10.5;

// Wildcard slot dials (context/archive/wildcard_slot/).

/// Wildcard slots per page: positions reserved for cards drawn from beyond
/// the reachable horizon (rank > `DEEP_POOL_FLOOR`), least-shown first, so the
/// deep pool accrues impressions at all. 0 reverts to pure band serving.
const WILDCARD_SLOTS: i64 = 1;

/// The reachable horizon: mirrors the client add-stack hard cap (zwiper
/// `MAX_CARDS_IN_STACK`, `action_history.rs`, which zerver can't import). Cards
/// ranked past this are structurally unreachable by band pagination within a
/// session, so no signal can ever accrue for them without the wildcard probe.
const DEEP_POOL_FLOOR: i64 = 500;

/// 0-based in-page index where a wildcard lands: deep enough not to lead the
/// hand, early enough to be reliably seen before the page is exhausted.
const WILDCARD_POSITION: usize = 17;

/// Days a left swipe keeps a card out of the deck's swipe pile. Skipping it
/// again restarts the clock (the insert refreshes `suppressed_at`).
const SKIP_TTL_DAYS: i32 = 7;

/// Days a deliberate removal keeps a card out of the pile: longer than a
/// skip, since taking a card out of the deck is the stronger "doesn't fit".
const REMOVAL_TTL_DAYS: i32 = 30;

// Related-card tiers: cards outside the synergy payload are ranked above
// UNSCORED_ANCHOR by how well their otags cover the deck's tag profile
// (tier 2), then by overall popularity (tier 3). W_NEIGHBOR + W_POPULAR stays
// under the 0.5 gap between UNSCORED_ANCHOR and the payload's scoreless floor
// (-10.0), so every payload card still outranks every related card.

/// Lift for a card covering the whole deck tag profile (coverage is 0..1).
const W_NEIGHBOR: f64 = 0.4;

/// Lift for the most popular card overall, fading with Scryfall's rank.
const W_POPULAR: f64 = 0.05;

/// The serve-time deck tag profile that ranks related cards keeps its
/// strongest this-many tags.
const SERVE_PROFILE_TAG_LIMIT: i64 = 40;

// == suggested commanders ==

/// Weight of the color term in a commander suggestion's score: the best
/// Jaccard match (0..1) of the candidate's color identity against the color
/// identities of the user's decks, times this. Tag overlap is a sum
/// of products of two profiles that each sum to 1, so it lands well under 1
/// and this keeps colors a tiebreak rather than the ranking.
const W_COLOR_AFFINITY: f64 = 0.15;

/// Shared tags a suggestion carries as its reasons.
const SUGGESTION_TAG_LIMIT: i64 = 5;

/// Days a dismissal keeps a commander out of the suggestions. Dismissing it
/// again restarts the clock.
const DISMISSAL_TTL_DAYS: i32 = 7;

/// Base ranking for the commander-select ordering: decks-helmed popularity
/// first (`commander_popularity`, written by the synergy worker), then
/// `edhrec_rank` as the fallback for commanders the table doesn't cover, then
/// name, then id for a stable tiebreak. Shared by the wildcard CTE's
/// `row_number()` and the non-wildcard banded ORDER BY. Absent popularity rows
/// sort last, so an empty table degrades to pure `edhrec_rank`; the revert
/// lever. (`context/archive/commander_select_ordering.md` §2.)
const POPULARITY_RANK: &str = "pop.pop_decks DESC NULLS LAST, latest_cards.edhrec_rank ASC NULLS LAST, latest_cards.name ASC, latest_cards.id";

/// The popularity join, aliased so it exposes only `pop_decks`, never a bare
/// `name`/`oracle_id`/`decks`, which would collide with `latest_cards` and the
/// shared WHERE filters (name search, exclude-oracle_ids) and make those
/// columns ambiguous. Keyed on the real card's `oracle_id`.
const POPULARITY_JOIN: &str = "LEFT JOIN (SELECT oracle_id AS pop_oracle_id, decks AS pop_decks FROM commander_popularity) pop ON pop.pop_oracle_id = latest_cards.oracle_id";

/// First-party select-signal join for the commander-select wildcard's
/// least-shown deep slice, aliased for the same ambiguity reason as
/// [`POPULARITY_JOIN`]. An absent row COALESCEs to 0 impressions, so an empty
/// table leaves the deep slice on the daily shuffle alone; the dormant-until-
/// data-accrues behavior (`context/archive/commander_select_signal.md` §3).
const SELECT_SIGNAL_JOIN: &str = "LEFT JOIN (SELECT commander_oracle_id AS sel_oid, shown AS sel_shown FROM commander_select_signal) sel ON sel.sel_oid = latest_cards.oracle_id";

impl CardRepository for MyPostgres {
    // == create ==
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
        let query_sql = "INSERT INTO zervice_metrics \
             (started_at, ended_at, duration_in_seconds, status, received_count, upserted_count, skipped_count, error_count, errors) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *";
        let database_zervice_metrics: DatabaseZerviceMetrics = query_as(query_sql)
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
        // Overlay zwipe-core's wholly-UB set list into oou_sets before the
        // refresh, so the view's printing preference always matches the
        // deployed const. Delete-then-fill keeps removals honest; the pair
        // shares a transaction so the refresh never sees a half-applied list.
        let codes: Vec<String> =
            zwipe_core::domain::card::models::scryfall_data::universe::all_set_codes()
                .map(str::to_string)
                .collect();
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to begin oou_sets overlay transaction")?;
        sqlx::query("DELETE FROM oou_sets WHERE code <> ALL($1)")
            .bind(&codes)
            .execute(&mut *tx)
            .await
            .context("failed to prune oou_sets")?;
        sqlx::query(
            "INSERT INTO oou_sets (code) SELECT unnest($1::text[]) ON CONFLICT (code) DO NOTHING",
        )
        .bind(&codes)
        .execute(&mut *tx)
        .await
        .context("failed to fill oou_sets")?;
        tx.commit()
            .await
            .context("failed to commit oou_sets overlay")?;
        tracing::info!("oou sets overlay: {} wholly-UB set codes", codes.len());

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

    async fn refresh_otag_context_signal_rollup(&self) -> anyhow::Result<()> {
        sqlx::query("REFRESH MATERIALIZED VIEW otag_context_signal_rollup")
            .execute(&self.pool)
            .await
            .context("failed to refresh otag_context_signal_rollup materialized view")?;
        Ok(())
    }

    async fn sync_oracle_tags(
        &self,
        tags: &[crate::inbound::external::scryfall::oracle_tag::OracleTag],
    ) -> anyhow::Result<(u32, u32)> {
        crate::outbound::sqlx::card::helpers::oracle_tags::sync_oracle_tags(&self.pool, tags).await
    }

    async fn refresh_card_oracle_tags(&self) -> anyhow::Result<()> {
        // Aggregate oracle tags per oracle_id once, then fan out to every printing's
        // card_profile. LEFT JOIN so cards that lost all tags reset to '[]'.
        sqlx::query(
            "WITH oracle_agg AS (
                 SELECT oracle_id, jsonb_agg(oracle_tag ORDER BY oracle_tag) AS tags
                 FROM card_oracle_tags
                 GROUP BY oracle_id
             )
             UPDATE card_profiles cp
             SET oracle_tags = COALESCE(oa.tags, '[]'::jsonb)
             FROM scryfall_data sd
             LEFT JOIN oracle_agg oa ON oa.oracle_id = sd.oracle_id
             WHERE cp.scryfall_data_id = sd.id",
        )
        .execute(&self.pool)
        .await
        .context("failed to refresh card_profiles.oracle_tags projection")?;
        Ok(())
    }

    // == get ==
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
    /// `oracle_id`). Joins `card_profiles` for `is_token` / `card_roles` filters.
    /// Filter clauses are composed with `AND` via `QueryBuilder::separated`.
    async fn search_scryfall_data(
        &self,
        request: &CardQuery,
    ) -> Result<Vec<ScryfallData>, SearchScryfallDataError> {
        self.search_scryfall_data_deck_aware(request, DeckServeContext::default())
            .await
    }

    /// `search_scryfall_data` plus the deck-aware extras: `oracle_id` exclusion,
    /// suppression filtering for `deck_id` (skipped/removed cards), synergy-score
    /// default ordering, and (when `synergy_only`) a membership constraint to
    /// the commander's synergy pool. The plain search is this with no extras.
    ///
    /// `commander_seed` switches the engine into commander-select mode
    /// (`context/archive/commander_select_ordering.md`): decks-helmed popularity
    /// ordering, banded + wildcarded by that caller-supplied seed (typically
    /// `{user_id}:{date}`, no deck required), and token/emblem printings
    /// excluded from the candidate pool. Exposed as `search_commanders`; every
    /// other caller passes a default context and is unaffected.
    #[expect(
        clippy::too_many_lines,
        reason = "one QueryBuilder composed in order: the filters, the deck extras, then the ordering"
    )]
    async fn search_scryfall_data_deck_aware(
        &self,
        request: &CardQuery,
        context: DeckServeContext<'_>,
    ) -> Result<Vec<ScryfallData>, SearchScryfallDataError> {
        // Strip punctuation from DB columns for punctuation-insensitive text search.
        // The query values are already stripped by CardQueryBuilder setters.
        const STRIP_NAME: &str = "regexp_replace(name, '[^a-zA-Z0-9 ]', '', 'g') ILIKE ";
        const STRIP_TYPE: &str = "regexp_replace(type_line, '[^a-zA-Z0-9 ]', '', 'g') ILIKE ";
        const STRIP_ORACLE: &str = "regexp_replace(oracle_text, '[^a-zA-Z0-9 ]', '', 'g') ILIKE ";

        // Deck-serving inputs arrive bundled; unpack them into the locals the
        // QueryBuilder threads them into (never stored beyond this call).
        let DeckServeContext {
            deck_id,
            exclude_oracle_ids,
            synergy_scores,
            synergy_only,
            neighbor_weights,
            commander_seed,
            deck_oracle_tags,
            mvp_card_roles,
            exclude_universes_beyond,
            universes_beyond_exception_set_names,
        } = context;
        // WHERE clauses read the predicate fields; LIMIT/OFFSET/ORDER BY read
        // the query config: the CardCriteria/CardQuery split, mirrored here.
        let criteria = request.criteria();
        // The deck whose suppressions apply: set for the swipe pile, unset for
        // quick add (`include_skipped`) and every deckless search.
        let returned_deck = deck_id.filter(|_| !request.include_skipped());
        // Sort key for returning cards: the WHERE clause drops every live
        // suppression, so a row left for a served card is an expired one.
        // False sorts first, so cards nobody has passed on lead the pile.
        let push_returned = |qb: &mut QueryBuilder<Postgres>, deck_id: uuid::Uuid| {
            qb.push("EXISTS (SELECT 1 FROM deck_card_suppressions ret WHERE ret.deck_id = ");
            qb.push_bind(deck_id);
            qb.push(" AND ret.oracle_id = latest_cards.oracle_id)");
        };
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
        //         global rate; a card with no impressions adds zero.
        let push_score = |qb: &mut QueryBuilder<Postgres>, scores: &serde_json::Value| {
            qb.push("COALESCE((");
            qb.push_bind(scores.clone());
            // Cards outside the score map: the anchor plus the related-card
            // tiers. COALESCE skips this branch for scored cards.
            qb.push(format!(" ->> LOWER(name))::float8, {UNSCORED_ANCHOR}"));
            if let Some(weights) = neighbor_weights {
                qb.push(format!(
                    " + {W_NEIGHBOR} * LEAST(1.0, (SELECT COALESCE(SUM(("
                ));
                qb.push_bind(weights.clone());
                qb.push(
                    " ->> tag)::float8), 0) \
                     FROM jsonb_array_elements_text(card_profiles.oracle_tags) tag))",
                );
            }
            qb.push(format!(
                " + {W_POPULAR} / (1.0 + COALESCE(latest_cards.edhrec_rank, 1000000) / 1000.0))"
            ));
            qb.push(format!(
                " + {W_SIGNAL} * ((COALESCE(sig.net, 0) + {SHRINK_K} * g.rate) \
                   / (COALESCE(sig.shown, 0) + {SHRINK_K}) - g.rate)"
            ));
            // Oracle-tag term: a flat lift for cards whose otags overlap
            // the deck's selected otags. Dormant when the deck selected none, so
            // decks without otags keep byte-identical ordering. Mirrors the
            // filter's `?|` jsonb-array overlap probe.
            if !deck_oracle_tags.is_empty() {
                qb.push(format!(
                    " + {W_ORACLE_TAG} * (card_profiles.oracle_tags ?| "
                ));
                qb.push_bind(deck_oracle_tags.to_vec());
                qb.push(")::int");
            }
            // MVP steering (deck-MVPs phase 3): a flat lift for cards sharing a
            // role with the deck's mainboard MVPs. The MVPs themselves are
            // already in the deck and thus excluded from the pool, so this
            // steers by kind rather than resurfacing the starred cards. Dormant
            // when the deck has no MVPs, so those decks keep byte-identical
            // ordering.
            if !mvp_card_roles.is_empty() {
                qb.push(format!(" + {W_STEER} * (card_profiles.card_roles ?| "));
                qb.push_bind(mvp_card_roles.to_vec());
                qb.push(")::int");
            }
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
        let mut qb: QueryBuilder<Postgres> = if wildcard_serving
            && signal_ordering
            && let (Some(scores), Some(seed)) = (synergy_scores, band_seed.clone())
        {
            // Synergy wildcard CTE: rank + shuffle + exposure computed once; the
            // two slices at the end page it two ways. The predicate pushes below
            // land inside the CTE, so filters, legality, and suppressions bound
            // both slices.
            let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
                "WITH pool AS (SELECT latest_cards.*, row_number() OVER (ORDER BY ",
            );
            push_score(&mut qb, scores);
            qb.push(
                " DESC, name ASC, latest_cards.id) AS rn, \
                     hashtext(COALESCE(latest_cards.oracle_id::text, '') || ",
            );
            qb.push_bind(seed);
            qb.push(") AS shuffle, ");
            match returned_deck {
                Some(d) => push_returned(&mut qb, d),
                None => {
                    qb.push("FALSE");
                }
            }
            qb.push(
                " AS pool_returned, COALESCE(sig.shown, 0) AS pool_shown
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
            // to the daily shuffle alone, byte-identical to the pre-signal
            // behavior (context/archive/commander_select_signal.md §3). Both
            // joins are 1:1 (PK on oracle_id) and aliased so no bare
            // `name`/`oracle_id` collides with the shared WHERE filters.
            let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
                "WITH pool AS (SELECT latest_cards.*, row_number() OVER (ORDER BY ",
            );
            qb.push(POPULARITY_RANK);
            qb.push(
                ") AS rn, \
                     hashtext(COALESCE(latest_cards.oracle_id::text, '') || ",
            );
            qb.push_bind(seed);
            qb.push(format!(
                ") AS shuffle, FALSE AS pool_returned, COALESCE(sel.sel_shown, 0) AS pool_shown
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
        let mut sep: Separated<'_, Postgres, &'static str> = qb.separated(" AND ");
        // Seed an always-true clause so the baked `WHERE` is valid even when no
        // filter conditions are pushed (e.g. an empty filter). It also lets every
        // real condition below rely on the `AND` separator: the first real push
        // becomes the second element, so it is correctly prefixed with ` AND `.
        sep.push("TRUE");

        // Commander-select candidate pool must exclude token/emblem printings.
        // The popularity table (written by the synergy worker) keys on the real
        // card's oracle_id and excludes these layouts, so serving must agree;
        // otherwise a same-named token could be offered as a commander and would
        // never join a real deck. Applies to every commander search, even one
        // with an explicit sort. (context/archive/commander_select_ordering.md §1.)
        if commander_select {
            sep.push("latest_cards.layout NOT IN ('token', 'double_faced_token', 'emblem')");
        }

        if let Some(query_string) = &criteria.name_contains() {
            sep.push(STRIP_NAME);
            sep.push_bind_unseparated(format!("%{query_string}%"));
        }

        if let Some(query_string) = &criteria.name_not_contains() {
            sep.push("NOT (");
            sep.push_unseparated(STRIP_NAME);
            sep.push_bind_unseparated(format!("%{query_string}%"));
            sep.push_unseparated(")");
        }

        if let Some(query_string) = &criteria.type_line_contains() {
            sep.push(STRIP_TYPE);
            sep.push_bind_unseparated(format!("%{query_string}%"));
        }

        if let Some(query_string_array) = &criteria.type_line_contains_any() {
            sep.push(" (");
            for (i, query_string) in query_string_array.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(" OR ");
                }
                sep.push_unseparated(STRIP_TYPE);
                sep.push_bind_unseparated(format!("%{query_string}%"));
            }
            sep.push_unseparated(") ");
        }

        if let Some(card_types) = &criteria.card_type_contains_any() {
            sep.push(" (");
            for (i, query_string) in card_types.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(" OR ");
                }
                sep.push_unseparated(STRIP_TYPE);
                sep.push_bind_unseparated(format!("%{query_string}%"));
            }
            sep.push_unseparated(") ");
        }

        if let Some(query_string_array) = &criteria.type_line_contains_all() {
            for query_string in *query_string_array {
                sep.push(STRIP_TYPE);
                sep.push_bind_unseparated(format!("%{query_string}%"));
            }
        }

        if let Some(card_types) = &criteria.card_type_contains_all() {
            for card_type in *card_types {
                sep.push(STRIP_TYPE);
                sep.push_bind_unseparated(format!("%{card_type}%"));
            }
        }

        if let Some(query_string) = &criteria.type_line_not_contains() {
            sep.push("(type_line IS NULL OR NOT (");
            sep.push_unseparated(STRIP_TYPE);
            sep.push_bind_unseparated(format!("%{query_string}%"));
            sep.push_unseparated("))");
        }

        if let Some(query_string_array) = &criteria.type_line_excludes_any() {
            sep.push("(type_line IS NULL OR NOT (");
            for (i, query_string) in query_string_array.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(" OR ");
                }
                sep.push_unseparated(STRIP_TYPE);
                sep.push_bind_unseparated(format!("%{query_string}%"));
            }
            sep.push_unseparated(")) ");
        }

        if let Some(card_types) = &criteria.card_type_excludes_any() {
            sep.push("(type_line IS NULL OR NOT (");
            for (i, query_string) in card_types.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(" OR ");
                }
                sep.push_unseparated(STRIP_TYPE);
                sep.push_bind_unseparated(format!("%{query_string}%"));
            }
            sep.push_unseparated(")) ");
        }

        if let Some(sets) = criteria.set_equals_any() {
            // Printing-aware: matches a card with a printing in any of these
            // sets, not just the view's picked printing (printing_set_names
            // aggregates every printing). && is array overlap.
            sep.push("printing_set_names && ");
            sep.push_bind_unseparated(sets);
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
            // Printing-aware: a card survives unless EVERY set it was printed
            // in is excluded (<@ is "contained by"). One excluded set can no
            // longer hide a staple whose picked printing happened to be there,
            // the Sol Ring / Secret Lair Drop bug this replaced.
            sep.push("NOT (printing_set_names <@ ");
            sep.push_bind_unseparated(sets);
            sep.push_unseparated(")");
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
        // empty/missing prices into NULL (excluded, no cast error), matching the
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

        if let Some(query_string) = &criteria.oracle_text_contains() {
            sep.push(STRIP_ORACLE);
            sep.push_bind_unseparated(format!("%{query_string}%"));
        }

        if let Some(query_string_array) = &criteria.oracle_text_contains_any() {
            sep.push(" (");
            for (i, query_string) in query_string_array.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(" OR ");
                }
                sep.push_unseparated(STRIP_ORACLE);
                sep.push_bind_unseparated(format!("%{query_string}%"));
            }
            sep.push_unseparated(") ");
        }

        if let Some(query_string_array) = &criteria.oracle_text_contains_all() {
            for query_string in *query_string_array {
                sep.push(STRIP_ORACLE);
                sep.push_bind_unseparated(format!("%{query_string}%"));
            }
        }

        if let Some(query_string) = &criteria.oracle_text_not_contains() {
            sep.push("(oracle_text IS NULL OR NOT (");
            sep.push_unseparated(STRIP_ORACLE);
            sep.push_bind_unseparated(format!("%{query_string}%"));
            sep.push_unseparated("))");
        }

        if let Some(query_string_array) = &criteria.oracle_text_excludes_any() {
            sep.push("(oracle_text IS NULL OR NOT (");
            for (i, query_string) in query_string_array.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(" OR ");
                }
                sep.push_unseparated(STRIP_ORACLE);
                sep.push_bind_unseparated(format!("%{query_string}%"));
            }
            sep.push_unseparated(")) ");
        }

        // Keywords are stored capitalized (e.g. "Flying") but UI lowercases them.
        // Use array_lowercase() via subquery to compare case-insensitively.
        if let Some(keywords) = &criteria.keywords_contains_any() {
            sep.push("(SELECT array_agg(lower(k)) FROM unnest(keywords) k) && ARRAY[");
            for (i, kw) in keywords.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(kw.to_lowercase());
            }
            sep.push_unseparated("]::text[]");
        }

        if let Some(keywords) = &criteria.keywords_contains_all() {
            sep.push("(SELECT array_agg(lower(k)) FROM unnest(keywords) k) @> ARRAY[");
            for (i, kw) in keywords.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(kw.to_lowercase());
            }
            sep.push_unseparated("]::text[]");
        }

        if let Some(keywords) = &criteria.keywords_excludes() {
            sep.push("(keywords IS NULL OR NOT ((SELECT array_agg(lower(k)) FROM unnest(keywords) k) && ARRAY[");
            for (i, kw) in keywords.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(kw.to_lowercase());
            }
            sep.push_unseparated("]::text[]))");
        }

        if let Some(colors) = &criteria.produced_mana_contains_any() {
            sep.push("produced_mana && ARRAY[");
            for (i, c) in colors.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(c.to_uppercase());
            }
            sep.push_unseparated("]::text[]");
        }

        if let Some(colors) = &criteria.produced_mana_contains_all() {
            sep.push("produced_mana @> ARRAY[");
            for (i, c) in colors.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(c.to_uppercase());
            }
            sep.push_unseparated("]::text[]");
        }

        if let Some(colors) = &criteria.produced_mana_excludes() {
            sep.push("(produced_mana IS NULL OR NOT (produced_mana && ARRAY[");
            for (i, c) in colors.iter().enumerate() {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(c.to_uppercase());
            }
            sep.push_unseparated("]::text[]))");
        }

        if let Some(query_string) = &criteria.flavor_text_contains() {
            sep.push("regexp_replace(flavor_text, '[^a-zA-Z0-9 ]', '', 'g') ILIKE ");
            sep.push_bind_unseparated(format!("%{query_string}%"));
        }

        if let Some(query_string) = &criteria.flavor_text_not_contains() {
            sep.push("(flavor_text IS NULL OR NOT (regexp_replace(flavor_text, '[^a-zA-Z0-9 ]', '', 'g') ILIKE ");
            sep.push_bind_unseparated(format!("%{query_string}%"));
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
            // A meld result (Brisela, Titania Gaea Incarnate) is its own card
            // object with a legendary creature type line and no " // " to
            // split, so the front-face rule below cannot see it. It can never
            // be a commander: it exists only by melding two cards already on
            // the battlefield. Its two parts share `layout = 'meld'` and are
            // often legal commanders, so the layout alone cannot separate
            // them; each meld card names its own role in `all_parts`.
            //
            // Applied before the match because it holds for every format.
            // `all_parts` is nullable and `jsonb_array_elements` over NULL
            // yields no rows, so a null column leaves the card alone.
            sep.push(
                "NOT (latest_cards.layout = 'meld' AND EXISTS (\
                 SELECT 1 FROM jsonb_array_elements(latest_cards.all_parts) mp \
                 WHERE mp->>'component' = 'meld_result' \
                   AND mp->>'name' = latest_cards.name))",
            );

            // Mirrors `zwipe_core::domain::card::is_valid_commander`. Every type
            // test reads the FRONT face: Scryfall joins both faces into one
            // `type_line` with " // ", so matching the whole string makes a card
            // eligible for what its back face is (Westvale Abbey is a Land).
            //
            // Each `split_part` is paired with the same test on the raw column.
            // That conjunct is logically redundant, since the front face is a
            // substring of the whole line, but `type_line ILIKE` can use the
            // trigram index while `split_part(type_line, ...)` cannot. Keeping
            // both lets the index narrow first: measured at 12ms, against 57ms
            // with the raw test dropped.
            match format {
                // Legendary creature, legendary vehicle with P/T, or "can be your commander"
                Format::Commander | Format::Duel | Format::Predh => {
                    sep.push(
                        "((type_line ILIKE '%Legendary%' \
                           AND split_part(type_line, ' // ', 1) ILIKE '%Legendary%' \
                           AND ((type_line ILIKE '%Creature%' \
                                 AND split_part(type_line, ' // ', 1) ILIKE '%Creature%') \
                                OR (power IS NOT NULL AND toughness IS NOT NULL))) \
                         OR oracle_text ILIKE '%can be your commander%')",
                    );
                }
                // Legendary creature or legendary planeswalker
                Format::Brawl | Format::StandardBrawl | Format::HistoricBrawl => {
                    sep.push(
                        "(type_line ILIKE '%Legendary%' \
                         AND split_part(type_line, ' // ', 1) ILIKE '%Legendary%' \
                         AND ((type_line ILIKE '%Creature%' \
                               AND split_part(type_line, ' // ', 1) ILIKE '%Creature%') \
                              OR (type_line ILIKE '%Planeswalker%' \
                                  AND split_part(type_line, ' // ', 1) ILIKE '%Planeswalker%')))",
                    );
                }
                // Uncommon creature: legendary or not. Two fixes here:
                //   1. Rarity is stored as the short code ('U'), not the word
                //      'uncommon'.
                //   2. PDH eligibility is "has appeared at uncommon in ANY
                //      printing", not "this cached printing is uncommon", so we
                //      check all printings via scryfall_data (catches cards whose
                //      preferred printing is common but were printed uncommon).
                Format::PauperCommander => {
                    sep.push(
                        "(type_line ILIKE '%Creature%' \
                         AND split_part(type_line, ' // ', 1) ILIKE '%Creature%' AND EXISTS (\
                         SELECT 1 FROM scryfall_data sd2 \
                         WHERE sd2.oracle_id = latest_cards.oracle_id \
                         AND sd2.rarity = 'U'))",
                    );
                }
                // Any planeswalker
                Format::Oathbreaker => {
                    sep.push(
                        "(type_line ILIKE '%Planeswalker%' \
                         AND split_part(type_line, ' // ', 1) ILIKE '%Planeswalker%')",
                    );
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
            // Mirrors `zwipe_core::domain::card::is_valid_signature_spell_type`.
            // Oathbreaker wants an instant or sorcery CARD, so an Adventure or
            // `prepare` creature does not qualify however its back half reads.
            // Split cards are both halves at once (CR 709.4) and keep the whole
            // line; everything else is judged on its front face (CR 712.4a).
            //
            // The raw ILIKE pair stays in front for the trigram index, as in the
            // commander filter above.
            sep.push(
                "((type_line ILIKE '%Instant%' OR type_line ILIKE '%Sorcery%') \
                 AND (layout = 'split' \
                      OR split_part(type_line, ' // ', 1) ILIKE '%Instant%' \
                      OR split_part(type_line, ' // ', 1) ILIKE '%Sorcery%'))",
            );
        }

        // mechanical category filters
        if let Some(categories) = criteria.card_roles_contains_any() {
            sep.push("(card_profiles.card_roles ?| ");
            sep.push_bind_unseparated(categories.to_vec());
            sep.push_unseparated(")");
        }

        if let Some(categories) = criteria.card_roles_contains_all() {
            let json = serde_json::to_value(categories).unwrap_or_default();
            sep.push("(card_profiles.card_roles @> ");
            sep.push_bind_unseparated(json);
            sep.push_unseparated(")");
        }

        if let Some(categories) = criteria.card_roles_excludes() {
            sep.push("NOT (card_profiles.card_roles ?| ");
            sep.push_bind_unseparated(categories.to_vec());
            sep.push_unseparated(")");
        }

        if let Some(tags) = criteria.oracle_tags_contains_any() {
            sep.push("(card_profiles.oracle_tags ?| ");
            sep.push_bind_unseparated(tags.to_vec());
            sep.push_unseparated(")");
        }

        if let Some(tags) = criteria.oracle_tags_contains_all() {
            let json = serde_json::to_value(tags).unwrap_or_default();
            sep.push("(card_profiles.oracle_tags @> ");
            sep.push_bind_unseparated(json);
            sep.push_unseparated(")");
        }

        if let Some(tags) = criteria.oracle_tags_excludes() {
            sep.push("NOT (card_profiles.oracle_tags ?| ");
            sep.push_bind_unseparated(tags.to_vec());
            sep.push_unseparated(")");
        }

        // Deck-aware exclusion: omit cards already in the deck. Null-oracle
        // printings are kept; they can't match a deck's oracle_ids anyway,
        // and a bare NOT(= ANY) would NULL them out of the results.
        if !exclude_oracle_ids.is_empty() {
            sep.push("(oracle_id IS NULL OR NOT (oracle_id = ANY(");
            sep.push_bind_unseparated(exclude_oracle_ids.to_vec());
            sep.push_unseparated(")))");
        }

        // Suppression filtering: the deck's skipped cards stay out of the
        // deck-aware search for SKIP_TTL_DAYS and removed cards for
        // REMOVAL_TTL_DAYS (Clear skips brings them back sooner). Expired rows
        // stay in the table, and the ORDER BY below serves those cards after
        // everything unseen. Quick add opts out via `include_skipped`: a typed
        // card name is explicit intent, so suppressed cards stay findable
        // there while the swipe pile keeps respecting them. NOT EXISTS rather
        // than a bind array; the set can be thousands of rows. Null-oracle
        // printings pass, matching the exclusion clause above.
        if let Some(deck_id) = returned_deck {
            sep.push("NOT EXISTS (SELECT 1 FROM deck_card_suppressions dcs WHERE dcs.deck_id = ");
            sep.push_bind_unseparated(deck_id);
            sep.push_unseparated(format!(
                " AND dcs.oracle_id = latest_cards.oracle_id \
                 AND dcs.suppressed_at > now() - make_interval(days => CASE dcs.source \
                 WHEN 'removal' THEN {REMOVAL_TTL_DAYS} ELSE {SKIP_TTL_DAYS} END))"
            ));
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

        // Universes Beyond preference: serve a card only when its pick is
        // in-universe (the view's pick preference makes that a card-level
        // truth: an OOU pick means no in-universe printing exists) or it has
        // a printing in a whitelisted franchise. An empty whitelist overlaps
        // nothing and the OR term is inert.
        if exclude_universes_beyond {
            let ub_codes: Vec<String> =
                zwipe_core::domain::card::models::scryfall_data::universe::all_set_codes()
                    .map(str::to_string)
                    .collect();
            sep.push("(NOT (COALESCE(security_stamp, '') = 'triangle' OR set = ANY(");
            sep.push_bind_unseparated(ub_codes);
            sep.push_unseparated(")) OR printing_set_names && ");
            sep.push_bind_unseparated(universes_beyond_exception_set_names);
            sep.push_unseparated(")");
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
            if let Some(d) = returned_deck {
                push_returned(&mut qb, d);
                qb.push(", ");
            }
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
                qb.push(match request.direction() {
                    SortDirection::Ascending => " ASC",
                    SortDirection::Descending => " DESC",
                });
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
            //         width) for pools that hold still between pages. The
            //         deck swipe pile doesn't: swiped cards leave it, so the
            //         deck service always asks for page 0 and the next page
            //         is simply the top of what is left.
            //   deep slice: the probe, cards beyond the reachable horizon,
            //         least-shown first, then the daily shuffle. In a still
            //         pool, pages walk the deep list so a probe never repeats
            //         within a day; in the swipe pile a swiped probe leaves
            //         the pool, so page 0 serves the next one.
            //         A pool that never reaches the horizon (tight filters,
            //         synergy ON) yields an empty slice and pure band
            //         serving, automatically.
            // The outer ORDER BY re-sorts deterministically (UNION ALL order
            // is not guaranteed): band cards in band order, probes last;
            // the Rust splice below lifts them to WILDCARD_POSITION.
            let limit = i64::from(request.limit().min(MAX_SEARCH_LIMIT));
            let band_limit = (limit - WILDCARD_SLOTS).max(1);
            let page_index = i64::from(request.offset().min(i32::MAX as u32)) / limit.max(1);
            qb.push(format!(
                ") SELECT * FROM ((SELECT *, 0 AS slice FROM pool \
                 ORDER BY pool_returned, (rn - 1) / {BAND_SIZE}, shuffle, name ASC LIMIT "
            ));
            qb.push_bind(band_limit);
            qb.push(" OFFSET ");
            qb.push_bind(page_index * band_limit);
            qb.push(format!(
                ") UNION ALL (SELECT *, 1 AS slice FROM pool WHERE rn > {DEEP_POOL_FLOOR} \
                 ORDER BY pool_returned, pool_shown ASC, shuffle LIMIT {WILDCARD_SLOTS} OFFSET "
            ));
            qb.push_bind(page_index * WILDCARD_SLOTS);
            qb.push(format!(
                ")) AS combined ORDER BY slice ASC, pool_returned, (rn - 1) / {BAND_SIZE}, shuffle, name ASC"
            ));
        } else if let Some(scores) = synergy_scores {
            // Default synergy ordering: band shuffle over (base + signal)
            // (context/plans/suggestion_signal.md).
            //   score: the commander's synergy score (unscored cards anchor
            //          below the scored floor, see UNSCORED_ANCHOR) + the
            //          pooled net-rate, shrunk toward and centered on the
            //          global rate; a card with no impressions adds zero.
            //   bands: cards ranked by score are cut into BAND_SIZE hands.
            //          Bands stay in strict order; position *within* a band
            //          is purely the (card, deck, day) hash; a different
            //          opening hand per deck per day, while a band-2 card
            //          can never lead band 1.
            qb.push(" ORDER BY ");
            if let Some(d) = returned_deck {
                push_returned(&mut qb, d);
                qb.push(", ");
            }
            if let Some(deck_id) = deck_id {
                let seed = format!("{deck_id}:{}", Utc::now().date_naive());
                qb.push("(row_number() OVER (ORDER BY ");
                push_score(&mut qb, scores);
                qb.push(format!(
                    " DESC, name ASC, latest_cards.id) - 1) / {BAND_SIZE}, "
                ));
                // COALESCE the oracle_id: it is nullable, and NULL || seed
                // would NULL the shuffle key.
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
        } else if let Some(d) = returned_deck {
            // No sort and no synergy data (cold cache): the pool is otherwise
            // unordered, but returning cards still come after the rest.
            qb.push(" ORDER BY ");
            push_returned(&mut qb, d);
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
        // page, so any rows past the band width are wildcards: lift them to
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

    async fn featured_flavor_id(&self, hour_key: &str) -> anyhow::Result<Option<uuid::Uuid>> {
        // Pool = the flavor UI's standing filters (has flavor text, English,
        // physical, no content warning) + the 12-month recency rule, with an
        // all-time fallback so the feature never serves nothing. Ordering by
        // md5(id || hour_key) makes the pick deterministic per hour: the
        // serving cache is an optimization, never load-bearing state. The
        // WHERE clause is duplicated across the two literals because sqlx's
        // injection lint (rightly) refuses assembled SQL strings.
        let recent: Option<uuid::Uuid> = sqlx::query_scalar(
            "SELECT id FROM latest_cards \
             WHERE flavor_text IS NOT NULL AND flavor_text != '' \
               AND lang = 'en' AND digital = false AND oversized = false \
               AND (content_warning = false OR content_warning IS NULL) \
               AND released_at >= (CURRENT_DATE - INTERVAL '12 months') \
             ORDER BY md5(id::text || $1) LIMIT 1",
        )
        .bind(hour_key)
        .fetch_optional(&self.pool)
        .await?;
        if recent.is_some() {
            return Ok(recent);
        }

        Ok(sqlx::query_scalar(
            "SELECT id FROM latest_cards \
             WHERE flavor_text IS NOT NULL AND flavor_text != '' \
               AND lang = 'en' AND digital = false AND oversized = false \
               AND (content_warning = false OR content_warning IS NULL) \
             ORDER BY md5(id::text || $1) LIMIT 1",
        )
        .bind(hour_key)
        .fetch_optional(&self.pool)
        .await?)
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

    /// Returns all printings of a card by `oracle_id`, ordered by release date (newest first).
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

    async fn get_latest_cards_by_oracle_ids(
        &self,
        oracle_ids: &[uuid::Uuid],
    ) -> Result<Vec<Card>, GetCardError> {
        if oracle_ids.is_empty() {
            return Ok(vec![]);
        }
        // latest_cards holds one row per oracle, so ANY($1) yields at most
        // one printing per requested id.
        let db_rows: Vec<DatabaseScryfallData> =
            query_as("SELECT * FROM latest_cards WHERE oracle_id = ANY($1)")
                .bind(oracle_ids)
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

    /// Returns the full oracle tag catalog: every row of `oracle_tags` with its
    /// `parent_ids` resolved to parent slugs, ordered by slug.
    async fn get_oracle_tags(&self) -> Result<Vec<OracleTag>, GetOracleTagsError> {
        let oracle_tags: Vec<OracleTag> = query_as!(
            DatabaseOracleTag,
            r#"SELECT
                 o.slug AS "slug!",
                 o.label AS "label!",
                 o.description,
                 COALESCE(
                     ARRAY(
                         SELECT p.slug FROM oracle_tags p
                         WHERE p.id = ANY(o.parent_ids)
                         ORDER BY p.slug
                     ),
                     ARRAY[]::text[]
                 ) AS "parent_slugs!"
               FROM oracle_tags o
               ORDER BY o.slug ASC"#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GetOracleTagsError::Database(e.into()))?
        .into_iter()
        .map(OracleTag::from)
        .collect();
        Ok(oracle_tags)
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

    async fn get_universes_beyond_preferences(
        &self,
        user_id: uuid::Uuid,
    ) -> anyhow::Result<(bool, Vec<String>)> {
        let row: Option<(bool, Vec<String>)> = sqlx::query_as(
            "SELECT exclude_universes_beyond, universes_beyond_exceptions \
             FROM user_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .context("failed to read universes beyond preferences")?;
        let (exclude, slugs) = row.unwrap_or((false, Vec::new()));
        Ok((
            exclude,
            zwipe_core::domain::card::models::scryfall_data::universe::exception_set_names(&slugs),
        ))
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
            "SELECT scryfall_data_id, is_token, card_roles, oracle_tags, oracle_tags_by_role, other_oracle_tags, created_at, updated_at FROM card_profiles WHERE scryfall_data_id = $1",
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
            "SELECT scryfall_data_id, is_token, card_roles, oracle_tags, oracle_tags_by_role, other_oracle_tags, created_at, updated_at
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
            "SELECT scryfall_data_id, is_token, card_roles, oracle_tags, oracle_tags_by_role, other_oracle_tags, created_at, updated_at
            FROM card_profiles WHERE scryfall_data_id = ANY($1)",
            &**request
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
        Ok(card_profiles)
    }

    async fn get_card_profiles_with_scryfall_data_ids(
        &self,
        request: &ScryfallDataIds,
    ) -> Result<Vec<CardProfile>, GetCardProfileError> {
        let card_profiles: Vec<CardProfile> = query_as!(
            DatabaseCardProfile,
            "SELECT scryfall_data_id, is_token, card_roles, oracle_tags, oracle_tags_by_role, other_oracle_tags, created_at, updated_at
            FROM card_profiles WHERE scryfall_data_id = ANY($1)",
            &**request
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Into::into)
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
        // Match the full name, and also the front face of double-faced cards, so
        // "Boggart Trawler // Boggart Bog" resolves whether the entry is the full
        // name, just the front ("Boggart Trawler"), or a "/" spelling. Each entry
        // contributes both itself and its pre-slash front to the match set, and
        // the query checks each card's full name and its front face.
        let mut lowered: std::collections::HashSet<String> = std::collections::HashSet::new();
        for name in names {
            let lower = name.to_lowercase();
            let front = import_deck_cards::entry_front_face(&lower);
            if !front.is_empty() {
                lowered.insert(front.to_string());
            }
            lowered.insert(lower);
        }
        let lowered: Vec<String> = lowered.into_iter().collect();
        // ORDER BY makes the row order stable, so when an entry is ambiguous
        // (two double-faced cards share a front face) resolution is deterministic.
        let db_rows: Vec<DatabaseScryfallData> = query_as(
            "SELECT * FROM latest_cards \
             WHERE LOWER(name) = ANY($1) \
                OR LOWER(split_part(name, ' // ', 1)) = ANY($1) \
             ORDER BY name",
        )
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
        context: DeckServeContext<'_>,
    ) -> Result<Vec<Card>, SearchCardsError> {
        // Card-level wrapper: forward the deck-serve context straight through
        // (its callers never set a commander seed; that path is commander-select).
        let scryfall_data = self
            .search_scryfall_data_deck_aware(request, context)
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

    /// First-class commander search (`context/archive/commander_select_ordering.md`):
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
        // Same degrade-to-everything read as the deck-aware serve: the UB
        // preference should never fail commander select.
        let (exclude_universes_beyond, universes_beyond_exception_set_names) = self
            .get_universes_beyond_preferences(user_id)
            .await
            .unwrap_or_default();
        let scryfall_data = self
            .search_scryfall_data_deck_aware(
                request,
                DeckServeContext {
                    commander_seed: Some(seed),
                    exclude_universes_beyond,
                    universes_beyond_exception_set_names,
                    ..Default::default()
                },
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

    async fn deck_tag_profile(
        &self,
        oracle_ids: &[uuid::Uuid],
        lowercased_names: &[String],
    ) -> Result<Option<serde_json::Value>, SearchCardsError> {
        // Weight = how many profile cards carry the tag × ln(tagged cards /
        // cards with the tag), so a shared rare tag counts far more than a
        // common one. Cycle tags group cards by print run, not by what they
        // do, so they carry no synergy and are left out. The strongest
        // SERVE_PROFILE_TAG_LIMIT tags are kept, normalized to sum to 1.
        let profile: Option<serde_json::Value> = sqlx::query_scalar(
            "WITH profile_cards AS (
                 SELECT unnest($1::uuid[]) AS oracle_id
                 UNION
                 SELECT oracle_id FROM latest_cards
                 WHERE LOWER(name) = ANY($2) AND oracle_id IS NOT NULL
             ),
             tf AS (
                 SELECT t.oracle_tag, count(*)::float8 AS n
                 FROM card_oracle_tags t JOIN profile_cards p USING (oracle_id)
                 WHERE t.oracle_tag NOT LIKE 'cycle-%'
                 GROUP BY t.oracle_tag
             ),
             df AS (
                 SELECT oracle_tag, count(*)::float8 AS n FROM card_oracle_tags
                 WHERE oracle_tag IN (SELECT oracle_tag FROM tf)
                 GROUP BY oracle_tag
             ),
             total AS (SELECT count(DISTINCT oracle_id)::float8 AS n FROM card_oracle_tags),
             weighted AS (
                 SELECT tf.oracle_tag, tf.n * ln(total.n / df.n) AS w
                 FROM tf JOIN df USING (oracle_tag) CROSS JOIN total
                 WHERE df.n < total.n
                 ORDER BY w DESC
                 LIMIT $3
             )
             SELECT jsonb_object_agg(oracle_tag, w / (SELECT sum(w) FROM weighted))
             FROM weighted",
        )
        .bind(oracle_ids)
        .bind(lowercased_names)
        .bind(SERVE_PROFILE_TAG_LIMIT)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SearchScryfallDataError::Database(e.into()))?;
        Ok(profile)
    }

    async fn deck_tag_profiles(
        &self,
        decks: &[Vec<uuid::Uuid>],
    ) -> Result<Vec<Option<DeckTagProfile>>, SearchCardsError> {
        /// Deck index, tag weights, and each kept tag's card count.
        type ProfileRow = (i32, serde_json::Value, Json<HashMap<String, u32>>);

        // Lift weighting, the steps in zwipe_core::domain::deck::tag_profile
        // run for every deck in one statement. The decks bind as two parallel
        // arrays, one (deck index, oracle id) pair per card, since a
        // uuid[][] does not bind. Only tags under a card role (the shared
        // membership CTEs, the same lookup tag_roles reads) and outside the
        // excluded prefix count. A deck's tagged cards and the corpus's
        // tagged cards both count cards with any oracle tag. The lift test
        // compares the cross products, which are exact in float8 for these
        // counts, so it agrees with lift_weight. The corpus counts do not
        // depend on the deck, so they are counted once; the per-deck rank and
        // the normalizing sum are window functions partitioned by deck. Each
        // kept tag's card count rides along as a second object. A deck whose
        // cards carry no usable tags produces no row and stays None.
        let mut deck_indices: Vec<i32> = Vec::new();
        let mut oracle_ids: Vec<uuid::Uuid> = Vec::new();
        for (index, deck) in decks.iter().enumerate() {
            let index = i32::try_from(index)
                .context("deck index")
                .map_err(SearchScryfallDataError::Database)?;
            deck_indices.extend(std::iter::repeat_n(index, deck.len()));
            oracle_ids.extend_from_slice(deck);
        }
        let excluded = format!("{PROFILE_EXCLUDED_TAG_PREFIX}%");
        // The SQL text is literals plus parameter numbers; the data is all
        // bound.
        let binds = RoleMembershipBinds::new();
        let sql = format!(
            "WITH RECURSIVE {},
             profile_cards AS (
                 SELECT DISTINCT deck, oracle_id
                 FROM unnest($7::int4[], $8::uuid[]) AS c(deck, oracle_id)
             ),
             deck_tagged AS (
                 SELECT p.deck, count(*)::float8 AS n
                 FROM profile_cards p
                 WHERE EXISTS (SELECT 1 FROM card_oracle_tags t WHERE t.oracle_id = p.oracle_id)
                 GROUP BY p.deck
             ),
             tf AS (
                 SELECT p.deck, t.oracle_tag, count(*)::float8 AS n
                 FROM card_oracle_tags t JOIN profile_cards p USING (oracle_id)
                 WHERE t.oracle_tag NOT LIKE $10
                   AND t.oracle_tag IN (SELECT slug FROM membership)
                 GROUP BY p.deck, t.oracle_tag
             ),
             df AS (
                 SELECT oracle_tag, count(*)::float8 AS n FROM card_oracle_tags
                 WHERE oracle_tag IN (SELECT oracle_tag FROM tf)
                 GROUP BY oracle_tag
             ),
             total AS (SELECT count(DISTINCT oracle_id)::float8 AS n FROM card_oracle_tags),
             weighted AS (
                 SELECT tf.deck, tf.oracle_tag, tf.n AS cards,
                        tf.n * ln((tf.n / dt.n) / (df.n / total.n)) AS w
                 FROM tf
                 JOIN df USING (oracle_tag)
                 JOIN deck_tagged dt USING (deck)
                 CROSS JOIN total
                 WHERE tf.n * total.n > df.n * dt.n
             ),
             ranked AS (
                 SELECT deck, oracle_tag, cards, w,
                        row_number() OVER (PARTITION BY deck ORDER BY w DESC, oracle_tag) AS rn
                 FROM weighted
             ),
             normalized AS (
                 SELECT deck, oracle_tag, cards, w / sum(w) OVER (PARTITION BY deck) AS w
                 FROM ranked WHERE rn <= $9
             )
             SELECT deck, jsonb_object_agg(oracle_tag, w),
                    jsonb_object_agg(oracle_tag, cards::int4)
             FROM normalized GROUP BY deck",
            oracle_tag_groups::role_membership_ctes(1)
        );
        let rows: Vec<ProfileRow> = binds
            .bind(sqlx::query(sqlx::AssertSqlSafe(sql)))
            .bind(&deck_indices)
            .bind(&oracle_ids)
            .bind(i64::from(PROFILE_TAG_LIMIT))
            .bind(&excluded)
            .map(|row: sqlx::postgres::PgRow| (row.get(0), row.get(1), row.get(2)))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SearchScryfallDataError::Database(e.into()))?;
        let mut profiles: Vec<Option<DeckTagProfile>> = vec![None; decks.len()];
        for (deck, weights, Json(cards)) in rows {
            let slot = usize::try_from(deck)
                .ok()
                .and_then(|index| profiles.get_mut(index));
            if let Some(slot) = slot {
                *slot = Some(DeckTagProfile { weights, cards });
            }
        }
        Ok(profiles)
    }

    async fn score_commander_suggestions(
        &self,
        user_id: uuid::Uuid,
        decks: &serde_json::Value,
        exclude_oracle_ids: &[uuid::Uuid],
        limit: i64,
    ) -> Result<Vec<ScoredCommander>, CommanderSuggestionsError> {
        // The statement and its scoring rules are documented on
        // SCORE_COMMANDER_SUGGESTIONS_SQL.
        let rows: Vec<(uuid::Uuid, Vec<String>, f64, Vec<String>)> =
            sqlx::query(SCORE_COMMANDER_SUGGESTIONS_SQL)
                .bind(decks)
                .bind(exclude_oracle_ids)
                .bind(user_id)
                .bind(DISMISSAL_TTL_DAYS)
                .bind(SUGGESTION_TAG_LIMIT)
                .bind(W_COLOR_AFFINITY)
                .bind(limit)
                .map(|row: sqlx::postgres::PgRow| (row.get(0), row.get(1), row.get(2), row.get(3)))
                .fetch_all(&self.pool)
                .await
                .map_err(|e| CommanderSuggestionsError::Database(e.into()))?;
        Ok(rows
            .into_iter()
            .map(
                |(oracle_id, color_identity, _score, tag_slugs)| ScoredCommander {
                    oracle_id,
                    color_identity,
                    tag_slugs,
                },
            )
            .collect())
    }

    async fn tag_roles(
        &self,
        slugs: &[String],
    ) -> Result<HashMap<String, String>, CommanderSuggestionsError> {
        if slugs.is_empty() {
            return Ok(HashMap::new());
        }
        // The SQL text is literals plus parameter numbers; the data is all
        // bound.
        let binds = RoleMembershipBinds::new();
        let sql = format!(
            "WITH RECURSIVE {}
             SELECT slug, min(role) FROM membership
             WHERE slug = ANY($7::text[])
             GROUP BY slug",
            oracle_tag_groups::role_membership_ctes(1)
        );
        let rows: Vec<(String, String)> = binds
            .bind(sqlx::query(sqlx::AssertSqlSafe(sql)))
            .bind(slugs)
            .map(|row: sqlx::postgres::PgRow| (row.get(0), row.get(1)))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CommanderSuggestionsError::Database(e.into()))?;
        Ok(rows.into_iter().collect())
    }

    // == category derivation ==

    async fn get_cards_batch(&self, ids: &[uuid::Uuid]) -> Result<Vec<Card>, anyhow::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let scryfall_data_ids: ScryfallDataIds = ids.iter().copied().collect();
        self.get_cards(&scryfall_data_ids)
            .await
            .map_err(|e| anyhow::anyhow!("failed to fetch card batch: {e}"))
    }

    async fn update_card_roles(
        &self,
        updates: &[(
            uuid::Uuid,
            Vec<zwipe_core::domain::card::card_role::CardRole>,
        )],
    ) -> Result<(), anyhow::Error> {
        if updates.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for (id, categories) in updates {
            let cat_strings: Vec<String> = categories.iter().map(ToString::to_string).collect();
            let json = serde_json::to_value(&cat_strings)?;
            sqlx::query!(
                "UPDATE card_profiles SET card_roles = $1, updated_at = NOW() WHERE scryfall_data_id = $2",
                json,
                id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn derive_oracle_tag_categories(&self) -> anyhow::Result<u64> {
        crate::outbound::sqlx::card::helpers::derive_categories::derive_categories(&self.pool).await
    }

    async fn refresh_oracle_tag_groups(&self) -> anyhow::Result<u64> {
        oracle_tag_groups::refresh_oracle_tag_groups(&self.pool).await
    }

    async fn get_card_ids_with_oracle_text(&self) -> Result<Vec<uuid::Uuid>, anyhow::Error> {
        let ids: Vec<uuid::Uuid> = sqlx::query_scalar(
            "SELECT cp.scryfall_data_id FROM card_profiles cp \
             JOIN scryfall_data sd ON cp.scryfall_data_id = sd.id \
             WHERE sd.oracle_text IS NOT NULL AND sd.oracle_id IS NOT NULL",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(ids)
    }
}
