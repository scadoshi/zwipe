//! Send to's boards row sits as far above its Search row as the Search row sits above the deck
//! chips: one spacing token for both gaps.

const SCREENS: &str = include_str!("../assets/screens/screens-b.css");

/// The declarations of the first rule whose selector is exactly `selector`, if there is one.
fn rule<'css>(css: &'css str, selector: &str) -> Option<&'css str> {
    let (_, rest) = css.split_once(&format!("\n{selector} {{"))?;
    let (body, _) = rest.split_once('}')?;
    Some(body)
}

/// The value `property` takes in `declarations`, if it is declared there.
fn value<'css>(declarations: &'css str, property: &str) -> Option<&'css str> {
    let prefix = format!("{property}:");
    declarations
        .lines()
        .find_map(|line| line.trim().strip_prefix(prefix.as_str()))
        .map(|v| v.trim().trim_end_matches(';'))
}

#[test]
fn the_boards_row_clears_the_search_row_by_the_gap_above_the_chips() {
    let grid = rule(SCREENS, ".tag-grid").unwrap_or_default();
    let above_chips = value(grid, "padding")
        .and_then(|padding| padding.split_whitespace().next())
        .unwrap_or_default();
    assert_eq!(
        above_chips, "var(--space-2)",
        "the chips' top padding: {grid}"
    );
    let boards = rule(SCREENS, ".tag-controls > .send-to-boards").unwrap_or_default();
    assert_eq!(
        value(boards, "margin-bottom"),
        Some(above_chips),
        "the boards row's gap below it: {boards}"
    );
}
