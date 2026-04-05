//! In-memory card grouping for local `Vec<Card>` slices.
//!
//! Partitions a `Vec<Card>` into labelled groups based on card type, mana value,
//! or color identity. Works alongside `filter_cards.rs` — the caller is expected
//! to `filter_by` first (which handles sorting), then `group_by`.
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::card::models::search_card::group_cards::{GroupCards, GroupByOption};
//!
//! let groups = deck_cards.group_by(GroupByOption::CardType);
//! for group in &groups {
//!     println!("{} · {}", group.label, group.cards.len());
//! }
//! ```

use crate::domain::card::{Card, scryfall_data::colors::Color};

/// Grouping strategies for partitioning cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupByOption {
    /// Group by card type (Land, Creature, Planeswalker, etc.).
    CardType,
    /// Group by converted mana cost (0–5, 6+).
    Cmc,
    /// Group by color identity (WUBRG, multicolor, colorless).
    Color,
}

impl GroupByOption {
    /// Returns all grouping options.
    pub fn all() -> Vec<Self> {
        vec![Self::CardType, Self::Cmc, Self::Color]
    }
}

impl std::fmt::Display for GroupByOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CardType => write!(f, "type"),
            Self::Cmc => write!(f, "cmc"),
            Self::Color => write!(f, "color"),
        }
    }
}

/// A labelled group of cards.
#[derive(Debug, Clone)]
pub struct CardGroup {
    /// Display label for this group (e.g., "creatures", "3", "blue").
    pub label: String,
    /// Cards belonging to this group, in the order they were received.
    pub cards: Vec<Card>,
}

/// Extension trait for grouping a `Vec<Card>` into labelled buckets.
pub trait GroupCards {
    /// Partitions cards into groups according to the given option.
    ///
    /// Groups are emitted in a fixed order (not alphabetical). Empty groups are
    /// skipped. Card order within each group is preserved from the input.
    fn group_by(self, option: GroupByOption) -> Vec<CardGroup>;
}

impl GroupCards for Vec<Card> {
    fn group_by(self, option: GroupByOption) -> Vec<CardGroup> {
        let labels: Vec<&str> = match option {
            GroupByOption::CardType => vec![
                "lands",
                "creatures",
                "planeswalkers",
                "artifacts",
                "enchantments",
                "instants",
                "sorceries",
                "other",
            ],
            GroupByOption::Cmc => vec!["0", "1", "2", "3", "4", "5", "6+"],
            GroupByOption::Color => vec![
                "white",
                "blue",
                "black",
                "red",
                "green",
                "multicolor",
                "colorless",
            ],
        };
        let mut buckets: Vec<Vec<Card>> = vec![Vec::new(); labels.len()];
        self.into_iter().for_each(|card| {
            if let Some((_, bucket)) = buckets
                .iter_mut()
                .enumerate()
                .find(|(i, _)| *i == classify(&card, option))
            {
                bucket.push(card);
            }
        });
        labels
            .into_iter()
            .zip(buckets)
            .filter(|(_, cards)| !cards.is_empty())
            .map(|(label, cards)| CardGroup {
                label: label.to_string(),
                cards,
            })
            .collect()
    }
}

/// Returns the bucket index for a card under the given grouping option.
fn classify(card: &Card, option: GroupByOption) -> usize {
    match option {
        GroupByOption::CardType => classify_card_type(card),
        GroupByOption::Cmc => classify_cmc(card),
        GroupByOption::Color => classify_color(card),
    }
}

/// Card type classification — first match wins.
///
/// Priority: Land → Creature → Planeswalker → Artifact → Enchantment →
/// Instant → Sorcery → Other.
fn classify_card_type(card: &Card) -> usize {
    let type_line = match &card.scryfall_data.type_line {
        Some(tl) => tl.as_str(),
        None => return 7, // "other"
    };

    const CHECKS: &[&str] = &[
        "Land",
        "Creature",
        "Planeswalker",
        "Artifact",
        "Enchantment",
        "Instant",
        "Sorcery",
    ];

    CHECKS
        .iter()
        .position(|keyword| type_line.contains(keyword))
        .unwrap_or(7) // "other"
}

/// CMC classification — floor to integer, cap at 6.
fn classify_cmc(card: &Card) -> usize {
    let cmc = card.scryfall_data.cmc.unwrap_or(0.0);
    let floored = cmc.floor() as usize;
    floored.min(6) // indices 0–5 for "0"–"5", index 6 for "6+"
}

/// Color identity classification — uses WUBRG order + multicolor + colorless.
fn classify_color(card: &Card) -> usize {
    let ci = &card.scryfall_data.color_identity;
    if ci.is_empty() {
        return 6; // "colorless"
    }
    if ci.len() >= 2 {
        return 5; // "multicolor"
    }
    let Some(color) = ci.first() else {
        return 6; // "colorless
    };
    // Exactly one color — map to WUBRG index (0–4)
    match color {
        Color::White => 0,
        Color::Blue => 1,
        Color::Black => 2,
        Color::Red => 3,
        Color::Green => 4,
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::{GroupByOption, GroupCards};
    use crate::domain::card::{
        Card,
        card_profile::CardProfile,
        scryfall_data::{
            ScryfallData,
            colors::{Color, Colors},
            legalities::Legalities,
            prices::Prices,
            rarity::Rarity,
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

    // ── GroupByOption::CardType ────────────────────────────────────────────────

    #[test]
    fn test_group_by_type_creature() {
        let mut card = make_card("Grizzly Bears");
        card.scryfall_data.type_line = Some("Creature — Bear".to_string());
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "creatures");
    }

    #[test]
    fn test_group_by_type_land() {
        let mut card = make_card("Forest");
        card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "lands");
    }

    #[test]
    fn test_group_by_type_instant() {
        let mut card = make_card("Lightning Bolt");
        card.scryfall_data.type_line = Some("Instant".to_string());
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "instants");
    }

    #[test]
    fn test_group_by_type_other() {
        let card = make_card("Mystery"); // type_line = None
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "other");
    }

    #[test]
    fn test_group_by_type_first_match_wins() {
        // "Artifact Creature" contains both keywords; Creature (index 1) wins over Artifact (index 3)
        let mut card = make_card("Phyrexian Juggernaut");
        card.scryfall_data.type_line = Some("Artifact Creature — Juggernaut".to_string());
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "creatures");
    }

    #[test]
    fn test_group_by_type_empty_groups_skipped() {
        let mut card = make_card("Forest");
        card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "lands");
        assert!(result.iter().all(|g| g.label != "creatures"));
    }

    // ── GroupByOption::Cmc ─────────────────────────────────────────────────────

    #[test]
    fn test_group_by_cmc_zero() {
        let mut card = make_card("Mox Pearl");
        card.scryfall_data.cmc = Some(0.0);
        let result = vec![card].group_by(GroupByOption::Cmc);
        assert_eq!(result[0].label, "0");
    }

    #[test]
    fn test_group_by_cmc_five() {
        let mut card = make_card("Mulldrifter");
        card.scryfall_data.cmc = Some(5.0);
        let result = vec![card].group_by(GroupByOption::Cmc);
        assert_eq!(result[0].label, "5");
    }

    #[test]
    fn test_group_by_cmc_six_plus() {
        let mut card = make_card("Emrakul");
        card.scryfall_data.cmc = Some(15.0);
        let result = vec![card].group_by(GroupByOption::Cmc);
        assert_eq!(result[0].label, "6+");
    }

    #[test]
    fn test_group_by_cmc_none_treated_as_zero() {
        let card = make_card("Ancestral Vision"); // cmc = None
        let result = vec![card].group_by(GroupByOption::Cmc);
        assert_eq!(result[0].label, "0");
    }

    #[test]
    fn test_group_by_cmc_boundary_exactly_six() {
        let mut card = make_card("Wurmcoil Engine");
        card.scryfall_data.cmc = Some(6.0);
        let result = vec![card].group_by(GroupByOption::Cmc);
        assert_eq!(result[0].label, "6+");
    }

    // ── GroupByOption::Color ───────────────────────────────────────────────────

    #[test]
    fn test_group_by_color_mono_white() {
        let mut card = make_card("Plains");
        card.scryfall_data.color_identity = Colors::from([Color::White]);
        let result = vec![card].group_by(GroupByOption::Color);
        assert_eq!(result[0].label, "white");
    }

    #[test]
    fn test_group_by_color_multicolor() {
        let mut card = make_card("Atraxa");
        card.scryfall_data.color_identity =
            Colors::from([Color::White, Color::Blue, Color::Black, Color::Green]);
        let result = vec![card].group_by(GroupByOption::Color);
        assert_eq!(result[0].label, "multicolor");
    }

    #[test]
    fn test_group_by_color_colorless() {
        let card = make_card("Eldrazi"); // color_identity = empty by default
        let result = vec![card].group_by(GroupByOption::Color);
        assert_eq!(result[0].label, "colorless");
    }

    #[test]
    fn test_group_by_color_group_order() {
        let mut white = make_card("White Card");
        white.scryfall_data.color_identity = Colors::from([Color::White]);
        let mut red = make_card("Red Card");
        red.scryfall_data.color_identity = Colors::from([Color::Red]);
        let colorless = make_card("Colorless Card"); // empty color_identity
        // Groups emitted in fixed order: white(0), red(3), colorless(6)
        let result = vec![colorless, red, white].group_by(GroupByOption::Color);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].label, "white");
        assert_eq!(result[1].label, "red");
        assert_eq!(result[2].label, "colorless");
    }
}
