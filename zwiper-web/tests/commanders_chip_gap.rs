//! The Commanders Suggested list's Show row sits a chip row's gap (`--space-2`) over the list, as
//! the Maybeboard's rows beside it do, rather than on the list's top edge.

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
