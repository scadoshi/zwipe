//! A deck's Cards layer opens and closes as a route change does: there at once, with no slide.
//! Its keyframes still run, for a moment, since the closing one's end drops the layer.

const MAGIC: &str = include_str!("../assets/magic.css");

/// The body of the first rule that starts with `head`, if there is one.
fn rule(head: &str) -> Option<&'static str> {
    let at = MAGIC.find(head)?;
    MAGIC.get(at..).and_then(|s| s.split('}').next())
}

#[test]
fn the_cards_layer_appears_and_leaves_at_once_with_nothing_moving() {
    for name in ["deck-cards-layer-in", "deck-cards-layer-out"] {
        let frames = rule(&format!("@keyframes {name} {{")).unwrap_or_default();
        assert!(!frames.is_empty(), "no keyframes {name}");
        assert!(
            !frames.contains("translate"),
            "{name} moves nothing: {frames}"
        );
    }
    let layer = rule(".deck-cards-layer,\n.app-cards-layer > .screen {").unwrap_or_default();
    assert!(layer.contains("1ms"), "{layer}");
    let leaving = rule(".deck-cards-layer.leaving,\n.app-cards-layer.leaving > .screen {")
        .unwrap_or_default();
    assert!(leaving.contains("1ms"), "{leaving}");
}
