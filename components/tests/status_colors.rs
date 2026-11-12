//! Status colors mean an outcome. Decoration draws from the palette slots. The
//! rules in the Magic sheet that may name one: the Danger zone title, over a
//! panel of destructive actions, and the starred Star toggle, which wears the
//! warning gold the Star hint draws in (Scotty's pick), a gold no palette
//! slot holds in every theme.

use components::status_uses;

const ALLOWED: [&str; 2] = [
    ".settings-list-title.danger-zone-title",
    ".card-action-btn.card-action-icon[data-action=\"star\"][aria-pressed=\"true\"]",
];

#[test]
fn status_colors_only_where_they_mean_something() {
    let offenders: Vec<String> = status_uses(include_str!("../assets/magic.css"))
        .into_iter()
        .filter(|u| !ALLOWED.contains(&u.selector.as_str()))
        .map(|u| format!("{} uses {}", u.selector, u.variable))
        .collect();
    assert!(
        offenders.is_empty(),
        "status colors used as decoration (move to a --palette-N slot, or add the selector to ALLOWED if it marks an outcome):\n  {}",
        offenders.join("\n  ")
    );
}
