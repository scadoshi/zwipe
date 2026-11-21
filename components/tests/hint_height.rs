//! A hint grows to a margin short of the window; one taller than that keeps its title and its
//! buttons in place and scrolls only its body between them, with no bounce past either end.

const COMPONENTS: &str = include_str!("../assets/screens/components.css");

/// The hint dialog's selector, as the rules open with it.
const HINT: &str = ".alert-dialog:has(.hint-body, .hint-line, .hint-bullets)";

/// The body of the rule in `css` that opens with `selector`.
fn rule<'css>(css: &'css str, selector: &str) -> Option<&'css str> {
    let at = css.find(selector)?;
    css.get(at..)?.split('}').next()
}

/// The bodies of every rule in `css` whose selector is exactly `selector`, joined.
fn rules_for(css: &str, selector: &str) -> String {
    css.split('}')
        .filter_map(|chunk| chunk.split_once('{'))
        .filter(|(head, _)| head.rsplit("*/").next().map(str::trim) == Some(selector))
        .map(|(_, body)| body)
        .collect()
}

#[test]
fn a_hint_reaches_to_a_margin_short_of_the_window_and_never_scrolls_whole() {
    let dialog = rule(COMPONENTS, &format!("{HINT} {{")).expect("the hint's own rule");
    assert!(
        dialog.contains(
            "max-height: calc(100dvh - env(safe-area-inset-top, 0px) - env(safe-area-inset-bottom, 0px) - 2rem);"
        ),
        "{dialog}"
    );
    assert!(
        !dialog.contains("overflow"),
        "the dialog keeps no scroll of its own, so its title and buttons stay put: {dialog}"
    );
}

#[test]
fn a_hints_body_scrolls_between_the_rules_without_bouncing() {
    let body = rule(
        COMPONENTS,
        &format!("{HINT} > .alert-dialog-description {{"),
    )
    .expect("the body's rule");
    assert!(body.contains("overscroll-behavior: none;"), "{body}");
    for gone in ["flex: none", "overflow: visible"] {
        assert!(
            !body.contains(gone),
            "the body keeps the kit's own scroll: {gone} in {body}"
        );
    }
    let kit = rules_for(components::APP_CSS, ".alert-dialog-description");
    for want in ["flex: 1 1 auto;", "min-height: 0;", "overflow-y: auto;"] {
        assert!(kit.contains(want), "{want} in {kit}");
    }
}
