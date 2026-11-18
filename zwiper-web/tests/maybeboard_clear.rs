//! The Maybeboard list's Clear is off while there is nothing on it to clear, as the app's is.

/// The Commanders screen's Maybeboard list's source.
const SRC: &str = include_str!("../src/lib/inbound/screens/commanders/maybeboard.rs");

#[test]
fn an_empty_maybeboard_s_clear_is_off() {
    let action = SRC
        .split("DangerZoneAction {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .unwrap_or_default();
    assert!(
        action.contains("disabled: !has_entries,"),
        "Clear is off with no commanders saved: {action}"
    );
}
