//! Every search field is the filter's compact input, the same height, padding, text size,
//! radius and border as Name contains.

#[test]
fn every_picker_and_dictionary_search_is_the_filter_s_compact_input() {
    for (name, src) in [
        (
            "oracle tag dictionary",
            include_str!("../src/lib/inbound/screens/oracle_tag_dictionary.rs"),
        ),
        (
            "format select",
            include_str!("../src/lib/inbound/screens/deck/components/format_select.rs"),
        ),
        (
            "tag select",
            include_str!("../src/lib/inbound/screens/deck/components/tag_select.rs"),
        ),
        (
            "oracle tag select",
            include_str!("../src/lib/inbound/screens/deck/components/oracle_tag_select.rs"),
        ),
    ] {
        assert!(
            !src.contains("input { class: \"input\",") && !src.contains("class: \"input\",\n"),
            "{name} still draws a full-size input"
        );
        assert!(src.contains("input input-compact"), "{name}");
    }
}
