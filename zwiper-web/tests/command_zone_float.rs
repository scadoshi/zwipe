//! The command zone's typeahead leaves its own label row in view on a wide window: its results
//! hang below the field there, over the fields under it, and float up over the label only on a
//! phone, where the keyboard covers what is below.

/// The command zone picker's source.
const SRC: &str = include_str!("../src/lib/inbound/screens/deck/form/command_zone.rs");

#[test]
fn a_wide_window_hangs_the_results_below_the_field() {
    let (before_field, after_field) = SRC.split_once("ClearWrap {").unwrap_or_default();
    assert!(
        after_field.contains("if dropdown() && wide {"),
        "on a wide window the results come after the field: {after_field}"
    );
    assert!(
        SRC.contains(r#""search-float-results search-float-below""#),
        "and hang below it"
    );
    assert!(
        before_field.contains("if dropdown() && !wide {"),
        "a phone keeps them over the label, clear of its keyboard: {before_field}"
    );
}
