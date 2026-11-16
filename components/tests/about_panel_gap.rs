//! About's panels sit a panel gap below what precedes them, and the first, with nothing above
//! it, sits where every screen's first panel does.

/// The screens' base sheet.
const BASE: &str = include_str!("../assets/screens/base.css");

#[test]
fn the_first_about_panel_takes_no_gap() {
    assert!(
        BASE.contains(".about-panel {\n    margin-top: var(--panel-gap);\n}"),
        "a later panel keeps its gap"
    );
    assert!(
        BASE.contains(".about-panel:first-child {\n    margin-top: 0;\n}"),
        "the first one takes none"
    );
}
