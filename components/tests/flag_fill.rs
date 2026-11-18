//! The flag fills the way the star does: one rule fills a pressed Star or Flag button's icon, so
//! each shows an outline off and a filled shape on, and a flagged card's tile mark, which only
//! shows while the card is flagged, is filled too.

const MAGIC: &str = include_str!("../assets/magic.css");

/// The pressed star's and flag's shared fill rule.
const PRESSED_FILL: &str = "\n.card-action-icon:is([data-action=\"star\"], [data-action=\"flag\"])[aria-pressed=\"true\"] > .action-dock-icon {\n    fill: currentColor;\n}";

#[test]
fn one_rule_fills_a_pressed_star_or_flag() {
    assert!(MAGIC.contains(PRESSED_FILL), "the shared fill rule");
    assert!(
        !MAGIC.contains("[data-action=\"flag\"][aria-pressed=\"true\"] > .action-dock-icon"),
        "the flag has no fill rule of its own"
    );
}

#[test]
fn a_flagged_card_s_tile_mark_is_filled() {
    assert!(MAGIC.contains(
        "\n.card-row-flag > .action-dock-icon,\n.card-stack-flag > .action-dock-icon,\n.card-stack-star > .action-dock-icon {\n    fill: currentColor;\n}"
    ));
}
