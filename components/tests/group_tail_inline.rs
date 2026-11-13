//! A List group's price and cart share the title's line, at its right end, and wrap under the
//! title only when the line can't hold both: neither the title nor the tail claims a whole line.

const MAGIC: &str = include_str!("../assets/magic.css");

/// The declarations of the rule whose selector is exactly `selector`.
fn rule(selector: &str) -> Option<&'static str> {
    let start = MAGIC.find(&format!("\n{selector} {{"))?;
    let body = MAGIC.get(start..)?;
    let open = body.find('{')?;
    let close = body.find('}')?;
    body.get(open.checked_add(1)?..close)
}

#[test]
fn the_title_and_its_tail_share_a_line() {
    for selector in [
        ".card-group-header-tailed > .card-group-toggle",
        ".card-group-header-tailed > .card-group-tail",
    ] {
        let Some(body) = rule(selector) else {
            panic!("no rule for {selector}");
        };
        assert!(
            !body.contains("100%"),
            "{selector} takes a whole line, so the tail can never sit beside the title: {body}"
        );
    }
}

#[test]
fn the_tail_keeps_to_the_right_edge() {
    let body = rule(".card-group-header-tailed > .card-group-tail").unwrap_or_default();
    assert!(body.contains("margin-left: auto"), "{body}");
    assert!(body.contains("flex: none"), "{body}");
}
