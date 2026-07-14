//! Changelog contracts (release-history feed).

use crate::content::changelog::{RELEASES, Release, UPCOMING};
use serde::{Deserialize, Serialize};

/// One release, owned and serializable for the wire.
///
/// The compiled-in source ([`Release`]) borrows `&'static str`; this is its
/// owned projection so it can round-trip through JSON on the client.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HttpRelease {
    /// Semantic version string, e.g. `"1.6.0"`.
    pub version: String,
    /// Human-readable release date, e.g. `"Jul 12, 2026"`.
    pub date: String,
    /// User-facing release notes, one bullet per entry.
    pub entries: Vec<String>,
}

impl From<&Release> for HttpRelease {
    fn from(release: &Release) -> Self {
        Self {
            version: release.version.to_string(),
            date: release.date.to_string(),
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
#[allow(clippy::expect_used, clippy::indexing_slicing)]
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
        assert_eq!(latest.date, RELEASES[0].date);
        assert_eq!(latest.entries.len(), RELEASES[0].entries.len());
    }

    #[test]
    fn round_trips_through_json() {
        let changelog = HttpChangelog::current();
        let json = serde_json::to_string(&changelog).expect("serialize");
        let parsed: HttpChangelog = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(changelog, parsed);
    }
}
