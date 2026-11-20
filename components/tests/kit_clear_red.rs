//! The kit draws every clear × in the clear red, so the app's filter sheet adds no rule of its
//! own for it.

#[test]
fn the_filter_sheet_leaves_the_clear_s_color_to_the_kit() {
    let filter = include_str!("../assets/filter.css");
    assert!(
        !filter.contains("\n.input-clear-btn {"),
        "a second red rule"
    );
    assert!(
        components::COMPONENTS_CSS.contains("border: 1px solid var(--color-error);"),
        "the kit's red clear"
    );
}
