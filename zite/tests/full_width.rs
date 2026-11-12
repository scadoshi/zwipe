//! zwipe.net runs the window's width: the page has no fixed cap, every
//! section shares one width so their edges line up with the card grids, and
//! the grids add columns as it grows.

const STYLE: &str = include_str!("../assets/style.css");

/// Every top-level rule whose selector line is exactly `selector`, joined: each from
/// its selector line to its closing brace.
fn rule(selector: &str) -> String {
    let lines: Vec<&str> = STYLE.lines().collect();
    let open = format!("{selector} {{");
    let found: Vec<String> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_end() == open)
        .map(|(at, _)| {
            lines
                .iter()
                .skip(at)
                .take_while(|line| *line != &"}")
                .copied()
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect();
    assert!(!found.is_empty(), "no rule {selector}");
    found.join("\n")
}

#[test]
fn the_page_has_no_fixed_width() {
    let page = rule(".page");
    assert!(!page.contains("max-width"), "{page}");
}

#[test]
fn every_section_and_the_hero_share_the_page_s_width() {
    assert!(rule("body .page > *").contains("max-width: var(--zite-page-max)"));
    assert!(rule(".hero-card").contains("max-width: var(--zite-page-max)"));
    assert!(rule(":root").contains("--zite-page-max: calc(7 * 26rem"));
}

#[test]
fn card_grids_fill_the_page_s_width_from_1280() {
    let at = STYLE.find("@media (min-width: 80rem) {").unwrap();
    let block = STYLE.get(at..).unwrap();
    let block = block.get(..block.find("\n}\n").unwrap()).unwrap();
    assert!(
        block.contains("repeat(auto-fit, minmax(22rem, 1fr))"),
        "{block}"
    );
}

#[test]
fn the_home_band_takes_one_row_from_1920() {
    let at = STYLE.find("@media (min-width: 120rem) {").unwrap();
    let block = STYLE.get(at..).unwrap();
    let block = block.get(..block.find("\n}\n").unwrap()).unwrap();
    assert!(block.contains(".features-stack"), "{block}");
    assert!(block.contains("repeat(3, minmax(0, 1fr))"), "{block}");
}

#[test]
fn intro_panels_keep_their_prose_to_a_readable_measure() {
    let at = STYLE
        .find("/* Running text keeps a readable measure. */")
        .unwrap();
    let rule = STYLE.get(at..).unwrap();
    let rule = rule.get(..rule.find('}').unwrap()).unwrap();
    assert!(
        rule.contains("body .page > :is(.section, .page-header) > .panel-card p"),
        "About's and Contribute's intro panels: {rule}"
    );
    assert!(
        rule.contains("body .page > .section > .section-head > .panel-card p"),
        "About's System architecture summary: {rule}"
    );
    assert!(rule.contains("max-width: 70ch"), "{rule}");
}
