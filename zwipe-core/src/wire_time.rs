//! Wire-format compatibility for timestamps.
//!
//! The database stores `TIMESTAMPTZ` and Rust models use `DateTime<Utc>`, but
//! the JSON wire format must stay byte-identical to the legacy `NaiveDateTime`
//! representation — no `Z`/offset suffix — so that already-installed mobile
//! clients (which still deserialize these fields as `NaiveDateTime`) keep
//! working. `chrono`'s `NaiveDateTime` deserializer rejects a trailing `Z`, so
//! emitting one would break every un-updated app on login and card search.
//!
//! - **Serialize**: delegate to `NaiveDateTime`'s own impl via `naive_utc()`,
//!   so the output is identical to the pre-migration format.
//! - **Deserialize**: lenient — accept the bare no-offset form (assumed UTC)
//!   *and* an RFC3339 `Z`/offset form, so a future coordinated switch to a `Z`
//!   wire format needs no second change on the read path.
//!
//! Apply with `#[serde(with = "crate::wire_time::utc")]` on `DateTime<Utc>`
//! fields and `#[serde(with = "crate::wire_time::utc_opt")]` on
//! `Option<DateTime<Utc>>` fields that cross the wire to clients.

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Parses either a bare naive timestamp (assumed UTC) or an RFC3339 value.
fn parse_utc<E: serde::de::Error>(s: &str) -> Result<DateTime<Utc>, E> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    s.parse::<NaiveDateTime>()
        .map(|n| n.and_utc())
        .map_err(serde::de::Error::custom)
}

/// `#[serde(with = "...")]` adapter for `DateTime<Utc>`.
pub mod utc {
    use super::*;

    pub fn serialize<S: Serializer>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error> {
        dt.naive_utc().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<DateTime<Utc>, D::Error> {
        let s = String::deserialize(d)?;
        parse_utc(&s)
    }
}

/// `#[serde(with = "...")]` adapter for `Option<DateTime<Utc>>`.
pub mod utc_opt {
    use super::*;

    pub fn serialize<S: Serializer>(
        dt: &Option<DateTime<Utc>>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        dt.map(|d| d.naive_utc()).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        match Option::<String>::deserialize(d)? {
            Some(s) => parse_utc(&s).map(Some),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Wrap {
        #[serde(with = "utc")]
        at: DateTime<Utc>,
        #[serde(with = "utc_opt")]
        maybe: Option<DateTime<Utc>>,
    }

    fn sample() -> DateTime<Utc> {
        "2026-06-08T04:40:20Z".parse().unwrap()
    }

    #[test]
    fn serializes_without_z_suffix() {
        let w = Wrap { at: sample(), maybe: Some(sample()) };
        let json = serde_json::to_string(&w).unwrap();
        assert!(!json.contains('Z'), "wire format must not contain Z: {json}");
        assert!(json.contains("2026-06-08T04:40:20"));
    }

    #[test]
    fn deserializes_legacy_naive_form() {
        // Exactly what pre-migration servers/clients emit.
        let json = r#"{"at":"2026-06-08T04:40:20","maybe":"2026-06-08T04:40:20"}"#;
        let w: Wrap = serde_json::from_str(json).unwrap();
        assert_eq!(w.at, sample());
        assert_eq!(w.maybe, Some(sample()));
    }

    #[test]
    fn deserializes_rfc3339_z_form_too() {
        let json = r#"{"at":"2026-06-08T04:40:20Z","maybe":null}"#;
        let w: Wrap = serde_json::from_str(json).unwrap();
        assert_eq!(w.at, sample());
        assert_eq!(w.maybe, None);
    }

    #[test]
    fn round_trips_with_fractional_seconds() {
        let at = "2026-06-08T04:40:20.123456Z".parse::<DateTime<Utc>>().unwrap();
        let w = Wrap { at, maybe: None };
        let json = serde_json::to_string(&w).unwrap();
        assert!(!json.contains('Z'));
        let back: Wrap = serde_json::from_str(&json).unwrap();
        assert_eq!(w, back);
    }
}
