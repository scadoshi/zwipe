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

#[test]
fn every_app_search_has_its_label_chip_on_its_left_on_one_line() {
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
        let row = src
            .split(r#"div { class: "chip-row","#)
            .nth(1)
            .unwrap_or_default();
        let label = row.find(r#"class: "stat-chip stat-chip-p1 chip-row-label", r#for:"#);
        let field = row.find("input input-compact");
        assert!(
            label.is_some_and(|l| field.is_some_and(|f| l < f)),
            "{name}: the Search chip sits left of its field in one row"
        );
    }
}
