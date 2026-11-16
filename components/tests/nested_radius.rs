//! Nested corners: a rounded box that sits near another's corner takes the outer radius minus
//! the border and the inset between them (never under 0.15rem), so the gap between the two
//! curves stays even around the corner. The outer radius and the inset are tokens on the
//! container, as in orangui's dock, Cairn's tiles and the portfolio's phone frame.

const SCREENS_COMPONENTS: &str = include_str!("../assets/screens/components.css");
const SCREENS_BASE: &str = include_str!("../assets/screens/base.css");
const WEB: &str = include_str!("../../zwiper-web/assets/web.css");

/// The body of the rule for `selector` in `sheet`.
fn rule(sheet: &'static str, selector: &str) -> &'static str {
    let Some(at) = sheet.find(&format!("\n{selector} {{")) else {
        return "";
    };
    sheet
        .get(at..)
        .and_then(|rest| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

/// The nested radius of a deck row's art: the row's corner less its border and the art's own
/// inset, which is smaller than the text's, so the art sits nearer the corner and rounder.
const DECK_ART_RADIUS: &str =
    "border-radius: max(0.15rem, calc(var(--deck-row-radius) - 1px - var(--deck-row-art-inset)));";

#[test]
fn a_deck_rows_art_follows_the_rows_corner() {
    let row = rule(SCREENS_COMPONENTS, ".deck-row");
    assert!(row.contains("--deck-row-radius: 1rem;"), "{row:?}");
    assert!(row.contains("--deck-row-inset: var(--space-3);"), "{row:?}");
    assert!(
        row.contains("padding: var(--deck-row-inset) var(--space-4);"),
        "{row:?}"
    );
    assert!(
        row.contains("border-radius: var(--deck-row-radius);"),
        "{row:?}"
    );
    assert!(
        row.contains("--deck-row-art-inset: var(--space-1);"),
        "{row:?}"
    );
    for art in [".deck-row-art-img", ".skeleton-deck-list-art"] {
        let body = rule(SCREENS_COMPONENTS, art);
        assert!(body.contains(DECK_ART_RADIUS), "{art}: {body:?}");
    }
}

#[test]
fn a_hovered_deck_row_on_the_web_keeps_the_rows_corner() {
    assert!(
        WEB.contains(
            ".card-row.deck-row:not(.expanded):hover {\n        border-radius: var(--deck-row-radius);"
        ),
        "the web's hover corner is the row's token"
    );
}

#[test]
fn a_docks_text_buttons_follow_the_docks_corner() {
    let button = rule(SCREENS_BASE, ".action-dock > .btn-action");
    assert!(
        button.contains("margin: var(--dock-inset) 0;"),
        "{button:?}"
    );
    assert!(
        button.contains(
            "border-radius: max(0.15rem, calc(var(--dock-radius) - 1px - var(--dock-inset)));"
        ),
        "as orangui's dock buttons: {button:?}"
    );
}

#[test]
fn a_deck_rows_art_reaches_out_to_its_own_inset() {
    // Out past the text's padding to the art's inset at the top and bottom, and the first crop
    // at the left, filling the row's height, so the text and chips stay where they were.
    let reach = "calc(var(--deck-row-art-inset) - var(--deck-row-inset))";
    for art in [".deck-row-art-img", ".skeleton-deck-list-art"] {
        let body = rule(SCREENS_COMPONENTS, art);
        assert!(body.contains("align-self: stretch;"), "{art}: {body:?}");
        // The art fills the height the text and chips set and never drives it: no size of its
        // own (size containment) and no minimum.
        assert!(body.contains("contain: size;"), "{art}: {body:?}");
        assert!(!body.contains("min-height"), "{art}: {body:?}");
        assert!(
            body.contains(&format!("margin-top: {reach};")),
            "{art}: {body:?}"
        );
        assert!(
            body.contains(&format!("margin-bottom: {reach};")),
            "{art}: {body:?}"
        );
    }
    let first = rule(
        SCREENS_COMPONENTS,
        ".deck-list-row > :is(.deck-row-art-img, .skeleton-deck-list-art):first-child",
    );
    assert!(
        first.contains("margin-left: calc(var(--deck-row-art-inset) - var(--space-4));"),
        "{first:?}"
    );
}
