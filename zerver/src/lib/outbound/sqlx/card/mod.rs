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

use zwipe_core::domain::card::{
    Card,
    card_profile::CardProfile,
    scryfall_data::ScryfallData,
    search_card::{
        card_filter::{CardFilter, order_by_option::OrderByOption},
        filter_cards::PLAYABLE_LAYOUTS,
    },
};
use crate::domain::card::models::{
    helpers::SleeveCardProfile,
    search_card::error::SearchCardsError,
    zervice_metrics::ZerviceMetrics,
};
use crate::domain::card::requests::{
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
};
use crate::outbound::sqlx::card::card_profile::DatabaseCardProfile;
use crate::outbound::sqlx::card::helpers::upsert_card::BatchDeltaUpsertWithTx;
use crate::outbound::sqlx::card::models::DatabaseScryfallData;
use crate::outbound::sqlx::card::zervice_metrics::DatabaseZerviceMetrics;
use crate::outbound::sqlx::postgres::Postgres as MyPostgres;
use crate::{
    domain::card::ports::CardRepository,
    outbound::sqlx::card::helpers::upsert_card::{
        BatchUpsertWithTx, BulkUpsertWithTx, SingleUpsertWithTx,
    },
};
use zwipe_core::domain::deck::Format;

use anyhow::Context;
use chrono::NaiveDateTime;
use sqlx::{Postgres, query_as, query_scalar};
use sqlx::{QueryBuilder, query_builder::Separated};

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

    /// Searches Scryfall data with a CTE-based deduplication strategy.
    ///
    /// Uses `ROW_NUMBER() OVER (PARTITION BY COALESCE(oracle_id, id))` to keep only the
    /// latest printing per unique card. Filter clauses are composed with `AND` via
    /// `QueryBuilder::separated`. Color identity uses `@>`/`<@` for exact match and
    /// power-set enumeration for "within" queries.
    async fn search_scryfall_data(
        &self,
        request: &CardFilter,
    ) -> Result<Vec<ScryfallData>, SearchScryfallDataError> {
        let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new(
            "WITH deduplicated_cards AS (
               SELECT scryfall_data.id,
                      ROW_NUMBER() OVER (
                        PARTITION BY COALESCE(scryfall_data.oracle_id, scryfall_data.id)
                        ORDER BY scryfall_data.released_at DESC
                      ) as rn
               FROM scryfall_data
               JOIN card_profiles ON scryfall_data.id = card_profiles.scryfall_data_id
               WHERE ",
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

        if let Some(query_string_array) = &request.type_line_contains_all() {
            for query_string in query_string_array.iter() {
                sep.push("type_line ILIKE ");
                sep.push_bind_unseparated(format!("%{}%", query_string));
            }
        }

        if let Some(card_types) = &request.card_type_contains_all() {
            for card_type in card_types.iter() {
                sep.push("type_line ILIKE ");
                sep.push_bind_unseparated(format!("%{}%", card_type));
            }
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
            sep.push_bind_unseparated(rarities.to_short_names());
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
            sep.push_bind_unseparated(colors.to_short_names());
            sep.push("color_identity <@ ");
            sep.push_bind_unseparated(colors.to_short_names());
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

        if let Some(query_string_array) = &request.oracle_text_contains_any() {
            sep.push(" (");
            query_string_array
                .iter()
                .enumerate()
                .for_each(|(i, query_string)| {
                    if i > 0 {
                        sep.push_unseparated(" OR ");
                    }
                    sep.push_unseparated("oracle_text ILIKE ");
                    sep.push_bind_unseparated(format!("%{}%", query_string));
                });
            sep.push_unseparated(") ");
        }

        if let Some(query_string_array) = &request.oracle_text_contains_all() {
            for query_string in query_string_array.iter() {
                sep.push("oracle_text ILIKE ");
                sep.push_bind_unseparated(format!("%{}%", query_string));
            }
        }

        if let Some(keywords) = &request.keywords_contains_any() {
            sep.push("keywords && ARRAY[");
            keywords.iter().enumerate().for_each(|(i, kw)| {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(kw.to_lowercase());
            });
            sep.push_unseparated("]::text[]");
        }

        if let Some(keywords) = &request.keywords_contains_all() {
            sep.push("keywords @> ARRAY[");
            keywords.iter().enumerate().for_each(|(i, kw)| {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(kw.to_lowercase());
            });
            sep.push_unseparated("]::text[]");
        }

        if let Some(colors) = &request.produced_mana_contains_any() {
            sep.push("produced_mana && ARRAY[");
            colors.iter().enumerate().for_each(|(i, c)| {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(c.to_uppercase());
            });
            sep.push_unseparated("]::text[]");
        }

        if let Some(colors) = &request.produced_mana_contains_all() {
            sep.push("produced_mana @> ARRAY[");
            colors.iter().enumerate().for_each(|(i, c)| {
                if i > 0 {
                    sep.push_unseparated(", ");
                }
                sep.push_bind_unseparated(c.to_uppercase());
            });
            sep.push_unseparated("]::text[]");
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
            sep.push_bind_unseparated(language);
        }

        if let Some(formats) = request.legalities_contains_any() {
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

        if let Some(format) = request.is_commander_in_format() {
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
                // Uncommon creature
                Format::PauperCommander => {
                    sep.push("(type_line ILIKE '%Creature%' AND rarity = 'uncommon')");
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
        if let Some(true) = request.is_partner() {
            sep.push(
                "(type_line ILIKE '%Legendary%' AND type_line ILIKE '%Creature%' AND (\
                 keywords @> ARRAY['Partner']::text[] \
                 OR keywords @> ARRAY['Friends forever']::text[] \
                 OR keywords @> ARRAY['Doctor''s companion']::text[] \
                 OR oracle_text ILIKE '%partner with%'))",
            );
        }

        if let Some(true) = request.is_background() {
            sep.push(
                "(type_line ILIKE '%Legendary%' AND type_line ILIKE '%Enchantment%' \
                 AND type_line ILIKE '%Background%')",
            );
        }

        if let Some(true) = request.is_signature_spell() {
            sep.push(
                "(type_line ILIKE '%Instant%' OR type_line ILIKE '%Sorcery%')",
            );
        }

        // Filter out NULLs for sorted field
        if let Some(order_by) = request.order_by() {
            let null_filter = match order_by {
                OrderByOption::Power => Some("power IS NOT NULL AND power ~ '^\\d+$'"),
                OrderByOption::Toughness => Some("toughness IS NOT NULL AND toughness ~ '^\\d+$'"),
                OrderByOption::PriceUsd => {
                    Some("prices->>'usd' IS NOT NULL AND prices->>'usd' != ''")
                }
                OrderByOption::PriceEur => {
                    Some("prices->>'eur' IS NOT NULL AND prices->>'eur' != ''")
                }
                OrderByOption::PriceTix => {
                    Some("prices->>'tix' IS NOT NULL AND prices->>'tix' != ''")
                }
                _ => None,
            };
            if let Some(filter) = null_filter {
                sep.push(filter);
            }
        }

        // Close CTE and select scryfall_data for latest printings only
        qb.push(
            ") SELECT scryfall_data.* FROM scryfall_data
                  JOIN deduplicated_cards ON scryfall_data.id = deduplicated_cards.id
                  WHERE deduplicated_cards.rn = 1 ",
        );

        // ORDER BY
        if let Some(order_by) = request.order_by() {
            qb.push(" ORDER BY ");
            let col = match order_by {
                OrderByOption::Name => "name",
                OrderByOption::Cmc => "cmc",
                OrderByOption::Power => "CAST(NULLIF(power, '') AS INT)",
                OrderByOption::Toughness => "CAST(NULLIF(toughness, '') AS INT)",
                OrderByOption::Rarity => "rarity",
                OrderByOption::ReleasedAt => "released_at",
                OrderByOption::PriceUsd => "(prices->>'usd')::NUMERIC",
                OrderByOption::PriceEur => "(prices->>'eur')::NUMERIC",
                OrderByOption::PriceTix => "(prices->>'tix')::NUMERIC",
                OrderByOption::Random => "RANDOM()",
            };
            qb.push(col);
            if order_by != OrderByOption::Random {
                qb.push(if request.ascending() { " ASC" } else { " DESC" });
            }
        }

        qb.push(" LIMIT ");
        qb.push_bind(request.limit() as i32);

        qb.push(" OFFSET ");
        qb.push_bind(request.offset() as i32);

        let db_rows: Vec<DatabaseScryfallData> = qb.build_query_as().fetch_all(&self.pool).await?;
        let scryfall_data: Vec<ScryfallData> = db_rows
            .into_iter()
            .map(ScryfallData::try_from)
            .collect::<Result<_, _>>()
            .map_err(SearchScryfallDataError::Database)?;
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

    /// Extracts distinct card types by tokenizing `type_line` with `STRING_TO_ARRAY`.
    async fn get_card_types(&self) -> Result<Vec<String>, GetCardTypesError> {
        // Stop words: see domain::card::models::search_card::stop_words::TYPE_STOP_WORDS
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

    /// Returns distinct keyword abilities from the `keywords` array column.
    async fn get_keywords(&self) -> Result<Vec<String>, GetKeywordsError> {
        let keywords: Vec<String> = query_scalar!(
            "SELECT DISTINCT LOWER(TRIM(keyword)) AS keyword
             FROM scryfall_data, UNNEST(keywords) AS keyword
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
             FROM scryfall_data, REGEXP_SPLIT_TO_TABLE(oracle_text, '\\s+') AS word
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

    async fn get_languages(&self) -> Result<Vec<String>, GetLanguagesError> {
        let languages: Vec<String> = query_scalar!(
            "SELECT DISTINCT lang FROM scryfall_data
             WHERE lang IS NOT NULL
             ORDER BY lang"
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
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

    async fn get_last_sync_date(&self) -> anyhow::Result<Option<NaiveDateTime>> {
        let last_sync_date: Option<NaiveDateTime> = query_scalar(
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
        let db_rows: Vec<DatabaseScryfallData> = query_as(
            "WITH deduplicated_cards AS (
                SELECT sd.id,
                       ROW_NUMBER() OVER (
                         PARTITION BY COALESCE(sd.oracle_id, sd.id)
                         ORDER BY sd.released_at DESC
                       ) as rn
                FROM scryfall_data sd
                JOIN card_profiles cp ON sd.id = cp.scryfall_data_id
                WHERE LOWER(sd.name) = ANY($1)
            )
            SELECT sd.* FROM scryfall_data sd
            JOIN deduplicated_cards dc ON sd.id = dc.id
            WHERE dc.rn = 1",
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
        self.get_cards(&scryfall_data_ids).await
            .map_err(|e| anyhow::anyhow!("failed to fetch card batch: {e}"))
    }

    async fn update_mechanical_categories(
        &self,
        updates: &[(uuid::Uuid, Vec<zwipe_core::domain::card::mechanical_category::MechanicalCategory>)],
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
        sqlx::query!("UPDATE card_profiles SET mechanical_categories = '[]'::jsonb")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
