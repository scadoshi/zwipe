//! A phone's card actions stand at a comfortable touch size, while a wide window keeps them
//! compact.

const MAGIC: &str = include_str!("../assets/magic.css");

/// The body of the first rule in `css` from `selector`, after `from`.
fn rule<'css>(css: &'css str, from: usize, selector: &str) -> Option<&'css str> {
    let at = css.get(from..)?.find(selector)?.checked_add(from)?;
    css.get(at..)?.split('}').next()
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
    for want in ["content: \"\";", "position: absolute;", "inset: -0.375rem;"] {
        assert!(reach.contains(want), "{want} in {reach}");
    }
    let row = rule(MAGIC, media, ".card-action-row {").expect("the row rule");
    assert!(
        row.contains("gap: 0.75rem;"),
        "reaches don't overlap: {row}"
    );
    assert!(
        row.contains("margin-top: 0.75rem;"),
        "a stacked row stands clear of the one above: {row}"
    );
    let last = rule(MAGIC, media, ".card-action-row:last-child {").expect("the last row's rule");
    assert!(
        last.contains("margin-bottom: 0.375rem;"),
        "the bottom row's reach stays inside a clipping row: {last}"
    );
}
