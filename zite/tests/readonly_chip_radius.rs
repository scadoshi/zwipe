//! The site's read-only chips (a flavor quote's price, the hero's facts, a release's shipped tag)
//! take the kit's read-only corner, `--radius-readonly`; the rating link keeps the button corner.

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
fn read_only_chips_take_the_read_only_corner() {
    for selector in [".ff-price", ".hero-chip", ".shipped-tag"] {
        let body = rule(selector);
        assert!(
            body.contains("border-radius: var(--radius-readonly);"),
            "{selector}: {body:?}"
        );
    }
}

#[test]
fn the_rating_link_keeps_the_control_corner() {
    let body = rule(".rating-tag");
    assert!(
        body.contains("border-radius: var(--radius-control);"),
        "{body:?}"
    );
}
