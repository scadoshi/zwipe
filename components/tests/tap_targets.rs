//! A phone's card actions stand 8px apart as a wide window's do, each reaching half the gap
//! past its edges, so neighboring reaches meet at the gap's middle without overlapping.

const MAGIC: &str = include_str!("../assets/magic.css");

/// The body of the first rule in `css` from `selector`, after `from`.
fn rule<'css>(css: &'css str, from: usize, selector: &str) -> Option<&'css str> {
    let at = css.get(from..)?.find(selector)?.checked_add(from)?;
    css.get(at..)?.split('}').next()
}

/// A length in px from `2rem`, `1px`, `var(--space-1)` or `calc(-1 * var(--space-1) - 1px)`'s
/// terms, at a 16px rem with `--space-1` at 0.25rem and `--space-2` at 0.5rem; the sign is
/// dropped.
fn px(length: &str) -> Option<f64> {
    length
        .strip_prefix("calc(")
        .and_then(|inner| inner.strip_suffix(')'))
        .unwrap_or(length)
        .split(" - ")
        .map(|term| {
            let term = term
                .trim()
                .trim_start_matches("-1 * ")
                .trim_start_matches('-');
            match term {
                "var(--space-1)" => return Some(4.0),
                "var(--space-2)" => return Some(8.0),
                _ => {}
            }
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
fn a_phone_s_card_action_reaches_half_the_gap_past_its_border() {
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
    let row = rule(MAGIC, media, ".card-action-row {").expect("the row rule");
    let gap = value(row, "gap").and_then(px).expect("the row's gap");
    let past_edge = inset - border;
    assert!(
        (2.0 * past_edge - gap).abs() < 0.01,
        "{past_edge}px past each edge, {gap}px gap"
    );
    let reached = height - 2.0 * border + 2.0 * inset;
    assert!(
        (reached - 40.0).abs() < 0.01,
        "{height}px tall, {border}px border, {inset}px reach: {reached}px"
    );
}

#[test]
fn a_phone_s_card_actions_keep_their_spacing() {
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
        "inset: calc(-1 * var(--space-1) - 1px);",
    ] {
        assert!(reach.contains(want), "{want} in {reach}");
    }
    let row = rule(MAGIC, media, ".card-action-row {").expect("the row rule");
    assert!(
        row.contains("gap: var(--space-2);"),
        "two 4px reaches meet in an 8px gap: {row}"
    );
    assert!(
        row.contains("margin-top: var(--space-2);"),
        "a stacked row stands as far from the one above: {row}"
    );
    let last = rule(MAGIC, media, ".card-action-row:last-child {").expect("the last row's rule");
    assert!(
        last.contains("margin-bottom: var(--space-1);"),
        "the bottom row's reach stays inside a clipping row: {last}"
    );
}
