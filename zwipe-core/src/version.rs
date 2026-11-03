//! App versions: the changelog's release numbers and the client min-version
//! gate.
//!
//! A version is an `x.y.z` numeric triple. No `semver` crate; Zwipe versions
//! never carry pre-release or build metadata.

use std::fmt;
use std::str::FromStr;

use thiserror::Error;

/// An `x.y.z` app version. Orders numerically, so `1.0.10` is newer than
/// `1.0.9`. Every triple is valid; only parsing a string can fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    /// Bumped for a change to what the product is.
    pub major: u64,
    /// Bumped when a release adds anything a user can see.
    pub minor: u64,
    /// Bumped when a release only fixes or polishes.
    pub patch: u64,
}

impl Version {
    /// The version `major.minor.patch`.
    #[must_use]
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A string that is not an `x.y.z` version.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid version {0:?} (expected x.y.z)")]
pub struct InvalidVersion(pub String);

/// Segments in an `x.y.z` version: major, minor and patch.
const VERSION_SEGMENTS: usize = 3;

impl FromStr for Version {
    type Err = InvalidVersion;

    /// Tolerates surrounding whitespace and fewer than three segments
    /// (`"1.2"` reads as `1.2.0`).
    fn from_str(version: &str) -> Result<Self, Self::Err> {
        let invalid = || InvalidVersion(version.to_owned());
        let mut segments = version.trim().splitn(VERSION_SEGMENTS, '.');
        let mut next = |missing_ok: bool| -> Option<u64> {
            match segments.next() {
                Some(segment) => segment.trim().parse().ok(),
                None if missing_ok => Some(0),
                None => None,
            }
        };
        let major = next(false).ok_or_else(invalid)?;
        let minor = next(true).ok_or_else(invalid)?;
        let patch = next(true).ok_or_else(invalid)?;
        Ok(Self::new(major, minor, patch))
    }
}

/// Whether `current` satisfies `minimum`.
///
/// **Fails open**: if either side doesn't parse, returns `true`; a malformed
/// version string must never lock users out of the app.
#[must_use]
pub fn version_at_least(current: &str, minimum: &str) -> bool {
    match (current.parse::<Version>(), minimum.parse::<Version>()) {
        (Ok(current), Ok(minimum)) => current >= minimum,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(version: &str) -> Option<Version> {
        version.parse().ok()
    }

    #[test]
    fn parses_three_segments() {
        assert_eq!(parse("1.0.4"), Some(Version::new(1, 0, 4)));
    }

    #[test]
    fn missing_segments_read_as_zero() {
        assert_eq!(parse("1.2"), Some(Version::new(1, 2, 0)));
        assert_eq!(parse("2"), Some(Version::new(2, 0, 0)));
    }

    #[test]
    fn garbage_does_not_parse() {
        assert_eq!(parse("1.0.4-beta"), None);
        assert_eq!(parse("abc"), None);
        assert_eq!(parse(""), None);
        assert_eq!(parse(&"9".repeat(4096)), None);
    }

    #[test]
    fn displays_as_the_triple() {
        assert_eq!(
            parse(" 1.11.2 ").map(|v| v.to_string()),
            Some("1.11.2".to_owned())
        );
        assert_eq!(
            parse("1.2").map(|v| v.to_string()),
            Some("1.2.0".to_owned())
        );
    }

    #[test]
    fn orders_numerically_not_lexicographically() {
        assert!(Version::new(1, 0, 10) > Version::new(1, 0, 9));
        assert!(Version::new(1, 1, 0) > Version::new(1, 0, 9));
        assert!(Version::new(2, 0, 0) > Version::new(1, 9, 9));
    }

    #[test]
    fn equal_versions_satisfy() {
        assert!(version_at_least("1.0.5", "1.0.5"));
    }

    #[test]
    fn newer_satisfies() {
        assert!(version_at_least("1.0.6", "1.0.5"));
        assert!(version_at_least("1.1.0", "1.0.9"));
        assert!(version_at_least("2.0.0", "1.9.9"));
    }

    #[test]
    fn older_does_not_satisfy() {
        assert!(!version_at_least("1.0.4", "1.0.5"));
        assert!(!version_at_least("1.0.9", "1.1.0"));
        assert!(!version_at_least("1.9.9", "2.0.0"));
    }

    #[test]
    fn gate_open_sentinel_allows_everyone() {
        assert!(version_at_least("0.0.1", "0.0.0"));
        assert!(version_at_least("1.0.4", "0.0.0"));
    }

    #[test]
    fn unparseable_fails_open() {
        assert!(version_at_least("garbage", "1.0.5"));
        assert!(version_at_least("1.0.5", "garbage"));
    }
}
