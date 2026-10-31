//! The minimum-version gate.

use zwipe_core::version;

/// What the gate should become after the server names `minimum`, or `None`
/// when it stays as `current`. The gate goes both ways: a lowered minimum
/// lets a blocked build back in. A version either side cannot parse never
/// blocks.
pub fn gate_change(current: bool, app_version: &str, minimum: &str) -> Option<bool> {
    let required = !version::version_at_least(app_version, minimum);
    (required != current).then_some(required)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_build_below_the_minimum_closes_the_gate() {
        assert_eq!(gate_change(false, "1.15.9", "1.16.0"), Some(true));
    }

    #[test]
    fn a_lowered_minimum_opens_it_again() {
        assert_eq!(gate_change(true, "1.16.2", "1.16.0"), Some(false));
    }

    #[test]
    fn no_change_is_none() {
        assert_eq!(gate_change(false, "1.16.2", "1.16.2"), None);
        assert_eq!(gate_change(true, "1.0.0", "2.0.0"), None);
    }

    #[test]
    fn an_unparsable_minimum_never_blocks() {
        assert_eq!(gate_change(false, "1.16.2", "soon"), None);
        assert_eq!(gate_change(true, "1.16.2", "soon"), Some(false));
    }
}
