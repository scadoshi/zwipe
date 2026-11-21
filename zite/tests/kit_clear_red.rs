//! The kit draws every clear × in the clear red, so zite's stylesheet adds no rule of its own for
//! the card search's.

#[test]
fn the_card_search_clear_takes_the_kit_s_red() {
    let style = include_str!("../assets/style.css");
    assert!(
        !style.contains(".card-search .input-clear-btn"),
        "a second red rule"
    );
}
