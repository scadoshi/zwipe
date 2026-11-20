//! Show more's ghost rows on a card list: they fold open and fade in at the foot of the list the
//! moment Show more is pressed, then lie under the rows its page deals and fade as those rows ease
//! in over them, and stay hidden between the two runs of rows once gone. Under reduced motion they
//! only fade, and the dealt rows fade in.

#![expect(
    clippy::panic,
    reason = "test helpers: a rule that is missing fails the test"
)]

use zwipe_ui::commander::more_ghosts::{ENTER_ANIMATION, FADE_IN_ANIMATION, LEAVE_ANIMATION};

const SCREENS_B: &str = include_str!("../assets/screens/screens-b.css");

/// The declarations of the top-level rule whose selector is exactly `selector`.
fn rule(selector: &str) -> &'static str {
    let opening = format!("\n{selector} {{");
    let (_, rest) = SCREENS_B
        .split_once(&opening)
        .unwrap_or_else(|| panic!("a rule for {selector}"));
    let (body, _) = rest
        .split_once('}')
        .unwrap_or_else(|| panic!("the end of {selector}"));
    body
}

#[test]
fn the_sheet_defines_every_animation_the_model_waits_on() {
    for name in [ENTER_ANIMATION, FADE_IN_ANIMATION, LEAVE_ANIMATION] {
        assert!(
            SCREENS_B.contains(&format!("@keyframes {name} {{")),
            "no @keyframes {name}"
        );
    }
}

#[test]
fn the_ghosts_fold_open_as_they_fade_in() {
    let ghosts = rule(".card-list-ghosts");
    assert!(ghosts.contains("display: grid;"), "{ghosts}");
    assert!(ghosts.contains("grid-template-rows: 1fr;"), "{ghosts}");
    assert!(rule(".card-list-ghosts > *").contains("min-height: 0;"));
    let entering = rule(".card-list-ghosts-entering");
    assert!(entering.contains("overflow: hidden;"), "{entering}");
    assert!(
        entering.contains(
            "animation: card-list-ghosts-fade-in var(--enter-duration) var(--ease-enter) both;"
        ),
        "{entering}"
    );
    assert!(
        SCREENS_B.contains(
            "@media (prefers-reduced-motion: no-preference) {\n    .card-list-ghosts-entering {\n        animation-name: card-list-ghosts-enter;\n    }\n}"
        ),
        "with motion allowed they fold open too"
    );
    assert!(SCREENS_B.contains(
        "@keyframes card-list-ghosts-enter {\n    from {\n        grid-template-rows: 0fr;\n        opacity: 0;\n    }\n}"
    ));
    assert!(SCREENS_B.contains(
        "@keyframes card-list-ghosts-fade-in {\n    from {\n        opacity: 0;\n    }\n}"
    ));
}

#[test]
fn leaving_they_lie_under_the_dealt_rows_and_fade() {
    let leaving = rule(".card-list-ghosts-leaving");
    for line in [
        "position: absolute;",
        "left: 0;",
        "right: 0;",
        "z-index: -1;",
        "pointer-events: none;",
        "animation: card-list-ghosts-leave var(--enter-duration) var(--ease-enter) forwards;",
    ] {
        assert!(leaving.contains(line), "{line} in {leaving}");
    }
    assert!(
        !leaving.contains("top:"),
        "no top, so they stay where the dealt rows start"
    );
    let holder = rule(".collapsible-inner:has(> .card-list-ghosts-leaving)");
    assert!(holder.contains("position: relative;"), "{holder}");
    assert!(
        holder.contains("z-index: 0;"),
        "a stacking context, so they sit over the panel and under its rows"
    );
    assert!(
        SCREENS_B.contains(
            "@keyframes card-list-ghosts-leave {\n    to {\n        opacity: 0;\n    }\n}"
        )
    );
}

#[test]
fn gone_they_take_no_room_and_take_no_line_from_the_last_row() {
    assert!(rule(".card-list-ghosts-gone").contains("display: none;"));
    assert!(
        rule(".card-row:has(+ .card-list-ghosts-gone:last-child)").contains("border-bottom: none;")
    );
}

#[test]
fn the_dealt_rows_ease_in_at_once_and_fade_in_under_reduced_motion() {
    let cascade = SCREENS_B
        .find(".maybeboard-list .card-row:nth-child(8) {")
        .unwrap_or(0);
    let dealt = SCREENS_B
        .find(".maybeboard-list .card-list-ghosts ~ .card-row {\n    animation-delay: 0ms;\n}")
        .unwrap_or_else(|| panic!("the dealt rows wait on no cascade"));
    assert!(dealt > cascade, "it follows the cascade it overrides");
    let reduce = SCREENS_B
        .rfind("@media (prefers-reduced-motion: reduce) {\n    .maybeboard-list .card-list-ghosts ~ .card-row {\n        animation: card-list-ghosts-fade-in var(--enter-duration) var(--ease-enter) both;\n    }\n}")
        .unwrap_or_else(|| panic!("the dealt rows fade in under reduced motion"));
    let quiet = SCREENS_B
        .find("    .maybeboard-list .card-row {\n        animation: none;\n    }")
        .unwrap_or(0);
    assert!(
        reduce > quiet,
        "it follows the reduced-motion rule it overrides"
    );
}
