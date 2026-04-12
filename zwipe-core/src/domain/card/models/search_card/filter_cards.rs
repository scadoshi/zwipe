//! In-memory card filtering for local `Vec<Card>` slices.
//!
//! Mirrors the SQL adapter's filtering logic exactly, enabling the same filter
//! criteria (`CardFilter`) to be applied against a local collection of cards
//! without a server round-trip.
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::card::models::filter_cards::FilterCards;
//!
//! let displayed = match filter_builder.build() {
//!     Ok(filter) => deck_cards.filter_by(&filter),
//!     Err(_) => deck_cards,
//! };
//! ```

use crate::domain::card::{
    Card,
    search_card::card_filter::{CardFilter, builder::CardFilterBuilder, order_by_option::OrderByOption},
};
use crate::domain::deck::DeckEntry;
use rand::seq::SliceRandom;

/// Layouts representing cards playable in Magic formats.
///
/// Unknown layouts **default to hidden** (safe behavior) — new Scryfall layouts
/// won't appear in results until explicitly whitelisted here.
pub const PLAYABLE_LAYOUTS: &[&str] = &[
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

/// Extension trait for filtering a `Vec<Card>` in memory using a `CardFilter`.
pub trait FilterCards {
    /// Filters, sorts, and paginates cards according to the given filter criteria.
    fn filter_by(self, filter: &CardFilter) -> Vec<Card>;
}

impl FilterCards for Vec<Card> {
    fn filter_by(self, filter: &CardFilter) -> Vec<Card> {
        let mut cards: Vec<Card> = self
            .into_iter()
            .filter(|card| {
                let sd = &card.scryfall_data;
                let cp = &card.card_profile;

                // ── text ──────────────────────────────────────────────────────
                if let Some(q) = filter.name_contains()
                    && !sd.name.to_lowercase().contains(&q.to_lowercase())
                {
                    return false;
                }

                if let Some(q) = filter.oracle_text_contains() {
                    match &sd.oracle_text {
                        Some(text) if text.to_lowercase().contains(&q.to_lowercase()) => {}
                        _ => return false,
                    }
                }

                if let Some(values) = filter.oracle_text_contains_any() {
                    let matches = match &sd.oracle_text {
                        Some(text) => values
                            .iter()
                            .any(|v| text.to_lowercase().contains(&v.to_lowercase())),
                        None => false,
                    };
                    if !matches {
                        return false;
                    }
                }

                if let Some(values) = filter.oracle_text_contains_all() {
                    let matches = match &sd.oracle_text {
                        Some(text) => values
                            .iter()
                            .all(|v| text.to_lowercase().contains(&v.to_lowercase())),
                        None => false,
                    };
                    if !matches {
                        return false;
                    }
                }

                // ── keywords ──────────────────────────────────────────────────
                if let Some(values) = filter.keywords_contains_any() {
                    let matches = match &sd.keywords {
                        Some(kw) => values.iter().any(|v| kw.iter().any(|k| k.eq_ignore_ascii_case(v))),
                        None => false,
                    };
                    if !matches {
                        return false;
                    }
                }

                if let Some(values) = filter.keywords_contains_all() {
                    let matches = match &sd.keywords {
                        Some(kw) => values.iter().all(|v| kw.iter().any(|k| k.eq_ignore_ascii_case(v))),
                        None => false,
                    };
                    if !matches {
                        return false;
                    }
                }

                // ── mechanical categories ───────────────────────────────────
                if let Some(values) = filter.mechanical_categories_contains_any() {
                    let matches = card
                        .card_profile
                        .mechanical_categories
                        .iter()
                        .any(|cat| values.iter().any(|v| cat.to_string().eq_ignore_ascii_case(v)));
                    if !matches {
                        return false;
                    }
                }

                if let Some(values) = filter.mechanical_categories_contains_all() {
                    let matches = values.iter().all(|v| {
                        card.card_profile
                            .mechanical_categories
                            .iter()
                            .any(|cat| cat.to_string().eq_ignore_ascii_case(v))
                    });
                    if !matches {
                        return false;
                    }
                }

                // ── produced mana ────────────────────────────────────────────
                if let Some(values) = filter.produced_mana_contains_any() {
                    let matches = match &sd.produced_mana {
                        Some(pm) => values.iter().any(|v| pm.iter().any(|p| p.eq_ignore_ascii_case(v))),
                        None => false,
                    };
                    if !matches {
                        return false;
                    }
                }

                if let Some(values) = filter.produced_mana_contains_all() {
                    let matches = match &sd.produced_mana {
                        Some(pm) => values.iter().all(|v| pm.iter().any(|p| p.eq_ignore_ascii_case(v))),
                        None => false,
                    };
                    if !matches {
                        return false;
                    }
                }

                if let Some(q) = filter.flavor_text_contains() {
                    match &sd.flavor_text {
                        Some(text) if text.to_lowercase().contains(&q.to_lowercase()) => {}
                        _ => return false,
                    }
                }

                if let Some(want_flavor) = filter.has_flavor_text() {
                    let has = sd
                        .flavor_text
                        .as_ref()
                        .map(|t| !t.is_empty())
                        .unwrap_or(false);
                    if has != want_flavor {
                        return false;
                    }
                }

                // ── types ─────────────────────────────────────────────────────
                if let Some(q) = filter.type_line_contains() {
                    match &sd.type_line {
                        Some(tl) if tl.to_lowercase().contains(&q.to_lowercase()) => {}
                        _ => return false,
                    }
                }

                if let Some(values) = filter.type_line_contains_any() {
                    let matches = match &sd.type_line {
                        Some(tl) => values
                            .iter()
                            .any(|v| tl.to_lowercase().contains(&v.to_lowercase())),
                        None => false,
                    };
                    if !matches {
                        return false;
                    }
                }

                if let Some(card_types) = filter.card_type_contains_any() {
                    let matches = match &sd.type_line {
                        Some(tl) => card_types
                            .iter()
                            .any(|ct| tl.to_lowercase().contains(&ct.to_string())),
                        None => false,
                    };
                    if !matches {
                        return false;
                    }
                }

                if let Some(values) = filter.type_line_contains_all() {
                    let matches = match &sd.type_line {
                        Some(tl) => values
                            .iter()
                            .all(|v| tl.to_lowercase().contains(&v.to_lowercase())),
                        None => false,
                    };
                    if !matches {
                        return false;
                    }
                }

                if let Some(card_types) = filter.card_type_contains_all() {
                    let matches = match &sd.type_line {
                        Some(tl) => card_types
                            .iter()
                            .all(|ct| tl.to_lowercase().contains(&ct.to_string())),
                        None => false,
                    };
                    if !matches {
                        return false;
                    }
                }

                // ── mana ──────────────────────────────────────────────────────
                if let Some(val) = filter.cmc_equals()
                    && sd.cmc != Some(val)
                {
                    return false;
                }

                if let Some((min, max)) = filter.cmc_range() {
                    let lo = min.min(max);
                    let hi = min.max(max);
                    match sd.cmc {
                        Some(cmc) if cmc >= lo && cmc <= hi => {}
                        _ => return false,
                    }
                }

                if let Some(filter_colors) = filter.color_identity_equals() {
                    let card_ci = &sd.color_identity;
                    let set_eq = filter_colors.len() == card_ci.len()
                        && filter_colors.iter().all(|c| card_ci.contains(c));
                    if !set_eq {
                        return false;
                    }
                }

                if let Some(filter_colors) = filter.color_identity_within()
                    && !sd.color_identity.iter().all(|c| filter_colors.contains(c))
                {
                    return false;
                }

                // ── combat ────────────────────────────────────────────────────
                if let Some(val) = filter.power_equals() {
                    let parsed = sd.power.as_deref().and_then(|p| p.parse::<i32>().ok());
                    if parsed != Some(val) {
                        return false;
                    }
                }

                if let Some((min, max)) = filter.power_range() {
                    let lo = min.min(max);
                    let hi = min.max(max);
                    match sd.power.as_deref().and_then(|p| p.parse::<i32>().ok()) {
                        Some(p) if p >= lo && p <= hi => {}
                        _ => return false,
                    }
                }

                if let Some(val) = filter.toughness_equals() {
                    let parsed = sd.toughness.as_deref().and_then(|t| t.parse::<i32>().ok());
                    if parsed != Some(val) {
                        return false;
                    }
                }

                if let Some((min, max)) = filter.toughness_range() {
                    let lo = min.min(max);
                    let hi = min.max(max);
                    match sd.toughness.as_deref().and_then(|t| t.parse::<i32>().ok()) {
                        Some(t) if t >= lo && t <= hi => {}
                        _ => return false,
                    }
                }

                // ── metadata ──────────────────────────────────────────────────
                if let Some(rarities) = filter.rarity_equals_any()
                    && !rarities.contains(&sd.rarity)
                {
                    return false;
                }

                if let Some(sets) = filter.set_equals_any()
                    && !sets.iter().any(|s| s == &sd.set)
                {
                    return false;
                }

                if let Some(artists) = filter.artist_equals_any()
                    && !artists
                        .iter()
                        .any(|a| Some(a.as_str()) == sd.artist.as_deref())
                {
                    return false;
                }

                if let Some(lang) = filter.language()
                    && sd.lang != lang
                {
                    return false;
                }

                // ── flags ─────────────────────────────────────────────────────


                if let Some(val) = filter.is_token()
                    && cp.is_token != val
                {
                    return false;
                }

                if let Some(want_playable) = filter.is_playable() {
                    let is_playable = PLAYABLE_LAYOUTS.contains(&sd.layout.as_str());
                    if is_playable != want_playable {
                        return false;
                    }
                }

                if let Some(val) = filter.digital()
                    && sd.digital != val
                {
                    return false;
                }

                if let Some(val) = filter.oversized()
                    && sd.oversized != val
                {
                    return false;
                }

                if let Some(val) = filter.promo()
                    && sd.promo != val
                {
                    return false;
                }

                if let Some(want_warning) = filter.content_warning() {
                    let has_warning = sd.content_warning == Some(true);
                    if has_warning != want_warning {
                        return false;
                    }
                }

                true
            })
            .collect();

        // ── sort ──────────────────────────────────────────────────────────────
        if let Some(order_by) = filter.order_by() {
            if order_by == OrderByOption::Random {
                let mut rng = rand::rng();
                cards.shuffle(&mut rng);
            } else {
                let ascending = filter.ascending();
                cards.sort_by(|a, b| {
                    let sd_a = &a.scryfall_data;
                    let sd_b = &b.scryfall_data;
                    let ord = match order_by {
                        OrderByOption::Name => sd_a.name.cmp(&sd_b.name),
                        OrderByOption::Cmc => {
                            let ca = sd_a.cmc.unwrap_or(f64::MAX);
                            let cb = sd_b.cmc.unwrap_or(f64::MAX);
                            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        OrderByOption::Power => {
                            let pa = sd_a
                                .power
                                .as_deref()
                                .and_then(|p| p.parse::<i32>().ok())
                                .unwrap_or(i32::MAX);
                            let pb = sd_b
                                .power
                                .as_deref()
                                .and_then(|p| p.parse::<i32>().ok())
                                .unwrap_or(i32::MAX);
                            pa.cmp(&pb)
                        }
                        OrderByOption::Toughness => {
                            let ta = sd_a
                                .toughness
                                .as_deref()
                                .and_then(|t| t.parse::<i32>().ok())
                                .unwrap_or(i32::MAX);
                            let tb = sd_b
                                .toughness
                                .as_deref()
                                .and_then(|t| t.parse::<i32>().ok())
                                .unwrap_or(i32::MAX);
                            ta.cmp(&tb)
                        }
                        OrderByOption::Rarity => {
                            sd_a.rarity.to_long_name().cmp(&sd_b.rarity.to_long_name())
                        }
                        OrderByOption::ReleasedAt => sd_a.released_at.cmp(&sd_b.released_at),
                        OrderByOption::PriceUsd => {
                            let pa = sd_a
                                .prices
                                .usd
                                .as_deref()
                                .and_then(|p| p.parse::<f64>().ok())
                                .unwrap_or(f64::MAX);
                            let pb = sd_b
                                .prices
                                .usd
                                .as_deref()
                                .and_then(|p| p.parse::<f64>().ok())
                                .unwrap_or(f64::MAX);
                            pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        OrderByOption::PriceEur => {
                            let pa = sd_a
                                .prices
                                .eur
                                .as_deref()
                                .and_then(|p| p.parse::<f64>().ok())
                                .unwrap_or(f64::MAX);
                            let pb = sd_b
                                .prices
                                .eur
                                .as_deref()
                                .and_then(|p| p.parse::<f64>().ok())
                                .unwrap_or(f64::MAX);
                            pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        OrderByOption::PriceTix => {
                            let pa = sd_a
                                .prices
                                .tix
                                .as_deref()
                                .and_then(|p| p.parse::<f64>().ok())
                                .unwrap_or(f64::MAX);
                            let pb = sd_b
                                .prices
                                .tix
                                .as_deref()
                                .and_then(|p| p.parse::<f64>().ok())
                                .unwrap_or(f64::MAX);
                            pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        OrderByOption::Random => std::cmp::Ordering::Equal,
                    };
                    if ascending { ord } else { ord.reverse() }
                });
            }
        }

        // ── pagination ────────────────────────────────────────────────────────
        let offset = filter.offset() as usize;
        let limit = filter.limit() as usize;
        cards.into_iter().skip(offset).take(limit).collect()
    }
}

/// Extension trait for sorting a `Vec<Card>` in-place using a [`CardFilterBuilder`].
///
/// Unlike [`FilterCards`], which requires a fully-built [`CardFilter`] and returns `Err`
/// when only `order_by` is set, `SortCards` operates on the builder directly so the
/// sort-only case is always safe to apply.
///
/// No-op when `builder.order_by()` is `None`.
pub trait SortCards {
    /// Sorts `self` in-place according to the `order_by` field in `builder`.
    fn sort_by_filter(&mut self, builder: &CardFilterBuilder);
}

impl SortCards for Vec<Card> {
    fn sort_by_filter(&mut self, builder: &CardFilterBuilder) {
        use OrderByOption::*;
        let Some(order_by) = builder.order_by() else { return };

        if order_by == Random {
            self.shuffle(&mut rand::rng());
            return;
        }

        let ascending = builder.ascending();
        self.sort_by(|a, b| {
            let sd_a = &a.scryfall_data;
            let sd_b = &b.scryfall_data;
            let ord = match order_by {
                Name => sd_a.name.cmp(&sd_b.name),
                Cmc => {
                    let ca = sd_a.cmc.unwrap_or(f64::MAX);
                    let cb = sd_b.cmc.unwrap_or(f64::MAX);
                    ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
                }
                Power => {
                    let pa = sd_a.power.as_deref().and_then(|p| p.parse::<i32>().ok()).unwrap_or(i32::MAX);
                    let pb = sd_b.power.as_deref().and_then(|p| p.parse::<i32>().ok()).unwrap_or(i32::MAX);
                    pa.cmp(&pb)
                }
                Toughness => {
                    let ta = sd_a.toughness.as_deref().and_then(|t| t.parse::<i32>().ok()).unwrap_or(i32::MAX);
                    let tb = sd_b.toughness.as_deref().and_then(|t| t.parse::<i32>().ok()).unwrap_or(i32::MAX);
                    ta.cmp(&tb)
                }
                Rarity => sd_a.rarity.to_long_name().cmp(&sd_b.rarity.to_long_name()),
                ReleasedAt => sd_a.released_at.cmp(&sd_b.released_at),
                PriceUsd => {
                    let pa = sd_a.prices.usd.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    let pb = sd_b.prices.usd.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
                }
                PriceEur => {
                    let pa = sd_a.prices.eur.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    let pb = sd_b.prices.eur.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
                }
                PriceTix => {
                    let pa = sd_a.prices.tix.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    let pb = sd_b.prices.tix.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
                }
                Random => std::cmp::Ordering::Equal,
            };
            if ascending { ord } else { ord.reverse() }
        });
    }
}

impl SortCards for Vec<DeckEntry> {
    fn sort_by_filter(&mut self, builder: &CardFilterBuilder) {
        let mut cards: Vec<Card> = self.iter().map(|e| e.card.clone()).collect();
        cards.sort_by_filter(builder);
        let order: Vec<uuid::Uuid> = cards.iter().map(|c| c.scryfall_data.id).collect();
        self.sort_by_key(|e| {
            order
                .iter()
                .position(|id| *id == e.card.scryfall_data.id)
                .unwrap_or(usize::MAX)
        });
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::FilterCards;
    use crate::domain::card::{
        Card,
        card_profile::CardProfile,
        scryfall_data::{
            ScryfallData,
            colors::{Color, Colors},
            legalities::Legalities,
            prices::Prices,
            rarity::{Rarities, Rarity},
        },
        search_card::card_filter::{
            builder::CardFilterBuilder,
            order_by_option::OrderByOption,
        },
    };
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn make_card(name: &str) -> Card {
        Card {
            card_profile: CardProfile {
                scryfall_data_id: Uuid::new_v4(),
                is_token: false,
                mechanical_categories: vec![],
                created_at: NaiveDate::from_ymd_opt(2021, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2021, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
            },
            scryfall_data: ScryfallData {
                arena_id: None,
                id: Uuid::new_v4(),
                lang: "en".to_string(),
                mtgo_id: None,
                mtgo_foil_id: None,
                multiverse_ids: None,
                tcgplayer_id: None,
                tcgplayer_etched_id: None,
                cardmarket_id: None,
                object: "card".to_string(),
                layout: "normal".to_string(),
                oracle_id: None,
                prints_search_uri: String::new(),
                rulings_uri: String::new(),
                scryfall_uri: String::new(),
                uri: String::new(),
                all_parts: None,
                card_faces: None,
                cmc: None,
                color_identity: Colors::from([]),
                color_indicator: None,
                colors: None,
                defense: None,
                edhrec_rank: None,
                game_changer: None,
                hand_modifier: None,
                keywords: None,
                legalities: Legalities::default(),
                life_modifier: None,
                loyalty: None,
                mana_cost: None,
                name: name.to_string(),
                oracle_text: None,
                penny_rank: None,
                power: None,
                produced_mana: None,
                reserved: false,
                toughness: None,
                type_line: None,
                artist: None,
                artist_ids: None,
                attraction_lights: None,
                booster: false,
                border_color: String::new(),
                card_back_id: None,
                collector_number: String::new(),
                content_warning: None,
                digital: false,
                finishes: vec![],
                flavor_name: None,
                flavor_text: None,
                frame_effects: None,
                frame: String::new(),
                full_art: false,
                games: None,
                highres_image: false,
                illustration_id: None,
                image_status: String::new(),
                image_uris: None,
                oversized: false,
                prices: Prices {
                    usd: None,
                    usd_foil: None,
                    usd_etched: None,
                    eur: None,
                    eur_foil: None,
                    eur_etched: None,
                    tix: None,
                },
                printed_name: None,
                printed_text: None,
                printed_type_line: None,
                promo: false,
                promo_types: None,
                purchase_uris: None,
                rarity: Rarity::Common,
                related_uris: serde_json::Value::Null,
                released_at: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                reprint: false,
                scryfall_set_uri: String::new(),
                set_name: String::new(),
                set_search_uri: String::new(),
                set_type: String::new(),
                set_uri: String::new(),
                set: "m21".to_string(),
                set_id: Uuid::new_v4(),
                story_spotlight: false,
                textless: false,
                variation: false,
                variation_of: None,
                security_stamp: None,
                watermark: None,
                preview_previewed_at: None,
                preview_source_uri: None,
                preview_source: None,
            },
        }
    }

    // ── keywords_contains_any ────────────────────────────────────────────────

    #[test]
    fn test_keywords_contains_any_matches_when_one_keyword_present() {
        let mut card = make_card("Serra Angel");
        card.scryfall_data.keywords = Some(vec!["flying".to_string(), "vigilance".to_string()]);
        let filter = CardFilterBuilder::with_keywords_contains_any(["flying", "trample"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Serra Angel");
    }

    #[test]
    fn test_keywords_contains_any_or_logic_either_keyword_matches() {
        let mut angel = make_card("Serra Angel");
        angel.scryfall_data.keywords = Some(vec!["flying".to_string(), "vigilance".to_string()]);
        let mut wurm = make_card("Carnage Wurm");
        wurm.scryfall_data.keywords = Some(vec!["trample".to_string()]);
        let forest = make_card("Forest");
        let filter = CardFilterBuilder::with_keywords_contains_any(["flying", "trample"])
            .build()
            .unwrap();
        let result = vec![angel, wurm, forest].filter_by(&filter);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_keywords_contains_any_excludes_when_no_keyword_matches() {
        let mut card = make_card("Grizzly Bears");
        card.scryfall_data.keywords = Some(vec!["haste".to_string()]);
        let filter = CardFilterBuilder::with_keywords_contains_any(["flying", "trample"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keywords_contains_any_skips_card_with_no_keywords() {
        let card = make_card("Forest"); // keywords = None
        let filter = CardFilterBuilder::with_keywords_contains_any(["flying"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keywords_contains_any_is_case_insensitive() {
        let mut card = make_card("Serra Angel");
        card.scryfall_data.keywords = Some(vec!["Flying".to_string()]);
        let filter = CardFilterBuilder::with_keywords_contains_any(["flying"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert_eq!(result.len(), 1);
    }

    // ── keywords_contains_all ────────────────────────────────────────────────

    #[test]
    fn test_keywords_contains_all_matches_when_all_keywords_present() {
        let mut card = make_card("Serra Angel");
        card.scryfall_data.keywords = Some(vec!["flying".to_string(), "vigilance".to_string()]);
        let filter = CardFilterBuilder::with_keywords_contains_all(["flying", "vigilance"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Serra Angel");
    }

    #[test]
    fn test_keywords_contains_all_excludes_when_only_one_keyword_matches() {
        let mut card = make_card("Serra Angel");
        card.scryfall_data.keywords = Some(vec!["flying".to_string(), "vigilance".to_string()]);
        let filter = CardFilterBuilder::with_keywords_contains_all(["flying", "trample"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keywords_contains_all_excludes_when_no_keywords_match() {
        let mut card = make_card("Grizzly Bears");
        card.scryfall_data.keywords = Some(vec!["haste".to_string()]);
        let filter = CardFilterBuilder::with_keywords_contains_all(["flying", "trample"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keywords_contains_all_skips_card_with_no_keywords() {
        let card = make_card("Forest"); // keywords = None
        let filter = CardFilterBuilder::with_keywords_contains_all(["flying"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keywords_contains_all_is_case_insensitive() {
        let mut card = make_card("Serra Angel");
        card.scryfall_data.keywords = Some(vec!["Flying".to_string(), "Vigilance".to_string()]);
        let filter = CardFilterBuilder::with_keywords_contains_all(["flying", "vigilance"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert_eq!(result.len(), 1);
    }

    // ── produced_mana_contains_any ──────────────────────────────────────────

    #[test]
    fn test_produced_mana_contains_any_matches_when_one_color_present() {
        let mut card = make_card("Sol Ring");
        card.scryfall_data.produced_mana = Some(vec!["C".to_string()]);
        let filter = CardFilterBuilder::with_produced_mana_contains_any(["C", "R"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_produced_mana_contains_any_or_logic() {
        let mut sol_ring = make_card("Sol Ring");
        sol_ring.scryfall_data.produced_mana = Some(vec!["C".to_string()]);
        let mut birds = make_card("Birds of Paradise");
        birds.scryfall_data.produced_mana =
            Some(vec!["W".to_string(), "U".to_string(), "B".to_string(), "R".to_string(), "G".to_string()]);
        let forest = make_card("Forest");
        let filter = CardFilterBuilder::with_produced_mana_contains_any(["R", "G"])
            .build()
            .unwrap();
        let result = vec![sol_ring, birds, forest].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Birds of Paradise");
    }

    #[test]
    fn test_produced_mana_contains_any_excludes_when_no_color_matches() {
        let mut card = make_card("Sol Ring");
        card.scryfall_data.produced_mana = Some(vec!["C".to_string()]);
        let filter = CardFilterBuilder::with_produced_mana_contains_any(["W", "U"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_produced_mana_contains_any_skips_card_with_no_produced_mana() {
        let card = make_card("Lightning Bolt"); // produced_mana = None
        let filter = CardFilterBuilder::with_produced_mana_contains_any(["R"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_produced_mana_contains_any_is_case_insensitive() {
        let mut card = make_card("Arcane Signet");
        card.scryfall_data.produced_mana = Some(vec!["W".to_string(), "U".to_string()]);
        let filter = CardFilterBuilder::with_produced_mana_contains_any(["w"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert_eq!(result.len(), 1);
    }

    // ── produced_mana_contains_all ──────────────────────────────────────────

    #[test]
    fn test_produced_mana_contains_all_matches_when_all_colors_present() {
        let mut card = make_card("Birds of Paradise");
        card.scryfall_data.produced_mana =
            Some(vec!["W".to_string(), "U".to_string(), "B".to_string(), "R".to_string(), "G".to_string()]);
        let filter = CardFilterBuilder::with_produced_mana_contains_all(["W", "U"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_produced_mana_contains_all_excludes_when_only_one_matches() {
        let mut card = make_card("Arcane Signet");
        card.scryfall_data.produced_mana = Some(vec!["W".to_string(), "U".to_string()]);
        let filter = CardFilterBuilder::with_produced_mana_contains_all(["W", "R"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_produced_mana_contains_all_skips_card_with_no_produced_mana() {
        let card = make_card("Lightning Bolt");
        let filter = CardFilterBuilder::with_produced_mana_contains_all(["R"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_produced_mana_contains_all_is_case_insensitive() {
        let mut card = make_card("Birds of Paradise");
        card.scryfall_data.produced_mana =
            Some(vec!["W".to_string(), "U".to_string(), "B".to_string(), "R".to_string(), "G".to_string()]);
        let filter = CardFilterBuilder::with_produced_mana_contains_all(["w", "u"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert_eq!(result.len(), 1);
    }

    // ── text ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_name_contains_case_insensitive() {
        let cards = vec![make_card("Lightning Bolt"), make_card("Forest")];
        let filter = CardFilterBuilder::with_name_contains("lightning").build().unwrap();
        let result = cards.filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Lightning Bolt");
    }

    #[test]
    fn test_name_contains_no_match() {
        let cards = vec![make_card("Forest")];
        let filter = CardFilterBuilder::with_name_contains("bolt").build().unwrap();
        let result = cards.filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains() {
        let mut bolt = make_card("Lightning Bolt");
        bolt.scryfall_data.oracle_text = Some("deal 3 damage to any target".to_string());
        let forest = make_card("Forest");
        let filter = CardFilterBuilder::with_oracle_text_contains("3 damage").build().unwrap();
        let result = vec![bolt, forest].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Lightning Bolt");
    }

    #[test]
    fn test_oracle_text_contains_skips_none() {
        let card = make_card("Forest"); // oracle_text = None
        let filter = CardFilterBuilder::with_oracle_text_contains("damage").build().unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_any_matches_when_one_keyword_present() {
        let mut bolt = make_card("Lightning Bolt");
        bolt.scryfall_data.oracle_text = Some("deals 3 damage to any target".to_string());
        let forest = make_card("Forest");
        let filter = CardFilterBuilder::with_oracle_text_contains_any(["damage", "draw"])
            .build()
            .unwrap();
        let result = vec![bolt, forest].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Lightning Bolt");
    }

    #[test]
    fn test_oracle_text_contains_any_or_logic_either_keyword_matches() {
        let mut bolt = make_card("Lightning Bolt");
        bolt.scryfall_data.oracle_text = Some("deals 3 damage to any target".to_string());
        let mut divination = make_card("Divination");
        divination.scryfall_data.oracle_text = Some("draw two cards".to_string());
        let forest = make_card("Forest");
        let filter = CardFilterBuilder::with_oracle_text_contains_any(["damage", "draw"])
            .build()
            .unwrap();
        let result = vec![bolt, divination, forest].filter_by(&filter);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_oracle_text_contains_any_excludes_when_no_keyword_matches() {
        let mut card = make_card("Island");
        card.scryfall_data.oracle_text = Some("tap: add blue mana".to_string());
        let filter = CardFilterBuilder::with_oracle_text_contains_any(["damage", "draw"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_any_skips_card_with_no_oracle_text() {
        let card = make_card("Forest"); // oracle_text = None
        let filter = CardFilterBuilder::with_oracle_text_contains_any(["flying"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_any_is_case_insensitive() {
        let mut hawk = make_card("Snapping Drake");
        hawk.scryfall_data.oracle_text = Some("Flying".to_string());
        let filter = CardFilterBuilder::with_oracle_text_contains_any(["flying"])
            .build()
            .unwrap();
        let result = vec![hawk].filter_by(&filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_oracle_text_contains_all_matches_when_all_words_present() {
        let mut bolt = make_card("Lightning Bolt");
        bolt.scryfall_data.oracle_text = Some("deals 3 damage to any target".to_string());
        let forest = make_card("Forest");
        let filter = CardFilterBuilder::with_oracle_text_contains_all(["damage", "target"])
            .build()
            .unwrap();
        let result = vec![bolt, forest].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Lightning Bolt");
    }

    #[test]
    fn test_oracle_text_contains_all_excludes_when_only_one_word_matches() {
        let mut bolt = make_card("Lightning Bolt");
        bolt.scryfall_data.oracle_text = Some("deals 3 damage to any target".to_string());
        let filter = CardFilterBuilder::with_oracle_text_contains_all(["damage", "flying"])
            .build()
            .unwrap();
        let result = vec![bolt].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_all_excludes_when_no_words_match() {
        let mut card = make_card("Island");
        card.scryfall_data.oracle_text = Some("tap: add blue mana".to_string());
        let filter = CardFilterBuilder::with_oracle_text_contains_all(["damage", "flying"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_all_skips_card_with_no_oracle_text() {
        let card = make_card("Forest"); // oracle_text = None
        let filter = CardFilterBuilder::with_oracle_text_contains_all(["flying"])
            .build()
            .unwrap();
        let result = vec![card].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_all_is_case_insensitive() {
        let mut hawk = make_card("Snapping Drake");
        hawk.scryfall_data.oracle_text = Some("Flying and Vigilance".to_string());
        let filter = CardFilterBuilder::with_oracle_text_contains_all(["flying", "vigilance"])
            .build()
            .unwrap();
        let result = vec![hawk].filter_by(&filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_has_flavor_text_true() {
        let mut with_flavor = make_card("Mox Pearl");
        with_flavor.scryfall_data.flavor_text = Some("Worth its weight".to_string());
        let without_flavor = make_card("Forest");
        let filter = CardFilterBuilder::with_has_flavor_text(true).build().unwrap();
        let result = vec![with_flavor, without_flavor].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Mox Pearl");
    }

    #[test]
    fn test_has_flavor_text_false() {
        let mut with_flavor = make_card("Mox Pearl");
        with_flavor.scryfall_data.flavor_text = Some("Worth its weight".to_string());
        let without_flavor = make_card("Forest");
        let filter = CardFilterBuilder::with_has_flavor_text(false).build().unwrap();
        let result = vec![with_flavor, without_flavor].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Forest");
    }

    #[test]
    fn test_type_line_contains() {
        let mut dragon = make_card("Shivan Dragon");
        dragon.scryfall_data.type_line = Some("Legendary Creature — Dragon".to_string());
        let forest = make_card("Forest");
        let filter = CardFilterBuilder::with_type_line_contains("Dragon").build().unwrap();
        let result = vec![dragon, forest].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Shivan Dragon");
    }

    // ── mana ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_cmc_equals() {
        let mut bolt = make_card("Lightning Bolt");
        bolt.scryfall_data.cmc = Some(1.0);
        let mut doom = make_card("Doom Blade");
        doom.scryfall_data.cmc = Some(2.0);
        let filter = CardFilterBuilder::with_cmc_equals(1.0).build().unwrap();
        let result = vec![bolt, doom].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Lightning Bolt");
    }

    #[test]
    fn test_cmc_range_inclusive() {
        let mut a = make_card("A");
        a.scryfall_data.cmc = Some(2.0);
        let mut b = make_card("B");
        b.scryfall_data.cmc = Some(3.0);
        let mut c = make_card("C");
        c.scryfall_data.cmc = Some(4.0);
        let filter = CardFilterBuilder::with_cmc_range((2.0, 3.0)).build().unwrap();
        let result = vec![a, b, c].filter_by(&filter);
        assert_eq!(result.len(), 2);
    }

    // ── color identity ─────────────────────────────────────────────────────────

    #[test]
    fn test_color_identity_equals_exact() {
        let mut red = make_card("Bolt");
        red.scryfall_data.color_identity = Colors::from([Color::Red]);
        let mut rg = make_card("Gruul");
        rg.scryfall_data.color_identity = Colors::from([Color::Red, Color::Green]);
        let filter = CardFilterBuilder::with_color_identity_equals([Color::Red]).build().unwrap();
        let result = vec![red, rg].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Bolt");
    }

    #[test]
    fn test_color_identity_within_excludes_superset() {
        let mut mono_red = make_card("Bolt");
        mono_red.scryfall_data.color_identity = Colors::from([Color::Red]);
        let mut rg = make_card("Gruul");
        rg.scryfall_data.color_identity = Colors::from([Color::Red, Color::Green]);
        // within Red only — Gruul has Green so it's excluded
        let filter = CardFilterBuilder::with_color_identity_within([Color::Red]).build().unwrap();
        let result = vec![mono_red, rg].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Bolt");
    }

    // ── combat ────────────────────────────────────────────────────────────────

    #[test]
    fn test_power_equals() {
        let mut a = make_card("3/3");
        a.scryfall_data.power = Some("3".to_string());
        let mut b = make_card("5/5");
        b.scryfall_data.power = Some("5".to_string());
        let filter = CardFilterBuilder::with_power_equals(3).build().unwrap();
        let result = vec![a, b].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "3/3");
    }

    #[test]
    fn test_non_numeric_power_excluded() {
        let mut star = make_card("Tarmogoyf");
        star.scryfall_data.power = Some("*".to_string());
        let filter = CardFilterBuilder::with_power_equals(2).build().unwrap();
        let result = vec![star].filter_by(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_toughness_range() {
        let mut a = make_card("A");
        a.scryfall_data.toughness = Some("1".to_string());
        let mut b = make_card("B");
        b.scryfall_data.toughness = Some("3".to_string());
        let mut c = make_card("C");
        c.scryfall_data.toughness = Some("5".to_string());
        let filter = CardFilterBuilder::with_toughness_range((2, 4)).build().unwrap();
        let result = vec![a, b, c].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "B");
    }

    // ── flags ─────────────────────────────────────────────────────────────────



    #[test]
    fn test_is_playable_filters_token_layout() {
        let mut token_card = make_card("Soldier Token");
        token_card.scryfall_data.layout = "token".to_string();
        let regular = make_card("Forest");
        // Default builder has is_playable=Some(true); "token" is not in PLAYABLE_LAYOUTS
        let filter = CardFilterBuilder::with_name_contains("").build().unwrap();
        let result = vec![token_card, regular].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Forest");
    }

    // ── metadata ──────────────────────────────────────────────────────────────

    #[test]
    fn test_rarity_equals_any_single() {
        let common = make_card("Forest");
        let mut rare = make_card("Shockland");
        rare.scryfall_data.rarity = Rarity::Rare;
        let filter = CardFilterBuilder::with_rarity_equals_any(Rarities::from([Rarity::Common]))
            .build()
            .unwrap();
        let result = vec![common, rare].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Forest");
    }

    #[test]
    fn test_rarity_equals_any_multiple() {
        let common = make_card("Common Card");
        let mut rare = make_card("Rare Card");
        rare.scryfall_data.rarity = Rarity::Rare;
        let mut mythic = make_card("Mythic Card");
        mythic.scryfall_data.rarity = Rarity::Mythic;
        let filter = CardFilterBuilder::with_rarity_equals_any(Rarities::from([
            Rarity::Rare,
            Rarity::Mythic,
        ]))
        .build()
        .unwrap();
        let result = vec![common, rare, mythic].filter_by(&filter);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_set_equals_any() {
        let m21_card = make_card("M21 Card"); // set = "m21" by default
        let mut mh2_card = make_card("MH2 Card");
        mh2_card.scryfall_data.set = "mh2".to_string();
        let filter = CardFilterBuilder::with_set_contains(["mh2"]).build().unwrap();
        let result = vec![m21_card, mh2_card].filter_by(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "MH2 Card");
    }

    // ── sort ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_order_by_name_ascending() {
        let cards = vec![make_card("Zap"), make_card("Aardvark"), make_card("Mox")];
        let mut b = CardFilterBuilder::with_name_contains("");
        b.set_order_by(OrderByOption::Name).set_ascending(true);
        let filter = b.build().unwrap();
        let result = cards.filter_by(&filter);
        assert_eq!(result[0].scryfall_data.name, "Aardvark");
        assert_eq!(result[2].scryfall_data.name, "Zap");
    }

    #[test]
    fn test_order_by_name_descending() {
        let cards = vec![make_card("Zap"), make_card("Aardvark"), make_card("Mox")];
        let mut b = CardFilterBuilder::with_name_contains("");
        b.set_order_by(OrderByOption::Name).set_ascending(false);
        let filter = b.build().unwrap();
        let result = cards.filter_by(&filter);
        assert_eq!(result[0].scryfall_data.name, "Zap");
        assert_eq!(result[2].scryfall_data.name, "Aardvark");
    }

    #[test]
    fn test_order_by_cmc_ascending() {
        let mut a = make_card("A");
        a.scryfall_data.cmc = Some(5.0);
        let mut b = make_card("B");
        b.scryfall_data.cmc = Some(1.0);
        let mut c = make_card("C");
        c.scryfall_data.cmc = Some(3.0);
        let mut builder = CardFilterBuilder::with_name_contains("");
        builder.set_order_by(OrderByOption::Cmc);
        let filter = builder.build().unwrap();
        let result = vec![a, b, c].filter_by(&filter);
        assert_eq!(result[0].scryfall_data.name, "B");
        assert_eq!(result[2].scryfall_data.name, "A");
    }

    // ── pagination ────────────────────────────────────────────────────────────

    #[test]
    fn test_limit_caps_results() {
        let cards = vec![make_card("A"), make_card("B"), make_card("C")];
        let mut b = CardFilterBuilder::with_name_contains("");
        b.set_limit(2);
        let filter = b.build().unwrap();
        let result = cards.filter_by(&filter);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_offset_skips_results() {
        let cards = vec![make_card("A"), make_card("B"), make_card("C")];
        let mut b = CardFilterBuilder::with_name_contains("");
        b.set_order_by(OrderByOption::Name).set_offset(1);
        let filter = b.build().unwrap();
        let result = cards.filter_by(&filter);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].scryfall_data.name, "B");
    }
}
