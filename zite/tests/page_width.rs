//! zwipe.net's pages share one column, 1400px at most (Scotty's pick between the old 74rem column
//! and the window's width), with running text at a readable 70ch inside it.

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
fn running_text_keeps_a_readable_measure() {
    let at = CSS.find("max-width: 70ch").expect("a 70ch rule");
    let selectors = CSS.get(..at).unwrap();
    let selectors = selectors.get(selectors.rfind('}').unwrap()..).unwrap();
    for text in [
        ".guide-content .guide-p",
        ".privacy-section > p",
        ".hero .tagline",
        ".panel-card p",
    ] {
        assert!(selectors.contains(text), "{text} in {selectors}");
    }
}
