//! The server card-search request: criteria + pagination + ordering.
//!
//! [`CardQuery`] is what zwiper POSTs to `/api/card/search` and the deck-aware
//! search. Its [`CardCriteria`] is `#[serde(flatten)]`ed, so the wire JSON is
//! identical to the pre-split `CardFilter` (criteria fields at the top level
//! alongside `limit` / `offset` / `order_by` / `ascending` / `synergy`).
//!
//! `limit` is untrusted pagination input, so it is a clamping [`Limit`] — the
//! in-memory path ([`Cards`](crate::domain::card::search_card::cards::Cards))
//! cannot express a limit at all.

use crate::domain::card::search_card::card_filter::{
    card_sort_key::CardSortKey, criteria::CardCriteria,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::skip_serializing_none;

/// Bounded result-page size for the server search.
///
/// Construction clamps to [`Limit::MAX`] — including on deserialize, so an
/// over-large value from the wire can never reach the database. (zerver keeps
/// its own SQL-side clamp as defense-in-depth.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Limit(u32);

impl Limit {
    /// The hard cap on a single search page.
    pub const MAX: u32 = 250;

    /// Creates a limit, clamping `value` to [`Limit::MAX`].
    pub fn new(value: u32) -> Self {
        Self(value.min(Self::MAX))
    }

    /// The bounded value.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl Default for Limit {
    /// The pre-split default page size (25).
    fn default() -> Self {
        Self(25)
    }
}

impl From<u32> for Limit {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for Limit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u32::deserialize(deserializer).map(Self::new)
    }
}

/// A server card-search request: what to match, plus pagination and ordering.
///
/// Built via [`CardQueryBuilder`](super::builder::CardQueryBuilder). The
/// in-memory filter path uses bare [`CardCriteria`] instead and cannot carry
/// pagination — that split is the point.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CardQuery {
    /// The predicate fields, flattened so the wire stays the pre-split shape.
    #[serde(flatten)]
    criteria: CardCriteria,
    #[serde(default)]
    limit: Limit,
    #[serde(default)]
    offset: u32,
    /// Sort key; serialized as `order_by` to preserve the wire contract.
    #[serde(rename = "order_by")]
    sort: Option<CardSortKey>,
    #[serde(default = "default_ascending")]
    ascending: bool,
    /// Deck-aware search only: when true, constrain results to the commander's
    /// synergy pool (membership), then sort by `sort` within it. Ignored by
    /// the plain (non-deck) search. `#[serde(default)]` so older clients that
    /// omit it parse to `false` (full-pool behavior).
    #[serde(default)]
    synergy: bool,
}

fn default_ascending() -> bool {
    true
}

impl CardQuery {
    /// Assembles a query from already-validated criteria plus config. Only the
    /// builder constructs these (validation lives there).
    pub(super) fn new(
        criteria: CardCriteria,
        limit: Limit,
        offset: u32,
        sort: Option<CardSortKey>,
        ascending: bool,
        synergy: bool,
    ) -> Self {
        Self {
            criteria,
            limit,
            offset,
            sort,
            ascending,
            synergy,
        }
    }

    /// The predicate fields.
    pub fn criteria(&self) -> &CardCriteria {
        &self.criteria
    }

    /// Bounded page size.
    pub fn limit(&self) -> u32 {
        self.limit.get()
    }

    /// Pagination offset.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Sort key (`order_by` on the wire).
    pub fn sort(&self) -> Option<CardSortKey> {
        self.sort
    }

    /// Sort direction.
    pub fn ascending(&self) -> bool {
        self.ascending
    }

    /// Deck-aware synergy membership mode (see field docs).
    pub fn synergy(&self) -> bool {
        self.synergy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::search_card::card_filter::builder::CardQueryBuilder;

    // ── the Option A wire gate ────────────────────────────────────────────
    // The split must not change the POST-body JSON. These tests pin the wire
    // to the pre-split `CardFilter` shape in both directions.

    /// A pre-split `CardFilter` JSON body, as an already-shipped client sends
    /// it: criteria fields at the top level alongside the config fields.
    const OLD_CLIENT_JSON: &str = r#"{
        "name_contains": "bolt",
        "cmc_range": [1.0, 3.0],
        "card_type_contains_any": ["Instant"],
        "is_playable": true,
        "digital": false,
        "content_warning": false,
        "language": "en",
        "limit": 50,
        "offset": 25,
        "order_by": "Cmc",
        "ascending": false,
        "synergy": true
    }"#;

    #[test]
    fn old_client_json_deserializes_into_card_query() {
        let query: CardQuery = serde_json::from_str(OLD_CLIENT_JSON).unwrap();
        assert_eq!(query.criteria().name_contains(), Some("bolt"));
        assert_eq!(query.criteria().cmc_range(), Some((1.0, 3.0)));
        assert_eq!(query.limit(), 50);
        assert_eq!(query.offset(), 25);
        assert_eq!(query.sort(), Some(CardSortKey::Cmc));
        assert!(!query.ascending());
        assert!(query.synergy());
    }

    #[test]
    fn card_query_serializes_to_the_old_flat_shape() {
        let mut builder = CardQueryBuilder::with_name_contains("bolt");
        builder.set_cmc_range((1.0, 3.0));
        builder.set_limit(50);
        builder.set_offset(25);
        builder.set_sort(CardSortKey::Cmc);
        builder.set_ascending(false);
        builder.set_synergy(true);
        let query = builder.build().unwrap();

        let value = serde_json::to_value(&query).unwrap();
        let expected = serde_json::json!({
            "name_contains": "bolt",
            "cmc_range": [1.0, 3.0],
            // builder defaults, present pre-split too
            "is_playable": true,
            "digital": false,
            "oversized": false,
            "content_warning": false,
            "language": "en",
            // config, flat at the top level
            "limit": 50,
            "offset": 25,
            "order_by": "Cmc",
            "ascending": false,
            "synergy": true
        });
        assert_eq!(value, expected);
    }

    #[test]
    fn unset_criteria_are_omitted_from_the_wire() {
        let query = CardQueryBuilder::with_name_contains("bolt")
            .build()
            .unwrap();
        let value = serde_json::to_value(&query).unwrap();
        let obj = value.as_object().unwrap();
        assert!(!obj.contains_key("cmc_range"));
        assert!(!obj.contains_key("order_by"), "None sort must be omitted");
        assert!(obj.contains_key("limit"));
    }

    #[test]
    fn synergy_defaults_false_when_omitted() {
        // A client predating the synergy flag omits it; must parse to false
        // (full-pool behavior), keeping the endpoint server-first safe.
        let query: CardQuery = serde_json::from_str(r#"{"name_contains":"bolt"}"#).unwrap();
        assert!(!query.synergy());
        // And the config defaults hold.
        assert_eq!(query.limit(), 25);
        assert_eq!(query.offset(), 0);
        assert!(query.ascending());
    }

    #[test]
    fn wire_limit_is_clamped_on_deserialize() {
        let query: CardQuery =
            serde_json::from_str(r#"{"name_contains":"bolt","limit":10000}"#).unwrap();
        assert_eq!(query.limit(), Limit::MAX);
    }

    #[test]
    fn limit_new_clamps() {
        assert_eq!(Limit::new(10_000).get(), 250);
        assert_eq!(Limit::new(25).get(), 25);
    }
}
