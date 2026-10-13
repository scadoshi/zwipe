//! Status colors mean an outcome. Decoration draws from the palette slots. No
//! rule in the Magic sheet marks an outcome, so none may name a status color.

use components::status_uses;

#[test]
fn status_colors_only_where_they_mean_something() {
    let offenders: Vec<String> = status_uses(include_str!("../assets/magic.css"))
        .iter()
        .map(|u| format!("{} uses {}", u.selector, u.variable))
        .collect();
    assert!(
        offenders.is_empty(),
        "status colors used as decoration (move to a --palette-N slot):\n  {}",
        offenders.join("\n  ")
    );
}
