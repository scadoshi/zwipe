use std::time::{Duration, Instant};

use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::otag_examples;
use crate::test_support::{render, screen_text, user};

fn tag(slug: &str, description: Option<&str>, parents: &[&str]) -> OracleTag {
    OracleTag {
        slug: slug.to_owned(),
        label: slug.replace('-', " "),
        description: description.map(str::to_owned),
        parent_slugs: parents.iter().map(|&p| p.to_owned()).collect(),
        curated: false,
    }
}

fn catalog() -> Vec<OracleTag> {
    vec![
        tag("ramp", Some("Puts extra mana into play"), &[]),
        tag("removal", Some("Gets rid of a permanent"), &[]),
        tag(
            "artifact-ramp",
            Some("Ramp from artifacts"),
            &["ramp", "mana-rock"],
        ),
        tag("anthem", None, &[]),
        tag("40k-model", None, &[]),
    ]
}

/// Home with the dictionary open over it, its hint already seen, and nothing queued.
fn on_dictionary() -> Harness {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let mut me = user(true);
    me.hints_shown
        .insert(HINT_ORACLE_TAG_DICTIONARY.to_owned(), true);
    harness.app.shared.user = Some(me);
    harness.open(open);
    harness.clear();
    harness
}

/// The dictionary over home with the catalog loaded.
fn loaded() -> Harness {
    let mut harness = on_dictionary();
    harness.apply(Update::Catalog(catalog()));
    harness
}

fn view(harness: &Harness) -> &OracleTagsView {
    match harness.screen() {
        Screen::OracleTags(view) => view,
        other => panic!("not the dictionary: {other:?}"),
    }
}

fn shown_slugs(harness: &Harness) -> Vec<String> {
    let tags = harness.app.shared.caches.oracle_tags.tags().unwrap();
    view(harness)
        .rows(tags)
        .into_iter()
        .map(|t| t.slug)
        .collect()
}

fn toasts(harness: &Harness) -> Vec<String> {
    harness
        .app
        .shared
        .toasts
        .items()
        .iter()
        .map(|t| t.message.clone())
        .collect()
}

#[test]
fn opening_fetches_the_catalog_once_and_shows_the_hint_the_first_time() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.app.shared.user = Some(user(true));
    harness.open(open);
    assert!(matches!(harness.screen(), Screen::OracleTags(_)));
    assert_eq!(
        harness.pending(),
        [
            Request::Catalog.into(),
            screens::Request::MarkHintShown(HINT_ORACLE_TAG_DICTIONARY.to_owned()),
        ]
    );
    let hint = harness.app.shared.hint.as_ref().unwrap();
    assert_eq!(hint.content.title, "Oracle tags");
    assert!(
        hint.content
            .text()
            .contains("• Use adds the tag where you came from")
    );

    harness.app.shared.hint = None;
    harness.press(KeyCode::Esc);
    harness.clear();
    harness.open(open);
    assert!(
        harness.pending().is_empty(),
        "one fetch in flight at a time"
    );
}

#[test]
fn a_day_old_catalog_refetches_but_stays_on_screen() {
    let mut harness = loaded();
    let day_ago = Instant::now()
        .checked_sub(Duration::from_hours(25))
        .unwrap();
    if let Catalog::Loaded { fetched_at, .. } = &mut harness.app.shared.caches.oracle_tags {
        *fetched_at = day_ago;
    }
    harness.open(ensure_catalog);
    assert_eq!(harness.pending(), [Request::Catalog.into()]);
    assert!(harness.app.shared.caches.oracle_tags.tags().is_some());
    harness.apply(Update::CatalogFailed);
    assert!(
        harness.app.shared.caches.oracle_tags.tags().is_some(),
        "a failed refresh keeps the old tags"
    );
    assert!(toasts(&harness).is_empty(), "no toast for a failed refresh");
    assert!(screen_text(&harness.app).contains("artifact-ramp"));
}

#[test]
fn the_catalog_lands_in_the_cache_with_the_dictionary_closed() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(ensure_catalog);
    harness.apply(Update::Catalog(catalog()));
    assert_eq!(
        harness.app.shared.caches.oracle_tags.tags().map(<[_]>::len),
        Some(5)
    );
}

#[test]
fn a_failed_catalog_says_so_and_toasts_once() {
    let mut harness = on_dictionary();
    harness.apply(Update::CatalogFailed);
    harness.apply(Update::CatalogFailed);
    assert_eq!(
        toasts(&harness),
        ["Couldn't load the oracle-tag dictionary"]
    );
    assert!(screen_text(&harness.app).contains(FAILED));
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
}

#[test]
fn browsing_shows_a_letter_by_slug_and_the_rail_moves() {
    let mut harness = loaded();
    assert_eq!(shown_slugs(&harness), ["anthem", "artifact-ramp"]);
    harness.press(KeyCode::Left);
    assert_eq!(view(&harness).letter, 'a', "the rail stops at A");
    harness.press(KeyCode::Right);
    assert_eq!(view(&harness).letter, 'b');
    assert!(screen_text(&harness.app).contains("No tags start with B."));
    for _ in 0..30 {
        harness.press(KeyCode::Char('l'));
    }
    assert_eq!(view(&harness).letter, OTHER, "# ends the rail");
    assert_eq!(shown_slugs(&harness), ["40k-model"]);
}

#[test]
fn the_rail_has_no_hash_without_a_slug_for_it() {
    let tags = vec![tag("ramp", None, &[])];
    assert_eq!(letters(&tags).len(), 26);
    assert_eq!(letters(&catalog()).last(), Some(&OTHER));
    assert_eq!(bucket_of("Ramp"), 'r');
    assert_eq!(bucket_of("5c-good"), OTHER);
}

#[test]
fn searching_ranks_matches_and_esc_clears_it() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('/'));
    harness.type_text("RAMP ");
    assert_eq!(shown_slugs(&harness), ["ramp", "artifact-ramp"]);
    assert!(view(&harness).typing);
    harness.press(KeyCode::Down);
    assert_eq!(view(&harness).selected, 1);
    harness.press(KeyCode::Esc);
    assert!(!view(&harness).typing);
    assert_eq!(view(&harness).query(), "");
    assert!(
        matches!(harness.screen(), Screen::OracleTags(_)),
        "Esc in the box leaves the screen open"
    );
}

#[test]
fn a_search_with_no_match_quotes_the_query() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('/'));
    harness.type_text("Zzz");
    harness.press(KeyCode::Enter);
    assert!(!view(&harness).typing, "Enter keeps the search");
    assert!(screen_text(&harness.app).contains("No tags match \u{201c}zzz\u{201d}."));
}

#[test]
fn a_letter_clears_the_search() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('/'));
    harness.type_text("rem");
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Right);
    assert_eq!(view(&harness).query(), "");
    assert_eq!(view(&harness).letter, 'b');
}

#[test]
fn use_hands_the_slug_to_the_host_and_goes_back() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Char('u'));
    assert!(matches!(harness.screen(), Screen::Home(_)));
    assert_eq!(
        harness.app.shared.caches.handoff.oracle_tag.as_deref(),
        Some("artifact-ramp")
    );
}

#[test]
fn opening_again_drops_a_tag_nobody_took() {
    let mut harness = loaded();
    harness.press(KeyCode::Enter);
    assert!(harness.app.shared.caches.handoff.oracle_tag.is_some());
    harness.open(open);
    assert!(harness.app.shared.caches.handoff.oracle_tag.is_none());
}

#[test]
fn x_opens_the_examples_for_the_selected_tag() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('x'));
    let Screen::OtagExamples(examples) = harness.screen() else {
        panic!("no examples");
    };
    assert_eq!(examples.slug, "anthem");
    assert!(matches!(
        harness.pending().first(),
        Some(screens::Request::OtagExamples(
            otag_examples::Request::Page { offset: 0, .. }
        ))
    ));
}

#[test]
fn question_mark_shows_the_hint_again() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('?'));
    assert_eq!(
        harness
            .app
            .shared
            .hint
            .as_ref()
            .map(|h| h.content.title.as_str()),
        Some("Oracle tags")
    );
}

#[test]
fn the_list_draws_descriptions_parents_and_the_selected_row_keys() {
    let harness = loaded();
    let screen = screen_text(&harness.app);
    for expected in [
        "Oracle tags",
        "Search tags or descriptions",
        "A",
        "anthem",
        "No description yet",
        "artifact-ramp",
        "Ramp from artifacts",
        "ramp  mana-rock",
        "x Examples",
        "u Use",
        "Esc Back",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn the_skeleton_ghosts_everything_while_loading() {
    let harness = on_dictionary();
    let screen = screen_text(&harness.app);
    assert!(screen.contains('░'));
    assert!(!screen.contains("Search tags"));
    assert!(!screen.contains("No description yet"));
}

#[test]
fn the_dictionary_fits_a_tiny_terminal() {
    let mut harness = loaded();
    render(&harness.app, 8, 3);
    render(&harness.app, 1, 1);
    harness.press(KeyCode::Char('/'));
    render(&harness.app, 8, 3);
    render(&harness.app, 30, 8);
    let loading = on_dictionary();
    render(&loading.app, 8, 3);
    render(&loading.app, 1, 1);
}
