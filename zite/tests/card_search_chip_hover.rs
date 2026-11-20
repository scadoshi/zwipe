//! The card search's result chips light and lift under a mouse as the site's other chips do: they
//! share the chip hover rules.

#[test]
fn a_result_chip_hovers_as_the_other_chips_do() {
    let css = include_str!("../assets/style.css");
    for rule in [
        ".chip:not(.disabled):not(.selected):hover,\n    .card-search-chip:hover {",
        ".chip:not(.disabled):hover,\n    .card-search-chip:hover {",
    ] {
        assert!(
            css.contains(rule),
            "a hover rule for the result chips: {rule}"
        );
    }
}
