//! Quick add's field keeps its width when its first letter brings in the ×: the × keeps its room
//! while the field is empty and only fades in.

/// The filter sheet both clients load.
const FILTER: &str = include_str!("../assets/filter.css");

#[test]
fn the_clear_button_keeps_its_room_beside_quick_add() {
    let selector =
        ".input-clear-wrap:has(> #quick-add, > #commander-maybeboard-search) > .input-clear-btn {";
    let at = FILTER
        .find(selector)
        .unwrap_or_else(|| panic!("no rule {selector}"));
    let body = FILTER
        .get(at..)
        .and_then(|s| s.split('}').next())
        .unwrap_or_default();
    assert!(body.contains("width: 2.2rem;"), "{body}");
    assert!(body.contains("margin-left: var(--space-2);"), "{body}");
}
