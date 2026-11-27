//! The card page on a wide window is three columns that scroll as one page: the stats widgets at
//! the left, the card in the middle and its details at the right. No column scrolls inside itself,
//! and the card is sized by the window's height too, so it fits under the nav at any size.

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

const WIDE: &str = "@media (min-width: 900px) {\n    /* The card page is three columns";

#[test]
fn the_page_is_three_columns_that_scroll_as_one() {
    let page = rule(WIDE, ".web-cards-swipe.screen-content.web-card-page");
    assert!(
        page.contains(
            "grid-template-columns: var(--web-dock-panel-width) minmax(0, 1fr) var(--web-dock-panel-width);"
        ),
        "{page}"
    );
    assert!(page.contains("overflow: visible;"), "{page}");
}

#[test]
fn neither_side_column_scrolls_inside_itself() {
    let details = rule(WIDE, ".web-card-page .web-swipe-details");
    for declaration in [
        "position: static;",
        "max-height: none;",
        "overflow: visible;",
        "animation: none;",
    ] {
        assert!(details.contains(declaration), "{declaration} in {details}");
    }
    let stats = rule(WIDE, ".web-card-page .web-card-stats");
    assert!(stats.contains("grid-column: 1;"), "{stats}");
    assert!(stats.contains("animation: none;"), "{stats}");
    assert!(
        !WEB.contains("overscroll-behavior: contain;\n        box-sizing: border-box;\n    }\n\n    /* Each widget keeps its height"),
        "the stats column no longer docks and scrolls on its own"
    );
}

#[test]
fn the_card_fits_the_window_height_and_stays_in_view() {
    let image = rule(WIDE, ".web-card-page .web-card-image");
    assert!(
        image.contains("max-width: min(26rem, calc(var(--web-card-room) * 488 / 680));"),
        "{image}"
    );
    let main = rule(WIDE, ".web-card-page .web-cards-swipe-main");
    assert!(main.contains("position: sticky;"), "{main}");
}
