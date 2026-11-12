//! zwipe.net runs the window's width: the page has no fixed cap, its text
//! keeps to the hero's width, and the card grids add columns as it grows.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "test helpers: a missing rule fails the test"
)]

const STYLE: &str = include_str!("../assets/style.css");

/// The body of the first rule whose selector line is exactly `selector`.
fn rule(selector: &str) -> &'static str {
    let at = STYLE
        .lines()
        .position(|line| line.trim_end() == format!("{selector} {{"))
        .unwrap_or_else(|| panic!("no rule {selector}"));
    let body: Vec<&str> = STYLE
        .lines()
        .skip(at)
        .take_while(|l| l.trim() != "}")
        .collect();
    let start = STYLE.find(body.first().unwrap()).unwrap();
    STYLE
        .get(start..)
        .unwrap()
        .get(..body.join("\n").len())
        .unwrap()
}

#[test]
fn the_page_has_no_fixed_width() {
    let page = rule(".page");
    assert!(!page.contains("max-width"), "{page}");
}

#[test]
fn text_sections_keep_to_the_hero_s_width() {
    assert!(rule("body .page > *").contains("max-width: var(--zite-text-max)"));
}

#[test]
fn card_grids_add_columns_up_to_seven() {
    let at = STYLE.find("@media (min-width: 80rem) {").unwrap();
    let block = STYLE.get(at..).unwrap();
    let block = block.get(..block.find("\n}\n").unwrap()).unwrap();
    assert!(
        block.contains("repeat(auto-fit, minmax(22rem, 26rem))"),
        "{block}"
    );
    assert!(block.contains("calc(7 * 26rem"), "{block}");
}
