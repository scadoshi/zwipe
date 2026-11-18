//! A phone's card actions stand at a comfortable touch size, while a wide window keeps them
//! compact.

const MAGIC: &str = include_str!("../assets/magic.css");

/// The body of the first rule in `css` from `selector`, after `from`.
fn rule<'css>(css: &'css str, from: usize, selector: &str) -> Option<&'css str> {
    let at = css.get(from..)?.find(selector)?.checked_add(from)?;
    css.get(at..)?.split('}').next()
}

/// A length in px from `0.375rem`, `1px` or `calc(-0.375rem - 1px)`'s terms, at a 16px rem;
/// the sign is dropped.
fn px(length: &str) -> Option<f64> {
    length
        .trim_start_matches("calc(")
        .trim_end_matches(')')
        .split(" - ")
        .map(|term| {
            let term = term.trim().trim_start_matches('-');
            match term.strip_suffix("rem") {
                Some(rem) => rem.parse::<f64>().ok().map(|rem| rem * 16.0),
                None => term.strip_suffix("px")?.parse::<f64>().ok(),
            }
        })
        .sum()
}

/// The value of `property` in a rule body.
fn value<'css>(body: &'css str, property: &str) -> Option<&'css str> {
    let (_, rest) = body.split_once(&format!("{property}: "))?;
    rest.split(';').next()
}

#[test]
fn a_phone_s_card_action_reaches_44px_from_inside_its_border() {
    // An absolute reach is placed from the padding box, inside the border.
    let media = MAGIC
        .find("@media (max-width: 899px) {\n    /* A card's actions on a phone")
        .expect("the phone block");
    let base = rule(MAGIC, 0, ".card-action-btn {").expect("the button's own rule");
    let border = value(base, "border")
        .and_then(|border| px(border.split(' ').next()?))
        .expect("the border's width");
    let button = rule(MAGIC, media, ".card-action-btn {").expect("the button rule");
    let height = value(button, "min-height")
        .and_then(px)
        .expect("the phone height");
    let reach = rule(MAGIC, media, ".card-action-btn::before {").expect("the reach rule");
    let inset = value(reach, "inset").and_then(px).expect("the reach");
    let reached = height - 2.0 * border + 2.0 * inset;
    assert!(
        (reached - 44.0).abs() < 0.01,
        "{height}px tall, {border}px border, {inset}px reach: {reached}px"
    );
}

#[test]
fn a_phone_s_card_actions_reach_a_touch_size_and_keep_their_spacing() {
    let media = MAGIC
        .find("@media (max-width: 899px) {\n    /* A card's actions on a phone")
        .expect("the phone block");
    let button = rule(MAGIC, media, ".card-action-btn {").expect("the button rule");
    assert!(button.contains("min-height: 2rem;"), "{button}");
    assert!(button.contains("position: relative;"), "{button}");
    let reach = rule(MAGIC, media, ".card-action-btn::before {").expect("the reach rule");
    for want in [
        "content: \"\";",
        "position: absolute;",
        "inset: calc(-0.375rem - 1px);",
    ] {
        assert!(reach.contains(want), "{want} in {reach}");
    }
    let row = rule(MAGIC, media, ".card-action-row {").expect("the row rule");
    assert!(
        row.contains("gap: var(--space-3);"),
        "reaches don't overlap: 0.75rem is two 0.375rem reaches: {row}"
    );
    assert!(
        row.contains("margin-top: var(--space-3);"),
        "a stacked row stands clear of the one above: {row}"
    );
    let last = rule(MAGIC, media, ".card-action-row:last-child {").expect("the last row's rule");
    assert!(
        last.contains("margin-bottom: 0.375rem;"),
        "the bottom row's reach stays inside a clipping row: {last}"
    );
}
