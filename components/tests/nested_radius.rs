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

/// The nested radius of a deck row's art: the row's corner, the panel radius, less its border
/// and the art's own inset, which is smaller than the text's, so the art sits nearer the corner
/// and rounder.
const DECK_ART_RADIUS: &str =
    "border-radius: max(0.15rem, calc(var(--panel-radius) - 1px - var(--deck-row-art-inset)));";

#[test]
fn a_deck_rows_art_follows_the_rows_corner() {
    let row = rule(SCREENS_COMPONENTS, ".deck-row");
    assert!(row.contains("--deck-row-inset: var(--space-3);"), "{row:?}");
    assert!(
        row.contains("padding: var(--deck-row-inset) var(--space-4);"),
        "{row:?}"
    );
    assert!(
        row.contains("border-radius: var(--panel-radius);"),
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
            ".card-row.deck-row:not(.expanded):hover {\n        border-radius: var(--panel-radius);"
        ),
        "the web's hover corner is the row's, the panel radius"
    );
}

#[test]
fn a_docks_text_buttons_take_the_chip_corner_the_dock_is_built_around() {
    let button = rule(SCREENS_BASE, ".action-dock > .btn-action");
    assert!(
        button.contains("margin: var(--dock-inset) 0;"),
        "{button:?}"
    );
    assert!(
        button.contains("border-radius: var(--chip-radius);"),
        "as orangui's dock buttons: {button:?}"
    );
}

#[test]
fn a_deck_rows_art_is_a_square_tucked_into_the_rows_corner() {
    // A fixed square at the art's inset from the top and the left, cropped from its center,
    // that never stretches to the row's height or drives it: the reach above and below
    // takes the inset back out of the room it asks the row for.
    let reach = "calc(var(--deck-row-art-inset) - var(--deck-row-inset))";
    for art in [".deck-row-art-img", ".skeleton-deck-list-art"] {
        let body = rule(SCREENS_COMPONENTS, art);
        for want in [
            "width: var(--deck-row-art-size);",
            "height: var(--deck-row-art-size);",
            "align-self: flex-start;",
            &format!("margin-top: {reach};"),
            &format!("margin-bottom: {reach};"),
        ] {
            assert!(body.contains(want), "{art}: {want} in {body:?}");
        }
        for gone in ["stretch", "contain:", "min-height"] {
            assert!(!body.contains(gone), "{art}: no {gone} in {body:?}");
        }
    }
    let img = rule(SCREENS_COMPONENTS, ".deck-row-art-img");
    assert!(img.contains("object-fit: cover;"), "{img:?}");
    assert!(img.contains("object-position: center;"), "{img:?}");
    let row = rule(SCREENS_COMPONENTS, ".deck-row");
    assert!(row.contains("--deck-row-art-size: 4.5rem;"), "{row:?}");
    let first = rule(
        SCREENS_COMPONENTS,
        ".deck-list-row > :is(.deck-row-art-img, .skeleton-deck-list-art):first-child",
    );
    assert!(
        first.contains("margin-left: calc(var(--deck-row-art-inset) - var(--space-4));"),
        "{first:?}"
    );
}
