//! Status colors mean an outcome. Decoration draws from the palette slots. The
//! one rule in the Magic sheet that marks an outcome is the Danger zone title,
//! over a panel of destructive actions, and the guides' sheet colors a swipe's
//! outcome on its legend; no other rule in either may name a status color.

use components::status_uses;

const ALLOWED: [&str; 4] = [
    ".settings-list-title.danger-zone-title",
    // A swipe's direction on a guide's legend: add, skip, maybe.
    ".swipe-add",
    ".swipe-maybe",
    ".swipe-skip",
];

#[test]
fn status_colors_only_where_they_mean_something() {
    let offenders: Vec<String> = [
        include_str!("../assets/magic.css"),
        include_str!("../assets/guides.css"),
    ]
    .into_iter()
    .flat_map(status_uses)
    .filter(|u| !ALLOWED.contains(&u.selector.as_str()))
    .map(|u| format!("{} uses {}", u.selector, u.variable))
    .collect();
    assert!(
        offenders.is_empty(),
        "status colors used as decoration (move to a --palette-N slot, or add the selector to ALLOWED if it marks an outcome):\n  {}",
        offenders.join("\n  ")
    );
}
