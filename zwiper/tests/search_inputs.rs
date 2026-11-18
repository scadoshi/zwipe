//! Every search field is the filter's compact input, the same height, padding, text size,
//! radius and border as Name contains, its Search label chip on its left. The pickers take
//! all of it from the shared `PickerSearch`; the dictionary draws its own.

#[test]
fn every_picker_and_dictionary_search_is_the_filter_s_compact_input() {
    {
        let name = "oracle tag dictionary";
        let src = include_str!("../src/lib/inbound/screens/oracle_tag_dictionary.rs");
        assert!(
            !src.contains("input { class: \"input\",") && !src.contains("class: \"input\",\n"),
            "{name} still draws a full-size input"
        );
        assert!(src.contains("input input-compact"), "{name}");
    }
}

#[test]
fn every_app_search_has_its_label_chip_on_its_left_on_one_line() {
    {
        let name = "oracle tag dictionary";
        let src = include_str!("../src/lib/inbound/screens/oracle_tag_dictionary.rs");
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

#[test]
fn the_app_pickers_search_with_the_shared_picker_search_and_mark_enter_s_target() {
    for (name, src) in [
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
            src.contains("PickerSearch {"),
            "{name}: the shared search, its label chip and ×"
        );
        assert!(
            src.contains("on_enter:"),
            "{name}: Enter acts on the top match"
        );
        assert!(
            src.contains("PickerChoice {"),
            "{name}: the top match shows as Enter's target"
        );
        assert!(
            !src.contains("input input-compact"),
            "{name}: no search field of its own"
        );
    }
}
