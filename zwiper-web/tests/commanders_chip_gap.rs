//! The Commanders Suggested list's Show row sits a chip row's gap (`--space-2`) over the list, as
//! the Maybeboard's rows beside it do, and a panel gap under the Maybeboard's Danger zone once the
//! columns stack. Its loading ghost draws its header's `?` and Swipe, and its Show more sits at
//! the panel's content inset.

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
    let (_, ghost) = SUGGESTED.split_once("if is_loading() {").unwrap();
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

#[test]
fn show_more_sits_at_the_panel_s_content_inset_clear_of_its_corner() {
    assert!(
        FOLD.contains(r#"class: "chip changelog-more card-list-more","#),
        "Show more carries the card list's own class"
    );
    assert!(
        WEB.contains(
            ".card-list-more {\n    display: flex;\n    width: fit-content;\n    margin: var(--space-3) var(--space-4) var(--space-4);\n}"
        ),
        "16px in from the left as the header and rows are, and 16px up, the panel's own radius, so the chip's corner clears the panel's"
    );
}
