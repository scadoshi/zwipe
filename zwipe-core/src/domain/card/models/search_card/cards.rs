//! The in-memory card collection.
//!
//! [`Cards`] wraps a `Vec<Card>` already in hand (a loaded deck, a fetched
//! stack) and exposes the two in-memory operations as explicit steps:
//! [`matching`](Cards::matching) (predicate, via
//! [`CardCriteria::matches`]) and [`sorted`](Cards::sorted) (ordering, via
//! [`CardSortKey::compare`]). There is deliberately **no limit/offset here** —
//! pagination is a [`CardQuery`](super::card_filter::CardQuery) concern; the
//! in-memory path cannot express one, by construction.
//!
//! # Example
//!
//! ```rust,ignore
//! let shown: Vec<Card> = Cards::from(deck_cards)
//!     .matching(&criteria)
//!     .sorted(CardSortKey::Cmc, true)
//!     .into();
//! ```

use crate::domain::{
    card::{
        Card,
        search_card::card_filter::{card_sort_key::CardSortKey, criteria::CardCriteria},
    },
    deck::DeckEntry,
};
use rand::seq::SliceRandom;

/// A collection of cards already loaded in memory.
///
/// Operations take bare [`CardCriteria`] — never a query — and read like a
/// slice via `Deref`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cards(Vec<Card>);

impl From<Vec<Card>> for Cards {
    fn from(cards: Vec<Card>) -> Self {
        Self(cards)
    }
}

impl From<Cards> for Vec<Card> {
    fn from(cards: Cards) -> Self {
        cards.0
    }
}

impl std::ops::Deref for Cards {
    type Target = [Card];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Cards {
    type Item = Card;
    type IntoIter = std::vec::IntoIter<Card>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Cards {
    /// Keeps only the cards matching `criteria`.
    pub fn matching(self, criteria: &CardCriteria) -> Cards {
        Cards(
            self.0
                .into_iter()
                .filter(|card| criteria.matches(card))
                .collect(),
        )
    }

    /// Returns the cards ordered by `sort`. `None` is a no-op (pass a
    /// [`CardSortKey`] directly or a builder's `sort()`); `Random` shuffles and
    /// ignores `ascending`.
    pub fn sorted(mut self, sort: impl Into<Option<CardSortKey>>, ascending: bool) -> Cards {
        let Some(sort) = sort.into() else {
            return self;
        };
        if sort == CardSortKey::Random {
            self.0.shuffle(&mut rand::rng());
            return self;
        }
        self.0.sort_by(|a, b| {
            let ord = sort.compare(&a.scryfall_data, &b.scryfall_data);
            if ascending { ord } else { ord.reverse() }
        });
        self
    }

    /// True if any card matches `criteria`. The single-card membership test
    /// (`Cards::from(vec![card]).any_match(...)`) needs no allocation dance —
    /// prefer calling [`CardCriteria::matches`] directly when you hold one card.
    pub fn any_match(&self, criteria: &CardCriteria) -> bool {
        self.0.iter().any(|card| criteria.matches(card))
    }
}

/// Sorts deck entries by their card under `sort`, mirroring [`Cards::sorted`]
/// (`None` no-op, `Random` shuffles, missing values last ascending).
pub fn sort_deck_entries(
    entries: &mut [DeckEntry],
    sort: impl Into<Option<CardSortKey>>,
    ascending: bool,
) {
    let Some(sort) = sort.into() else {
        return;
    };
    if sort == CardSortKey::Random {
        entries.shuffle(&mut rand::rng());
        return;
    }
    entries.sort_by(|a, b| {
        let ord = sort.compare(&a.card.scryfall_data, &b.card.scryfall_data);
        if ascending { ord } else { ord.reverse() }
    });
}
#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::Cards;
    use crate::domain::card::{
        Card,
        card_profile::CardProfile,
        scryfall_data::{
            ScryfallData,
            colors::{Color, Colors},
            legalities::{Legalities, LegalityKind},
            prices::Prices,
            rarity::{Rarities, Rarity},
        },
        search_card::card_filter::{builder::CardQueryBuilder, card_sort_key::CardSortKey},
    };
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn make_card(name: &str) -> Card {
        Card {
            card_profile: CardProfile {
                scryfall_data_id: Uuid::new_v4(),
                is_token: false,
                mechanical_categories: vec![],
                oracle_tags: vec![],
                oracle_tags_by_role: Default::default(),
                other_oracle_tags: vec![],
                created_at: NaiveDate::from_ymd_opt(2021, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc(),
                updated_at: NaiveDate::from_ymd_opt(2021, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc(),
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
                legalities: Legalities {
                    vintage: Some(LegalityKind::Legal),
                    ..Legalities::default()
                },
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
        let filter = CardQueryBuilder::with_keywords_contains_any(["flying", "trample"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
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
        let filter = CardQueryBuilder::with_keywords_contains_any(["flying", "trample"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![angel, wurm, forest]).matching(&filter);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_keywords_contains_any_excludes_when_no_keyword_matches() {
        let mut card = make_card("Grizzly Bears");
        card.scryfall_data.keywords = Some(vec!["haste".to_string()]);
        let filter = CardQueryBuilder::with_keywords_contains_any(["flying", "trample"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keywords_contains_any_skips_card_with_no_keywords() {
        let card = make_card("Forest"); // keywords = None
        let filter = CardQueryBuilder::with_keywords_contains_any(["flying"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keywords_contains_any_is_case_insensitive() {
        let mut card = make_card("Serra Angel");
        card.scryfall_data.keywords = Some(vec!["Flying".to_string()]);
        let filter = CardQueryBuilder::with_keywords_contains_any(["flying"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert_eq!(result.len(), 1);
    }

    // ── keywords_contains_all ────────────────────────────────────────────────

    #[test]
    fn test_keywords_contains_all_matches_when_all_keywords_present() {
        let mut card = make_card("Serra Angel");
        card.scryfall_data.keywords = Some(vec!["flying".to_string(), "vigilance".to_string()]);
        let filter = CardQueryBuilder::with_keywords_contains_all(["flying", "vigilance"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Serra Angel");
    }

    #[test]
    fn test_keywords_contains_all_excludes_when_only_one_keyword_matches() {
        let mut card = make_card("Serra Angel");
        card.scryfall_data.keywords = Some(vec!["flying".to_string(), "vigilance".to_string()]);
        let filter = CardQueryBuilder::with_keywords_contains_all(["flying", "trample"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keywords_contains_all_excludes_when_no_keywords_match() {
        let mut card = make_card("Grizzly Bears");
        card.scryfall_data.keywords = Some(vec!["haste".to_string()]);
        let filter = CardQueryBuilder::with_keywords_contains_all(["flying", "trample"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keywords_contains_all_skips_card_with_no_keywords() {
        let card = make_card("Forest"); // keywords = None
        let filter = CardQueryBuilder::with_keywords_contains_all(["flying"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_keywords_contains_all_is_case_insensitive() {
        let mut card = make_card("Serra Angel");
        card.scryfall_data.keywords = Some(vec!["Flying".to_string(), "Vigilance".to_string()]);
        let filter = CardQueryBuilder::with_keywords_contains_all(["flying", "vigilance"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert_eq!(result.len(), 1);
    }

    // ── produced_mana_contains_any ──────────────────────────────────────────

    #[test]
    fn test_produced_mana_contains_any_matches_when_one_color_present() {
        let mut card = make_card("Sol Ring");
        card.scryfall_data.produced_mana = Some(vec!["C".to_string()]);
        let filter = CardQueryBuilder::with_produced_mana_contains_any(["C", "R"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_produced_mana_contains_any_or_logic() {
        let mut sol_ring = make_card("Sol Ring");
        sol_ring.scryfall_data.produced_mana = Some(vec!["C".to_string()]);
        let mut birds = make_card("Birds of Paradise");
        birds.scryfall_data.produced_mana = Some(vec![
            "W".to_string(),
            "U".to_string(),
            "B".to_string(),
            "R".to_string(),
            "G".to_string(),
        ]);
        let forest = make_card("Forest");
        let filter = CardQueryBuilder::with_produced_mana_contains_any(["R", "G"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![sol_ring, birds, forest]).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Birds of Paradise");
    }

    #[test]
    fn test_produced_mana_contains_any_excludes_when_no_color_matches() {
        let mut card = make_card("Sol Ring");
        card.scryfall_data.produced_mana = Some(vec!["C".to_string()]);
        let filter = CardQueryBuilder::with_produced_mana_contains_any(["W", "U"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_produced_mana_contains_any_skips_card_with_no_produced_mana() {
        let card = make_card("Lightning Bolt"); // produced_mana = None
        let filter = CardQueryBuilder::with_produced_mana_contains_any(["R"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_produced_mana_contains_any_is_case_insensitive() {
        let mut card = make_card("Arcane Signet");
        card.scryfall_data.produced_mana = Some(vec!["W".to_string(), "U".to_string()]);
        let filter = CardQueryBuilder::with_produced_mana_contains_any(["w"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert_eq!(result.len(), 1);
    }

    // ── produced_mana_contains_all ──────────────────────────────────────────

    #[test]
    fn test_produced_mana_contains_all_matches_when_all_colors_present() {
        let mut card = make_card("Birds of Paradise");
        card.scryfall_data.produced_mana = Some(vec![
            "W".to_string(),
            "U".to_string(),
            "B".to_string(),
            "R".to_string(),
            "G".to_string(),
        ]);
        let filter = CardQueryBuilder::with_produced_mana_contains_all(["W", "U"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_produced_mana_contains_all_excludes_when_only_one_matches() {
        let mut card = make_card("Arcane Signet");
        card.scryfall_data.produced_mana = Some(vec!["W".to_string(), "U".to_string()]);
        let filter = CardQueryBuilder::with_produced_mana_contains_all(["W", "R"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_produced_mana_contains_all_skips_card_with_no_produced_mana() {
        let card = make_card("Lightning Bolt");
        let filter = CardQueryBuilder::with_produced_mana_contains_all(["R"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_produced_mana_contains_all_is_case_insensitive() {
        let mut card = make_card("Birds of Paradise");
        card.scryfall_data.produced_mana = Some(vec![
            "W".to_string(),
            "U".to_string(),
            "B".to_string(),
            "R".to_string(),
            "G".to_string(),
        ]);
        let filter = CardQueryBuilder::with_produced_mana_contains_all(["w", "u"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert_eq!(result.len(), 1);
    }

    // ── text ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_name_contains_case_insensitive() {
        let cards = vec![make_card("Lightning Bolt"), make_card("Forest")];
        let filter = CardQueryBuilder::with_name_contains("lightning")
            .build_criteria()
            .unwrap();
        let result = Cards::from(cards).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Lightning Bolt");
    }

    #[test]
    fn test_name_contains_no_match() {
        let cards = vec![make_card("Forest")];
        let filter = CardQueryBuilder::with_name_contains("bolt")
            .build_criteria()
            .unwrap();
        let result = Cards::from(cards).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_name_contains_ignores_punctuation() {
        let cards = vec![make_card("Akroma's Will"), make_card("Forest")];
        let filter = CardQueryBuilder::with_name_contains("akromas will")
            .build_criteria()
            .unwrap();
        let result = Cards::from(cards).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Akroma's Will");
    }

    #[test]
    fn test_name_contains_ignores_commas() {
        let cards = vec![make_card("Satya, Aetherflux Genius"), make_card("Forest")];
        let filter = CardQueryBuilder::with_name_contains("satya aetherflux genius")
            .build_criteria()
            .unwrap();
        let result = Cards::from(cards).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Satya, Aetherflux Genius");
    }

    #[test]
    fn test_name_contains_trims_whitespace() {
        let cards = vec![make_card("Lightning Bolt")];
        let filter = CardQueryBuilder::with_name_contains("  lightning bolt  ")
            .build_criteria()
            .unwrap();
        let result = Cards::from(cards).matching(&filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_oracle_text_contains_ignores_punctuation() {
        let mut card = make_card("Test Card");
        card.scryfall_data.oracle_text =
            Some("Target creature's controller loses 3 life.".to_string());
        let filter = CardQueryBuilder::with_oracle_text_contains("creatures controller")
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_oracle_text_contains() {
        let mut bolt = make_card("Lightning Bolt");
        bolt.scryfall_data.oracle_text = Some("deal 3 damage to any target".to_string());
        let forest = make_card("Forest");
        let filter = CardQueryBuilder::with_oracle_text_contains("3 damage")
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![bolt, forest]).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Lightning Bolt");
    }

    #[test]
    fn test_oracle_text_contains_skips_none() {
        let card = make_card("Forest"); // oracle_text = None
        let filter = CardQueryBuilder::with_oracle_text_contains("damage")
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_any_matches_when_one_keyword_present() {
        let mut bolt = make_card("Lightning Bolt");
        bolt.scryfall_data.oracle_text = Some("deals 3 damage to any target".to_string());
        let forest = make_card("Forest");
        let filter = CardQueryBuilder::with_oracle_text_contains_any(["damage", "draw"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![bolt, forest]).matching(&filter);
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
        let filter = CardQueryBuilder::with_oracle_text_contains_any(["damage", "draw"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![bolt, divination, forest]).matching(&filter);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_oracle_text_contains_any_excludes_when_no_keyword_matches() {
        let mut card = make_card("Island");
        card.scryfall_data.oracle_text = Some("tap: add blue mana".to_string());
        let filter = CardQueryBuilder::with_oracle_text_contains_any(["damage", "draw"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_any_skips_card_with_no_oracle_text() {
        let card = make_card("Forest"); // oracle_text = None
        let filter = CardQueryBuilder::with_oracle_text_contains_any(["flying"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_any_is_case_insensitive() {
        let mut hawk = make_card("Snapping Drake");
        hawk.scryfall_data.oracle_text = Some("Flying".to_string());
        let filter = CardQueryBuilder::with_oracle_text_contains_any(["flying"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![hawk]).matching(&filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_oracle_text_contains_all_matches_when_all_words_present() {
        let mut bolt = make_card("Lightning Bolt");
        bolt.scryfall_data.oracle_text = Some("deals 3 damage to any target".to_string());
        let forest = make_card("Forest");
        let filter = CardQueryBuilder::with_oracle_text_contains_all(["damage", "target"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![bolt, forest]).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Lightning Bolt");
    }

    #[test]
    fn test_oracle_text_contains_all_excludes_when_only_one_word_matches() {
        let mut bolt = make_card("Lightning Bolt");
        bolt.scryfall_data.oracle_text = Some("deals 3 damage to any target".to_string());
        let filter = CardQueryBuilder::with_oracle_text_contains_all(["damage", "flying"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![bolt]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_all_excludes_when_no_words_match() {
        let mut card = make_card("Island");
        card.scryfall_data.oracle_text = Some("tap: add blue mana".to_string());
        let filter = CardQueryBuilder::with_oracle_text_contains_all(["damage", "flying"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_all_skips_card_with_no_oracle_text() {
        let card = make_card("Forest"); // oracle_text = None
        let filter = CardQueryBuilder::with_oracle_text_contains_all(["flying"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![card]).matching(&filter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_oracle_text_contains_all_is_case_insensitive() {
        let mut hawk = make_card("Snapping Drake");
        hawk.scryfall_data.oracle_text = Some("Flying and Vigilance".to_string());
        let filter = CardQueryBuilder::with_oracle_text_contains_all(["flying", "vigilance"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![hawk]).matching(&filter);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_has_flavor_text_true() {
        let mut with_flavor = make_card("Mox Pearl");
        with_flavor.scryfall_data.flavor_text = Some("Worth its weight".to_string());
        let without_flavor = make_card("Forest");
        let filter = CardQueryBuilder::with_has_flavor_text(true)
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![with_flavor, without_flavor]).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Mox Pearl");
    }

    #[test]
    fn test_has_flavor_text_false() {
        let mut with_flavor = make_card("Mox Pearl");
        with_flavor.scryfall_data.flavor_text = Some("Worth its weight".to_string());
        let without_flavor = make_card("Forest");
        let filter = CardQueryBuilder::with_has_flavor_text(false)
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![with_flavor, without_flavor]).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Forest");
    }

    #[test]
    fn test_type_line_contains() {
        let mut dragon = make_card("Shivan Dragon");
        dragon.scryfall_data.type_line = Some("Legendary Creature — Dragon".to_string());
        let forest = make_card("Forest");
        let filter = CardQueryBuilder::with_type_line_contains("Dragon")
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![dragon, forest]).matching(&filter);
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
        let filter = CardQueryBuilder::with_cmc_equals(1.0)
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![bolt, doom]).matching(&filter);
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
        let filter = CardQueryBuilder::with_cmc_range((2.0, 3.0))
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![a, b, c]).matching(&filter);
        assert_eq!(result.len(), 2);
    }

    // ── color identity ─────────────────────────────────────────────────────────

    #[test]
    fn test_color_identity_equals_exact() {
        let mut red = make_card("Bolt");
        red.scryfall_data.color_identity = Colors::from([Color::Red]);
        let mut rg = make_card("Gruul");
        rg.scryfall_data.color_identity = Colors::from([Color::Red, Color::Green]);
        let filter = CardQueryBuilder::with_color_identity_equals([Color::Red])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![red, rg]).matching(&filter);
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
        let filter = CardQueryBuilder::with_color_identity_within([Color::Red])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![mono_red, rg]).matching(&filter);
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
        let filter = CardQueryBuilder::with_power_equals(3)
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![a, b]).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "3/3");
    }

    #[test]
    fn test_non_numeric_power_excluded() {
        let mut star = make_card("Tarmogoyf");
        star.scryfall_data.power = Some("*".to_string());
        let filter = CardQueryBuilder::with_power_equals(2)
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![star]).matching(&filter);
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
        let filter = CardQueryBuilder::with_toughness_range((2, 4))
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![a, b, c]).matching(&filter);
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
        let filter = {
            let mut b = CardQueryBuilder::default();
            b.set_legalities_contains_any(vec!["vintage".to_string()]);
            b
        }
        .build_criteria()
        .unwrap();
        let result = Cards::from(vec![token_card, regular]).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "Forest");
    }

    // ── metadata ──────────────────────────────────────────────────────────────

    #[test]
    fn test_rarity_equals_any_single() {
        let common = make_card("Forest");
        let mut rare = make_card("Shockland");
        rare.scryfall_data.rarity = Rarity::Rare;
        let filter = CardQueryBuilder::with_rarity_equals_any(Rarities::from([Rarity::Common]))
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![common, rare]).matching(&filter);
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
        let filter = CardQueryBuilder::with_rarity_equals_any(Rarities::from([
            Rarity::Rare,
            Rarity::Mythic,
        ]))
        .build_criteria()
        .unwrap();
        let result = Cards::from(vec![common, rare, mythic]).matching(&filter);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_set_equals_any() {
        let mut m21_card = make_card("M21 Card");
        m21_card.scryfall_data.set_name = "m21".to_string();
        let mut mh2_card = make_card("MH2 Card");
        mh2_card.scryfall_data.set_name = "mh2".to_string();
        let filter = CardQueryBuilder::with_set_contains(["mh2"])
            .build_criteria()
            .unwrap();
        let result = Cards::from(vec![m21_card, mh2_card]).matching(&filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scryfall_data.name, "MH2 Card");
    }

    // ── sort ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_sorted_by_name_ascending() {
        let cards = vec![make_card("Zap"), make_card("Aardvark"), make_card("Mox")];
        let result = Cards::from(cards).sorted(CardSortKey::Name, true);
        assert_eq!(result[0].scryfall_data.name, "Aardvark");
        assert_eq!(result[2].scryfall_data.name, "Zap");
    }

    #[test]
    fn test_sorted_by_name_descending() {
        let cards = vec![make_card("Zap"), make_card("Aardvark"), make_card("Mox")];
        let result = Cards::from(cards).sorted(CardSortKey::Name, false);
        assert_eq!(result[0].scryfall_data.name, "Zap");
        assert_eq!(result[2].scryfall_data.name, "Aardvark");
    }

    #[test]
    fn test_sorted_by_cmc_ascending() {
        let mut a = make_card("A");
        a.scryfall_data.cmc = Some(5.0);
        let mut b = make_card("B");
        b.scryfall_data.cmc = Some(1.0);
        let mut c = make_card("C");
        c.scryfall_data.cmc = Some(3.0);
        let result = Cards::from(vec![a, b, c]).sorted(CardSortKey::Cmc, true);
        assert_eq!(result[0].scryfall_data.name, "B");
        assert_eq!(result[2].scryfall_data.name, "A");
    }

    #[test]
    fn test_sorted_none_is_a_noop() {
        let cards = vec![make_card("Zap"), make_card("Aardvark")];
        let result = Cards::from(cards).sorted(None, true);
        assert_eq!(result[0].scryfall_data.name, "Zap");
    }

    #[test]
    fn test_sort_deck_entries_orders_by_card() {
        use crate::domain::deck::{Board, DeckCard, DeckEntry, Quantity};
        use uuid::Uuid;
        let entry = |name: &str| DeckEntry {
            deck_card: DeckCard {
                deck_id: Uuid::new_v4(),
                scryfall_data_id: Uuid::new_v4(),
                oracle_id: Uuid::new_v4(),
                quantity: Quantity::one(),
                board: Board::Deck,
                mvp_at: None,
            },
            card: make_card(name),
        };
        let mut entries = vec![entry("Zap"), entry("Aardvark"), entry("Mox")];
        super::sort_deck_entries(&mut entries, CardSortKey::Name, true);
        assert_eq!(entries[0].card.scryfall_data.name, "Aardvark");
        assert_eq!(entries[2].card.scryfall_data.name, "Zap");
    }
}
