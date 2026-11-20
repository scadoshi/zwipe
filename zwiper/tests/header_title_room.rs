//! The app header's title column shrinks below its title's width, so a long title truncates in
//! the middle column instead of running under the back arrow, the search's glass or the menu.

const MAIN: &str = include_str!("../assets/main.css");

#[test]
fn the_title_column_may_shrink_below_its_text() {
    let (_, rest) = MAIN
        .split_once("\n.app-nav .nav-persistent {")
        .unwrap_or_else(|| panic!("the title column"));
    let rule = rest.split_once('}').map_or("", |(body, _)| body);
    for declaration in ["min-width: 0;", "max-width: 100%;"] {
        assert!(rule.contains(declaration), "{declaration} in {rule}");
    }
}
