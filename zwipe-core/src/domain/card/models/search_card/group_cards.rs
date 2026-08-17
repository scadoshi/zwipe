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

use crate::domain::card::{Card, card_role::role_label, scryfall_data::colors::Color};
use std::collections::BTreeMap;

/// Grouping strategies for partitioning cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupByOption {
    /// Group by card type (Land, Creature, Planeswalker, etc.).
    CardType,
    /// Group by converted mana cost (0–5, 6+).
    Cmc,
    /// Group by color identity (WUBRG, multicolor, colorless).
    Color,
    /// Group by card role (ramp, draw, removal, etc.).
    /// Cards can appear in multiple groups.
    CardRole,
}

impl GroupByOption {
    /// Returns all grouping options.
    pub fn all() -> Vec<Self> {
        vec![Self::CardType, Self::Cmc, Self::Color, Self::CardRole]
    }
}

impl std::fmt::Display for GroupByOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CardType => write!(f, "Type"),
            Self::Cmc => write!(f, "Mana value"),
            Self::Color => write!(f, "Color"),
            Self::CardRole => write!(f, "Card role"),
        }
    }
}

/// A labelled group of cards.
#[derive(Debug, Clone)]
pub struct CardGroup {
    /// Display label for this group (e.g., "creatures", "3", "blue").
    pub label: String,
    /// Color-identity pips, in WUBRG order, when grouping by
    /// [`GroupByOption::Color`]. The view renders these in place of a color
    /// word, so `label` is empty for colored groups (colorless keeps its
    /// word — there's no pip for "no colors"). `None` for every other
    /// grouping option.
    pub pips: Option<Vec<Color>>,
    /// Cards belonging to this group, in the order they were received.
    pub cards: Vec<Card>,
}

impl CardGroup {
    /// Stable identity for this group, used as the collapse key. Colored
    /// groups carry no label, so they key off their pips instead — without
    /// this every colored group would share the empty string and collapse as
    /// one.
    pub fn key(&self) -> String {
        if !self.label.is_empty() {
            return self.label.clone();
        }
        self.pips
            .as_ref()
            .map(|pips| {
                pips.iter()
                    .map(|c| c.to_short_name())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default()
    }
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
        // Card-role grouping is multi-bucket — a card can appear in multiple groups
        if option == GroupByOption::CardRole {
            return group_by_card_role(self);
        }
        // Color grouping has an open-ended bucket set (one per color combination),
        // so it can't use the fixed-label path below.
        if option == GroupByOption::Color {
            return group_by_color(self);
        }

        let labels: Vec<&str> = match option {
            GroupByOption::CardType => vec![
                "Lands",
                "Creatures",
                "Planeswalkers",
                "Artifacts",
                "Enchantments",
                "Instants",
                "Sorceries",
                "Other",
            ],
            GroupByOption::Cmc => vec!["0", "1", "2", "3", "4", "5", "6+"],
            GroupByOption::Color => unreachable!("color uses group_by_color"),
            GroupByOption::CardRole => unreachable!("card role uses group_by_card_role"),
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
                pips: None,
                cards,
            })
            .collect()
    }
}

/// Groups by exact color identity: one group per distinct combination, so
/// Azorius cards sit together and apart from Jeskai, rather than every
/// multicolored card sharing one "Multicolor" bucket. This mirrors how the
/// deck list groups decks by color.
///
/// Colored groups carry [`CardGroup::pips`] and an empty label — the view
/// renders pips where a color word used to be. Colorless keeps its word and
/// sorts last.
fn group_by_color(cards: Vec<Card>) -> Vec<CardGroup> {
    let mut groups: Vec<CardGroup> = Vec::new();
    for card in cards {
        let pips = identity_pips(&card);
        match groups
            .iter_mut()
            .find(|g| g.pips.as_deref() == Some(pips.as_slice()))
        {
            Some(group) => group.cards.push(card),
            None => {
                let label = if pips.is_empty() {
                    "Colorless".to_string()
                } else {
                    String::new()
                };
                groups.push(CardGroup {
                    label,
                    pips: Some(pips),
                    cards: vec![card],
                });
            }
        }
    }
    // Fewest colors first, each in WUBRG order, colorless last — the same
    // ordering the deck list uses for its color groups.
    groups.sort_by_key(|g| {
        let pips = g.pips.clone().unwrap_or_default();
        let order: Vec<usize> = pips
            .iter()
            .map(|c| Color::all().iter().position(|a| a == c).unwrap_or(0))
            .collect();
        (pips.is_empty(), pips.len(), order)
    });
    groups
}

/// A card's color identity in WUBRG order, so two cards with the same colors
/// always produce the same pip sequence and land in the same group.
fn identity_pips(card: &Card) -> Vec<Color> {
    let identity = &card.scryfall_data.color_identity;
    Color::all()
        .into_iter()
        .filter(|c| identity.contains(c))
        .collect()
}

/// Groups cards by card role. Cards with multiple roles appear in each matching
/// group (cloned); cards with no roles go into "uncategorized". Buckets are keyed
/// by the card's role **slugs** (`card_profile.card_roles`), so a server-added
/// role groups without a client release; labels come from [`role_label`].
fn group_by_card_role(cards: Vec<Card>) -> Vec<CardGroup> {
    // slug -> its cards, ordered by slug (BTreeMap) for a stable display order.
    let mut buckets: BTreeMap<String, Vec<Card>> = BTreeMap::new();
    let mut uncategorized: Vec<Card> = Vec::new();

    for card in cards {
        if card.card_profile.card_roles.is_empty() {
            uncategorized.push(card);
        } else {
            for slug in &card.card_profile.card_roles {
                buckets.entry(slug.clone()).or_default().push(card.clone());
            }
        }
    }

    let mut groups: Vec<CardGroup> = buckets
        .into_iter()
        .map(|(slug, cards)| CardGroup {
            label: role_label(&slug),
            pips: None,
            cards,
        })
        .collect();
    if !uncategorized.is_empty() {
        groups.push(CardGroup {
            label: "uncategorized".to_string(),
            pips: None,
            cards: uncategorized,
        });
    }
    groups
}

/// Returns the bucket index for a card under the given grouping option.
fn classify(card: &Card, option: GroupByOption) -> usize {
    match option {
        GroupByOption::CardType => classify_card_type(card),
        GroupByOption::Cmc => classify_cmc(card),
        GroupByOption::Color => unreachable!("color uses group_by_color"),
        GroupByOption::CardRole => unreachable!("card role uses group_by_card_role"),
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
                card_roles: vec![],
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
        assert_eq!(result[0].label, "Creatures");
    }

    #[test]
    fn test_group_by_type_land() {
        let mut card = make_card("Forest");
        card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "Lands");
    }

    #[test]
    fn test_group_by_type_instant() {
        let mut card = make_card("Lightning Bolt");
        card.scryfall_data.type_line = Some("Instant".to_string());
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "Instants");
    }

    #[test]
    fn test_group_by_type_other() {
        let card = make_card("Mystery"); // type_line = None
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "Other");
    }

    #[test]
    fn test_group_by_type_first_match_wins() {
        // "Artifact Creature" contains both keywords; Creature (index 1) wins over Artifact (index 3)
        let mut card = make_card("Phyrexian Juggernaut");
        card.scryfall_data.type_line = Some("Artifact Creature — Juggernaut".to_string());
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "Creatures");
    }

    #[test]
    fn test_group_by_type_empty_groups_skipped() {
        let mut card = make_card("Forest");
        card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
        let result = vec![card].group_by(GroupByOption::CardType);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, "Lands");
        assert!(result.iter().all(|g| g.label != "Creatures"));
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
        // Colored groups render as pips, so the label stays empty.
        assert_eq!(result[0].label, "");
        assert_eq!(result[0].pips.as_deref(), Some([Color::White].as_slice()));
    }

    /// Each color combination is its own group — the old behavior funnelled
    /// every multicolored card into one "Multicolor" bucket.
    #[test]
    fn test_group_by_color_splits_combinations() {
        let mut azorius = make_card("Azorius Card");
        azorius.scryfall_data.color_identity = Colors::from([Color::White, Color::Blue]);
        let mut azorius_two = make_card("Another Azorius Card");
        azorius_two.scryfall_data.color_identity = Colors::from([Color::White, Color::Blue]);
        let mut rakdos = make_card("Rakdos Card");
        rakdos.scryfall_data.color_identity = Colors::from([Color::Black, Color::Red]);

        let result = vec![azorius, rakdos, azorius_two].group_by(GroupByOption::Color);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].pips.as_deref(),
            Some([Color::White, Color::Blue].as_slice())
        );
        assert_eq!(result[0].cards.len(), 2);
        assert_eq!(
            result[1].pips.as_deref(),
            Some([Color::Black, Color::Red].as_slice())
        );
        assert_eq!(result[1].cards.len(), 1);
    }

    /// Color identity normalizes to WUBRG order, so the same colors group
    /// together however the source listed them.
    #[test]
    fn test_group_by_color_normalizes_pip_order() {
        let mut a = make_card("A");
        a.scryfall_data.color_identity = Colors::from([Color::Red, Color::White]);
        let mut b = make_card("B");
        b.scryfall_data.color_identity = Colors::from([Color::White, Color::Red]);
        let result = vec![a, b].group_by(GroupByOption::Color);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].pips.as_deref(),
            Some([Color::White, Color::Red].as_slice())
        );
    }

    #[test]
    fn test_group_by_color_colorless() {
        let card = make_card("Eldrazi"); // color_identity = empty by default
        let result = vec![card].group_by(GroupByOption::Color);
        assert_eq!(result[0].label, "Colorless");
        assert!(result[0].pips.as_deref().is_some_and(<[Color]>::is_empty));
    }

    /// Fewest colors first in WUBRG order, colorless last.
    #[test]
    fn test_group_by_color_group_order() {
        let mut white = make_card("White Card");
        white.scryfall_data.color_identity = Colors::from([Color::White]);
        let mut red = make_card("Red Card");
        red.scryfall_data.color_identity = Colors::from([Color::Red]);
        let mut boros = make_card("Boros Card");
        boros.scryfall_data.color_identity = Colors::from([Color::White, Color::Red]);
        let colorless = make_card("Colorless Card"); // empty color_identity

        let result = vec![colorless, boros, red, white].group_by(GroupByOption::Color);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].pips.as_deref(), Some([Color::White].as_slice()));
        assert_eq!(result[1].pips.as_deref(), Some([Color::Red].as_slice()));
        assert_eq!(
            result[2].pips.as_deref(),
            Some([Color::White, Color::Red].as_slice())
        );
        assert_eq!(result[3].label, "Colorless");
    }

    /// Collapse keys must differ per group; colored groups have no label to
    /// key off, so they fall back to their pips.
    #[test]
    fn test_color_group_keys_are_distinct() {
        let mut white = make_card("White Card");
        white.scryfall_data.color_identity = Colors::from([Color::White]);
        let mut azorius = make_card("Azorius Card");
        azorius.scryfall_data.color_identity = Colors::from([Color::White, Color::Blue]);
        let colorless = make_card("Colorless Card");

        let result = vec![white, azorius, colorless].group_by(GroupByOption::Color);
        let keys: Vec<String> = result.iter().map(super::CardGroup::key).collect();
        assert_eq!(keys, vec!["W", "WU", "Colorless"]);
    }
}
