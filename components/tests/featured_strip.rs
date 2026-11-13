//! The featured strip's cards keep one width whatever their count, so the strip's height holds
//! when a card joins or leaves, and a third card scrolls rather than shrinking the rest.

/// The body of the stylesheet's `.deck-featured-card` rule.
#[expect(
    clippy::expect_used,
    reason = "a test helper: a stylesheet without the rule fails the test"
)]
fn card_rule() -> &'static str {
    let css = include_str!("../assets/screens/screens-b.css");
    let start = css
        .find("\n.deck-featured-card {")
        .expect("screens-b.css has a .deck-featured-card rule");
    let body = &css[start..];
    &body[..body.find('}').expect("the rule closes")]
}

#[test]
fn a_featured_card_never_grows_or_shrinks_with_the_count() {
    let rule = card_rule();
    assert!(
        rule.contains("flex: 0 0 min(11rem, calc((100% - 0.5rem) / 2));"),
        "each card is half the row up to 11rem, neither growing nor shrinking: {rule}"
    );
    assert!(
        !rule.contains("transition: flex-grow"),
        "no width share to ease when a card joins: {rule}"
    );
}
