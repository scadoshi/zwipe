//! A picker that opens as a screen over another (the deck form's Format, Deck tags, Oracle tags
//! and Swipe select) draws its own header while the screen under it keeps its own, so its header
//! leaves the card search out: a picker is a choice to make, and two header searches would share
//! the page.

#[test]
fn an_overlay_picker_s_header_has_no_card_search() {
    for (name, src) in [
        (
            "format",
            include_str!("../src/lib/inbound/screens/deck/components/format_select.rs"),
        ),
        (
            "deck tags",
            include_str!("../src/lib/inbound/screens/deck/components/tag_select.rs"),
        ),
        (
            "oracle tags",
            include_str!("../src/lib/inbound/screens/deck/components/oracle_tag_select.rs"),
        ),
        (
            "swipe select",
            include_str!("../src/lib/inbound/screens/deck/components/swipe_select.rs"),
        ),
    ] {
        let (_, header) = src
            .split_once("ScreenHeader {")
            .unwrap_or_else(|| panic!("{name} draws a header"));
        let (props, _) = header
            .split_once('}')
            .unwrap_or_else(|| panic!("{name}'s header ends"));
        assert!(
            props.contains("search: false"),
            "{name}'s header leaves the card search out: {props}"
        );
    }
}
