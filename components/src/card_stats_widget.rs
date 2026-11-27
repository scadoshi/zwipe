//! The card page's stats widgets, each a regular table in the shared data table style, and the
//! same widgets for any card from the [`CardStatsSource`] the client provides: a swipe screen's
//! stats column and the end of a card's details.

use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::models::search_card::commander_eligibility;
use zwipe_core::domain::deck::models::format::Format;
use zwipe_core::http::contracts::card::HttpCardStats;
use zwipe_ui::screens::card_lookup::CardStats;
use zwipe_ui::screens::card_lookup::stats::{self, AS_COMMANDER, Cell, DECKS, SWIPES, Widget};

use crate::{DeckFold, LoadFailed};

/// The host's card stats: each card's fetch by its Oracle id, and the callback that starts one,
/// which writes [`CardStats::Loading`] and then its outcome into `results`. Components never call
/// the API; without a source the stats draw nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardStatsSource {
    /// Each card's stats asked for this session, by the card's Oracle id.
    pub results: Signal<HashMap<Uuid, CardStats>>,
    /// Fetches the stats of the card with this Oracle id.
    pub request: Callback<Uuid>,
}

impl CardStatsSource {
    /// Asks for the stats of the card with Oracle id `oracle_id` unless they are loaded or on
    /// their way.
    fn ensure(self, oracle_id: Uuid) {
        let asked = matches!(
            self.results.peek().get(&oracle_id),
            Some(CardStats::Loading | CardStats::Loaded(_))
        );
        if !asked {
            self.request.call(oracle_id);
        }
    }
}

/// Which folds of the stats are shut in a card's details: every one at first, and as the user
/// leaves them for the rest of the session, across swipes and screens.
static DETAILS_FOLDED: GlobalSignal<HashSet<String>> =
    Signal::global(|| [DECKS, SWIPES, AS_COMMANDER].map(String::from).into());

/// Which folds of a swipe screen's stats column are shut: none at first, as on the card page,
/// and as the user leaves them for the rest of the session.
static COLUMN_FOLDED: GlobalSignal<HashSet<String>> = Signal::global(HashSet::new);

/// Where a card's stats show, which decides how their folds start.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StatsPlace {
    /// At the end of the card's details, every fold shut at first.
    #[default]
    Details,
    /// In a swipe screen's column beside the card, every fold open at first.
    Column,
}

/// `card`'s stats widgets from the [`CardStatsSource`] in context, fetched while `active` (a
/// drawer fetches as it opens), with `next`'s fetched too so the next swipe lands on loaded
/// figures. The folds start as `place` says and hold for the session. A failed fetch offers
/// Retry. A card that can't lead a Commander deck shows no As a commander widget.
#[component]
pub fn CardStatsFolds(
    card: Card,
    #[props(default)] place: StatsPlace,
    #[props(default = true)] active: bool,
    #[props(default)] next: Option<Card>,
) -> Element {
    let source = try_use_context::<CardStatsSource>();
    let oracle_id = card.scryfall_data.oracle_id;
    let next_id = next.and_then(|n| n.scryfall_data.oracle_id);
    use_effect(use_reactive!(|(oracle_id, next_id, active)| {
        let Some(source) = source else {
            return;
        };
        if !active {
            return;
        }
        for id in oracle_id.into_iter().chain(next_id) {
            source.ensure(id);
        }
    }));
    let Some(source) = source else {
        return rsx! {};
    };
    let folded = match place {
        StatsPlace::Details => DETAILS_FOLDED.signal(),
        StatsPlace::Column => COLUMN_FOLDED.signal(),
    };
    // A card with no Oracle id has no stats to ask for, so its figures stay blank, as on the
    // card page.
    let stats = match oracle_id {
        Some(id) => source
            .results
            .read()
            .get(&id)
            .copied()
            .unwrap_or(CardStats::Loading),
        None => CardStats::Loaded(HttpCardStats::default()),
    };
    let leads = commander_eligibility::is_valid_commander(&card, &Format::Commander);
    rsx! {
        CardStatsList {
            stats,
            leads,
            folded,
            on_retry: move |()| {
                if let Some(id) = oracle_id {
                    source.request.call(id);
                }
            },
        }
    }
}

/// The stats widgets over `stats`, `leads` when the card can lead a Commander deck, with the
/// folds `folded` holds shut, or [`stats::LOAD_FAILED`] with Retry, which calls `on_retry`.
#[component]
pub fn CardStatsList(
    stats: CardStats,
    leads: bool,
    folded: Signal<HashSet<String>>,
    on_retry: EventHandler<()>,
) -> Element {
    let loaded = match stats {
        CardStats::Failed => {
            return rsx! {
                LoadFailed { message: stats::LOAD_FAILED, on_retry }
            };
        }
        CardStats::Loaded(counts) => Some(counts),
        CardStats::Loading => None,
    };
    rsx! {
        for widget in stats::widgets(loaded.as_ref(), leads) {
            CardStatsWidget { key: "{widget.title}", widget, folded }
        }
    }
}

/// `widget` as a fold holding a regular table in the shared data table style: a header row of
/// the column names, then a row per stat, its label and its figures, right-aligned. `folded`
/// holds the folds the user shut. While the stats load, the table is all ghost: every name,
/// label and figure a skeleton bar at its place.
#[component]
pub fn CardStatsWidget(widget: Widget, folded: Signal<HashSet<String>>) -> Element {
    let Widget { title, heads, rows } = widget;
    let loading = rows
        .iter()
        .any(|row| row.cells.iter().any(|cell| matches!(cell, Cell::Loading)));
    let (first, columns) = heads.split_first().map_or(("", &[][..]), |(f, c)| (*f, c));
    let text = move |text: &'static str| {
        if loading {
            ghost(text.chars().count())
        } else {
            rsx! { "{text}" }
        }
    };
    rsx! {
        DeckFold { fold_key: title, title, folded,
            table { class: "data-table",
                thead {
                    tr {
                        th { scope: "col", {text(first)} }
                        for head in columns.iter() {
                            th { key: "{head}", scope: "col", class: "data-table-num", {text(head)} }
                        }
                    }
                }
                tbody {
                    for row in rows {
                        tr { key: "{row.label}",
                            th { scope: "row", {text(row.label)} }
                            for cell in row.cells {
                                td { class: "data-table-num",
                                    match cell {
                                        Cell::Loading => ghost(FIGURE_GHOST_CHARS),
                                        Cell::Figure(figure) => rsx! { "{figure}" },
                                        Cell::Blank => rsx! {},
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// How many characters a figure's ghost spans.
const FIGURE_GHOST_CHARS: usize = 3;

/// A skeleton bar `chars` characters wide, in a data table's cell.
fn ghost(chars: usize) -> Element {
    rsx! {
        span { class: "skeleton-bar data-table-ghost", style: "width: {chars}ch;" }
    }
}
