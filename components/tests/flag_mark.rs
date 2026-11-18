//! A stack card's flag and quantity badge sit side by side at its top right, in one holder, so a
//! flagged card with copies shows both cleanly. The flag is the quantity badge's twin: the same
//! translucent box at the same size, with its icon at the badge's text size.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: slices the stylesheet after a match it found"
)]

const MAGIC: &str = include_str!("../assets/magic.css");
const CARD_STACK: &str = include_str!("../src/card_stack.rs");

/// The body of the first rule whose selector, on its own line, is `selector`.
fn rule(selector: &str) -> &'static str {
    let open = format!("\n{selector} {{");
    let Some(at) = MAGIC.find(&open) else {
        return "";
    };
    MAGIC[at + open.len()..]
        .split_once('}')
        .map_or("", |(body, _)| body)
}

#[test]
fn the_flag_and_the_quantity_share_one_holder_at_the_top_right() {
    let holder = rule(".card-stack-badges");
    for declaration in [
        "position: absolute;",
        "top: 0.35rem;",
        "right: 0.35rem;",
        "display: flex;",
        "gap: var(--space-1);",
    ] {
        assert!(holder.contains(declaration), "{declaration} in {holder:?}");
    }
    let badges = rule(".card-stack-qty,\n.card-stack-flag");
    assert!(
        !badges.contains("position: absolute"),
        "neither badge places itself, so they cannot overlap: {badges:?}"
    );
    let (_, after_holder) = CARD_STACK
        .split_once("div { class: \"card-stack-badges\",")
        .unwrap_or_default();
    let (flag, qty) = (
        after_holder.find("class: \"card-stack-flag\""),
        after_holder.find("class: \"card-stack-qty\""),
    );
    assert!(
        flag.is_some() && qty.is_some(),
        "both badges render inside the holder"
    );
}

#[test]
fn the_flag_wears_the_quantity_badge_s_box() {
    let badges = rule(".card-stack-qty,\n.card-stack-flag");
    for declaration in [
        "padding: 0.05rem var(--space-2);",
        "border: 1px solid var(--border-secondary);",
        "border-radius: 0.5rem;",
        "font-size: 0.75rem;",
        "background-color: color-mix(in srgb, var(--bg-primary) 85%, transparent);",
    ] {
        assert!(badges.contains(declaration), "{declaration} in {badges:?}");
    }
    let icon = rule(".card-stack-flag > .action-dock-icon");
    assert!(
        icon.contains("width: 1em;") && icon.contains("height: 1em;"),
        "the flag's icon is the badge's text size: {icon:?}"
    );
}
