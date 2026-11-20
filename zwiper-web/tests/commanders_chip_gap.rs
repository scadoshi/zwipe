//! The Commanders Suggested list's Show row sits a chip row's gap (`--space-2`) over the list, as
//! the Maybeboard's rows beside it do, and a panel gap under the Maybeboard's Danger zone once the
//! columns stack. Its loading ghost draws its header's `?` and Swipe, and Show more sits just
//! under the panel, offered once per deal.

const WEB: &str = include_str!("../assets/web.css");

/// The web skeletons that ghost a card list's header with its `?`.
const GHOSTS: [&str; 2] = [
    include_str!("../src/lib/inbound/screens/activity/mod.rs"),
    include_str!("../src/lib/inbound/screens/playstyle/skeleton.rs"),
];

#[test]
fn the_suggested_show_row_keeps_a_chip_row_s_gap_over_the_list() {
    assert!(
        WEB.contains(".web-suggested > .deck-controls {\n    margin-bottom: var(--space-2);\n}"),
        "the console's chip row zeroes its own margin, so the console carries it"
    );
}

#[test]
fn a_list_ghost_s_hint_follows_its_title() {
    for src in GHOSTS {
        let (_, header) = src
            .split_once(r#"class: "card-group-header card-group-header-with-hint","#)
            .unwrap();
        let header = header.split_once("for i in").unwrap().0;
        assert!(
            header.contains(r#"span { class: "card-list-fold-hint", GhostHint {} }"#),
            "{header}"
        );
    }
}

#[test]
fn stacked_under_the_maybeboard_the_suggested_column_keeps_a_panel_gap_over_its_show_row() {
    assert!(
        WEB.contains(
            "@media (max-width: 899px) {\n    .web-commanders-suggested {\n        margin-top: var(--panel-gap);\n    }\n}"
        ),
        "a phone stacks the columns with nothing between them, so the Show row would sit on the Danger zone"
    );
}

/// The Suggested list's source.
const SUGGESTED: &str = include_str!("../src/lib/inbound/screens/commanders/suggested.rs");

#[test]
fn the_suggested_ghost_draws_its_header_s_question_mark_and_swipe() {
    let (_, ghost) = SUGGESTED.split_once("if list.is_loading() {").unwrap();
    let ghost = ghost.split_once("for i in").unwrap().0;
    assert!(
        ghost.contains(r#"class: "card-group-header card-group-header-with-hint","#),
        "{ghost}"
    );
    let title = ghost.find("GhostText { chars: 9 }").unwrap();
    let hint = ghost
        .find(r#"span { class: "card-list-fold-hint", GhostHint {} }"#)
        .unwrap();
    let swipe = ghost.find("GhostButton { chars: 5 }").unwrap();
    assert!(title < hint && hint < swipe, "{ghost}");
}

/// The Suggested fold's source, which draws Show more.
const FOLD: &str = include_str!("../src/lib/inbound/screens/playstyle/suggestions.rs");

/// `SuggestedFold`'s body, from its signature to the next item.
fn suggested_fold() -> &'static str {
    let (_, body) = FOLD
        .split_once("pub(crate) fn SuggestedFold(")
        .unwrap_or_default();
    body.split_once("\n}\n").map_or(body, |(fold, _)| fold)
}

#[test]
fn show_more_sits_under_the_panel_not_as_a_row_in_it() {
    let body = suggested_fold();
    let (fold, after) = body
        .split_once("// Show more sits under the panel")
        .unwrap();
    assert!(
        !fold.contains("Show more\""),
        "no Show more row inside the fold: {fold}"
    );
    assert!(
        after.contains(r#"class: "chip card-list-more","#),
        "{after}"
    );
    assert!(body.contains("offers_more("), "{body}");
    assert!(after.contains("if more {"), "{after}");
}

#[test]
fn show_more_sits_a_chip_row_s_gap_under_its_panel_and_a_panel_gap_over_the_next() {
    assert!(
        WEB.contains(".card-group:has(+ .card-list-more) {\n    margin-bottom: var(--space-2);\n}"),
        "8px between the panel and its chip, as a chip row keeps over its panel"
    );
    assert!(
        WEB.contains(
            ".card-list-more {\n    display: flex;\n    width: fit-content;\n    margin: 0 0 var(--panel-gap);\n}"
        ),
        "the panel gap to what follows, as the panel itself kept"
    );
}

#[test]
fn the_suggested_ghost_draws_show_more_under_its_panel() {
    let (_, ghost) = SUGGESTED.split_once("if list.is_loading() {").unwrap();
    let ghost = ghost.split_once("} else if list.load_failed()").unwrap().0;
    assert!(
        ghost.contains(r#"div { class: "card-list-more", GhostChip { chars: 9, button: true } }"#),
        "{ghost}"
    );
}

#[test]
fn show_more_hides_while_the_panel_is_folded() {
    let body = suggested_fold();
    assert!(
        body.contains("let more = open() && state.offers_more();"),
        "a folded panel offers no Show more: {body}"
    );
    assert!(
        body.contains("CardListFold {\n            title,\n            open,"),
        "{body}"
    );
}
