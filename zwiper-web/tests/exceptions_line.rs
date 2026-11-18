//! Exceptions shows its grayed chips, its ? and its edit button while Universes Beyond shows, and
//! nothing under them: the grayed row says on its own that it is off.

const SCREEN: &str = include_str!("../src/lib/inbound/screens/config/mod.rs");
const SHEET: &str = include_str!("../../components/assets/screens/screens-b.css");

#[test]
fn exceptions_says_nothing_under_its_grayed_row() {
    assert!(
        !SCREEN.contains("ub-exceptions-reason") && !SCREEN.contains("exceptions_locked"),
        "the screen draws no line under Exceptions"
    );
    assert!(
        !SHEET.contains(".ub-exceptions-reason"),
        "no style is left for the line"
    );
}
