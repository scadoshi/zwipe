//! The deck page's card details drawer leaves the page live wherever it opens from: a stack, a
//! featured card in the deck head or a row, in List as in Stacks.

/// The deck page's own sheet.
const SHEET: &str = components::DECK_PAGE_CSS;

#[test]
fn on_a_phone_the_details_drawer_dims_and_its_dim_only_closes_it() {
    let rule = ".sheet-form-drawer:has(> .deck-stack-details) > .sheet-backdrop {";
    let at = SHEET.find(rule).unwrap_or_else(|| panic!("no rule {rule}"));
    let before = SHEET.get(..at).unwrap_or_default();
    let opened = before.rfind("@media (min-width: 900px) {");
    let closed = before.rfind("\n}\n");
    assert!(
        opened.is_some() && opened > closed,
        "the undimmed drawer is the wide layout's only"
    );
}

#[test]
fn the_details_drawer_never_dims_the_deck_page_in_either_view() {
    let rule = ".sheet-form-drawer:has(> .deck-stack-details) > .sheet-backdrop {";
    let at = SHEET.find(rule).unwrap_or_else(|| panic!("no rule {rule}"));
    let body = SHEET
        .get(at..)
        .and_then(|s| s.split('}').next())
        .unwrap_or_default();
    assert!(body.contains("background: transparent;"), "{body}");
    assert!(body.contains("pointer-events: none;"), "{body}");
    assert!(
        !SHEET.contains("body:has(.deck-stacks) .sheet-form-drawer"),
        "the rule holds in List too, not only while Stacks shows"
    );
}
