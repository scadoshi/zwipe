//! Every search field is the filter's compact input, the same height, padding, text size,
//! radius and border as Name contains.

#[test]
fn the_oracle_tags_dictionary_search_is_the_filter_s_compact_input() {
    let src = include_str!("../src/lib/inbound/screens/oracle_tag_dictionary.rs");
    let field = src
        .split("id: SEARCH_ID")
        .next()
        .and_then(|head| head.rsplit("input {").next())
        .unwrap_or_default();
    assert!(
        field.contains(r#"class: "input input-compact","#),
        "{field}"
    );
}

#[test]
fn every_web_search_has_its_label_chip_on_its_left_on_one_line() {
    let dictionary = include_str!("../src/lib/inbound/screens/oracle_tag_dictionary.rs");
    let pickers = include_str!("../src/lib/inbound/screens/deck/form/pickers.rs");
    assert!(
        !dictionary.contains("tag-controls-head") && !pickers.contains("tag-search-label"),
        "no label above a field"
    );
    let row = dictionary
        .split(r#"div { class: "chip-row","#)
        .nth(1)
        .unwrap_or_default();
    let label = row.find("chip-row-label");
    let field = row.find("id: SEARCH_ID");
    assert!(label.is_some_and(|l| field.is_some_and(|f| l < f)), "{row}");
}
