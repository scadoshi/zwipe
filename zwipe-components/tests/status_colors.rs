//! Status colors mean an outcome. Decoration draws from the palette slots. Every
//! status reference in the crate's own stylesheets must be one of these.

use zwipe_components::status_uses;

const ALLOWED: [&str; 6] = [
    ".btn-danger, .util-btn-danger",
    ".status-doing",
    ".status-done",
    ".toast-container [data-type=\"error\"]",
    ".toast-container [data-type=\"success\"]",
    ".toast-container [data-type=\"warning\"]",
];

#[test]
fn status_colors_only_where_they_mean_something() {
    let sheets = [
        include_str!("../assets/components.css"),
        include_str!("../assets/toast.css"),
        include_str!("../assets/app.css"),
    ];
    let offenders: Vec<String> = sheets
        .iter()
        .flat_map(|css| status_uses(css))
        .filter(|u| !ALLOWED.contains(&u.selector.as_str()))
        .map(|u| format!("{} uses {}", u.selector, u.variable))
        .collect();
    assert!(
        offenders.is_empty(),
        "status colors used as decoration (move to a --palette-N slot, or add the selector to ALLOWED if it marks an outcome):\n  {}",
        offenders.join("\n  ")
    );
}
