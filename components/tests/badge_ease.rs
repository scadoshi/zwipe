//! A card's flag and quantity badges ease rather than snap: a badge that comes scales and fades
//! in, one that goes scales and fades out where it was, its neighbor slides over, and each change
//! pops the badge, a quick lift and scale-up that settles and starts over on the next tap. The
//! details drawer's count and Flag pop the same way. Transform and opacity only, on the kit's
//! entrance curve, and none of it under reduced motion.

const DECK_CARDS_VIEW: &str = include_str!("../src/deck_cards_view.rs");

#[test]
fn the_badges_script_is_installed_by_the_stacks_and_the_drawer() {
    let js = components::BADGE_EASE_JS;
    assert!(
        js.contains("if (window.__zwipeBadgeEase) return;"),
        "once a page: {js}"
    );
    for component in ["pub fn DeckStacks(", "pub fn DeckStackDrawer("] {
        let (_, body) = DECK_CARDS_VIEW.split_once(component).unwrap_or_default();
        let body = body.split("\n#[component]").next().unwrap_or_default();
        assert!(
            body.contains("crate::badge_ease::use_badge_ease();"),
            "{component} installs it"
        );
    }
}

#[test]
fn only_a_badge_coming_or_going_moves_never_a_card_or_a_deck_loading() {
    let js = components::BADGE_EASE_JS;
    assert!(
        js.contains("const isBadge = (node) => node.nodeType === 1 && (node.matches(BADGE) || node.matches(HOLDER)) && !node.hasAttribute('data-badge-copy');"),
        "a badge or its holder added or removed on its own, not inside a whole new card: {js}"
    );
}

#[test]
fn it_moves_by_transform_and_opacity_on_the_kit_s_curve_and_rests_under_reduced_motion() {
    let js = components::BADGE_EASE_JS;
    assert!(js.contains("getPropertyValue('--ease-enter')"), "{js}");
    assert!(
        js.contains("matchMedia('(prefers-reduced-motion: reduce)').matches"),
        "{js}"
    );
    for layout in ["width", "height", "margin", "left:", "top:"] {
        assert!(
            !js.split("animate(").skip(1).any(|call| call
                .split(')')
                .next()
                .unwrap_or_default()
                .contains(layout)),
            "no keyframe animates {layout}: {js}"
        );
    }
}

#[test]
fn each_change_pops_and_a_quick_run_of_taps_starts_each_pop_over() {
    let js = components::BADGE_EASE_JS;
    assert!(js.contains("const POP_MS = 200;"), "{js}");
    assert!(
        js.contains(
            "el.getAnimations().forEach((a) => { if (a.id === 'badge-pop') a.cancel(); });"
        ),
        "a new pop cancels the running one: {js}"
    );
    assert!(
        js.contains("composite: 'add'"),
        "the pop adds onto a slide in progress: {js}"
    );
    for target in [
        ".card-stack-qty",
        ".card-action-count",
        "[data-action=\"flag\"]",
    ] {
        assert!(js.contains(target), "{target} pops: {js}");
    }
}
