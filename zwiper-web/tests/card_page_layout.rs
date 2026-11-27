//! The card page lays out as a swipe screen: on a wide window the card in the middle, its details
//! the docked panel the eye hides and shows and its stats the swipe screens' column, the card
//! sized by the window's height too so it fits under the nav; below that the card alone, filling
//! the screen, its details behind the eye.

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

const WIDE: &str = "@media (min-width: 900px) {\n    /* The card page is a swipe screen's layout";
const NARROW: &str = "@media (max-width: 899px) {\n    /* Below the wide layout the card fills";

#[test]
fn the_card_fits_the_window_height_on_a_wide_window() {
    let image = rule(WIDE, ".web-card-page .web-card-image");
    assert!(
        image.contains("max-width: min(26rem, calc(var(--web-card-room) * 488 / 680));"),
        "{image}"
    );
}

#[test]
fn the_details_and_stats_are_the_swipe_screens_own() {
    for selector in [
        ".web-card-page .web-swipe-details {",
        ".web-card-page .web-card-stats {",
        "grid-template-columns: var(--web-dock-panel-width) minmax(0, 1fr) var(--web-dock-panel-width);",
    ] {
        assert!(
            !WEB.contains(selector),
            "the card page places none of its own: {selector}"
        );
    }
}

#[test]
fn below_the_wide_layout_the_card_fills_the_screen() {
    let page = rule(NARROW, ".web-card-page");
    assert!(page.contains("justify-content: center;"), "{page}");
    let image = rule(NARROW, ".web-card-page .web-card-image");
    assert!(image.contains("width: 100%;"), "{image}");
    assert!(
        image.contains("calc(var(--web-card-room) * 488 / 680)"),
        "no taller than the room: {image}"
    );
}
