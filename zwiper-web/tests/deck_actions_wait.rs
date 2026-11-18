//! The deck page's Clone and Share wait for the deck: both stay off until its profile has
//! loaded, in the Profile's dock and the Cards layer's alike, so Clone never opens without the
//! deck's name and Share never reads a shared deck as unshared. Create link keeps the Share
//! dialog open until the link lands, so Share can't be reopened over a link still on its way.

/// The deck page's own source, which builds the Profile's dock.
const VIEW: &str = include_str!("../src/lib/inbound/screens/deck/view/mod.rs");

/// The wide page's source, which builds the Cards layer's dock.
const PAGE: &str = include_str!("../src/lib/inbound/screens/deck/view/page.rs");

/// The deck page's dialogs.
const DIALOGS: &str = include_str!("../src/lib/inbound/screens/deck/view/dialogs.rs");

/// The `button { .. }` in `src` labeled `label`, up to its icon.
fn button(src: &str, label: &str) -> String {
    let at = format!("\"aria-label\": MoreItem::{label}.short_label(),");
    let (before, after) = src
        .split_once(&at)
        .unwrap_or_else(|| panic!("a {label} button"));
    let start = before.rfind("button {").unwrap_or_default();
    let end = after.find("Icon {}").unwrap_or(after.len());
    let head = before.get(start..).unwrap_or_default();
    let tail = after.get(..end).unwrap_or_default();
    // Joined by the label line the split took out.
    format!("{head}{at}{tail}")
}

#[test]
fn clone_and_share_stay_off_until_the_deck_has_loaded_in_both_docks() {
    for (name, src) in [
        ("the Profile's dock", VIEW),
        ("the Cards layer's dock", PAGE),
    ] {
        for label in ["CloneDeck", "ShareDeck"] {
            let button = button(src, label);
            assert!(
                button.contains("disabled: !profile_loaded,"),
                "{label} in {name} waits for the deck: {button}"
            );
        }
    }
    assert!(
        VIEW.contains("let profile_loaded = matches!(loaded_profile, Some(Ok(_)));"),
        "the deck has loaded once its profile has"
    );
}

#[test]
fn create_link_keeps_share_open_until_the_link_lands() {
    assert!(
        DIALOGS.contains("ActionButton::new(\"Create link\")"),
        "Create link is the dialog's own busy action"
    );
    let create = DIALOGS
        .split("ActionButton::new(\"Create link\")")
        .nth(1)
        .unwrap_or_default();
    assert!(
        create.trim_start().starts_with(".busy(\"Creating link…\""),
        "it shows the wait and closes once the link is in: {create}"
    );
    assert!(
        !DIALOGS.contains("label: \"Create link\".into(),"),
        "no Create link that closes the dialog before the link lands"
    );
}
