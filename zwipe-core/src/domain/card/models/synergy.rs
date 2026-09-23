//! The commander-synergy cache payload (`commander_synergy.payload`) and its
//! reduction to a name → score map for ordering card searches.
//!
//! Written by the synergy worker, read by zerver. This type lives here, in the
//! crate both sides already depend on, so there is exactly ONE definition of
//! the shape. That matters because the reader is tolerant by design: if writer
//! and reader each kept their own copy, renaming a field on the writing side
//! would parse cleanly over here and simply yield nothing — no error, no log,
//! the synergy feature quietly off. Sharing the type turns that class of
//! mistake into a build failure.
//!
//! Tolerance is still required and deliberate: the table holds payloads
//! written by *older* worker versions, so every field but `name` has a
//! default. Shape drift must degrade to "no signal", never to a failed search.
//! Defaults affect deserialization only — the writer always fills them in.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Top-level cached payload: a set of named card lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyPayload {
    /// Card lists keyed by machine tag (high synergy, top cards, per-type...).
    #[serde(default)]
    pub lists: Vec<SynergyList>,
}

/// One themed list of cards within the payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyList {
    /// Stable machine key for the list (e.g. `highsynergycards`).
    #[serde(default)]
    pub tag: String,
    /// Human-readable list title.
    #[serde(default)]
    pub header: String,
    /// Cards in this list.
    #[serde(default)]
    pub cards: Vec<SynergyCard>,
}

/// One card's standing with the commander this payload belongs to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyCard {
    /// Exact card name, resolved against `scryfall_data.name`. The only
    /// required field: a card entry without one carries no usable signal.
    pub name: String,
    /// The source's sanitized card slug, kept as a fallback matching aid.
    #[serde(default)]
    pub slug: String,
    /// Synergy score (roughly −1..1); absent or null for some lists.
    #[serde(default)]
    pub synergy: Option<f64>,
    /// Decks with this commander that include the card.
    #[serde(default)]
    pub num_decks: Option<i64>,
    /// Decks with this commander that could include the card.
    #[serde(default)]
    pub potential_decks: Option<i64>,
}

impl SynergyPayload {
    /// Total cards across all lists. Used by the worker for logging and as a
    /// sanity check — a payload that parses but holds zero cards means the
    /// upstream shape moved, and is treated as a fetch failure rather than
    /// cached over good data.
    pub fn card_count(&self) -> usize {
        self.lists.iter().map(|l| l.cards.len()).sum()
    }

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

    /// Sharing this type stops writer and reader from disagreeing, but it does
    /// NOT make a rename safe: `commander_synergy` already holds rows
    /// serialized with these key names. Rename a field and both crates still
    /// compile — every stored payload just stops deserializing into anything,
    /// silently, because the fields below default to empty.
    ///
    /// So these key names are a data contract with the table, not only with
    /// the other service, and this test is what fails when someone changes
    /// one. Changing a name here means migrating or re-fetching every cached
    /// row.
    #[test]
    fn serialized_keys_are_the_stored_contract() {
        let payload = SynergyPayload {
            lists: vec![SynergyList {
                tag: "highsynergycards".into(),
                header: "High Synergy Cards".into(),
                cards: vec![SynergyCard {
                    name: "Sol Ring".into(),
                    slug: "sol-ring".into(),
                    synergy: Some(0.27),
                    num_decks: Some(27430),
                    potential_decks: Some(41838),
                }],
            }],
        };

        let json = serde_json::to_value(&payload).unwrap();
        let lists = json.get("lists").unwrap().as_array().unwrap();
        let list = lists.first().unwrap();
        let card = list
            .get("cards")
            .unwrap()
            .as_array()
            .unwrap()
            .first()
            .unwrap();

        // Names the ordering path binds to. Types matter as much as names:
        // `name` must stay a string and `synergy` a number.
        assert!(card.get("name").unwrap().is_string());
        assert!(card.get("synergy").unwrap().is_number());
        // Names the worker fills but the reader ignores today.
        for key in ["slug", "num_decks", "potential_decks"] {
            assert!(card.get(key).is_some(), "card lost the `{key}` key");
        }
        for key in ["tag", "header"] {
            assert!(list.get(key).is_some(), "list lost the `{key}` key");
        }
    }

    /// A payload as it sits in the table today must still read back. Pinned
    /// from a real cached row so a future change that only *looks* harmless
    /// has to prove itself against stored data.
    #[test]
    fn a_stored_payload_still_deserializes() {
        const STORED: &str = r#"{"lists":[{"tag":"highsynergycards",
            "header":"High Synergy Cards","cards":[{"name":"Tekuthal, Inquiry Dominus",
            "slug":"tekuthal-inquiry-dominus","synergy":0.27,"num_decks":27430,
            "potential_decks":41838}]}]}"#;

        let payload: SynergyPayload = serde_json::from_str(STORED).unwrap();
        assert_eq!(payload.card_count(), 1);
        let scores = payload.into_scores();
        assert_eq!(scores.get("tekuthal, inquiry dominus").copied(), Some(0.27));
    }
}
