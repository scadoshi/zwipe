//! The command zone's typeahead covers no label at any width: its results sit in the form's flow
//! under the field, pushing the rows below down while they show, instead of floating over the
//! label row above or the field below.

/// The command zone picker's source.
const SRC: &str = include_str!("../src/lib/inbound/screens/deck/form/command_zone.rs");

/// The web app's own sheet.
const WEB_CSS: &str = include_str!("../assets/web.css");

#[test]
fn the_results_sit_under_the_field_in_the_form_s_flow() {
    let markup = SRC
        .split(r#"div { class: "label-row","#)
        .nth(1)
        .unwrap_or_default();
    let (above_field, below_field) = markup.split_once("ClearWrap {").unwrap_or_default();
    assert!(
        !above_field.contains("floats") && !above_field.contains("search-float"),
        "nothing renders between the label row and the field: {above_field}"
    );
    assert!(
        below_field.contains("{floats}"),
        "the results come after the field: {below_field}"
    );
    assert!(
        SRC.contains(r#"div { class: "search-float-anchor search-float-flow","#),
        "in an anchor that takes their height"
    );
    let rule = WEB_CSS
        .split(".search-float-flow > .search-float-results {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .unwrap_or_default();
    assert!(
        rule.contains("position: static;"),
        "in the flow, so the rows below move down: {rule}"
    );
    assert!(
        WEB_CSS.contains(".search-float-anchor.search-float-flow {\n    height: auto;"),
        "the anchor takes the results' height"
    );
}
