//! The card results grid (the search's results, a card page's several cards) shows as many
//! cards across as the window fits, up to five, each at its own size: its column holds five
//! cards at their widest, their gaps and its side padding.

const WEB: &str = include_str!("../assets/web.css");

#[test]
fn the_results_column_holds_five_cards_at_most() {
    let (_, wide) = WEB
        .split_once("    .web-card-list {\n        max-width: ")
        .unwrap_or_else(|| panic!("the wide results column"));
    let width = wide.split_once(';').map_or("", |(w, _)| w);
    assert_eq!(
        width, "calc(5 * 15rem + 4 * var(--space-4) + 2 * var(--space-4))",
        "five 15rem cards, four gaps and the side padding"
    );
}
