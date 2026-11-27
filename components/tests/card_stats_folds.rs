//! A card's stats widgets draw what the host's [`CardStatsSource`] fetched: asked for while
//! active, with the next card's asked for too, folded at first at the end of the details and
//! open in the swipe screens' column.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed read or render fails the test"
)]

use std::collections::HashMap;

use components::{CardStatsFolds, CardStatsSource, StatsPlace};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::{HttpCardStats, HttpCardSwipes};
use zwipe_ui::screens::card_lookup::CardStats;

/// A real card as the API serves it, shared with zwipe-ui's tests: a common creature, which
/// can't lead a Commander deck.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

fn card() -> Card {
    serde_json::from_str(CARD_JSON).unwrap()
}

/// The fixture as a legendary creature, which can lead a Commander deck.
fn commander() -> Card {
    let mut card = card();
    card.scryfall_data.type_line = Some("Legendary Creature — Elf Wizard".to_owned());
    card
}

/// Another card's Oracle id.
const OTHER_ORACLE: Uuid = Uuid::from_u128(0x0714);

/// Another card, with its own Oracle id.
fn other() -> Card {
    let mut card = card();
    card.scryfall_data.oracle_id = Some(OTHER_ORACLE);
    card
}

/// Figures for the fixture's stats.
fn counts() -> HttpCardStats {
    HttpCardStats {
        in_decks: Some(214),
        swipes: Some(HttpCardSwipes {
            shown: 1_480,
            added: 402,
            ..HttpCardSwipes::default()
        }),
        ..HttpCardStats::default()
    }
}

/// Renders `app` once its effects have run.
fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

/// A source holding `fetched` for the fixture's Oracle id, recording each request in `asked`.
fn provide(fetched: Option<CardStats>, asked: Signal<Vec<Uuid>>) {
    let oracle = card().scryfall_data.oracle_id.unwrap();
    let mut asked = asked;
    let results = use_signal(|| {
        fetched
            .map(|f| (oracle, f))
            .into_iter()
            .collect::<HashMap<_, _>>()
    });
    use_context_provider(|| CardStatsSource {
        results,
        request: Callback::new(move |id| asked.write().push(id)),
    });
}

/// How many requests `asked` holds, as a paragraph the test reads.
fn asked_line(asked: Signal<Vec<Uuid>>) -> Element {
    rsx! {
        p { class: "asked", "{asked.read().len()}" }
    }
}

#[test]
fn in_the_details_every_fold_starts_shut_over_the_loaded_figures() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        provide(Some(CardStats::Loaded(counts())), asked);
        rsx! {
            CardStatsFolds { card: commander() }
            {asked_line(asked)}
        }
    }
    let html = render(app);
    assert_eq!(
        html.matches("deck-fold folded").count(),
        3,
        "three shut folds: {html}"
    );
    assert!(html.contains("As a commander"), "{html}");
    assert!(html.contains("1,480"), "the loaded figures: {html}");
    assert!(
        html.contains(r#"<p class="asked">0</p>"#),
        "loaded, so no request: {html}"
    );
}

#[test]
fn in_the_column_the_folds_start_open() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        provide(Some(CardStats::Loaded(counts())), asked);
        rsx! {
            CardStatsFolds { card: commander(), place: StatsPlace::Column }
        }
    }
    let html = render(app);
    assert!(!html.contains("deck-fold folded"), "{html}");
    assert!(html.contains("214"), "{html}");
}

#[test]
fn a_card_that_cannot_lead_shows_no_commander_widget() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        provide(Some(CardStats::Loaded(counts())), asked);
        rsx! {
            CardStatsFolds { card: card(), place: StatsPlace::Column }
        }
    }
    let html = render(app);
    assert!(!html.contains("As a commander"), "{html}");
    assert!(html.contains("Swipes"), "{html}");
}

#[test]
fn an_active_fold_asks_for_its_card_and_the_next_one() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        provide(None, asked);
        rsx! {
            CardStatsFolds { card: card(), next: Some(other()) }
            {asked_line(asked)}
        }
    }
    let html = render(app);
    assert!(
        html.contains(r#"<p class="asked">2</p>"#),
        "this card and the next: {html}"
    );
    assert!(
        html.contains("skeleton-bar"),
        "the ghost while they load: {html}"
    );
}

#[test]
fn an_inactive_fold_asks_for_nothing() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        provide(None, asked);
        rsx! {
            CardStatsFolds { card: card(), active: false }
            {asked_line(asked)}
        }
    }
    let html = render(app);
    assert!(
        html.contains(r#"<p class="asked">0</p>"#),
        "the drawer is shut: {html}"
    );
}

#[test]
fn a_failed_fetch_offers_retry() {
    fn app() -> Element {
        let asked = use_signal(Vec::new);
        provide(Some(CardStats::Failed), asked);
        rsx! {
            CardStatsFolds { card: card() }
            {asked_line(asked)}
        }
    }
    let html = render(app);
    assert!(
        html.contains("Couldn&#39;t load the card&#39;s stats"),
        "{html}"
    );
    assert!(html.contains("Retry"), "{html}");
    assert!(
        html.contains(r#"<p class="asked">1</p>"#),
        "shown anew, a failure asks again: {html}"
    );
}
