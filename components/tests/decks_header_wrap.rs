//! Home's Decks header in a narrow panel: the title starts at the left edge and the Create deck
//! and All decks buttons follow it in the flow, wrapping under it when the line can't hold both,
//! so the title never cuts off and the buttons keep their size.

/// The sheet the Home panels' rules live in.
const MAGIC: &str = include_str!("../assets/magic.css");

/// The body of the narrow panel's container query.
fn narrow() -> Option<&'static str> {
    let at = MAGIC.find("@container (max-width: 36rem) {")?;
    let body = MAGIC.get(at..)?;
    body.get(..body.find("\n}")?)
}

/// The body of the rule in `css` whose selector is `selector`.
fn rule<'css>(css: &'css str, selector: &str) -> Option<&'css str> {
    let at = css.find(&format!("{selector} {{"))?;
    let body = css.get(at..)?;
    body.get(body.find('{')?.saturating_add(1)..body.find('}')?)
}

#[test]
fn the_narrow_header_wraps_instead_of_reserving_the_buttons_width() {
    let head = rule(
        narrow().unwrap(),
        ".home-flavor.home-decks > .settings-list-header.home-decks-header-create",
    )
    .unwrap();
    assert!(head.contains("flex-wrap: wrap;"), "{head}");
    assert!(head.contains("padding-left: var(--space-4);"), "{head}");
    assert!(
        head.contains("padding-right: var(--space-4);"),
        "no reserve for the buttons: {head}"
    );
}

#[test]
fn the_narrow_header_s_buttons_sit_in_the_flow_at_the_right() {
    let action = rule(
        narrow().unwrap(),
        ".home-decks > .home-decks-header-create > .home-panel-header-action",
    )
    .unwrap();
    assert!(action.contains("position: static;"), "{action}");
    assert!(action.contains("transform: none;"), "{action}");
    assert!(action.contains("margin-left: auto;"), "{action}");
}
