//! zwipe.net's changelog page matches app.zwipe.net's: below 900px its page scrollbar hides
//! as the web app's does, so the panels span the window less the same gutter, and from 900px
//! the Changelog panel sits one `--space-4` under the nav as the web app's does.

/// zwipe.net's stylesheet.
const CSS: &str = include_str!("../assets/style.css");

/// The text of the `@media` block that opens with `query`.
fn media(query: &str) -> Option<&'static str> {
    let at = CSS
        .find(&format!(
            "@media {query} {{\n    html:has(.page > .changelog-panel)"
        ))
        .or_else(|| CSS.find(&format!("@media {query} {{\n    .page > .changelog-panel")))?;
    let body = CSS.get(at..)?;
    body.get(..body.find("\n}\n")?)
}

#[test]
fn below_the_web_apps_breakpoint_the_page_scrollbar_takes_no_room() {
    let narrow = media("(max-width: 899px)").expect("the narrow block");
    assert!(narrow.contains("scrollbar-gutter: auto;"), "{narrow}");
    assert!(narrow.contains("scrollbar-width: none;"), "{narrow}");
    assert!(
        narrow.contains("html:has(.page > .changelog-panel)::-webkit-scrollbar"),
        "{narrow}"
    );
}

#[test]
fn on_a_wide_window_the_panel_sits_one_step_under_the_nav() {
    let wide = media("(min-width: 900px)").expect("the wide block");
    assert!(
        wide.contains(".page > .changelog-panel {\n        margin-top: var(--space-4);"),
        "{wide}"
    );
}
