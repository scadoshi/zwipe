//! Reduced commander-synergy payload (`commander_synergy.payload`) and its
//! reduction to a name → score map for ordering card searches.
//!
//! The type itself lives in `zwipe-core` because the synergy worker writes
//! this payload and zerver reads it: one shared definition means a field
//! rename is a build failure on both sides instead of a silent outage here.
//! Parsing is deliberately lenient (unknown fields ignored, missing scores
//! tolerated); shape drift must degrade to "no signal", never to a failed
//! search. Shape contract: `context/plans/synergy_data_layer.md`.
//!
//! Re-exported at this path so every call site keeps its existing import.

pub use zwipe_core::domain::card::models::synergy::{SynergyCard, SynergyList, SynergyPayload};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
