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
    ] {
        assert!(SUGGESTED.contains(piece), "{piece}");
    }
    let fold = SUGGESTED.find("CardListFold {").expect("the fold");
    let more = SUGGESTED
        .find("ListMore {")
        .expect("Show more, the web's shared chip");
    let danger = SUGGESTED
        .find("label: model::CLEAR_SKIPS,")
        .expect("Clear skips");
    assert!(fold < more && more < danger, "Show more under the panel");
    assert!(
        SUGGESTED.contains("let offers = state.offers_more();")
            && SUGGESTED.contains("open: fold_open(),")
            && SUGGESTED.contains("leaves: true,"),
        "once per deal, and not while the panel is shut"
    );
    assert!(
        loop_root_is_keyed(SUGGESTED) && SUGGESTED.contains("MoreItem::Ghosts => rsx! {"),
        "its press eases in the ghost rows, as the web's does, in the rows' one keyed run"
    );
}

/// `.screen-content` scrolls, and iOS draws the screen's header and floating dock over a fixed
/// layer inside a scroll container, so Suggested's swipe showed the screen's funnel over its eye
/// button. Both swipes render at the screen's root, after the dock, as the web's do.
#[test]
fn both_swipes_render_at_the_screen_s_root_after_the_dock() {
    let panel = SUGGESTED.split("#[cfg(test)]").next().unwrap_or_default();
    assert!(
        !panel.contains("SwipeSelect {"),
        "Suggested's panel renders no swipe inside the page"
    );
    let dock = COMMANDERS
        .find("div { class: \"action-dock action-dock-right\",")
        .expect("the dock");
    let (page, root) = COMMANDERS.split_at(dock);
    assert!(!page.contains("SwipeSelect {"), "no swipe inside the page");
    assert_eq!(
        root.matches("SwipeSelect {").count(),
        2,
        "the Maybeboard's swipe and Suggested's, after the dock"
    );
    for piece in [
        "open: suggested_swipe_open,",
        "mode: suggested_swipe_mode,",
        "reasons_deck_count: Some(suggested_deck_count.into()),",
    ] {
        assert!(root.contains(piece), "{piece}");
    }
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

const MAYBEBOARD: &str = include_str!("../src/lib/inbound/screens/deck/commander_maybeboard.rs");

#[test]
fn the_maybeboard_shows_ten_at_a_time_under_its_own_show_more() {
    assert!(MAYBEBOARD.contains("let mut more_page = use_signal(MorePage::new);"));
    assert!(
        MAYBEBOARD.contains(
            "let list_view = use_memo(move || (driver.model.read().sort(), filter_reset_counter()));"
        ),
        "a new sort or an applied filter shows the first ten again"
    );
    assert!(
        MAYBEBOARD.contains(".layout(&filtered, |c| c.scryfall_data.oracle_id)")
            && loop_root_is_keyed(MAYBEBOARD),
        "the rows and the ghosts in one keyed run, so a later press moves no row"
    );
    let ghosts = MAYBEBOARD.find("MoreGhostRows {").expect("ghost rows");
    let more = MAYBEBOARD.find("ListMore {").expect("Show more");
    let danger = MAYBEBOARD
        .find("label: model::CLEAR_MAYBEBOARD,")
        .expect("Clear maybeboard");
    assert!(ghosts < more && more < danger, "Show more under the panel");
}

/// Whether `src` draws its Show more list in a loop whose body's first node is a keyed `Fragment`:
/// Dioxus reads a loop item's key off that node only, and a key inside a `match` there leaves the
/// list unkeyed.
fn loop_root_is_keyed(src: &str) -> bool {
    src.split_once("for (key, item) in items {")
        .is_some_and(|(_, body)| body.trim_start().starts_with("Fragment { key: \"{key}\","))
}
