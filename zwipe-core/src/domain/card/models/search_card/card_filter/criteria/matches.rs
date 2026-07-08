//! The in-memory card predicate.
//!
//! Mirrors the SQL adapter's filtering logic exactly, so the same criteria
//! applied against a local collection agree with a server-side search.

use crate::domain::{
    card::{
        Card,
        scryfall_data::legalities::LegalityKind,
        search_card::{
            card_filter::{
                criteria::CardCriteria, price_currency::PriceCurrency, strip_punctuation,
            },
            commander_eligibility::is_valid_commander,
        },
    },
    deck::Format,
};

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

impl CardCriteria {
    /// True if `card` satisfies every criterion set on `self`.
    ///
    /// Unset criteria (`None`) never exclude a card, so an empty criteria set
    /// matches everything. Parity note: `is_partner` / `is_background` /
    /// `is_signature_spell` are **not** evaluated here (matching the old
    /// in-memory filter) — they are command-zone pool constraints only the
    /// server-side search applies.
    pub fn matches(&self, card: &Card) -> bool {
        let sd = &card.scryfall_data;
        let cp = &card.card_profile;

        // ── text ──────────────────────────────────────────────────────
        if let Some(q) = self.name_contains()
            && !strip_punctuation(&sd.name)
                .to_lowercase()
                .contains(&q.to_lowercase())
        {
            return false;
        }

        if let Some(q) = self.name_not_contains()
            && strip_punctuation(&sd.name)
                .to_lowercase()
                .contains(&q.to_lowercase())
        {
            return false;
        }

        if let Some(q) = self.oracle_text_contains() {
            match &sd.oracle_text {
                Some(text)
                    if strip_punctuation(text)
                        .to_lowercase()
                        .contains(&q.to_lowercase()) => {}
                _ => return false,
            }
        }

        if let Some(q) = self.oracle_text_not_contains()
            && sd.oracle_text.as_ref().is_some_and(|text| {
                strip_punctuation(text)
                    .to_lowercase()
                    .contains(&q.to_lowercase())
            })
        {
            return false;
        }

        if let Some(values) = self.oracle_text_contains_any() {
            let matches = match &sd.oracle_text {
                Some(text) => {
                    let stripped = strip_punctuation(text).to_lowercase();
                    values.iter().any(|v| stripped.contains(&v.to_lowercase()))
                }
                None => false,
            };
            if !matches {
                return false;
            }
        }

        if let Some(values) = self.oracle_text_contains_all() {
            let matches = match &sd.oracle_text {
                Some(text) => {
                    let stripped = strip_punctuation(text).to_lowercase();
                    values.iter().all(|v| stripped.contains(&v.to_lowercase()))
                }
                None => false,
            };
            if !matches {
                return false;
            }
        }

        if let Some(values) = self.oracle_text_excludes_any() {
            let excluded = match &sd.oracle_text {
                Some(text) => {
                    let stripped = strip_punctuation(text).to_lowercase();
                    values.iter().any(|v| stripped.contains(&v.to_lowercase()))
                }
                None => false,
            };
            if excluded {
                return false;
            }
        }

        // ── keywords ──────────────────────────────────────────────────
        if let Some(values) = self.keywords_contains_any() {
            let matches = match &sd.keywords {
                Some(kw) => values
                    .iter()
                    .any(|v| kw.iter().any(|k| k.eq_ignore_ascii_case(v))),
                None => false,
            };
            if !matches {
                return false;
            }
        }

        if let Some(values) = self.keywords_contains_all() {
            let matches = match &sd.keywords {
                Some(kw) => values
                    .iter()
                    .all(|v| kw.iter().any(|k| k.eq_ignore_ascii_case(v))),
                None => false,
            };
            if !matches {
                return false;
            }
        }

        if let Some(values) = self.keywords_excludes() {
            let excluded = match &sd.keywords {
                Some(kw) => values
                    .iter()
                    .any(|v| kw.iter().any(|k| k.eq_ignore_ascii_case(v))),
                None => false,
            };
            if excluded {
                return false;
            }
        }

        // ── mechanical categories ───────────────────────────────────
        if let Some(values) = self.mechanical_categories_contains_any() {
            let matches = card.card_profile.mechanical_categories.iter().any(|cat| {
                values
                    .iter()
                    .any(|v| cat.to_string().eq_ignore_ascii_case(v))
            });
            if !matches {
                return false;
            }
        }

        if let Some(values) = self.mechanical_categories_contains_all() {
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

        if let Some(values) = self.mechanical_categories_excludes() {
            let excluded = card.card_profile.mechanical_categories.iter().any(|cat| {
                values
                    .iter()
                    .any(|v| cat.to_string().eq_ignore_ascii_case(v))
            });
            if excluded {
                return false;
            }
        }

        // ── produced mana ────────────────────────────────────────────
        if let Some(values) = self.produced_mana_contains_any() {
            let matches = match &sd.produced_mana {
                Some(pm) => values
                    .iter()
                    .any(|v| pm.iter().any(|p| p.eq_ignore_ascii_case(v))),
                None => false,
            };
            if !matches {
                return false;
            }
        }

        if let Some(values) = self.produced_mana_contains_all() {
            let matches = match &sd.produced_mana {
                Some(pm) => values
                    .iter()
                    .all(|v| pm.iter().any(|p| p.eq_ignore_ascii_case(v))),
                None => false,
            };
            if !matches {
                return false;
            }
        }

        if let Some(values) = self.produced_mana_excludes() {
            let excluded = match &sd.produced_mana {
                Some(pm) => values
                    .iter()
                    .any(|v| pm.iter().any(|p| p.eq_ignore_ascii_case(v))),
                None => false,
            };
            if excluded {
                return false;
            }
        }

        if let Some(q) = self.flavor_text_contains() {
            match &sd.flavor_text {
                Some(text)
                    if strip_punctuation(text)
                        .to_lowercase()
                        .contains(&q.to_lowercase()) => {}
                _ => return false,
            }
        }

        if let Some(q) = self.flavor_text_not_contains()
            && sd.flavor_text.as_ref().is_some_and(|text| {
                strip_punctuation(text)
                    .to_lowercase()
                    .contains(&q.to_lowercase())
            })
        {
            return false;
        }

        if let Some(want_flavor) = self.has_flavor_text() {
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
        if let Some(q) = self.type_line_contains() {
            match &sd.type_line {
                Some(tl)
                    if strip_punctuation(tl)
                        .to_lowercase()
                        .contains(&q.to_lowercase()) => {}
                _ => return false,
            }
        }

        if let Some(q) = self.type_line_not_contains()
            && sd.type_line.as_ref().is_some_and(|tl| {
                strip_punctuation(tl)
                    .to_lowercase()
                    .contains(&q.to_lowercase())
            })
        {
            return false;
        }

        if let Some(values) = self.type_line_contains_any() {
            let matches = match &sd.type_line {
                Some(tl) => {
                    let stripped = strip_punctuation(tl).to_lowercase();
                    values.iter().any(|v| stripped.contains(&v.to_lowercase()))
                }
                None => false,
            };
            if !matches {
                return false;
            }
        }

        if let Some(card_types) = self.card_type_contains_any() {
            let matches = match &sd.type_line {
                Some(tl) => {
                    let stripped = strip_punctuation(tl).to_lowercase();
                    card_types
                        .iter()
                        .any(|ct| stripped.contains(&ct.to_string().to_lowercase()))
                }
                None => false,
            };
            if !matches {
                return false;
            }
        }

        if let Some(values) = self.type_line_contains_all() {
            let matches = match &sd.type_line {
                Some(tl) => {
                    let stripped = strip_punctuation(tl).to_lowercase();
                    values.iter().all(|v| stripped.contains(&v.to_lowercase()))
                }
                None => false,
            };
            if !matches {
                return false;
            }
        }

        if let Some(card_types) = self.card_type_contains_all() {
            let matches = match &sd.type_line {
                Some(tl) => card_types
                    .iter()
                    .all(|ct| tl.to_lowercase().contains(&ct.to_string().to_lowercase())),
                None => false,
            };
            if !matches {
                return false;
            }
        }

        if let Some(values) = self.type_line_excludes_any() {
            let excluded = match &sd.type_line {
                Some(tl) => {
                    let stripped = strip_punctuation(tl).to_lowercase();
                    values.iter().any(|v| stripped.contains(&v.to_lowercase()))
                }
                None => false,
            };
            if excluded {
                return false;
            }
        }

        if let Some(card_types) = self.card_type_excludes_any() {
            let excluded = match &sd.type_line {
                Some(tl) => {
                    let stripped = strip_punctuation(tl).to_lowercase();
                    card_types
                        .iter()
                        .any(|ct| stripped.contains(&ct.to_string().to_lowercase()))
                }
                None => false,
            };
            if excluded {
                return false;
            }
        }

        // ── legality (any-of-formats) ────────────────────────────────
        if let Some(formats) = self.legalities_contains_any() {
            let is_legal_in_any = formats.iter().any(|format_key| {
                Format::try_from(format_key.as_str())
                    .ok()
                    .and_then(|fmt| sd.legalities.get(&fmt).cloned())
                    .is_some_and(|k| matches!(k, LegalityKind::Legal | LegalityKind::Restricted))
            });
            if !is_legal_in_any {
                return false;
            }
        }

        // ── commander eligibility ────────────────────────────────────
        if let Some(format) = self.is_commander_in_format()
            && !is_valid_commander(card, format)
        {
            return false;
        }

        // ── mana ──────────────────────────────────────────────────────
        if let Some(val) = self.cmc_equals()
            && sd.cmc != Some(val)
        {
            return false;
        }

        if let Some((min, max)) = self.cmc_range() {
            let lo = min.min(max);
            let hi = min.max(max);
            match sd.cmc {
                Some(cmc) if cmc >= lo && cmc <= hi => {}
                _ => return false,
            }
        }

        // ── price ─────────────────────────────────────────────────────
        if self.price_min().is_some() || self.price_max().is_some() {
            let price = match self.price_currency().unwrap_or_default() {
                PriceCurrency::Usd => sd.prices.usd.as_deref(),
                PriceCurrency::Eur => sd.prices.eur.as_deref(),
                PriceCurrency::Tix => sd.prices.tix.as_deref(),
            }
            .and_then(|p| p.parse::<f64>().ok());
            match price {
                // A missing/unparseable price can't be confirmed in budget.
                None => return false,
                Some(p) => {
                    if self.price_min().is_some_and(|min| p < min) {
                        return false;
                    }
                    if self.price_max().is_some_and(|max| p > max) {
                        return false;
                    }
                }
            }
        }

        if let Some(filter_colors) = self.color_identity_equals() {
            let card_ci = &sd.color_identity;
            let set_eq = filter_colors.len() == card_ci.len()
                && filter_colors.iter().all(|c| card_ci.contains(c));
            if !set_eq {
                return false;
            }
        }

        if let Some(filter_colors) = self.color_identity_within()
            && !sd.color_identity.iter().all(|c| filter_colors.contains(c))
        {
            return false;
        }

        // ── combat ────────────────────────────────────────────────────
        if let Some(val) = self.power_equals() {
            let parsed = sd.power.as_deref().and_then(|p| p.parse::<i32>().ok());
            if parsed != Some(val) {
                return false;
            }
        }

        if let Some((min, max)) = self.power_range() {
            let lo = min.min(max);
            let hi = min.max(max);
            match sd.power.as_deref().and_then(|p| p.parse::<i32>().ok()) {
                Some(p) if p >= lo && p <= hi => {}
                _ => return false,
            }
        }

        if let Some(val) = self.toughness_equals() {
            let parsed = sd.toughness.as_deref().and_then(|t| t.parse::<i32>().ok());
            if parsed != Some(val) {
                return false;
            }
        }

        if let Some((min, max)) = self.toughness_range() {
            let lo = min.min(max);
            let hi = min.max(max);
            match sd.toughness.as_deref().and_then(|t| t.parse::<i32>().ok()) {
                Some(t) if t >= lo && t <= hi => {}
                _ => return false,
            }
        }

        // ── metadata ──────────────────────────────────────────────────
        if let Some(rarities) = self.rarity_equals_any()
            && !rarities.contains(&sd.rarity)
        {
            return false;
        }

        if let Some(rarities) = self.rarity_excludes_any()
            && rarities.contains(&sd.rarity)
        {
            return false;
        }

        if let Some(sets) = self.set_equals_any()
            && !sets.iter().any(|s| s == &sd.set_name)
        {
            return false;
        }

        if let Some(sets) = self.set_excludes_any()
            && sets.iter().any(|s| s == &sd.set_name)
        {
            return false;
        }

        if let Some(artists) = self.artist_equals_any()
            && !artists
                .iter()
                .any(|a| Some(a.as_str()) == sd.artist.as_deref())
        {
            return false;
        }

        if let Some(artists) = self.artist_excludes_any()
            && artists
                .iter()
                .any(|a| Some(a.as_str()) == sd.artist.as_deref())
        {
            return false;
        }

        if let Some(lang) = self.language()
            && sd.lang != lang
        {
            return false;
        }

        // ── flags ─────────────────────────────────────────────────────

        if let Some(val) = self.is_token()
            && cp.is_token != val
        {
            return false;
        }

        if let Some(want_playable) = self.is_playable() {
            let is_playable = PLAYABLE_LAYOUTS.contains(&sd.layout.as_str());
            if is_playable != want_playable {
                return false;
            }
        }

        if let Some(val) = self.digital()
            && sd.digital != val
        {
            return false;
        }

        if let Some(val) = self.oversized()
            && sd.oversized != val
        {
            return false;
        }

        if let Some(val) = self.promo()
            && sd.promo != val
        {
            return false;
        }

        if let Some(want_warning) = self.content_warning() {
            let has_warning = sd.content_warning == Some(true);
            if has_warning != want_warning {
                return false;
            }
        }

        true
    }
}
