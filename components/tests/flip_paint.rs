//! A card's Flip and Meld buttons show only with the card's image: never inside a stack that
//! still holds its ghost or hides its cards for a reveal, where a phone's browser can paint the
//! button's own layer over an empty card, and fading in with the image rather than at full
//! strength over an image still fading in.

#![expect(
    clippy::panic,
    reason = "test helpers: a rule that is missing fails the test"
)]

const MAGIC: &str = include_str!("../assets/magic.css");
const SCREENS: &str = include_str!("../assets/screens/screens-a.css");

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

#[test]
fn no_flip_or_meld_paints_while_its_stack_holds_its_ghost_or_hides_its_cards() {
    let hidden = rule(
        SCREENS,
        ".swipe-stack.ghost-held .card-flip-button,\n.swipe-reveal.revealing .card-flip-button",
    );
    assert!(
        hidden.contains("display: none;"),
        "gone from the page, not only hidden by an ancestor: {hidden}"
    );
}

#[test]
fn flip_and_meld_fade_in_with_the_image() {
    let image = rule(MAGIC, ".flippable-card-wrapper img");
    assert!(
        image.contains("opacity 250ms ease"),
        "the image's fade: {image}"
    );
    let button = rule(MAGIC, ".card-flip-button");
    assert!(
        button.contains("animation: flip-button-in 250ms ease;"),
        "the button's own fade, as long as the image's: {button}"
    );
    let frames = rule(MAGIC, "@keyframes flip-button-in");
    assert!(frames.contains("from"), "{frames}");
    let still = rule(SCREENS, ".swipe-reveal > .swipe-stack .card-flip-button");
    assert!(
        still.contains("animation: none;"),
        "where the stack shows its images without a fade, its buttons show without one: {still}"
    );
}
