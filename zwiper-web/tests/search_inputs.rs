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
