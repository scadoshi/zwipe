//! On the web each card row sits in its own `.web-card-row`, so the rows Show more deals are the
//! ghosts' siblings through that wrapper: they ease in at once over the leaving ghosts rather than
//! on the cascade's tail, and fade in under reduced motion.

const WEB: &str = include_str!("../assets/web.css");

#[test]
fn the_dealt_rows_ease_in_at_once_through_their_wrapper() {
    let cascade = WEB
        .find(".maybeboard-list > .web-card-row:nth-child(8) > .card-row {")
        .unwrap_or(0);
    let dealt = WEB.find(
        "    .maybeboard-list .card-list-ghosts ~ .web-card-row > .card-row {\n        animation-delay: 0ms;\n    }",
    );
    assert!(
        dealt.is_some_and(|at| at > cascade),
        "after the cascade it overrides"
    );
}

#[test]
fn the_dealt_rows_fade_in_under_reduced_motion() {
    assert!(WEB.contains(
        "@media (prefers-reduced-motion: reduce) {\n    .maybeboard-list .card-list-ghosts ~ .web-card-row > .card-row {\n        animation: card-list-ghosts-fade-in var(--enter-duration) var(--ease-enter) both;\n    }\n}"
    ));
}
