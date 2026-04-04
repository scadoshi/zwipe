//! Deck-level aggregate metrics computed from a collection of cards.
//!
//! Single-pass computation producing mana curve histogram, type distribution,
//! color distribution, and summary stats. Supports both `Vec<Card>` (each card
//! counted once) and `Vec<DeckEntry>` (each card counted by its quantity).

use crate::domain::{
    card::{scryfall_data::colors::Color, Card},
    deck::deck::DeckEntry,
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
    /// Total deck price in USD (nonfoil preferred, foil/etched fallback). `None` if no cards priced.
    pub total_price_usd: Option<f64>,
    /// Average card price in USD. `None` if no cards priced.
    pub avg_price_usd: Option<f64>,
    /// Total deck price in EUR. `None` if no cards priced.
    pub total_price_eur: Option<f64>,
    /// Average card price in EUR. `None` if no cards priced.
    pub avg_price_eur: Option<f64>,
    /// Total deck price in MTGO Event Tickets. `None` if no cards priced.
    pub total_price_tix: Option<f64>,
    /// Average card price in MTGO Event Tickets. `None` if no cards priced.
    pub avg_price_tix: Option<f64>,
}


impl DeckMetrics {
    /// Computes metrics from deck entries, counting each card by its quantity.
    ///
    /// Maybeboard cards are excluded — metrics reflect the active deck only.
    pub fn from_entries(entries: &[DeckEntry]) -> Self {
        let active_entries: Vec<DeckEntry> = entries
            .iter()
            .filter(|e| !e.deck_card.maybeboard)
            .cloned()
            .collect();
        let entries = &active_entries;
        let mut cmc_histogram = [0usize; 7];
        let mut type_buckets = [0usize; 8];
        let mut color_buckets = [0usize; 7];
        let mut land_count = 0usize;
        let mut cmc_sum = 0.0f64;
        let mut total_cards = 0usize;
        let mut pip_consumed = [0usize; 5];
        let mut pip_produced = [0usize; 5];
        let mut usd_sum = 0.0f64;
        let mut usd_count = 0usize;
        let mut eur_sum = 0.0f64;
        let mut eur_count = 0usize;
        let mut tix_sum = 0.0f64;
        let mut tix_count = 0usize;

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

            // Price: prefer nonfoil → foil → etched
            let prices = &card.scryfall_data.prices;
            if let Some(p) = parse_price(&prices.usd)
                .or_else(|| parse_price(&prices.usd_foil))
                .or_else(|| parse_price(&prices.usd_etched))
            {
                usd_sum += p * qty as f64;
                usd_count += qty;
            }
            if let Some(p) = parse_price(&prices.eur)
                .or_else(|| parse_price(&prices.eur_foil))
                .or_else(|| parse_price(&prices.eur_etched))
            {
                eur_sum += p * qty as f64;
                eur_count += qty;
            }
            if let Some(p) = parse_price(&prices.tix) {
                tix_sum += p * qty as f64;
                tix_count += qty;
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

        let total_price_usd = if usd_count > 0 { Some(usd_sum) } else { None };
        let avg_price_usd = if usd_count > 0 { Some(usd_sum / usd_count as f64) } else { None };
        let total_price_eur = if eur_count > 0 { Some(eur_sum) } else { None };
        let avg_price_eur = if eur_count > 0 { Some(eur_sum / eur_count as f64) } else { None };
        let total_price_tix = if tix_count > 0 { Some(tix_sum) } else { None };
        let avg_price_tix = if tix_count > 0 { Some(tix_sum / tix_count as f64) } else { None };

        DeckMetrics {
            total_cards,
            avg_cmc,
            land_count,
            nonland_count,
            cmc_histogram,
            type_counts,
            color_counts,
            mana_balance,
            total_price_usd,
            avg_price_usd,
            total_price_eur,
            avg_price_eur,
            total_price_tix,
            avg_price_tix,
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

/// Parse an optional price string (e.g. "1.50") into f64.
fn parse_price(price: &Option<String>) -> Option<f64> {
    price.as_ref().and_then(|s| s.parse::<f64>().ok())
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
    use crate::domain::card::scryfall_data::colors::{Color, Colors};
    use crate::test_utils::make_entry;

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
        assert_eq!(metrics.total_price_usd, None);
        assert_eq!(metrics.avg_price_usd, None);
        assert_eq!(metrics.total_price_eur, None);
        assert_eq!(metrics.avg_price_eur, None);
        assert_eq!(metrics.total_price_tix, None);
        assert_eq!(metrics.avg_price_tix, None);
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

    #[test]
    fn price_aggregation_usd() {
        let mut a = make_entry("Sol Ring", 2);
        a.card.scryfall_data.prices.usd = Some("1.50".to_string());
        let mut b = make_entry("Mana Crypt", 3);
        b.card.scryfall_data.prices.usd = Some("1.50".to_string());

        let metrics = DeckMetrics::from_entries(&[a, b]);
        assert!((metrics.total_price_usd.unwrap() - 7.50).abs() < f64::EPSILON);
        assert!((metrics.avg_price_usd.unwrap() - 1.50).abs() < f64::EPSILON);
    }

    #[test]
    fn price_none_when_no_prices() {
        let entry = make_entry("Mystery Card", 1);
        let metrics = DeckMetrics::from_entries(&[entry]);
        assert_eq!(metrics.total_price_usd, None);
        assert_eq!(metrics.avg_price_usd, None);
        assert_eq!(metrics.total_price_eur, None);
        assert_eq!(metrics.avg_price_eur, None);
        assert_eq!(metrics.total_price_tix, None);
        assert_eq!(metrics.avg_price_tix, None);
    }

    #[test]
    fn price_mixed_availability() {
        let mut priced = make_entry("Sol Ring", 1);
        priced.card.scryfall_data.prices.usd = Some("2.00".to_string());
        let unpriced = make_entry("Mystery Card", 1);

        let metrics = DeckMetrics::from_entries(&[priced, unpriced]);
        assert!((metrics.total_price_usd.unwrap() - 2.00).abs() < f64::EPSILON);
        assert!((metrics.avg_price_usd.unwrap() - 2.00).abs() < f64::EPSILON);
    }

    #[test]
    fn price_foil_fallback() {
        let mut entry = make_entry("Foil Only", 1);
        entry.card.scryfall_data.prices.usd = None;
        entry.card.scryfall_data.prices.usd_foil = Some("5.00".to_string());

        let metrics = DeckMetrics::from_entries(&[entry]);
        assert!((metrics.total_price_usd.unwrap() - 5.00).abs() < f64::EPSILON);
    }

    #[test]
    fn price_quantity_multiplied() {
        let mut entry = make_entry("Lightning Bolt", 4);
        entry.card.scryfall_data.prices.usd = Some("1.00".to_string());

        let metrics = DeckMetrics::from_entries(&[entry]);
        assert!((metrics.total_price_usd.unwrap() - 4.00).abs() < f64::EPSILON);
        assert!((metrics.avg_price_usd.unwrap() - 1.00).abs() < f64::EPSILON);
    }
}
