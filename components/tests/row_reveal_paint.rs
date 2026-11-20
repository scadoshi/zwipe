//! An expanded card row is a layer of its own, so a reveal easing open inside it repaints the
//! whole row. Android's web view at version 133 otherwise leaves stale tiles behind when a role chip opens
//! its oracle tags: the action row drawn twice and the tags' first row missing, while the page
//! underneath is right.

#![expect(
    clippy::panic,
    reason = "test helpers: a rule that is missing fails the test"
)]

const MAGIC: &str = include_str!("../assets/magic.css");

/// The declarations of the rule whose selector is exactly `selector` in `css`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let opening = format!("\n{selector} {{");
    let (_, rest) = css
        .split_once(&opening)
        .unwrap_or_else(|| panic!("a rule for {selector}"));
    let (body, _) = rest
        .split_once('}')
        .unwrap_or_else(|| panic!("the end of {selector}"));
    body
}

#[test]
fn an_expanded_row_is_its_own_layer() {
    let expanded = rule(MAGIC, ".card-row.expanded");
    assert!(
        expanded.contains("will-change: transform;"),
        "the open row repaints as one layer: {expanded}"
    );
}

#[test]
fn only_the_open_row_is_a_layer() {
    let row = rule(MAGIC, ".card-row");
    assert!(
        !row.contains("will-change"),
        "a closed row stays in the list's own layer, so a long list costs no layer per row: {row}"
    );
}
