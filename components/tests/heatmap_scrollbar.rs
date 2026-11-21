//! The activity heatmap scrolls sideways with no scroll indicator, as Cairn's does, so nothing
//! draws over its bottom row of cells.

/// The body of the stylesheet's rule for `selector`.
#[expect(
    clippy::expect_used,
    reason = "a test helper: a stylesheet without the rule fails the test"
)]
fn rule(selector: &str) -> &'static str {
    let css = include_str!("../assets/screens/screens-b.css");
    let start = css
        .find(&format!("\n{selector} {{"))
        .expect("screens-b.css has the rule");
    let body = &css[start..];
    &body[..body.find('}').expect("the rule closes")]
}

#[test]
fn the_heatmap_scrolls_with_no_indicator_over_its_cells() {
    let scroll = rule(".activity-scroll");
    assert!(
        scroll.contains("scrollbar-width: none;"),
        "Firefox and the standard property hide it: {scroll}"
    );
    assert!(
        !scroll.contains("scrollbar-color"),
        "a hidden bar takes no colors: {scroll}"
    );
    let webkit = rule(".activity-scroll::-webkit-scrollbar");
    assert!(
        webkit.contains("display: none;"),
        "WebKit, the iPhone's engine, hides it too: {webkit}"
    );
}
