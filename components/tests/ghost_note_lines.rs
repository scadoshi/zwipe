//! A card ghost's note sits inside the card image's wrapper, which sets `line-height: 0` so the
//! image has no line box under it. The note sets its own line height, the body's, so a note that
//! wraps at a phone's width keeps its lines apart.

#![expect(
    clippy::panic,
    reason = "test helpers: a rule that is missing fails the test"
)]

const SCREENS: &str = include_str!("../assets/screens/screens-a.css");
const APP_BODY: &str = include_str!("../../zwiper/assets/main.css");
const WEB_BODY: &str = include_str!("../../zwiper-web/assets/web.css");

/// The declarations of the rule whose selector is exactly `selector` in `css`.
fn rule<'css>(css: &'css str, selector: &str) -> &'css str {
    let opening = format!("\n{selector} {{");
    let (_, rest) = css
        .split_once(&opening)
        .unwrap_or_else(|| panic!("a rule for {selector}"));
    let (body, _) = rest
        .split_once('}')
        .unwrap_or_else(|| panic!("the end of {selector}"));
    body
}

/// The `line-height` declared in `declarations`.
fn line_height(declarations: &str) -> Option<&str> {
    declared(declarations, "line-height")
}

/// The `line-height` the `body` rules of `css` declare.
fn body_line_height(css: &str) -> Option<&str> {
    css.split("\nbody {")
        .skip(1)
        .filter_map(|rest| rest.split_once('}'))
        .find_map(|(declarations, _)| line_height(declarations))
}

#[test]
fn a_ghost_note_keeps_the_body_s_line_height_on_the_web_and_in_the_app() {
    let note = line_height(rule(SCREENS, ".card-ghost-note"));
    assert!(note.is_some(), "the note sets its own line height");
    assert_eq!(note, body_line_height(APP_BODY), "the app's body");
    assert_eq!(note, body_line_height(WEB_BODY), "the web's body");
}

/// The value `property` takes in `declarations`.
fn declared<'css>(declarations: &'css str, property: &str) -> Option<&'css str> {
    let prefix = format!("{property}:");
    declarations
        .lines()
        .find_map(|line| line.trim().strip_prefix(prefix.as_str()))
        .map(|value| value.trim().trim_end_matches(';'))
}

#[test]
fn a_ghost_note_is_subtle_ink_at_normal_weight() {
    let note = rule(SCREENS, ".card-ghost-note");
    assert_eq!(
        declared(note, "color"),
        Some("var(--text-subtle)"),
        "the empty-state chips' ink, which clears 3:1 on the ghost in every theme"
    );
    assert_eq!(declared(note, "font-weight"), Some("400"));
}
