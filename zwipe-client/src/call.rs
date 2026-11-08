//! The one place an API request is built, sent and decoded.
//!
//! Every endpoint goes through [`ZwipeClient::call`]; the per-endpoint files
//! describe the call and hold no transport code.

use chrono::{DateTime, TimeDelta, Utc};
use reqwest::header::{AGE, HeaderMap, HeaderName, LAST_MODIFIED};
use tracing::info;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::endpoint::{Endpoint, Method};

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Sends `endpoint` and decodes its success body.
    ///
    /// `session` is required when `E::AUTH`; passing `None` there is a bug in
    /// the caller, so it fails as an unauthorized error rather than sending an
    /// anonymous request the server would reject anyway.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::Unauthorized`] when `E::AUTH` is set and `session` is `None`, and
    /// otherwise a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn call<E: Endpoint>(
        &self,
        endpoint: E,
        session: Option<&Session>,
    ) -> Result<E::Response, ClientError> {
        let response = self.send(&endpoint, session).await?;
        decode::<E>(response).await
    }

    /// Sends `endpoint` like [`Self::call`] and answers with its success body
    /// and when that copy was made: the response's `Age` before now when the
    /// caller can read it, else its `Last-Modified`, which an edge cache sets
    /// to when it fetched the copy. `None` when the response carries neither,
    /// as one straight from the origin does: the copy is new.
    ///
    /// # Errors
    ///
    /// As [`Self::call`].
    pub async fn call_dated<E: Endpoint>(
        &self,
        endpoint: E,
        session: Option<&Session>,
    ) -> Result<Dated<E::Response>, ClientError> {
        let response = self.send(&endpoint, session).await?;
        let made_at = made_at(response.headers(), Utc::now());
        let value = decode::<E>(response).await?;
        Ok(Dated { value, made_at })
    }

    /// Builds and sends `endpoint`'s request.
    async fn send<E: Endpoint>(
        &self,
        endpoint: &E,
        session: Option<&Session>,
    ) -> Result<reqwest::Response, ClientError> {
        let mut url = self.base_url.clone();
        url.set_path(&endpoint.path());
        url.set_query(endpoint.query().as_deref());

        let mut request = match E::METHOD {
            Method::Get => self.client.get(url.clone()),
            Method::Post => self.client.post(url.clone()),
            Method::Put => self.client.put(url.clone()),
            Method::Patch => self.client.patch(url.clone()),
            Method::Delete => self.client.delete(url.clone()),
        };

        if E::AUTH {
            let Some(session) = session else {
                return Err(ClientError::Unauthorized("not logged in".to_string()));
            };
            request = request.bearer_auth(session.access_token.value.as_str());
        }

        if let Some(body) = endpoint.body() {
            request = request.json(body);
        }

        // The browser keeps cross-origin cookies out of a fetch unless asked;
        // only the web session endpoints ask, so no other call sends one.
        #[cfg(target_arch = "wasm32")]
        if E::CREDENTIALS {
            request = request.fetch_credentials_include();
        }
        // A browser may hold a copy longer than the server's header says
        // (an edge can raise max-age), so a call that changes on a deploy
        // asks it to check with the server first; a current copy is a 304.
        #[cfg(target_arch = "wasm32")]
        if E::REVALIDATE {
            request = request.fetch_cache_no_cache();
        }

        info!("{} {}", E::METHOD.as_str(), url);
        hold_for_skeleton_audit(E::METHOD).await;
        Ok(request.send().await?)
    }
}

/// A response body with when the copy was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dated<T> {
    /// The decoded body.
    pub value: T,
    /// When the copy was made, `None` when the response does not say and it
    /// is new.
    pub made_at: Option<DateTime<Utc>>,
}

/// When a response's copy was made, read at `now`: `Age` seconds before
/// `now`, else `Last-Modified`. An `Age` that is not a whole number of
/// seconds, a negative one included, is ignored.
fn made_at(headers: &HeaderMap, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let text = |name: HeaderName| headers.get(name).and_then(|v| v.to_str().ok());
    let aged = text(AGE)
        .and_then(|age| age.trim().parse::<u32>().ok())
        .and_then(|age| TimeDelta::try_seconds(i64::from(age)))
        .and_then(|age| now.checked_sub_signed(age));
    aged.or_else(|| {
        text(LAST_MODIFIED)
            .and_then(|date| DateTime::parse_from_rfc2822(date).ok())
            .map(|date| date.with_timezone(&Utc))
    })
}

/// Holds every GET for `SKELETON_DELAY_MS` when the app is built with it set,
/// so each loading skeleton stays up long enough to look at. Unset, which is
/// every shipped build, this is nothing. Writes are never held.
#[cfg(not(target_arch = "wasm32"))]
async fn hold_for_skeleton_audit(method: Method) {
    let ms = option_env!("SKELETON_DELAY_MS").and_then(|ms| ms.parse::<u64>().ok());
    if let (Method::Get, Some(ms)) = (method, ms) {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    }
}

#[cfg(target_arch = "wasm32")]
#[expect(
    clippy::unused_async,
    reason = "keeps the native version's async signature, which sleeps"
)]
async fn hold_for_skeleton_audit(_method: Method) {}

/// Splits the response into success and failure, and turns a success body into
/// `E::Response`. Any 2xx is success: several endpoints answer 200 or 204 for
/// the same call, and no caller branches on which. An empty body reads as
/// `null` so those can declare `type Response = ()`.
async fn decode<E: Endpoint>(response: reqwest::Response) -> Result<E::Response, ClientError> {
    let status = response.status();
    if !status.is_success() {
        let message = response.text().await?;
        return Err((status, message).into());
    }

    let bytes = response.bytes().await?;
    let decoded = if bytes.is_empty() {
        serde_json::from_str::<E::Response>("null")?
    } else {
        serde_json::from_slice::<E::Response>(&bytes)?
    };
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderValue;

    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn an_edge_copy_is_dated_by_its_age() {
        let mut headers = HeaderMap::new();
        headers.insert("age", HeaderValue::from_static("600"));
        headers.insert(
            "last-modified",
            HeaderValue::from_static("Sat, 07 Nov 2026 07:00:00 GMT"),
        );
        let now = at("2026-11-07T08:00:00Z");
        assert_eq!(made_at(&headers, now), Some(at("2026-11-07T07:50:00Z")));
    }

    #[test]
    fn a_negative_age_is_ignored_for_the_last_modified() {
        let mut headers = HeaderMap::new();
        headers.insert("age", HeaderValue::from_static("-600"));
        headers.insert(
            "last-modified",
            HeaderValue::from_static("Sat, 07 Nov 2026 07:00:00 GMT"),
        );
        let now = at("2026-11-07T08:00:00Z");
        assert_eq!(made_at(&headers, now), Some(at("2026-11-07T07:00:00Z")));
    }

    #[test]
    fn without_a_readable_age_the_last_modified_dates_it() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "last-modified",
            HeaderValue::from_static("Sat, 07 Nov 2026 07:16:43 GMT"),
        );
        assert_eq!(
            made_at(&headers, at("2026-11-07T08:00:00Z")),
            Some(at("2026-11-07T07:16:43Z"))
        );
    }

    #[test]
    fn a_response_without_either_is_new() {
        assert_eq!(made_at(&HeaderMap::new(), at("2026-11-07T08:00:00Z")), None);
    }
}
