//! App version comparison for the client min-version gate.
//!
//! Deliberately tiny: versions are `x.y.z` numeric tuples (missing segments
//! read as 0). No `semver` crate — Zwipe versions never carry pre-release or
//! build metadata. Both sides use this: the client compares its own version
//! against the server's minimum, and tests pin the semantics.

/// Parses an `x.y.z` version string into a numeric tuple.
///
/// Tolerates fewer than three segments (`"1.2"` → `(1, 2, 0)`). Returns
/// `None` if any present segment isn't a non-negative integer.
pub fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut segments = version.trim().splitn(3, '.');
    let mut next = |missing_ok: bool| -> Option<u64> {
        match segments.next() {
            Some(segment) => segment.trim().parse().ok(),
            None if missing_ok => Some(0),
            None => None,
        }
    };
    let major = next(false)?;
    let minor = next(true)?;
    let patch = next(true)?;
    Some((major, minor, patch))
}

/// Whether `current` satisfies `minimum` (numeric tuple compare).
///
/// **Fails open**: if either side doesn't parse, returns `true` — a malformed
/// version string must never lock users out of the app.
pub fn version_at_least(current: &str, minimum: &str) -> bool {
    match (parse_version(current), parse_version(minimum)) {
        (Some(current), Some(minimum)) => current >= minimum,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_three_segments() {
        assert_eq!(parse_version("1.0.4"), Some((1, 0, 4)));
    }

    #[test]
    fn missing_segments_read_as_zero() {
        assert_eq!(parse_version("1.2"), Some((1, 2, 0)));
        assert_eq!(parse_version("2"), Some((2, 0, 0)));
    }

    #[test]
    fn garbage_does_not_parse() {
        assert_eq!(parse_version("1.0.4-beta"), None);
        assert_eq!(parse_version("abc"), None);
        assert_eq!(parse_version(""), None);
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
    fn numeric_not_lexicographic() {
        assert!(version_at_least("1.0.10", "1.0.9"));
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
