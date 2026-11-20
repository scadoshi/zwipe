//! A phone's stacks keep the main thread free: a raised card lifts and grows but holds still, and
//! a stack off the screen skips its layout and paint until it scrolls near.

const MAGIC: &str = include_str!("../assets/magic.css");

/// The body of the rule for `selector`, at the top level or nested once.
fn rule(selector: &str) -> &'static str {
    let open = format!("{selector} {{\n");
    MAGIC
        .split_once(&format!("\n{open}"))
        .or_else(|| MAGIC.split_once(&format!("\n    {open}")))
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_raised_card_in_a_tap_stack_holds_still() {
    let tap = rule(".card-stack.tap .floatable.floating");
    assert!(
        tap.contains("animation-play-state: paused;"),
        "no sway or bob, which restyle the page every frame: {tap:?}"
    );
    assert!(
        !tap.contains("--float-reach"),
        "the lift and grow still ease in: {tap:?}"
    );
    let hover = rule(".floatable.floating");
    assert!(
        hover.contains("animation-play-state: running;"),
        "a pointer's stacks keep the drift: {hover:?}"
    );
}

#[test]
fn a_tap_stack_off_the_screen_skips_its_work_and_keeps_its_place() {
    let stack = rule(".deck-stacks.deck-stacks-touch .deck-stack");
    assert!(stack.contains("content-visibility: auto;"), "{stack:?}");
    assert!(
        stack.contains("contain-intrinsic-size: auto"),
        "an unseen stack keeps the height it last had: {stack:?}"
    );
    assert!(
        stack.contains("padding: var(--stack-paint-room);"),
        "room inside the paint clip for the lift and the shadows: {stack:?}"
    );
    assert!(
        stack.contains(
            "margin: calc(-1 * var(--stack-paint-room)) calc(-1 * var(--stack-paint-room)) calc(var(--space-3) - var(--stack-paint-room));"
        ),
        "the room takes nothing from the layout: {stack:?}"
    );
    assert!(
        stack.contains("pointer-events: none;")
            && rule(".deck-stacks.deck-stacks-touch .deck-stack > *")
                .contains("pointer-events: auto;"),
        "the room takes no taps, the stack's own contents do: {stack:?}"
    );
}

#[test]
fn the_keyboard_cursor_s_result_card_floats_as_a_hovered_one_does() {
    let focused = rule(".card-list-results .card-stack-card.floatable:focus-visible");
    assert!(focused.contains("--float-reach: 1;"), "{focused:?}");
    assert!(
        focused.contains("animation-play-state: running;"),
        "{focused:?}"
    );
}
