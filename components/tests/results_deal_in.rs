//! A results page's next page deals its cards in from below, a little apart, with the kit's
//! enter duration and easing, by transform and opacity alone; under reduced motion they fade in.

const MAGIC: &str = include_str!("../assets/magic.css");
const GRID: &str = include_str!("../src/card_results_grid.rs");

/// The body of the rule whose selector line is `selector`, at the top level or nested once.
fn rule(selector: &str) -> &'static str {
    let open = format!("{selector} {{\n");
    MAGIC
        .split_once(&format!("\n{open}"))
        .or_else(|| MAGIC.split_once(&format!("\n    {open}")))
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_next_page_deals_up_a_little_apart_with_the_kit_s_enter_motion() {
    let dealt = rule(".card-list-results > .card-list-dealt");
    assert!(
        dealt.contains(
            "animation: card-list-fade var(--enter-duration) var(--ease-enter) calc(min(var(--deal-n), 15) * 30ms) backwards;"
        ),
        "{dealt:?}"
    );
    let keyframes = MAGIC
        .split_once("@keyframes card-list-deal {")
        .and_then(|(_, rest)| rest.split_once("\n}"))
        .map_or("", |(body, _)| body);
    assert!(keyframes.contains("opacity: 0;"), "{keyframes}");
    assert!(
        keyframes.contains("transform: translateY(1.5rem);"),
        "{keyframes}"
    );
    for layout in ["top", "height", "margin", "width"] {
        assert!(
            !keyframes.contains(&format!("{layout}:")),
            "transform and opacity only, no {layout}: {keyframes}"
        );
    }
}

#[test]
fn the_deal_runs_only_without_reduced_motion_and_a_fade_otherwise() {
    let marker = "@media (prefers-reduced-motion: no-preference) {\n    .card-list-results > .card-list-dealt {";
    let (_, moving) = MAGIC.split_once(marker).unwrap_or_default();
    assert!(
        moving
            .trim_start()
            .starts_with("animation-name: card-list-deal;"),
        "{moving:.120}"
    );
    assert!(MAGIC.contains("@keyframes card-list-fade {"));
}

#[test]
fn the_grid_marks_the_newest_page_s_cards_with_their_place_in_it() {
    assert!(GRID.contains("\"card-list-dealt\""), "{GRID}");
    assert!(GRID.contains("--deal-n: {n}"), "{GRID}");
}
