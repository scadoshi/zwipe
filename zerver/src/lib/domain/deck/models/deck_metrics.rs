//! Deck-level aggregate metrics computed from a collection of cards.
//!
//! Single-pass computation producing mana curve histogram, type distribution,
//! color distribution, and summary stats. Supports both `Vec<Card>` (each card
//! counted once) and `Vec<DeckEntry>` (each card counted by its quantity).

use crate::domain::{
    card::models::{scryfall_data::colors::Color, Card},
    deck::models::deck::DeckEntry,
};

/// Aggregate statistics for a collection of cards.
#[derive(Debug, Clone, PartialEq)]
pub struct DeckMetrics {
    /// Total unique card count.
    pub total_cards: usize,
    /// Average CMC of nonland cards (0.0 if all lands or empty).
    pub avg_cmc: f64,
    /// Number of land cards.
    pub land_count: usize,
    /// Number of nonland cards.
    pub nonland_count: usize,
    /// CMC histogram buckets: 0, 1, 2, 3, 4, 5, 6+.
    pub cmc_histogram: [usize; 7],
    /// Non-empty type counts in fixed order.
    pub type_counts: Vec<(&'static str, usize)>,
    /// Non-empty color counts in fixed order.
    pub color_counts: Vec<(&'static str, usize)>,
    /// Per-color (WUBRG) mana balance: (consumed_pips, produced_pips).
    /// Index 0=White 1=Blue 2=Black 3=Red 4=Green.
    pub mana_balance: [(usize, usize); 5],
}


impl DeckMetrics {
    /// Computes metrics from deck entries, counting each card by its quantity.
    pub fn from_entries(entries: &[DeckEntry]) -> Self {
        let mut cmc_histogram = [0usize; 7];
        let mut type_buckets = [0usize; 8];
        let mut color_buckets = [0usize; 7];
        let mut land_count = 0usize;
        let mut cmc_sum = 0.0f64;
        let mut total_cards = 0usize;
        let mut pip_consumed = [0usize; 5];
        let mut pip_produced = [0usize; 5];

        for entry in entries {
            let qty = (*entry.deck_card.quantity).max(1) as usize;
            let card = &entry.card;
            total_cards += qty;

            let type_idx = classify_type(card);
            if let Some(slot) = type_buckets.get_mut(type_idx) {
                *slot += qty;
            }
            let is_land = type_idx == 0;
            if is_land {
                land_count += qty;
            }

            if !is_land {
                let cmc_idx = classify_cmc(card);
                if let Some(slot) = cmc_histogram.get_mut(cmc_idx) {
                    *slot += qty;
                }
                cmc_sum += card.scryfall_data.cmc.unwrap_or(0.0) * qty as f64;
            }

            let color_idx = classify_color(card);
            if let Some(slot) = color_buckets.get_mut(color_idx) {
                *slot += qty;
            }

            // Pip consumed: parse mana_cost for colored symbols
            if let Some(mc) = &card.scryfall_data.mana_cost {
                let pips = count_color_pips(mc);
                for (slot, &pip) in pip_consumed.iter_mut().zip(pips.iter()) {
                    *slot += pip * qty;
                }
            }

            // Pip produced: walk produced_mana (covers lands, rocks, dorks — all mana producers)
            if let Some(produced) = &card.scryfall_data.produced_mana {
                for s in produced {
                    if let Some(idx) = produced_color_index(s)
                        && let Some(slot) = pip_produced.get_mut(idx)
                    {
                        *slot += qty;
                    }
                }
            }
        }

        let mana_balance: [(usize, usize); 5] = std::array::from_fn(|i| {
            (
                pip_consumed.get(i).copied().unwrap_or(0),
                pip_produced.get(i).copied().unwrap_or(0),
            )
        });
        let nonland_count = total_cards - land_count;
        let avg_cmc = if nonland_count > 0 {
            cmc_sum / nonland_count as f64
        } else {
            0.0
        };

        const TYPE_LABELS: [&str; 8] = [
            "lands",
            "creatures",
            "planeswalkers",
            "artifacts",
            "enchantments",
            "instants",
            "sorceries",
            "other",
        ];
        let type_counts: Vec<(&'static str, usize)> = TYPE_LABELS
            .iter()
            .zip(type_buckets.iter())
            .filter(|&(_, count)| *count > 0)
            .map(|(&label, &count)| (label, count))
            .collect();

        const COLOR_LABELS: [&str; 7] = [
            "white",
            "blue",
            "black",
            "red",
            "green",
            "multicolor",
            "colorless",
        ];
        let color_counts: Vec<(&'static str, usize)> = COLOR_LABELS
            .iter()
            .zip(color_buckets.iter())
            .filter(|&(_, count)| *count > 0)
            .map(|(&label, &count)| (label, count))
            .collect();

        DeckMetrics {
            total_cards,
            avg_cmc,
            land_count,
            nonland_count,
            cmc_histogram,
            type_counts,
            color_counts,
            mana_balance,
        }
    }
}

/// Card type classification — first match wins (matches group_cards.rs logic).
fn classify_type(card: &Card) -> usize {
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
        .unwrap_or(7)
}

/// CMC classification — floor to integer, cap at 6.
fn classify_cmc(card: &Card) -> usize {
    let cmc = card.scryfall_data.cmc.unwrap_or(0.0);
    (cmc.floor() as usize).min(6)
}

/// Count colored pip symbols in a Scryfall mana_cost string (e.g. "{2}{R}{R}").
/// Only counts pure single-color symbols — skips hybrid ({W/U}), phyrexian ({W/P}), etc.
fn count_color_pips(mana_cost: &str) -> [usize; 5] {
    let mut counts = [0usize; 5];
    for token in mana_cost.split('{').skip(1) {
        let sym = token.trim_end_matches('}');
        match sym {
            "W" => counts[0] += 1,
            "U" => counts[1] += 1,
            "B" => counts[2] += 1,
            "R" => counts[3] += 1,
            "G" => counts[4] += 1,
            _ => {} // {2}, {X}, {C}, {W/U}, {W/P} — ignored
        }
    }
    counts
}

/// Map a produced_mana string ("W","U","B","R","G") to WUBRG index.
/// Returns None for "C" (colorless), "S" (snow), or unknown values.
fn produced_color_index(s: &str) -> Option<usize> {
    match s {
        "W" => Some(0),
        "U" => Some(1),
        "B" => Some(2),
        "R" => Some(3),
        "G" => Some(4),
        _ => None,
    }
}

/// Color identity classification — WUBRG order + multicolor + colorless.
fn classify_color(card: &Card) -> usize {
    let ci = &card.scryfall_data.color_identity;
    if ci.is_empty() {
        return 6; // colorless
    }
    if ci.len() >= 2 {
        return 5; // multicolor
    }
    match ci.first() {
        Some(Color::White) => 0,
        Some(Color::Blue) => 1,
        Some(Color::Black) => 2,
        Some(Color::Red) => 3,
        Some(Color::Green) => 4,
        None => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        card::models::{
            card_profile::CardProfile,
            scryfall_data::{
                colors::{Color, Colors},
                legalities::Legalities,
                prices::Prices,
                rarity::Rarity,
                ScryfallData,
            },
        },
        deck::models::deck_card::{quantity::Quantity, DeckCard},
    };
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn make_entry(name: &str, qty: i32) -> DeckEntry {
        let deck_id = Uuid::new_v4();
        let card = make_card(name);
        let scryfall_data_id = card.scryfall_data.id;
        DeckEntry {
            card,
            deck_card: DeckCard {
                deck_id,
                scryfall_data_id,
                quantity: Quantity::new(qty).unwrap(),
            },
        }
    }

    fn make_card(name: &str) -> Card {
        Card {
            card_profile: CardProfile {
                scryfall_data_id: Uuid::new_v4(),
                is_valid_commander: false,
                is_token: false,
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

    #[test]
    fn empty_deck() {
        let metrics = DeckMetrics::from_entries(&[]);
        assert_eq!(metrics.total_cards, 0);
        assert_eq!(metrics.avg_cmc, 0.0);
        assert_eq!(metrics.land_count, 0);
        assert_eq!(metrics.nonland_count, 0);
        assert_eq!(metrics.cmc_histogram, [0; 7]);
        assert!(metrics.type_counts.is_empty());
        assert!(metrics.color_counts.is_empty());
    }

    #[test]
    fn single_land() {
        let mut entry = make_entry("Forest", 1);
        entry.card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
        entry.card.scryfall_data.cmc = Some(0.0);
        entry.card.scryfall_data.color_identity = Colors::from([Color::Green]);

        let metrics = DeckMetrics::from_entries(&[entry]);
        assert_eq!(metrics.total_cards, 1);
        assert_eq!(metrics.land_count, 1);
        assert_eq!(metrics.nonland_count, 0);
        assert_eq!(metrics.avg_cmc, 0.0);
        assert_eq!(metrics.cmc_histogram, [0; 7]); // lands excluded from mana curve
        assert_eq!(metrics.type_counts, vec![("lands", 1)]);
        assert_eq!(metrics.color_counts, vec![("green", 1)]);
    }

    #[test]
    fn mixed_deck() {
        let mut land = make_entry("Forest", 1);
        land.card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
        land.card.scryfall_data.cmc = Some(0.0);
        land.card.scryfall_data.color_identity = Colors::from([Color::Green]);

        let mut creature = make_entry("Llanowar Elves", 1);
        creature.card.scryfall_data.type_line = Some("Creature — Elf Druid".to_string());
        creature.card.scryfall_data.cmc = Some(1.0);
        creature.card.scryfall_data.color_identity = Colors::from([Color::Green]);

        let mut instant = make_entry("Counterspell", 1);
        instant.card.scryfall_data.type_line = Some("Instant".to_string());
        instant.card.scryfall_data.cmc = Some(2.0);
        instant.card.scryfall_data.color_identity = Colors::from([Color::Blue]);

        let mut artifact = make_entry("Sol Ring", 1);
        artifact.card.scryfall_data.type_line = Some("Artifact".to_string());
        artifact.card.scryfall_data.cmc = Some(1.0);

        let metrics = DeckMetrics::from_entries(&[land, creature, instant, artifact]);

        assert_eq!(metrics.total_cards, 4);
        assert_eq!(metrics.land_count, 1);
        assert_eq!(metrics.nonland_count, 3);
        assert!((metrics.avg_cmc - 4.0 / 3.0).abs() < f64::EPSILON);
        assert_eq!(metrics.cmc_histogram, [0, 2, 1, 0, 0, 0, 0]); // land excluded
        assert_eq!(
            metrics.type_counts,
            vec![("lands", 1), ("creatures", 1), ("artifacts", 1), ("instants", 1)]
        );
        assert_eq!(
            metrics.color_counts,
            vec![("blue", 1), ("green", 2), ("colorless", 1)]
        );
    }

    #[test]
    fn no_type_line_is_other() {
        let entry = make_entry("Mystery", 1);
        let metrics = DeckMetrics::from_entries(&[entry]);
        assert_eq!(metrics.type_counts, vec![("other", 1)]);
    }

    #[test]
    fn no_cmc_is_zero() {
        let entry = make_entry("Ancestral Vision", 1);
        let metrics = DeckMetrics::from_entries(&[entry]);
        assert_eq!(metrics.cmc_histogram[0], 1);
    }

    #[test]
    fn cmc_capped_at_six() {
        let mut entry = make_entry("Emrakul", 1);
        entry.card.scryfall_data.cmc = Some(15.0);
        entry.card.scryfall_data.type_line = Some("Creature — Eldrazi".to_string());

        let metrics = DeckMetrics::from_entries(&[entry]);
        assert_eq!(metrics.cmc_histogram[6], 1);
        assert_eq!(metrics.cmc_histogram[0..6], [0; 6]);
    }

    #[test]
    fn multicolor_classification() {
        let mut entry = make_entry("Atraxa", 1);
        entry.card.scryfall_data.color_identity =
            Colors::from([Color::White, Color::Blue, Color::Black, Color::Green]);
        entry.card.scryfall_data.type_line =
            Some("Creature — Phyrexian Angel Horror".to_string());

        let metrics = DeckMetrics::from_entries(&[entry]);
        assert_eq!(metrics.color_counts, vec![("multicolor", 1)]);
    }

    #[test]
    fn quantity_multiplies_counts() {
        let mut bolt = make_entry("Lightning Bolt", 4);
        bolt.card.scryfall_data.type_line = Some("Instant".to_string());
        bolt.card.scryfall_data.cmc = Some(1.0);
        bolt.card.scryfall_data.color_identity = Colors::from([Color::Red]);

        let mut forest = make_entry("Forest", 10);
        forest.card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
        forest.card.scryfall_data.cmc = Some(0.0);
        forest.card.scryfall_data.color_identity = Colors::from([Color::Green]);

        let metrics = DeckMetrics::from_entries(&[bolt, forest]);

        assert_eq!(metrics.total_cards, 14);
        assert_eq!(metrics.land_count, 10);
        assert_eq!(metrics.nonland_count, 4);
        // avg_cmc = (1.0 * 4) / 4 = 1.0
        assert!((metrics.avg_cmc - 1.0).abs() < f64::EPSILON);
        assert_eq!(metrics.cmc_histogram, [0, 4, 0, 0, 0, 0, 0]); // lands excluded
        assert_eq!(
            metrics.type_counts,
            vec![("lands", 10), ("instants", 4)]
        );
        assert_eq!(
            metrics.color_counts,
            vec![("red", 4), ("green", 10)]
        );
    }

    #[test]
    fn pip_consumed_counts_pips() {
        let mut entry = make_entry("Lightning Bolt", 1);
        entry.card.scryfall_data.mana_cost = Some("{2}{R}{R}".to_string());
        entry.card.scryfall_data.type_line = Some("Instant".to_string());

        let metrics = DeckMetrics::from_entries(&[entry]);
        // R=2, all others 0
        assert_eq!(metrics.mana_balance[3].0, 2); // R consumed
        assert_eq!(metrics.mana_balance[0].0, 0); // W consumed
    }

    #[test]
    fn pip_consumed_skips_hybrid() {
        let mut entry = make_entry("Hybrid Card", 1);
        entry.card.scryfall_data.mana_cost = Some("{W/U}".to_string());
        entry.card.scryfall_data.type_line = Some("Instant".to_string());

        let metrics = DeckMetrics::from_entries(&[entry]);
        assert_eq!(metrics.mana_balance, [(0, 0); 5]);
    }

    #[test]
    fn pip_produced_from_land() {
        let mut entry = make_entry("Forest", 1);
        entry.card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
        entry.card.scryfall_data.produced_mana = Some(vec!["G".to_string()]);

        let metrics = DeckMetrics::from_entries(&[entry]);
        assert_eq!(metrics.mana_balance[4].1, 1); // G produced
    }

    #[test]
    fn pip_produced_quantity_aware() {
        let mut entry = make_entry("Forest", 4);
        entry.card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
        entry.card.scryfall_data.produced_mana = Some(vec!["G".to_string()]);

        let metrics = DeckMetrics::from_entries(&[entry]);
        assert_eq!(metrics.mana_balance[4].1, 4); // G produced × 4
    }

    #[test]
    fn pip_surplus() {
        let mut land = make_entry("Forest", 3);
        land.card.scryfall_data.type_line = Some("Basic Land — Forest".to_string());
        land.card.scryfall_data.produced_mana = Some(vec!["G".to_string()]);

        let mut spell = make_entry("Giant Growth", 1);
        spell.card.scryfall_data.mana_cost = Some("{G}".to_string());
        spell.card.scryfall_data.type_line = Some("Instant".to_string());

        let metrics = DeckMetrics::from_entries(&[land, spell]);
        let (consumed, produced) = metrics.mana_balance[4]; // G
        assert_eq!(consumed, 1);
        assert_eq!(produced, 3);
        assert!(produced > consumed); // surplus
    }

    #[test]
    fn pip_produced_skips_colorless() {
        let mut entry = make_entry("Wastes", 1);
        entry.card.scryfall_data.type_line = Some("Basic Land".to_string());
        entry.card.scryfall_data.produced_mana = Some(vec!["C".to_string()]);

        let metrics = DeckMetrics::from_entries(&[entry]);
        assert_eq!(metrics.mana_balance, [(0, 0); 5]);
    }
}
