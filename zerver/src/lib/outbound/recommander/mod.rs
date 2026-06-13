//! Recommander API adapter: implements
//! [`CardRecommender`](crate::domain::recommendation::ports::CardRecommender)
//! via the public Recommander API (<https://recommander.cards/api>).
//!
//! Called live and deck-aware for mature decks (25+ non-land mainboard cards).
//! Commercial use is sanctioned for zwipe; attribution is required by their
//! terms. Any failure here is non-fatal — the caller falls back to the cached
//! synergy signal — so the tight per-request timeout doubles as the "it's too
//! slow, fall back" guardrail. See `plans/recommander-integration.md`.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::recommendation::{
    models::{CardRecommendation, RecommendError, RecommendQuery},
    ports::CardRecommender,
};

/// Recommander HTTP API adapter.
///
/// Construct once and clone into services — the inner [`reqwest::Client`] is a
/// connection pool. The per-request timeout is the fall-back trigger.
#[derive(Debug, Clone)]
pub struct Recommander {
    client: reqwest::Client,
    base_url: String,
    enabled: bool,
}

impl Recommander {
    /// Creates a new Recommander adapter.
    ///
    /// * `base_url` — e.g. `https://api.recommander.cards/public-release`.
    /// * `timeout` — per-request budget; on expiry the call errors and the
    ///   caller falls back to the cached signal.
    /// * `enabled` — kill switch; when false, [`recommend`](Self::recommend)
    ///   short-circuits to [`RecommendError::Disabled`] with no network call.
    pub fn new(base_url: String, timeout: Duration, enabled: bool) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_default();
        Self {
            client,
            base_url,
            enabled,
        }
    }
}

/// Request body — cards encoded as oracle_id strings.
#[derive(Serialize)]
struct WireQuery {
    card_format: &'static str,
    commander: String,
    partner: Option<String>,
    deck: Vec<String>,
}

/// Standard API envelope (`ApiResult<RecommendResult>`).
#[derive(Deserialize)]
struct WireEnvelope {
    result_code: String,
    #[serde(default)]
    data: Option<WireResult>,
    #[serde(default)]
    error: Option<WireError>,
}

#[derive(Deserialize)]
struct WireResult {
    #[serde(default)]
    recommendations: Vec<WireRecommendation>,
}

#[derive(Deserialize)]
struct WireRecommendation {
    oracle_id: Uuid,
    name: String,
    score: f64,
}

#[derive(Deserialize)]
struct WireError {
    #[serde(default)]
    messages: Vec<String>,
}

impl CardRecommender for Recommander {
    async fn recommend(
        &self,
        query: RecommendQuery,
    ) -> Result<Vec<CardRecommendation>, RecommendError> {
        if !self.enabled {
            return Err(RecommendError::Disabled);
        }

        let body = WireQuery {
            card_format: "oracle_id",
            commander: query.commander.to_string(),
            partner: query.partner.map(|p| p.to_string()),
            deck: query.deck.iter().map(Uuid::to_string).collect(),
        };

        let url = format!("{}/api/decks/recommend/top", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RecommendError::Network(e.into()))?;

        // Parse the body once, regardless of status: the API wraps errors in
        // the same envelope (e.g. result_code = error_rate_limited on a 429).
        // A non-JSON body on an error status (gateway 429/5xx) still degrades
        // to a non-fatal Api error so the caller falls back.
        let status = response.status().as_u16();
        let text = response
            .text()
            .await
            .map_err(|e| RecommendError::Network(e.into()))?;
        let envelope: WireEnvelope = match serde_json::from_str(&text) {
            Ok(env) => env,
            Err(_) if status >= 400 => {
                return Err(RecommendError::Api {
                    status,
                    code: format!("http_{status}"),
                    messages: vec![],
                });
            }
            Err(e) => return Err(RecommendError::Parse(e.into())),
        };

        if envelope.result_code != "success" {
            return Err(RecommendError::Api {
                status,
                code: envelope.result_code,
                messages: envelope.error.map(|e| e.messages).unwrap_or_default(),
            });
        }

        let recommendations = envelope
            .data
            .map(|d| d.recommendations)
            .unwrap_or_default()
            .into_iter()
            .map(|r| CardRecommendation {
                oracle_id: r.oracle_id,
                name: r.name,
                score: r.score,
            })
            .collect();
        Ok(recommendations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_success_envelope() {
        let json = r#"{
            "result_code": "success",
            "data": { "recommendations": [
                { "oracle_id": "795b096a-2bce-4588-a2c9-abc5ea40dc0c", "name": "Enchantress's Presence", "score": 0.9998 }
            ] },
            "error": null
        }"#;
        let env: WireEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(env.result_code, "success");
        assert_eq!(env.data.unwrap().recommendations.len(), 1);
    }

    #[test]
    fn parses_error_envelope() {
        let json = r#"{ "result_code": "error_rate_limited", "data": null, "error": { "messages": ["slow down"] } }"#;
        let env: WireEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(env.result_code, "error_rate_limited");
        assert!(env.data.is_none());
        assert_eq!(env.error.unwrap().messages, vec!["slow down".to_string()]);
    }

    #[test]
    fn tolerates_missing_optional_fields() {
        // A boot/model-loading response may omit data and error entirely.
        let env: WireEnvelope = serde_json::from_str(r#"{ "result_code": "error_booting" }"#).unwrap();
        assert!(env.data.is_none());
        assert!(env.error.is_none());
    }

    #[tokio::test]
    async fn disabled_short_circuits_without_network() {
        let rec = Recommander::new(
            "http://127.0.0.1:0".to_string(),
            Duration::from_millis(50),
            false,
        );
        let err = rec
            .recommend(RecommendQuery {
                commander: Uuid::nil(),
                partner: None,
                deck: vec![],
            })
            .await
            .unwrap_err();
        assert!(matches!(err, RecommendError::Disabled));
    }
}
