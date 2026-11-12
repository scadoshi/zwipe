//! zwipe.net's pages share one column, 1400px at most (Scotty's pick between the old 74rem column
//! and the window's width), with running text filling its panel across it.

/// zwipe.net's stylesheet.
const CSS: &str = include_str!("../assets/style.css");

/// The body of the first rule whose selector starts a line as `selector`.
fn rule<'a>(selector: &str) -> Option<&'a str> {
    let at = CSS.find(&format!("\n{selector} {{"))?;
    let body = CSS.get(at..)?;
    body.get(body.find('{')?.saturating_add(1)..body.find('}')?)
}

#[test]
fn every_page_shares_one_1400px_column() {
    let root = rule(":root").expect("the root rule");
    assert!(root.contains("--zite-page-max: 87.5rem"), "{root}");
    let page = rule(".page").expect("the page rule");
    assert!(page.contains("max-width: var(--zite-page-max)"), "{page}");
    let hero = rule(".hero-card").expect("the hero card rule");
    assert!(
        hero.contains("max-width: calc(var(--zite-page-max) - 3rem)"),
        "the hero's edges align with the page's content, its 1.5rem side padding inside"
    );
}

#[test]
fn running_text_fills_its_panel() {
    assert!(
        !CSS.contains("70ch"),
        "no text is capped to a reading measure"
    );
    let coming = rule(".coming-soon p").expect("the coming soon text rule");
    assert!(!coming.contains("max-width"), "{coming}");
}
