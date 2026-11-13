//! A hint grows to fit its text with no scroll of its own inside; one taller than the window
//! scrolls as a whole, title and buttons with it, like a page.

const COMPONENTS: &str = include_str!("../assets/screens/components.css");

/// The body of the rule in `css` that opens with `selector`.
fn rule<'css>(css: &'css str, selector: &str) -> Option<&'css str> {
    let at = css.find(selector)?;
    css.get(at..)?.split('}').next()
}

#[test]
fn a_hint_grows_to_its_text_and_scrolls_whole_past_the_window() {
    let hint = ".alert-dialog:has(.hint-body, .hint-line, .hint-bullets)";
    let dialog = rule(COMPONENTS, &format!("{hint} {{")).expect("the hint's own rule");
    for want in [
        "max-height: calc(100dvh - env(safe-area-inset-top, 0px) - env(safe-area-inset-bottom, 0px) - 2rem);",
        "overflow-y: auto;",
        "overscroll-behavior: contain;",
    ] {
        assert!(dialog.contains(want), "{want} in {dialog}");
    }
    let body = rule(
        COMPONENTS,
        &format!("{hint} > .alert-dialog-description {{"),
    )
    .expect("the body's rule");
    for want in ["flex: none;", "overflow: visible;"] {
        assert!(body.contains(want), "{want} in {body}");
    }
}
