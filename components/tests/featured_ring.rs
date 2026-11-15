//! The keyboard cursor's ring on a featured card follows the card's rounded corners: it rings
//! the rounded image, offset as a stack card's ring is, so the ring's radius is the card's plus
//! the gap, instead of squaring off the card's stop.

#![expect(
    clippy::panic,
    reason = "test helpers: a rule that is missing fails the test"
)]

const MAGIC: &str = include_str!("../assets/magic.css");
const SCREENS: &str = include_str!("../assets/screens/screens-b.css");

/// The declarations of the rule whose selector is exactly `selector`.
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

/// The stack card's ring offset, which the featured rings match.
fn stack_offset() -> &'static str {
    let ring = rule(MAGIC, ".card-stack-card:focus-visible");
    assert!(ring.contains("outline-offset: 2px;"), "{ring}");
    "outline-offset: 2px;"
}

#[test]
fn a_featured_card_in_the_wide_head_rings_its_rounded_image() {
    let stop = rule(MAGIC, ".deck-head-card-image:focus-visible");
    assert!(
        stop.contains("outline: none;"),
        "no square ring on the stop: {stop}"
    );
    let ring = rule(MAGIC, ".deck-head-card-image:focus-visible img");
    assert!(
        ring.contains("outline: 2px solid var(--accent-primary);"),
        "{ring}"
    );
    assert!(ring.contains(stack_offset()), "{ring}");
    let image = rule(MAGIC, ".deck-head-card .deck-head-card-image img");
    assert!(
        image.contains("border-radius: 4.75% / 3.5%;"),
        "the ringed image is rounded: {image}"
    );
}

#[test]
fn a_featured_card_in_the_phone_strip_rings_its_rounded_image() {
    let stop = rule(SCREENS, ".deck-featured-card:focus-visible");
    assert!(
        stop.contains("outline: none;"),
        "no square ring around image and chips: {stop}"
    );
    let ring = rule(
        SCREENS,
        ".deck-featured-card:focus-visible .deck-featured-image",
    );
    assert!(
        ring.contains("outline: 2px solid var(--accent-primary);"),
        "{ring}"
    );
    assert!(ring.contains(stack_offset()), "{ring}");
    let image = rule(SCREENS, ".deck-featured-image");
    assert!(
        image.contains("border-radius: 4.75% / 3.5%;"),
        "the ringed box is rounded: {image}"
    );
}
