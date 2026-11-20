//! The card search's results (and a name's several cards) and the card page sit the kit's 1rem
//! under the header, as every screen's first element does: a column's gap between its items does
//! not also open under the kit's header spacer, the column's first flex item.

const MAIN: &str = include_str!("../assets/main.css");

/// The body of the rule `selector` opens in main.css, or "" when there is none.
fn rule(selector: &str) -> &'static str {
    MAIN.split_once(&format!("\n{selector} {{"))
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn the_column_names_its_gap() {
    let list = rule(".card-page-list");
    assert!(
        list.contains("--card-page-list-gap: var(--space-2);"),
        "{list}"
    );
    assert!(list.contains("gap: var(--card-page-list-gap);"), "{list}");
}

#[test]
fn the_header_spacer_takes_the_gap_back() {
    let spacer = rule(".screen > .app-nav ~ .screen-content.card-page-list::before");
    assert!(
        spacer.contains("margin-bottom: calc(-1 * var(--card-page-list-gap));"),
        "{spacer}"
    );
}

#[test]
fn the_card_page_names_its_gap_and_its_header_spacer_takes_it_back() {
    let page = rule(".card-page");
    assert!(page.contains("--card-page-gap: var(--space-4);"), "{page}");
    assert!(page.contains("gap: var(--card-page-gap);"), "{page}");
    let spacer = rule(".screen > .app-nav ~ .screen-content.card-page::before");
    assert!(
        spacer.contains("margin-bottom: calc(-1 * var(--card-page-gap));"),
        "{spacer}"
    );
}
