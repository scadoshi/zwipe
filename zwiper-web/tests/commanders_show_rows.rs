//! Commanders' two lists keep only All and Art in their Show rows: colors come from the screen's
//! one shared filter, so neither list offers its own color-identity or color chips.

/// The Suggested list's source.
const SUGGESTED: &str = include_str!("../src/lib/inbound/screens/commanders/suggested.rs");

/// The Maybeboard list's source.
const MAYBEBOARD: &str = include_str!("../src/lib/inbound/screens/commanders/maybeboard.rs");

/// The chip-row in `src` that the `Show` label opens, up to the row after it.
fn show_row(src: &str) -> &str {
    let (_, rest) = src
        .split_once(r#"chip-row-label", "Show" }"#)
        .unwrap_or_default();
    rest.split_once(r#"class: "chip-row""#)
        .map_or(rest, |(row, _)| row)
}

/// The chip labels in `row`, in order: each line that is only a quoted label.
fn labels(row: &str) -> Vec<&str> {
    row.lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix('"')?.strip_suffix('"'))
        .collect()
}

#[test]
fn suggested_shows_all_and_art_without_identity_chips() {
    assert!(
        !SUGGESTED.contains("ShowIdentityChips"),
        "no color-identity chips"
    );
    let row = show_row(SUGGESTED);
    assert!(!row.is_empty(), "a Show row");
    assert!(!row.contains("IdentityPips"), "{row}");
    assert_eq!(labels(row), ["All", "Art"], "{row}");
}

#[test]
fn maybeboard_shows_all_and_art_without_color_chips() {
    let row = show_row(MAYBEBOARD);
    assert!(!row.is_empty(), "a Show row");
    assert!(!row.contains("Color::all()"), "no color pips: {row}");
    assert!(!row.contains("ms-cost"), "no mana pips: {row}");
    assert_eq!(labels(row), ["All", "Art"], "{row}");
}
