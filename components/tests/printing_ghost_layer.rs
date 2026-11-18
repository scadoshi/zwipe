//! A printing's ghost in the cover flow is a layer of its own from the start, so an image that
//! arrives once the flow shows changes no layers: its ghost stops pulsing under it, and a ghost
//! whose pulse alone made it a layer would leave the page's layers, which a phone's browser
//! regroups by drawing the sheet afresh, the page behind it showing through for a few frames.

const MAGIC: &str = include_str!("../assets/magic.css");

/// The declarations of the first rule whose selector is exactly `selector`.
fn rule<'css>(selector: &str) -> &'css str {
    let head = format!("\n{selector} {{\n");
    MAGIC
        .split_once(&head)
        .and_then(|(_, rest)| rest.split_once("\n}"))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_printing_s_ghost_is_a_layer_of_its_own_from_the_start() {
    let ghost = rule(".cover-flow-ghost");
    assert!(
        ghost.contains("animation: skeleton-pulse"),
        "it pulses: {ghost}"
    );
    assert!(
        ghost.contains("will-change: opacity;"),
        "it keeps its layer once its pulse stops: {ghost}"
    );
}

#[test]
fn a_ghost_under_a_shown_image_holds_still() {
    assert_eq!(
        rule(".cover-flow-ghost:has(+ img.shown)").trim(),
        "animation: none;"
    );
}
