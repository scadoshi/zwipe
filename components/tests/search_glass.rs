//! The search glass is a heavier, rounder lens: a 6.5 lens centered at 10.5 on the 24 box and
//! its handle to 20, drawn at a 2.75 stroke with round caps and joins, so it reads at 17px. Every
//! glass is the card search bar's, in the nav, the phone header and zwipe.net.

const CARD_SEARCH: &str = include_str!("../src/card_search.rs");

#[test]
fn the_glass_is_a_smaller_lens_and_a_shorter_handle() {
    assert_eq!(
        components::SEARCH_ICON,
        "M10.5 4a6.5 6.5 0 1 0 0 13 6.5 6.5 0 1 0 0-13M20 20l-4.8-4.8"
    );
}

#[test]
fn the_glass_draws_heavier_than_the_other_icons_with_round_ends() {
    assert_eq!(components::SEARCH_STROKE, "2.75");
    let path = CARD_SEARCH
        .split_once("path {\n")
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body);
    for want in [
        "d: SEARCH_ICON,",
        "stroke_width: SEARCH_STROKE,",
        "stroke_linecap: \"round\",",
        "stroke_linejoin: \"round\",",
    ] {
        assert!(path.contains(want), "{want} on the glass's path: {path:?}");
    }
}
