//! The card page's stats column on a wide window is docked at the left as the details are at the
//! right: it ends above the docks' line and scrolls inside itself, and it shrinks from its foot
//! as the footer scrolls in, so every widget can be brought into view at any window height.

#![expect(
    clippy::panic,
    reason = "a test: a rule that is missing fails the test"
)]

const WEB: &str = include_str!("../assets/web.css");

/// The declarations of the first rule whose selector list, after `after`, is exactly `selector`.
fn rule<'css>(after: &str, selector: &str) -> &'css str {
    let (_, rest) = WEB.split_once(after).unwrap_or_else(|| panic!("{after}"));
    let (_, rest) = rest
        .split_once(&format!("\n    {selector} {{"))
        .unwrap_or_else(|| panic!("a rule for {selector}"));
    rest.split_once('}').unwrap_or_else(|| panic!("its end")).0
}

#[test]
fn the_stats_column_scrolls_inside_itself_above_the_docks() {
    let stats = rule(
        "@media (min-width: 900px) {\n    /* The stats and details panels",
        ".web-card-stats",
    );
    for declaration in [
        "max-height: calc(100dvh - var(--web-dock-panel-top) - var(--web-dock-panel-bottom));",
        "overflow-y: auto;",
        "overscroll-behavior: contain;",
    ] {
        assert!(stats.contains(declaration), "{declaration} in {stats}");
    }
}

#[test]
fn the_stats_column_shrinks_as_the_footer_scrolls_in() {
    let (_, rest) = WEB
        .split_once("@media (min-width: 900px) {\n        .web-scroll-screen .web-swipe-details")
        .unwrap_or_else(|| panic!("the details' rise"));
    let (selectors, _) = rest.split_once('{').unwrap_or_else(|| panic!("its block"));
    assert!(
        selectors.contains(".web-scroll-screen .web-card-stats"),
        "the stats column rises with the details: {selectors}"
    );
}

#[test]
fn each_widget_keeps_its_height_and_the_column_takes_no_entrance() {
    let kids = rule(
        "@media (min-width: 900px) {\n    /* The stats and details panels",
        ".web-card-stats > *",
    );
    assert!(kids.contains("flex-shrink: 0;"), "{kids}");
    assert!(
        WEB.contains("> :not(.web-swipe-details, .web-examples-details, .web-card-stats) {\n        animation: content-enter"),
        "the column, fixed like the details, skips the swipe body's entrance, which would replace its rise"
    );
}
