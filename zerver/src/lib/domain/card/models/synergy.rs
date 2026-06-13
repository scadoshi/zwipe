//! Reduced commander-synergy payload (`commander_synergy.payload`) and its
//! reduction to a name → score map for ordering card searches.
//!
//! Written by the synergy worker, read here. Shape contract:
//! `context/plans/synergy-data-layer.md`. Parsing is deliberately lenient
//! (unknown fields ignored, missing scores tolerated) — shape drift upstream
//! must degrade to "no signal", never to a failed search.

use serde::Deserialize;
use std::collections::HashMap;

/// Which column a synergy score map is keyed by, for deck-aware default
/// ordering.
#[derive(Debug, Clone, Copy)]
pub enum SynergyKey {
    /// Map keyed by lowercased card name — the cached commander signal
    /// (`SynergyPayload::into_scores`).
    Name,
    /// Map keyed by oracle_id string — the live Recommander signal, which
    /// carries oracle_ids natively, so matching is exact.
    OracleId,
}

/// A synergy score map plus how it is keyed, passed to the deck-aware search to
/// drive default ordering. Borrowed so the JSON map isn't cloned at the call
/// site.
#[derive(Debug, Clone, Copy)]
pub struct SynergyOrder<'a> {
    /// Which card column to match the map's keys against.
    pub key: SynergyKey,
    /// The score map: `{ key -> score }` as JSON, bound directly into SQL.
    pub scores: &'a serde_json::Value,
}

/// Top-level cached payload: a set of named card lists.
#[derive(Debug, Deserialize)]
pub struct SynergyPayload {
    /// Card lists keyed by machine tag (high synergy, top cards, per-type...).
    #[serde(default)]
    pub lists: Vec<SynergyList>,
}

/// One list of cards within the payload.
#[derive(Debug, Deserialize)]
pub struct SynergyList {
    /// Cards in this list.
    #[serde(default)]
    pub cards: Vec<SynergyCard>,
}

/// One card entry. Only the fields ordering needs.
#[derive(Debug, Deserialize)]
pub struct SynergyCard {
    /// Exact card name — resolved against `scryfall_data.name`.
    pub name: String,
    /// Synergy score (roughly −1..1); absent/null for some lists.
    #[serde(default)]
    pub synergy: Option<f64>,
}

impl SynergyPayload {
    /// Flattens to a lowercased-name → score map, keeping the highest score
    /// when a card appears in multiple lists. Scoreless entries get a small
    /// floor score: still boosted above cards with no signal at all, but
    /// below anything actually scored.
    pub fn into_scores(self) -> HashMap<String, f64> {
        const SCORELESS_FLOOR: f64 = -10.0;
        let mut scores: HashMap<String, f64> = HashMap::new();
        for card in self.lists.into_iter().flat_map(|l| l.cards) {
            let score = card.synergy.unwrap_or(SCORELESS_FLOOR);
            scores
                .entry(card.name.to_lowercase())
                .and_modify(|s| *s = s.max(score))
                .or_insert(score);
        }
        scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A parse failure yields an empty map, which fails the score assertions.
    fn scores_from(payload: serde_json::Value) -> HashMap<String, f64> {
        serde_json::from_value::<SynergyPayload>(payload)
            .map(SynergyPayload::into_scores)
            .unwrap_or_default()
    }

    #[test]
    fn reduces_to_max_score_per_name() {
        let scores = scores_from(serde_json::json!({
            "lists": [
                {"tag": "a", "cards": [{"name": "Sol Ring", "synergy": 0.1}]},
                {"tag": "b", "cards": [{"name": "Sol Ring", "synergy": 0.9}]}
            ]
        }));
        assert_eq!(scores.len(), 1);
        assert_eq!(scores.get("sol ring").copied(), Some(0.9));
    }

    #[test]
    fn scoreless_entries_get_floor_not_skip() {
        let scores = scores_from(serde_json::json!({
            "lists": [{"cards": [{"name": "New Card", "synergy": null}]}]
        }));
        assert!(scores.get("new card").copied() < Some(-1.0));
    }

    #[test]
    fn tolerates_unknown_fields_and_empty_payload() {
        let scores = scores_from(serde_json::json!({
            "lists": [{"tag": "x", "header": "X", "cards": [
                {"name": "A", "synergy": 0.2, "color": "blue", "weight": 5}
            ]}],
            "future_field": true
        }));
        assert_eq!(scores.get("a").copied(), Some(0.2));
        assert!(scores_from(serde_json::json!({})).is_empty());
    }
}
