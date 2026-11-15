//! A collapsed card row keeps its detail mounted but closed, and does no motion work in it.

/// The body of the rule for `selector` in the Magic UI sheet.
fn rule(selector: &str) -> &'static str {
    let css = include_str!("../assets/magic.css");
    let Some(at) = css.find(&format!("\n{selector} {{")) else {
        return "";
    };
    css.get(at..)
        .and_then(|rest| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn the_buttons_of_a_closed_row_do_not_animate() {
    let closed = rule(".card-row-collapse:not(.open) .card-action-btn");
    assert!(closed.contains("transition: none;"), "{closed:?}");
}
