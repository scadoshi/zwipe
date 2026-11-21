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
fn the_glass_is_as_wide_as_the_store_icons_and_wears_the_web_app_s_class() {
    let css = include_str!("../assets/style.css");
    assert!(
        css.contains("nav .card-search-nav {\n    --card-search-glass: 1.85rem;\n}"),
        "zite tells the bar its glass is a store icon's width"
    );
    let source = include_str!("../src/components/card_search.rs");
    assert!(
        source.contains(r#"pub(crate) const GLASS_CLASS: &str = "btn-action nav-icon";"#),
        "the glass wears the web app's nav icon class, so it takes the header icons' color rather than the store icons' accent"
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
}

#[test]
fn a_finger_gets_the_glass_s_44px_tap_box_as_the_store_icons_do() {
    let css = include_str!("../assets/style.css");
    let (_, coarse) = css
        .split_once("@media (pointer: coarse) {")
        .unwrap_or_else(|| panic!("the touch block"));
    for opening in [
        ":where(button, [role=\"button\"], .chip, .chip-xs, a.nav-cta, a.panel-action, a.nav-brand, .card-search-label) {",
        ":is(button, [role=\"button\"], .chip, .chip-xs, a.nav-cta, a.panel-action, a.nav-brand, .card-search-label)::after {",
    ] {
        assert!(
            coarse.contains(opening),
            "the glass, a label, joins the tap box: {opening}"
        );
    }
}

#[test]
fn the_glass_glyph_is_the_store_glyphs_size_at_every_width() {
    let css = include_str!("../assets/style.css");
    assert!(
        css.contains(
            "\nnav .card-search-label.nav-icon svg {\n    width: 1.25em;\n    height: 1.25em;\n}"
        ),
        "the glass glyph takes the store glyphs' 1.25em, wide and folded"
    );
}

#[test]
fn wide_the_web_app_s_glass_keeps_the_store_icons_box() {
    let css = include_str!("../assets/style.css");
    let rule = "\nnav .card-search-label.nav-icon {\n    width: var(--card-search-glass);\n    height: var(--card-search-glass);\n    font-size: 0.8rem;\n}";
    let at = css
        .find(rule)
        .unwrap_or_else(|| panic!("the glass's box: {rule}"));
    let folded = css
        .find("    nav .card-search-header .card-search-label {")
        .unwrap_or(0);
    assert!(at < folded, "before the folded size, which overrides it");
}
