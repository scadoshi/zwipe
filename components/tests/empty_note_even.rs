//! An empty panel's note (`.activity-total`, such as "Swipe more cards to see your Oracle tags")
//! sits evenly in its panel: it adds no room of its own above or below, so the panel body's room
//! over it matches the panel's own padding under it. Alone under a card group's header, where no
//! panel body spaces it, it takes the same room on both sides.

const SCREENS_B: &str = include_str!("../assets/screens/screens-b.css");

fn rule(selector: &str) -> &'static str {
    SCREENS_B
        .split_once(&format!("\n{selector} {{"))
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_panel_s_empty_note_adds_no_room_above_or_below() {
    let note = rule(".activity-total");
    assert!(note.contains("padding: 0 var(--space-3);"), "{note}");
}

#[test]
fn alone_under_a_group_header_it_takes_even_room() {
    let group = rule(".activity-total-group");
    assert!(group.contains("padding-top: var(--space-3);"), "{group}");
    assert!(group.contains("padding-bottom: var(--space-3);"), "{group}");
}
