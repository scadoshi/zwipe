//! Read-only chips (a price, a set, a badge, a parent tag, a keyword in a guide, the ghost of a
//! stat chip) take the kit's read-only corner, `--radius-readonly`, the one the kit's stat chips
//! wear, so only a chip that acts has the button corner: the result chips you tap keep
//! `--radius-control` over the no-results chip they share a base with, and the search glass keeps
//! it over the stat chip it is drawn as.

const MAGIC: &str = include_str!("../assets/magic.css");
const GUIDES: &str = include_str!("../assets/guides.css");
const SCREEN_COMPONENTS: &str = include_str!("../assets/screens/components.css");
const SCREENS_A: &str = include_str!("../assets/screens/screens-a.css");
const SCREENS_B: &str = include_str!("../assets/screens/screens-b.css");
const FILTER: &str = include_str!("../assets/filter.css");

/// The body of the rule whose selector's last line is `selector`, in `sheet`.
fn rule(sheet: &'static str, selector: &str) -> &'static str {
    let Some(at) = sheet.find(&format!("\n{selector} {{")) else {
        return "";
    };
    sheet
        .get(at..)
        .and_then(|rest| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn read_only_chips_take_the_read_only_corner() {
    for (sheet, selector) in [
        (MAGIC, ".card-row-stat"),
        (MAGIC, ".card-row-colors i.ms.ms-cost"),
        (MAGIC, ".detail-chip"),
        (MAGIC, ".card-details-ghost .card-details-ghost-label"),
        (MAGIC, ".card-details-ghost .card-details-ghost-tag"),
        (MAGIC, ".card-prices > .card-price-ghost"),
        (MAGIC, ".chip-note"),
        (GUIDES, ".guide-kw"),
        (SCREEN_COMPONENTS, ".skeleton-chip.skeleton-tinted"),
        (SCREENS_A, ".skeleton-bar.skeleton-price-chip"),
        (SCREENS_B, ".flavor-price"),
        (SCREENS_B, ".chip-bubble"),
        (SCREENS_B, ".chip-bubble-error"),
        (SCREENS_B, ".badge-verified"),
        (SCREENS_B, ".badge-expiry"),
        (SCREENS_B, ".badge-unverified"),
        (SCREENS_B, ".dict-parent"),
        (SCREENS_B, ".deck-featured-chips > .stat-chip"),
    ] {
        let body = rule(sheet, selector);
        assert!(
            body.contains("border-radius: var(--radius-readonly);"),
            "{selector}: {body:?}"
        );
    }
}

#[test]
fn the_card_row_labels_keep_the_kit_corner() {
    for (sheet, selector) in [
        (MAGIC, ".keyword-chips > .chip-row-label"),
        (MAGIC, ".deck-controls .chip-row-label"),
        (GUIDES, ".guide-filter > .chip-row-label"),
        (SCREENS_B, ".chip-row-label"),
    ] {
        let body = rule(sheet, selector);
        assert!(!body.is_empty(), "{selector} is in its sheet");
        assert!(
            !body.contains("border-radius"),
            "{selector} takes the stat chip's read-only corner from the kit: {body:?}"
        );
    }
}

#[test]
fn tappable_result_chips_keep_the_control_corner() {
    for (sheet, selector) in [
        (MAGIC, ".card-search-chip"),
        (MAGIC, ".card-search-label"),
        (FILTER, ".card-filter-result"),
        (FILTER, ".search-float-chip"),
    ] {
        let body = rule(sheet, selector);
        assert!(
            body.contains("border-radius: var(--radius-control);"),
            "{selector} is a chip you tap: {body:?}"
        );
    }
}
