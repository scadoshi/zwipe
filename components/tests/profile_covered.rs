//! The app's deck Profile under its Cards layer: the Cards layer is a child of the Profile's
//! screen, so a covered Profile hides its own children and never itself, which would hide the
//! Cards with it.

const MAGIC: &str = include_str!("../assets/magic.css");

#[test]
fn a_covered_profile_hides_its_children_but_the_cards_layer() {
    let rule = ".screen.profile-covered > :not(.app-cards-layer) {";
    let at = MAGIC.find(rule);
    assert!(at.is_some(), "no rule {rule}");
    let body = at
        .and_then(|at| MAGIC.get(at..))
        .and_then(|s| s.split('}').next())
        .unwrap_or_default();
    assert!(body.contains("visibility: hidden !important;"), "{body}");
}
