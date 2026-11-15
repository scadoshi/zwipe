//! A stock card (a printing with no image) flips by touch, and its chips don't animate as its
//! text is fitted to the card.

#![expect(
    clippy::panic,
    reason = "test helpers: a rule that is missing fails the test"
)]

const MAGIC: &str = include_str!("../assets/magic.css");

/// The declarations of the rule whose selector is exactly `selector`.
fn rule(selector: &str) -> &'static str {
    let opening = format!("\n{selector} {{");
    let (_, rest) = MAGIC
        .split_once(&opening)
        .unwrap_or_else(|| panic!("a rule for {selector}"));
    let (body, _) = rest
        .split_once('}')
        .unwrap_or_else(|| panic!("the end of {selector}"));
    body
}

#[test]
fn a_stock_card_s_flip_takes_a_finger_sized_tap() {
    let button = rule(".card-proxy .card-action-btn[data-action]");
    assert!(button.contains("position: relative;"), "{button}");
    assert!(
        button.contains("z-index: 1;"),
        "its tap area sits over the cost and name beside it: {button}"
    );
    let area = rule(".card-proxy .card-action-btn[data-action]::after");
    assert!(
        area.contains("inset: -0.75rem;"),
        "the tap area reaches past the shrunken icon, in rem so the card's scale never shrinks it: {area}"
    );
}

#[test]
fn a_stock_card_s_chips_hold_still_as_its_text_is_fitted() {
    let still = rule(".card-proxy *");
    assert!(still.contains("transition: none;"), "{still}");
}
