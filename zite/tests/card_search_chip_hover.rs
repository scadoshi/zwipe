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

#[test]
fn the_glass_is_as_wide_as_the_store_icons_it_wears_the_class_of() {
    let css = include_str!("../assets/style.css");
    assert!(
        css.contains("nav .card-search-nav {\n    --card-search-glass: 1.85rem;\n}"),
        "zite tells the bar its glass is a store icon's width"
    );
    let source = include_str!("../src/components/card_search.rs");
    assert!(
        source.contains(r#"pub(crate) const GLASS_CLASS: &str = "nav-cta nav-cta-icon";"#),
        "the glass wears the store icons' class"
    );
}

#[test]
fn folded_the_glass_is_the_persistent_store_icons_size() {
    let css = include_str!("../assets/style.css");
    assert!(
        css.contains(
            "    nav .card-search-header .card-search-label {\n        width: 1.7rem;\n        height: 1.7rem;\n        font-size: 0.7rem;\n    }"
        ),
        "the folded glass takes the persistent store icons' size"
    );
    assert!(
        css.contains("    nav .card-search-header .card-search-label svg {\n        width: 1.25em;\n        height: 1.25em;\n    }"),
        "and its glyph the store glyphs' size"
    );
}
