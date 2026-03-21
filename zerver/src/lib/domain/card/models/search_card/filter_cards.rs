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

use crate::domain::card::models::{
    Card,
    search_card::card_filter::{CardFilter, order_by_options::OrderByOptions},
};
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
                if let Some(val) = filter.is_valid_commander()
                    && cp.is_valid_commander != val
                {
                    return false;
                }

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
            if order_by == OrderByOptions::Random {
                let mut rng = rand::rng();
                cards.shuffle(&mut rng);
            } else {
                let ascending = filter.ascending();
                cards.sort_by(|a, b| {
                    let sd_a = &a.scryfall_data;
                    let sd_b = &b.scryfall_data;
                    let ord = match order_by {
                        OrderByOptions::Name => sd_a.name.cmp(&sd_b.name),
                        OrderByOptions::Cmc => {
                            let ca = sd_a.cmc.unwrap_or(f64::MAX);
                            let cb = sd_b.cmc.unwrap_or(f64::MAX);
                            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        OrderByOptions::Power => {
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
                        OrderByOptions::Toughness => {
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
                        OrderByOptions::Rarity => {
                            sd_a.rarity.to_long_name().cmp(&sd_b.rarity.to_long_name())
                        }
                        OrderByOptions::ReleasedAt => sd_a.released_at.cmp(&sd_b.released_at),
                        OrderByOptions::PriceUsd => {
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
                        OrderByOptions::PriceEur => {
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
                        OrderByOptions::PriceTix => {
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
                        OrderByOptions::Random => std::cmp::Ordering::Equal,
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
