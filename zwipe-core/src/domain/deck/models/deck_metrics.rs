//! Deck-level aggregate metrics computed from a collection of cards.
//!
//! Single-pass computation producing mana curve histogram, type distribution,
//! color distribution, and summary stats. Supports both `Vec<Card>` (each card
//! counted once) and `Vec<DeckEntry>` (each card counted by its quantity).

use crate::domain::{
    card::{
        Card,
        mechanical_category::MechanicalCategory,
        scryfall_data::{ScryfallData, colors::Color},
        search_card::card_filter::price_currency::PriceCurrency,
    },
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
    /// Non-empty mechanical category counts, sorted by count descending.
    pub mechanical_category_counts: Vec<(&'static str, usize)>,
}

impl DeckMetrics {
    /// 5-letter abbreviation for type distribution labels.
    pub fn abbreviate_type(label: &str) -> &str {
        match label {
            "lands" => "lands",
            "creatures" => "creat",
            "planeswalkers" => "plnsw",
            "artifacts" => "artif",
            "enchantments" => "enchn",
            "instants" => "instn",
            "sorceries" => "sorcr",
            "other" => "other",
            _ => label,
        }
    }

    /// 5-letter abbreviation for color distribution labels.
    pub fn abbreviate_color(label: &str) -> &str {
        match label {
            "white" => "white",
            "blue" => "blue",
            "black" => "black",
            "red" => "red",
            "green" => "green",
            "multicolor" => "multi",
            "colorless" => "clrls",
            _ => label,
        }
    }

    /// Computes metrics from deck entries, counting each card by its quantity.
    ///
    /// Maybeboard and sideboard cards are excluded — metrics reflect the active deck only.
    pub fn from_entries(entries: &[DeckEntry]) -> Self {
        Self::from_entries_and_command_zone(entries, &[])
    }

    /// Like [`from_entries`](Self::from_entries), but also folds the command-zone
    /// cards (commander, partner, background, signature spell) into the card count
    /// and price totals. Command-zone cards contribute one copy each and are
    /// *not* mixed into the type/color/mana/category distributions (those reflect
    /// the mainboard). Cards already present as active entries are skipped so a
    /// commander that also sits in the deck list is not double-counted.
    pub fn from_entries_and_command_zone(entries: &[DeckEntry], command_zone: &[Card]) -> Self {
        let active_entries: Vec<DeckEntry> = entries
            .iter()
            .filter(|e| e.deck_card.board.is_active())
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
        let all_cats = MechanicalCategory::all();
        let mut cat_buckets = vec![0usize; all_cats.len()];
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

            // Mechanical categories
            for cat in &card.card_profile.mechanical_categories {
                if let Some(idx) = all_cats.iter().position(|c| c == cat)
                    && let Some(slot) = cat_buckets.get_mut(idx)
                {
                    *slot += qty;
                }
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

        // Fold the command zone into card count + price totals only (one copy
        // each), after mainboard-only stats (nonland_count, avg_cmc) are fixed.
        // Command-zone cards do not enter the type/color/mana/category
        // distributions. Cards already present as active entries are skipped so a
        // commander that also sits in the deck list is not double-counted.
        for card in command_zone {
            let already_active = entries
                .iter()
                .any(|e| e.card.scryfall_data.id == card.scryfall_data.id);
            if already_active {
                continue;
            }
            total_cards += 1;
            if let Some(p) = card_price(&card.scryfall_data, PriceCurrency::Usd) {
                usd_sum += p;
                usd_count += 1;
            }
            if let Some(p) = card_price(&card.scryfall_data, PriceCurrency::Eur) {
                eur_sum += p;
                eur_count += 1;
            }
            if let Some(p) = card_price(&card.scryfall_data, PriceCurrency::Tix) {
                tix_sum += p;
                tix_count += 1;
            }
        }

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

        let mut mechanical_category_counts: Vec<(&'static str, usize)> = all_cats
            .iter()
            .zip(cat_buckets.iter())
            .filter(|&(_, count)| *count > 0)
            .map(|(cat, &count)| (cat.to_short_name(), count))
            .collect();
        mechanical_category_counts.sort_by_key(|a| std::cmp::Reverse(a.1));

        let total_price_usd = if usd_count > 0 { Some(usd_sum) } else { None };
        let avg_price_usd = if usd_count > 0 {
            Some(usd_sum / usd_count as f64)
        } else {
            None
        };
        let total_price_eur = if eur_count > 0 { Some(eur_sum) } else { None };
        let avg_price_eur = if eur_count > 0 {
            Some(eur_sum / eur_count as f64)
        } else {
            None
        };
        let total_price_tix = if tix_count > 0 { Some(tix_sum) } else { None };
        let avg_price_tix = if tix_count > 0 {
            Some(tix_sum / tix_count as f64)
        } else {
            None
        };

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
            mechanical_category_counts,
        }
    }
}

/// Card type classification — first match wins (matches group_cards.rs logic).
#[cfg(test)]
mod budget_tests {
    use super::budget_tier;

    #[test]
    fn tiers_track_total_against_budget() {
        assert_eq!(budget_tier(100.0, 40.0), 0);
        assert_eq!(budget_tier(100.0, 50.0), 1);
        assert_eq!(budget_tier(100.0, 80.0), 2);
        assert_eq!(budget_tier(100.0, 100.0), 3);
        assert_eq!(budget_tier(100.0, 130.0), 3);
        // No budget → tier 0 (no alerts).
        assert_eq!(budget_tier(0.0, 50.0), 0);
    }

    #[test]
    fn fires_when_band_increases_between_before_and_after() {
        // Screens toast when budget_tier(after) > budget_tier(before): once per
        // upward crossing, re-firing if the total dips and crosses again.
        let crossed = |before, after| budget_tier(100.0, after) > budget_tier(100.0, before);
        assert!(crossed(10.0, 89.0)); // 0 → 2, single fire across two markers
        assert!(crossed(49.0, 52.0)); // crosses 50%
        assert!(!crossed(24.0, 25.0)); // same band, no fire
        assert!(!crossed(80.0, 85.0)); // same band, no fire
        assert!(!crossed(80.0, 40.0)); // downward, no fire
        // Teeter across 50%: down (no), back up (yes).
        assert!(!crossed(52.0, 48.0));
        assert!(crossed(48.0, 51.0));
    }
}

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

/// Price of a single card in the given currency, preferring nonfoil → foil →
/// etched (USD/EUR); TIX is nonfoil only. `None` when no price is available.
pub fn card_price(scryfall_data: &ScryfallData, currency: PriceCurrency) -> Option<f64> {
    let prices = &scryfall_data.prices;
    match currency {
        PriceCurrency::Usd => parse_price(&prices.usd)
            .or_else(|| parse_price(&prices.usd_foil))
            .or_else(|| parse_price(&prices.usd_etched)),
        PriceCurrency::Eur => parse_price(&prices.eur)
            .or_else(|| parse_price(&prices.eur_foil))
            .or_else(|| parse_price(&prices.eur_etched)),
        PriceCurrency::Tix => parse_price(&prices.tix),
    }
}

/// How many budget markers (50% / 75% / 100%) a total has reached: `0` (<50%),
/// `1` (≥50%), `2` (≥75%), `3` (≥100%). `0` when there is no budget.
///
/// Screens track the highest band already announced and only toast when the band
/// increases, so each marker alerts exactly once (no re-fire on re-crossing, and
/// nothing once already over). The toast itself reports the *exact* percentage.
pub fn budget_tier(budget: f64, total: f64) -> u8 {
    if budget <= 0.0 {
        0
    } else if total >= budget {
        3
    } else if total >= 0.75 * budget {
        2
    } else if total >= 0.5 * budget {
        1
    } else {
        0
    }
}

/// Total price of the mainboard (active board) in the given currency,
/// quantity-aware. Cards without a price in that currency contribute 0 (matching
/// the deck price estimate), so the budget total is a lower bound.
pub fn mainboard_total_price(entries: &[DeckEntry], currency: PriceCurrency) -> f64 {
    entries
        .iter()
        .filter(|e| e.deck_card.board.is_active())
        .map(|e| {
            card_price(&e.card.scryfall_data, currency).unwrap_or(0.0)
                * (*e.deck_card.quantity as f64)
        })
        .sum()
}

/// Full deck price in the given currency: the mainboard total plus the command
/// zone (commander, partner, background, signature spell). Command-zone cards
/// already present as active entries are skipped so they are not double-counted.
/// Cards without a price in that currency contribute 0.
pub fn deck_price(entries: &[DeckEntry], command_zone: &[Card], currency: PriceCurrency) -> f64 {
    let command_zone_total: f64 = command_zone
        .iter()
        .filter(|c| {
            !entries.iter().any(|e| {
                e.deck_card.board.is_active() && e.card.scryfall_data.id == c.scryfall_data.id
            })
        })
        .map(|c| card_price(&c.scryfall_data, currency).unwrap_or(0.0))
        .sum();
    mainboard_total_price(entries, currency) + command_zone_total
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
    use crate::{
        domain::card::scryfall_data::colors::{Color, Colors},
        test_utils::make_entry,
    };

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
            vec![
                ("lands", 1),
                ("creatures", 1),
                ("artifacts", 1),
                ("instants", 1)
            ]
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
        entry.card.scryfall_data.type_line = Some("Creature — Phyrexian Angel Horror".to_string());

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
        assert_eq!(metrics.type_counts, vec![("lands", 10), ("instants", 4)]);
        assert_eq!(metrics.color_counts, vec![("red", 4), ("green", 10)]);
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
    fn command_zone_folds_into_count_and_price() {
        let mut spell = make_entry("Sol Ring", 1);
        spell.card.scryfall_data.prices.usd = Some("2.00".to_string());
        spell.card.scryfall_data.type_line = Some("Artifact".to_string());

        let mut commander = make_entry("Atraxa", 1).card;
        commander.scryfall_data.prices.usd = Some("10.00".to_string());
        commander.scryfall_data.type_line = Some("Creature".to_string());
        commander.scryfall_data.cmc = Some(4.0);

        let metrics = DeckMetrics::from_entries_and_command_zone(&[spell], &[commander]);
        // Card count includes the commander.
        assert_eq!(metrics.total_cards, 2);
        // Price includes the commander (2.00 + 10.00).
        assert!((metrics.total_price_usd.unwrap() - 12.00).abs() < f64::EPSILON);
        assert!((metrics.avg_price_usd.unwrap() - 6.00).abs() < f64::EPSILON);
        // The commander does NOT dilute the mainboard-only avg_cmc (Sol Ring is
        // an artifact = nonland with cmc 0, so avg stays 0.0).
        assert_eq!(metrics.avg_cmc, 0.0);
    }

    #[test]
    fn command_zone_skips_card_already_in_entries() {
        let mut entry = make_entry("Sol Ring", 1);
        entry.card.scryfall_data.prices.usd = Some("2.00".to_string());
        let commander = entry.card.clone();

        let metrics = DeckMetrics::from_entries_and_command_zone(&[entry], &[commander]);
        // Same scryfall id already active — not double-counted.
        assert_eq!(metrics.total_cards, 1);
        assert!((metrics.total_price_usd.unwrap() - 2.00).abs() < f64::EPSILON);
    }

    #[test]
    fn deck_price_includes_command_zone() {
        let mut entry = make_entry("Forest", 1);
        entry.card.scryfall_data.prices.usd = Some("1.00".to_string());

        let mut commander = make_entry("Atraxa", 1).card;
        commander.scryfall_data.prices.usd = Some("10.00".to_string());

        let total = deck_price(&[entry], &[commander], PriceCurrency::Usd);
        assert!((total - 11.00).abs() < f64::EPSILON);
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
