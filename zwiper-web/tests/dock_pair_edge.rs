//! A swipe screen's dock pair ends as far from the right edge on its own as it does in the bottom
//! row beside the pile chips, so Suggested's swipe and the Maybeboard's put their docks on one
//! line at every width.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a missing rule fails the test"
)]

/// The web app's own sheet.
const WEB_CSS: &str = include_str!("../assets/web.css");

/// The shared screens' base sheet, which holds the swipe bottom row.
const BASE_CSS: &str = include_str!("../../components/assets/screens/base.css");

/// The declarations of the first rule whose selector is exactly `selector` in `css`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let (_, rest) = css.split_once(&format!("{selector} {{")).unwrap();
    rest.split_once('}').unwrap().0
}

/// The value `css`'s declarations give `property`.
fn value<'css>(css: &'css str, property: &str) -> &'css str {
    let (_, rest) = css.split_once(&format!("{property}: ")).unwrap();
    rest.split_once(';').unwrap().0
}

#[test]
fn the_lone_pair_takes_the_bottom_row_s_right_padding() {
    let pair = rule(WEB_CSS, "\n.web-dock-pair");
    let row = rule(BASE_CSS, "\n.swipe-bottom-row");
    assert_eq!(value(pair, "right"), "var(--pair-gap)", "{pair}");
    assert_eq!(value(pair, "--pair-gap"), value(row, "--row-gap"));
    assert_eq!(
        value(row, "padding"),
        "0 var(--row-gap) 0 var(--space-4)",
        "the row's dock ends --row-gap from the right edge: {row}"
    );
}

#[test]
fn no_width_moves_the_lone_pair_off_that_edge() {
    for block in WEB_CSS.split("@media").skip(1) {
        let Some((_, pair)) = block.split_once("\n    .web-dock-pair {") else {
            continue;
        };
        let pair = pair.split_once('}').unwrap().0;
        assert!(
            !pair.contains("\n        right:"),
            "a narrow screen keeps the pair's edge: {pair}"
        );
    }
}
