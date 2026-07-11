//! Heuristic classification for the handful of `MechanicalCategory` variants that
//! Oracle Tags cannot cleanly express: `Pump`, `Stax`, `Protection`, `GraveyardHate`.
//!
//! As of 2026-07-11 the other ~18 categories are derived from Oracle-tag subtrees
//! (the community gold standard) and `Tokens` from `all_parts` — see
//! `context/plans/otags/` (Phase 2). Those concepts map cleanly to otags. These four
//! do NOT: the otag taxonomy carves the space differently (no single "single-target
//! pump" or "tax/denial" concept, and `hate-graveyard` under-covers graveyard hate).
//! So we keep a small, stable regex heuristic *only* for them.
//!
//! This is deliberately NOT the old brittle 24-category guesswork — it is the residual
//! ~4 with no gold-standard equivalent. Everything else was retired to Oracle Tags.

use super::MechanicalCategory;
use crate::domain::card::Card;
use regex::Regex;
use std::sync::LazyLock;

/// Classifies a card into the subset of `{Pump, Stax, Protection, GraveyardHate}`
/// it matches — the categories with no clean Oracle-tag equivalent. All other
/// categories come from Oracle Tags (`helpers::derive_categories`), not here.
pub fn classify_oracle_tag_gaps(card: &Card) -> Vec<MechanicalCategory> {
    let sd = &card.scryfall_data;
    let oracle = sd.oracle_text.as_deref().unwrap_or("").to_lowercase();
    let keywords: Vec<String> = sd
        .keywords
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|k| k.to_lowercase())
        .collect();
    let has_keyword = |kw: &str| keywords.iter().any(|k| k == kw);

    let mut cats = Vec::new();

    // Protection: granted keywords or "protection from".
    if has_keyword("hexproof")
        || has_keyword("indestructible")
        || has_keyword("ward")
        || oracle.contains("protection from")
        || oracle.contains("gains hexproof")
        || oracle.contains("gains indestructible")
    {
        cats.push(MechanicalCategory::Protection);
    }

    // Pump: single-target buff (excluding team-wide, which is Anthem via otags).
    if PUMP.is_match(&oracle) && !oracle.contains("creatures you control") {
        cats.push(MechanicalCategory::Pump);
    }

    // Stax: "players can't …" denial or symmetric "each player …" taxes.
    if STAX_CANT.is_match(&oracle) || STAX_EACH.is_match(&oracle) {
        cats.push(MechanicalCategory::Stax);
    }

    // GraveyardHate: exile from graveyard(s).
    if GRAVEYARD_HATE.is_match(&oracle) {
        cats.push(MechanicalCategory::GraveyardHate);
    }

    cats.sort_by_key(|c| *c as u8);
    cats.dedup();
    cats
}

// Compiled regex patterns (lazy-initialized). Compile-time-known valid patterns.
#[allow(clippy::expect_used)]
mod patterns {
    use super::*;

    pub static PUMP: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(target|equipped|enchanted) creature.{0,15}gets? \+\d+/\+\d+")
            .expect("valid regex")
    });
    pub static STAX_CANT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(opponents?|players?) can'?t").expect("valid regex"));
    pub static STAX_EACH: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(each|all) (player|opponent).{0,30}(sacrifice|discard|lose|pay)")
            .expect("valid regex")
    });
    pub static GRAVEYARD_HATE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"exile.{0,30}(graveyard|all cards from)").expect("valid regex")
    });
}
use patterns::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::make_card;
    use MechanicalCategory::*;

    fn card_with(oracle: &str) -> Card {
        let mut card = make_card("Test");
        card.scryfall_data.oracle_text = Some(oracle.to_string());
        card
    }

    #[test]
    fn protection_from_text() {
        let cats =
            classify_oracle_tag_gaps(&card_with("Target creature gains protection from red."));
        assert!(cats.contains(&Protection), "got {cats:?}");
    }

    #[test]
    fn single_target_pump_not_team() {
        let pump =
            classify_oracle_tag_gaps(&card_with("Target creature gets +3/+3 until end of turn."));
        assert!(pump.contains(&Pump), "got {pump:?}");
        // team-wide buff is Anthem (otags), not Pump
        let team = classify_oracle_tag_gaps(&card_with("Creatures you control get +1/+1."));
        assert!(
            !team.contains(&Pump),
            "team buff should not be Pump: {team:?}"
        );
    }

    #[test]
    fn stax_cant_and_each() {
        assert!(
            classify_oracle_tag_gaps(&card_with(
                "Your opponents can't draw more than one card each turn."
            ))
            .contains(&Stax)
        );
        assert!(
            classify_oracle_tag_gaps(&card_with(
                "At the beginning of each player's upkeep, that player sacrifices a creature."
            ))
            .contains(&Stax)
        );
    }

    #[test]
    fn graveyard_hate_exile() {
        let cats = classify_oracle_tag_gaps(&card_with(
            "When this enters, exile all cards from all graveyards.",
        ));
        assert!(cats.contains(&GraveyardHate), "got {cats:?}");
    }

    #[test]
    fn no_false_positive_on_vanilla() {
        assert!(classify_oracle_tag_gaps(&card_with("Flying")).is_empty());
    }
}
