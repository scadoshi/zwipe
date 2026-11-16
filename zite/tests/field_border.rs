//! A form field's edge: the only thing that shows a field on a panel of the same fill, so it
//! borders in the kit's muted ink, held at 4.5:1 or better on every theme's panel and sink
//! (orangui's ink gate), as the kit's own `.input` does. Focus keeps its accent border.

const CSS: &str = include_str!("../assets/style.css");

/// The body of the rule for `selector`.
fn rule(selector: &str) -> &'static str {
    let Some(at) = CSS.find(&format!("\n{selector} {{")) else {
        return "";
    };
    CSS.get(at..)
        .and_then(|rest| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_form_field_borders_in_the_muted_ink() {
    let input = rule(".form-group input");
    assert!(
        input.contains("border: 1px solid var(--ink-text-muted, var(--text-muted));"),
        "{input:?}"
    );
}

#[test]
fn focus_keeps_its_accent_border() {
    let focus = rule(".form-group input:focus");
    assert!(
        focus.contains("border-color: var(--accent-primary);"),
        "{focus:?}"
    );
}
