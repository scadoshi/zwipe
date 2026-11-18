//! The commander maybeboard's Clear is off while there is nothing on it to clear, as the app's is.

/// The commander maybeboard screen's source.
const SRC: &str = include_str!("../src/lib/inbound/screens/commander_maybeboard.rs");

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
