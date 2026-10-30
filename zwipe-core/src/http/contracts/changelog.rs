//! Changelog contracts (release-history feed).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::content::changelog::{DISPLAY_DATE_FORMAT, RELEASES, Release, UPCOMING};

/// One release, owned and serializable for the wire.
///
/// The compiled-in source ([`Release`]) borrows `&'static str`; this is its
/// owned projection so it can round-trip through JSON on the client.
///
/// `date` is the display string every app has always shown, so it keeps its
/// shape: apps up to 1.13.0 render it as is. `released_on` is the same day as
/// a machine date, added in 1.14.0 and absent from older servers, so read the
/// day through [`HttpRelease::released_on`], which falls back to `date`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HttpRelease {
    /// Semantic version string, e.g. `"1.6.0"`.
    pub version: String,
    /// Human-readable release date, e.g. `"Jul 12, 2026"`. Empty for an
    /// upcoming release.
    pub date: String,
    /// The release day as an ISO date, e.g. `"2026-07-12"`. Left off the wire
    /// for an upcoming release, and missing from servers before 1.14.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub released_on: Option<NaiveDate>,
    /// User-facing release notes, one bullet per entry.
    pub entries: Vec<String>,
}

impl HttpRelease {
    /// The day this release shipped: `released_on` when the server sent it,
    /// otherwise parsed from the display `date` an older server sends. `None`
    /// for an upcoming release, or a `date` that does not parse.
    pub fn released_on(&self) -> Option<NaiveDate> {
        self.released_on
            .or_else(|| NaiveDate::parse_from_str(&self.date, DISPLAY_DATE_FORMAT).ok())
    }
}

impl From<&Release> for HttpRelease {
    fn from(release: &Release) -> Self {
        Self {
            version: release.version.to_string(),
            date: release
                .date
                .map(|date| date.format(DISPLAY_DATE_FORMAT).to_string())
                .unwrap_or_default(),
            released_on: release.date,
            entries: release.entries.iter().map(|e| e.to_string()).collect(),
        }
    }
}

/// The full changelog feed, served publicly at `/api/changelog`.
///
/// Clients fetch this at startup and render it, falling back to the copy
/// compiled into their binary if the fetch fails. `upcoming` teases the next
/// release; `releases` is the shipped history, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HttpChangelog {
    /// Versions in progress for the next release (rendered with an "Upcoming"
    /// badge). Usually empty.
    pub upcoming: Vec<HttpRelease>,
    /// Shipped releases, newest first.
    pub releases: Vec<HttpRelease>,
}

impl HttpChangelog {
    /// Build the response from the compiled-in changelog data.
    pub fn current() -> Self {
        Self {
            upcoming: UPCOMING.iter().map(HttpRelease::from).collect(),
            releases: RELEASES.iter().map(HttpRelease::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_projects_the_compiled_in_source() {
        let changelog = HttpChangelog::current();
        assert_eq!(changelog.upcoming.len(), UPCOMING.len());
        assert_eq!(changelog.releases.len(), RELEASES.len());

        // Newest first, notes preserved.
        let latest = changelog.releases.first().expect("at least one release");
        assert_eq!(latest.version, RELEASES[0].version);
        assert_eq!(latest.released_on, RELEASES[0].date);
        assert_eq!(latest.entries.len(), RELEASES[0].entries.len());
    }

    #[test]
    fn round_trips_through_json() {
        let changelog = HttpChangelog::current();
        let json = serde_json::to_string(&changelog).expect("serialize");
        let parsed: HttpChangelog = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(changelog, parsed);
    }

    /// The display string keeps the exact shape apps up to 1.13.0 render.
    #[test]
    fn date_keeps_the_display_format() {
        let release = Release {
            version: "1.11.2",
            date: NaiveDate::from_ymd_opt(2026, 10, 9),
            entries: &[],
        };
        let json = serde_json::to_value(HttpRelease::from(&release)).expect("serialize");
        assert_eq!(json["date"], "Oct 9, 2026");
        assert_eq!(json["released_on"], "2026-10-09");
    }

    /// An upcoming release keeps its empty `date` and sends no `released_on`.
    #[test]
    fn upcoming_leaves_released_on_off_the_wire() {
        let release = Release {
            version: "9.9.9",
            date: None,
            entries: &[],
        };
        let json = serde_json::to_value(HttpRelease::from(&release)).expect("serialize");
        assert_eq!(json["date"], "");
        assert!(json.get("released_on").is_none(), "{json}");
    }

    /// A server before 1.14.0 sends no `released_on`: it parses, and the day
    /// comes from the display string.
    #[test]
    fn released_on_falls_back_to_the_display_date() {
        let json = r#"{"version":"1.13.0","date":"Oct 22, 2026","entries":["x"]}"#;
        let release: HttpRelease = serde_json::from_str(json).expect("deserialize");
        assert_eq!(release.released_on, None);
        assert_eq!(release.released_on(), NaiveDate::from_ymd_opt(2026, 10, 22));
    }

    /// When both are present, `released_on` wins.
    #[test]
    fn released_on_prefers_the_machine_date() {
        let json =
            r#"{"version":"1.14.0","date":"whenever","released_on":"2026-11-02","entries":[]}"#;
        let release: HttpRelease = serde_json::from_str(json).expect("deserialize");
        assert_eq!(release.released_on(), NaiveDate::from_ymd_opt(2026, 11, 2));
    }

    /// An upcoming release from any server has no day.
    #[test]
    fn upcoming_has_no_day() {
        let json = r#"{"version":"1.14.0","date":"","entries":[]}"#;
        let release: HttpRelease = serde_json::from_str(json).expect("deserialize");
        assert_eq!(release.released_on(), None);
    }

    /// Every compiled-in display date parses back to its own day, so the
    /// fallback agrees with `released_on` for this server's history.
    #[test]
    fn every_display_date_parses_back() {
        for release in HttpChangelog::current().releases {
            let parsed = NaiveDate::parse_from_str(&release.date, DISPLAY_DATE_FORMAT).ok();
            assert_eq!(parsed, release.released_on, "{}", release.version);
        }
    }
}
