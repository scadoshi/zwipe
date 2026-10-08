//! Status colors mean an outcome. Decoration draws from the palette slots. Every
//! status reference in zite's stylesheet must be one of these.

use zwipe_components::status_uses;

const ALLOWED: [&str; 7] = [
    ".dead-end .panel-title",
    ".shipped-tag",
    ".status-message.error",
    ".status-message.success",
    ".swipe-add",
    ".swipe-maybe",
    ".swipe-skip",
];

#[test]
fn status_colors_only_where_they_mean_something() {
    let offenders: Vec<String> = status_uses(include_str!("../assets/style.css"))
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
