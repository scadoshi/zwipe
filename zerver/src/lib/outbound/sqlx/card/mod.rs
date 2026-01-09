pub mod card_profile;
pub mod error;
pub mod helpers;
pub mod scryfall_data;
pub mod sync_metrics;

use crate::domain::card::models::{
    Card,
    create_card::CreateCardError,
    get_artists::GetArtistsError,
    get_sets::GetSetsError,
    scryfall_data::get_scryfall_data::{GetScryfallData, GetScryfallDataError},
    search_card::{
        card_filter::{CardFilter, order_by_options::OrderByOptions},
        error::SearchCardsError,
    },
};
use crate::outbound::sqlx::card::card_profile::DatabaseCardProfile;
use crate::outbound::sqlx::card::helpers::upsert_card::BatchDeltaUpsertWithTx;
use crate::outbound::sqlx::postgres::Postgres as MyPostgres;
use crate::{
    domain::card::models::{
        card_profile::{
            CardProfile,
            get_card_profile::{CardProfileIds, GetCardProfile, GetCardProfileError},
        },
        get_card::GetCardError,
        get_card_types::GetCardTypesError,
        helpers::SleeveCardProfile,
        scryfall_data::get_scryfall_data::{ScryfallDataIds, SearchScryfallDataError},
    },
    outbound::sqlx::card::helpers::upsert_card::{
        BatchUpsertWithTx, BulkUpsertWithTx, SingleUpsertWithTx,
    },
};
use crate::{
    domain::card::{
        models::{scryfall_data::ScryfallData, sync_metrics::SyncMetrics},
        ports::CardRepository,
    },
    outbound::sqlx::card::sync_metrics::DatabaseSyncMetrics,
};

use anyhow::Context;
use chrono::NaiveDateTime;
use sqlx::{Postgres, query_as, query_scalar};
use sqlx::{QueryBuilder, query_builder::Separated};

// Playable layouts whitelist - layouts that represent cards playable in Magic formats
// Unknown layouts default to hidden (safe behavior)
const PLAYABLE_LAYOUTS: &[&str] = &[
    "normal",
    "split",
    "flip",
    "transform",
    "modal_dfc",
    "meld",
    "reversible_card",
    "leveler",
    "saga",
    "adventure",
    "mutate",
    "prototype",
    "battle",
    "class",
    "case",
];

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
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let cards = multiple_scryfall_data
            .batch_upsert_with_tx(&mut tx, batch_size, sync_metrics)
            .await?;
        tx.commit().await?;
        Ok(cards)
    }

    async fn batch_delta_upsert(
        &self,
        multiple_scryfall_data: &[ScryfallData],
        batch_size: usize,
        sync_metrics: &mut SyncMetrics,
    ) -> Result<Vec<Card>, CreateCardError> {
        let mut tx = self.pool.begin().await?;
        let cards = multiple_scryfall_data
            .batch_delta_upsert_with_tx(&mut tx, batch_size, sync_metrics)
            .await?;
        tx.commit().await?;
        Ok(cards)
    }

    async fn record_sync_metrics(
        &self,
        sync_metrics: &SyncMetrics,
    ) -> Result<SyncMetrics, anyhow::Error> {
        let mut tx = self.pool.begin().await?;
        let query_sql = "INSERT INTO scryfall_data_sync_metrics".to_string()
            + " (started_at, ended_at, duration_in_seconds, status, received_count, upserted_count, skipped_count, error_count, errors)"
            + " VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *";
        let database_sync_metrics: DatabaseSyncMetrics = query_as(&query_sql)
            .bind(sync_metrics.started_at())
            .bind(sync_metrics.ended_at())
            .bind(sync_metrics.duration_in_seconds())
            .bind(sync_metrics.status().to_string())
            .bind(sync_metrics.received_count())
            .bind(sync_metrics.upserted_count())
            .bind(sync_metrics.skipped_count())
            .bind(sync_metrics.error_count())
            .bind(sync_metrics.errors())
            .fetch_one(&mut *tx)
            .await?;
        let sync_metrics: SyncMetrics = database_sync_metrics.try_into()?;
        tx.commit().await?;
        Ok(sync_metrics)
    }

    // =====
    //  get
    // =====
    async fn get_scryfall_data(
        &self,
        request: &GetScryfallData,
    ) -> Result<ScryfallData, GetScryfallDataError> {
        let scryfall_data: ScryfallData = query_as("SELECT * FROM scryfall_data WHERE id = $1")
            .bind(request.id())
            .fetch_one(&self.pool)
            .await?;

        Ok(scryfall_data)
    }

    async fn get_multiple_scryfall_data(
        &self,
        request: &ScryfallDataIds,
    ) -> Result<Vec<ScryfallData>, GetScryfallDataError> {
        let scryfall_data: Vec<ScryfallData> =
            query_as("SELECT * FROM scryfall_data WHERE id = ANY($1)")
                .bind(request.ids())
                .fetch_all(&self.pool)
                .await?;

        Ok(scryfall_data)
    }

    async fn search_scryfall_data(
        &self,
        request: &CardFilter,
    ) -> Result<Vec<ScryfallData>, SearchScryfallDataError> {
        let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new(
            "SELECT scryfall_data.* FROM scryfall_data JOIN card_profiles ON scryfall_data.id = card_profiles.scryfall_data_id WHERE ",
        );
        let mut sep: Separated<Postgres, &'static str> = qb.separated(" AND ");

        if let Some(query_string) = &request.name_contains() {
            sep.push("name ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", query_string));
        }

        if let Some(query_string) = &request.type_line_contains() {
            sep.push("type_line ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", query_string));
        }

        if let Some(query_string_array) = &request.type_line_contains_any() {
            sep.push(" (");
            query_string_array
                .iter()
                .enumerate()
                .for_each(|(i, query_string)| {
                    if i > 0 {
                        sep.push_unseparated(" OR ");
                    }
                    sep.push_unseparated("type_line ILIKE ");
                    sep.push_bind_unseparated(format!("%{}%", query_string));
                });
            sep.push_unseparated(") ");
        }

        if let Some(card_types) = &request.card_type_contains_any() {
            sep.push(" (");
            card_types.iter().enumerate().for_each(|(i, query_string)| {
                if i > 0 {
                    sep.push_unseparated(" OR ");
                }
                sep.push_unseparated("type_line ILIKE ");
                sep.push_bind_unseparated(format!("%{}%", query_string));
            });
            sep.push_unseparated(") ");
        }

        if let Some(sets) = request.set_equals_any() {
            sep.push("set_name = ANY(");
            sep.push_bind_unseparated(sets);
            sep.push_unseparated(")");
        }

        if let Some(artists) = request.artist_equals_any() {
            sep.push("artist = ANY(");
            sep.push_bind_unseparated(artists);
            sep.push_unseparated(")");
        }

        if let Some(rarities) = request.rarity_equals_any() {
            sep.push("rarity = ANY(");
            sep.push_bind_unseparated(rarities);
            sep.push_unseparated(")");
        }

        if let Some(query_string) = request.cmc_equals() {
            sep.push("cmc = ");
            sep.push_bind_unseparated(query_string);
        }

        if let Some(cmc_range) = request.cmc_range() {
            let lower = cmc_range.0.min(cmc_range.1);
            let higher = cmc_range.0.max(cmc_range.1);
            sep.push("cmc between ");
            sep.push_bind_unseparated(lower);
            sep.push("");
            sep.push_bind_unseparated(higher);
        }

        if let Some(query_string) = request.power_equals() {
            sep.push("power ~ '^\\d+$' AND CAST(power AS INT) = ");
            sep.push_bind_unseparated(query_string);
        }

        if let Some(power_range) = request.power_range() {
            let lower = power_range.0.min(power_range.1);
            let higher = power_range.0.max(power_range.1);
            sep.push("power ~ '^\\d+$' AND CAST(power AS INT) between ");
            sep.push_bind_unseparated(lower);
            sep.push_unseparated(" AND ");
            sep.push_bind_unseparated(higher);
        }

        if let Some(query_string) = request.toughness_equals() {
            sep.push("toughness ~ '^\\d+$' AND CAST(toughness AS INT) = ");
            sep.push_bind_unseparated(query_string);
        }

        if let Some(toughness_range) = request.toughness_range() {
            let lower = toughness_range.0.min(toughness_range.1);
            let higher = toughness_range.0.max(toughness_range.1);
            sep.push("toughness ~ '^\\d+$' AND CAST(toughness AS INT) between ");
            sep.push_bind_unseparated(lower);
            sep.push_unseparated(" AND ");
            sep.push_bind_unseparated(higher);
        }

        if let Some(colors) = request.color_identity_equals() {
            sep.push("color_identity @> ");
            sep.push_bind_unseparated(colors);
            sep.push("color_identity <@ ");
            sep.push_bind_unseparated(colors);
        }

        if let Some(colors) = request.color_identity_within() {
            let all_combinations = (1..1 << colors.len()).fold(
                Vec::<Vec<String>>::new(),
                |mut all_combinations, bits| {
                    all_combinations.push(
                        colors
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| bits & (1 << i) != 0)
                            .map(|(_, c)| c.to_short_name())
                            .collect(),
                    );
                    all_combinations
                },
            );

            sep.push("(");
            all_combinations
                .into_iter()
                .enumerate()
                .for_each(|(i, subset)| {
                    if i > 0 {
                        sep.push_unseparated(" OR ");
                    }
                    sep.push_unseparated("color_identity <@ ");
                    sep.push_bind_unseparated(subset);
                });
            sep.push_unseparated(")");
        }

        if let Some(query_string) = &request.oracle_text_contains() {
            sep.push("oracle_text ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", query_string));
        }

        if let Some(query_string) = &request.flavor_text_contains() {
            sep.push("flavor_text ILIKE ");
            sep.push_bind_unseparated(format!("%{}%", query_string));
        }

        if let Some(has_flavor_text) = request.has_flavor_text() {
            if has_flavor_text {
                sep.push("flavor_text IS NOT NULL AND flavor_text != ''");
            } else {
                sep.push("(flavor_text IS NULL OR flavor_text = '')");
            }
        }

        // flag filters
        if let Some(is_commander) = request.is_valid_commander() {
            sep.push(" card_profiles.is_valid_commander = ");
            sep.push_bind_unseparated(is_commander);
        }

        if let Some(is_tok) = request.is_token() {
            sep.push(" card_profiles.is_token = ");
            sep.push_bind_unseparated(is_tok);
        }

        if let Some(is_playable) = request.is_playable()
            && is_playable
        {
            // Only playable layouts
            sep.push("scryfall_data.layout = ANY(");
            sep.push_bind_unseparated(PLAYABLE_LAYOUTS);
            sep.push_unseparated(")");
        } else if let Some(is_playable) = request.is_playable()
            && !is_playable
        {
            // Only non-playable layouts
            sep.push("scryfall_data.layout != ALL(");
            sep.push_bind_unseparated(PLAYABLE_LAYOUTS);
            sep.push_unseparated(")");
        }

        if let Some(is_digital) = request.digital() {
            sep.push("scryfall_data.digital = ");
            sep.push_bind_unseparated(is_digital);
        }

        if let Some(is_oversized) = request.oversized() {
            sep.push("scryfall_data.oversized = ");
            sep.push_bind_unseparated(is_oversized);
        }

        if let Some(is_promo) = request.promo() {
            sep.push("scryfall_data.promo = ");
            sep.push_bind_unseparated(is_promo);
        }

        if let Some(has_warning) = request.content_warning() {
            if has_warning {
                sep.push("scryfall_data.content_warning = true");
            } else {
                // Hide cards with warnings (include false OR null)
                sep.push("(scryfall_data.content_warning = false OR scryfall_data.content_warning IS NULL)");
            }
        }

        if let Some(language) = request.language() {
            sep.push("scryfall_data.lang = ");
            sep.push_bind_unseparated(language.to_code());
        }

        // Filter out NULLs for sorted field
        if let Some(order_by) = request.order_by() {
            let null_filter = match order_by {
                OrderByOptions::Power => Some("power IS NOT NULL AND power ~ '^\\d+$'"),
                OrderByOptions::Toughness => Some("toughness IS NOT NULL AND toughness ~ '^\\d+$'"),
                OrderByOptions::PriceUsd => {
                    Some("prices->>'usd' IS NOT NULL AND prices->>'usd' != ''")
                }
                OrderByOptions::PriceEur => {
                    Some("prices->>'eur' IS NOT NULL AND prices->>'eur' != ''")
                }
                OrderByOptions::PriceTix => {
                    Some("prices->>'tix' IS NOT NULL AND prices->>'tix' != ''")
                }
                _ => None,
            };
            if let Some(filter) = null_filter {
                sep.push(filter);
            }
        }

        // ORDER BY
        if let Some(order_by) = request.order_by() {
            qb.push(" ORDER BY ");
            let col = match order_by {
                OrderByOptions::Name => "name",
                OrderByOptions::Cmc => "cmc",
                OrderByOptions::Power => "CAST(NULLIF(power, '') AS INT)",
                OrderByOptions::Toughness => "CAST(NULLIF(toughness, '') AS INT)",
                OrderByOptions::Rarity => "rarity",
                OrderByOptions::ReleasedAt => "released_at",
                OrderByOptions::PriceUsd => "(prices->>'usd')::NUMERIC",
                OrderByOptions::PriceEur => "(prices->>'eur')::NUMERIC",
                OrderByOptions::PriceTix => "(prices->>'tix')::NUMERIC",
                OrderByOptions::Random => "RANDOM()",
            };
            qb.push(col);
            if order_by != OrderByOptions::Random {
                qb.push(if request.ascending() { " ASC" } else { " DESC" });
            }
        }

        qb.push(" LIMIT ");
        qb.push_bind(request.limit() as i32);

        qb.push(" OFFSET ");
        qb.push_bind(request.offset() as i32);

        let scryfall_data: Vec<ScryfallData> = qb.build_query_as().fetch_all(&self.pool).await?;
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

    async fn search_cards(&self, request: &CardFilter) -> Result<Vec<Card>, SearchCardsError> {
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

    async fn get_card_types(&self) -> Result<Vec<String>, GetCardTypesError> {
        let card_types: Vec<String> = query_scalar!(
            "SELECT DISTINCT subtype FROM (
                SELECT TRIM(BOTH ':-?, ' FROM UNNEST(STRING_TO_ARRAY(type_line, ' '))) subtype
                FROM scryfall_data
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

    async fn get_artists(&self) -> Result<Vec<String>, GetArtistsError> {
        let artists: Vec<String> = query_scalar!(
            "SELECT DISTINCT artist FROM scryfall_data
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
            query_scalar!("SELECT DISTINCT set_name FROM scryfall_data ORDER BY set_name")
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .collect();
        Ok(sets)
    }

    async fn get_card_profile_with_id(
        &self,
        request: &GetCardProfile,
    ) -> Result<CardProfile, GetCardProfileError> {
        let card_profile: CardProfile = query_as!(
            DatabaseCardProfile,
            "SELECT id, scryfall_data_id, is_valid_commander, is_token, created_at, updated_at FROM card_profiles WHERE id = $1",
            request.id()
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
            "SELECT id, scryfall_data_id, is_valid_commander, is_token, created_at, updated_at
            FROM card_profiles WHERE scryfall_data_id = $1",
            request.id()
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
            "SELECT id, scryfall_data_id, is_valid_commander, is_token, created_at, updated_at
            FROM card_profiles WHERE id = ANY($1)",
            request.ids()
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
            "SELECT id, scryfall_data_id, is_valid_commander, is_token, created_at, updated_at
            FROM card_profiles WHERE scryfall_data_id = ANY($1)",
            request.ids()
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|dcp| dcp.into())
        .collect();
        Ok(card_profiles)
    }

    async fn get_last_sync_date(&self) -> anyhow::Result<Option<NaiveDateTime>> {
        let last_sync_date: Option<NaiveDateTime> = query_scalar(
            "SELECT started_at FROM scryfall_data_sync_metrics
            ORDER BY started_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to get last sync date")?;
        Ok(last_sync_date)
    }
}
