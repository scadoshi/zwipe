//! The app's Commanders screen: the Maybeboard, then Suggested under it, each with its console,
//! Swipe and Danger zone; Playstyle keeps only its profile.

const COMMANDERS: &str = include_str!("../src/lib/inbound/screens/deck/commander_maybeboard.rs");
const SUGGESTED: &str = include_str!("../src/lib/inbound/screens/deck/commanders_suggested.rs");
const PLAYSTYLE: &str = include_str!("../src/lib/inbound/screens/deck/suggested_commanders.rs");

#[test]
fn suggested_follows_the_maybeboard_s_danger_zone() {
    let danger = COMMANDERS
        .find("label: model::CLEAR_MAYBEBOARD,")
        .expect("the Maybeboard's Danger zone");
    let suggested = COMMANDERS
        .find("SuggestedSection {")
        .expect("the Suggested list");
    assert!(danger < suggested, "Maybeboard first, then Suggested");
    assert!(
        COMMANDERS.contains("title: model::SCREEN_TITLE"),
        "the header names the Commanders screen"
    );
    assert!(
        COMMANDERS.contains("hint: OneTimeHint::Commanders"),
        "the Commanders hint"
    );
}

#[test]
fn one_filter_narrows_both_lists() {
    assert!(COMMANDERS.contains("DockFilter {"), "the dock's funnel");
    assert!(
        COMMANDERS.contains("filter: filter_builder,")
            && COMMANDERS.contains("filter_applied: filter_reset_counter,"),
        "Suggested deals under the Maybeboard's filter and again on Apply"
    );
}

#[test]
fn suggested_has_its_console_swipe_question_mark_show_more_and_danger_zone() {
    for piece in [
        "{maybeboard_model::SHOW}",
        "{maybeboard_model::ART}",
        "InfoButton { topic: HintTopic::SuggestionList }",
        "{maybeboard_model::SWIPE}",
        "label: model::CLEAR_SKIPS,",
        "mode: swipe_mode,",
    ] {
        assert!(SUGGESTED.contains(piece), "{piece}");
    }
    let fold = SUGGESTED.find("CardListFold {").expect("the fold");
    let more = SUGGESTED
        .find(r#"class: if leaving() { "card-list-more card-list-more-leaving" } else { "card-list-more" },"#)
        .expect("Show more, in the web's exit markup");
    let danger = SUGGESTED
        .find("label: model::CLEAR_SKIPS,")
        .expect("Clear skips");
    assert!(fold < more && more < danger, "Show more under the panel");
    assert!(
        SUGGESTED.contains("let more = fold_open() && (leaving() || state.offers_more());"),
        "once per deal, and not while the panel is shut"
    );
}

#[test]
fn playstyle_keeps_only_its_profile() {
    for gone in ["SwipeSelect {", "SuggestionRow {", "CLEAR_SKIPS"] {
        assert!(!PLAYSTYLE.contains(gone), "{gone} left Playstyle");
    }
    assert!(PLAYSTYLE.contains("BuilderProfileSections {"));
    assert!(PLAYSTYLE.contains("StaplesPanel {"));
}

const MAIN_CSS: &str = include_str!("../assets/main.css");

#[test]
fn suggested_keeps_a_panel_gap_under_the_maybeboard_s_danger_zone() {
    assert!(
        SUGGESTED.contains(r#"div { class: "flex-col commanders-suggested","#),
        "Suggested's list sits in its own column"
    );
    assert!(
        MAIN_CSS.contains(".commanders-suggested {\n    margin-top: var(--panel-gap);\n}"),
        "a panel gap over its Show row, as the web's stacked Suggested keeps"
    );
}
