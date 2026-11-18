//! The Commanders screen's Maybeboard list: the commanders a user saved for
//! later, with Quick add by name, the Show row's Art, the Sort row,
//! and Create deck or Remove on each. The screen's one filter narrows it too.
//!
//! `q` focuses Quick add, `/` opens the screen's filter at its name, and ↑
//! and ↓ move through Quick add's result chips with Enter adding the
//! highlighted one. Swipe sits in the list's header and Clear maybeboard is
//! the Danger zone panel under the list.

use std::collections::HashSet;

use components::{
    AlertDialog, Button, ButtonVariant, CardListFold, Chip, ClearWrap, DangerZoneAction,
    DialogAction, TOAST_LONG, TOAST_QUICK, spawn_write, use_mounted,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::format::Format;
use zwipe_ui::commander::maybeboard::{self, Sort, SortBy, Step};
use zwipe_ui::generation::{Generation, Ticket};
use zwipe_ui::screens::deck_list;
use zwipe_ui::timing::{QUICK_ADD_DEBOUNCE, QUICK_ADD_MIN_CHARS};

use crate::domain::commander_maybeboard;
use crate::inbound::caches::CreateDeckSeed;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_row::{CardRow, ShowRowArt};
use crate::inbound::cards::swipe_select::{SwipeMode, SwipeSelect};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::ghost::{
    GhostActionRow, GhostButton, GhostChipRow, GhostList, GhostShowRow, GhostText,
};
use crate::inbound::router::Route;
use crate::inbound::telemetry;
use crate::outbound::maybeboard_sort_store::MaybeboardSortStore;
use crate::outbound::timer;

/// The saved commanders' header.
const LIST_TITLE: &str = "Maybeboard";

/// The Quick add input's id, for `q`.
pub(super) const QUICK_ADD_ID: &str = "commander-maybeboard-search";

/// The saved list and its load state. Entries live in a signal so Remove
/// and Quick add edit the list in place. Each load takes a ticket, and an
/// edit made while a load is out starts a new one: that reply left before
/// the edit reached the server and could bring a removed row back.
#[derive(Clone, Copy)]
struct SavedList {
    entries: Signal<Vec<Card>>,
    is_loading: Signal<bool>,
    load_failed: Signal<bool>,
    /// Bumped to refetch; the load effect reads it.
    reload: Signal<u32>,
    loads: Signal<Generation>,
    /// Whether the newest load's reply is still out.
    load_out: Signal<bool>,
}

impl SavedList {
    /// Takes the ticket for a load about to go out.
    fn begin_load(&mut self) -> Ticket {
        self.load_out.set(true);
        self.loads.write().next_ticket()
    }

    /// Lands a load's reply (`None` on failure) unless a newer load or an
    /// edit has taken a ticket since.
    fn land(&mut self, ticket: Ticket, reply: Option<Vec<Card>>) {
        if !self.loads.peek().is_current(ticket) {
            return;
        }
        self.load_out.set(false);
        if let Some(cards) = reply {
            self.entries.set(cards);
            self.load_failed.set(false);
        } else {
            let empty = self.entries.peek().is_empty();
            self.load_failed.set(empty);
        }
        self.is_loading.set(false);
    }

    /// Drops any load still out and fetches again. Silent while entries show.
    fn refetch(&mut self) {
        self.loads.write().next_ticket();
        let tick = *self.reload.peek();
        self.reload.set(tick.wrapping_add(1));
    }

    /// Restarts a load that went out before a local edit.
    fn restart_stale_load(&mut self) {
        if *self.load_out.peek() {
            self.refetch();
        }
    }

    /// Drops a removed commander's row.
    fn remove(&mut self, oracle_id: Uuid) {
        self.entries
            .write()
            .retain(|c| c.scryfall_data.oracle_id != Some(oracle_id));
        self.restart_stale_load();
    }

    /// Empties the list after the server cleared it.
    fn clear(&mut self) {
        self.entries.set(Vec::new());
        self.restart_stale_load();
    }
}

/// The Maybeboard list over `entries`, the screen's copy of the saved
/// commanders, narrowed by the screen's `filter`. A row opens into
/// `expanded_card`, which the screen's card keys share.
#[component]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "keyboard_types::Key is a foreign enum of every key; only the keys handled act"
)]
pub(super) fn MaybeboardPanel(
    entries: Signal<Vec<Card>>,
    filter: Signal<CardQueryBuilder>,
    expanded_card: Signal<Option<Uuid>>,
) -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let seed: CreateDeckSeed = use_context();
    let stores = try_use_context::<CardStores>();
    let toast = use_toast();
    // A save or remove lands even after the screen goes; its list update waits on this.
    let mounted = use_mounted();
    let authed = use_authed();
    let usage = telemetry::use_usage();
    let mut show_clear_dialog = use_signal(|| false);

    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));

    let mut query = use_signal(String::new);
    let mut highlight: Signal<Option<usize>> = use_signal(|| None);
    let mut sort: Signal<Sort> = use_signal(|| MaybeboardSortStore::browser().load());

    let mut list = SavedList {
        entries,
        is_loading: use_signal(|| true),
        load_failed: use_signal(|| false),
        reload: use_signal(|| 0u32),
        loads: use_signal(Generation::default),
        load_out: use_signal(|| false),
    };
    let SavedList {
        mut entries,
        is_loading,
        load_failed,
        reload,
        ..
    } = list;
    // A save or remove from Swipe select or Playstyle reloads the list once it has landed, so
    // the list never reads the server ahead of the write.
    use_effect(move || {
        let _tick = reload();
        let _written = stores.map(|stores| (stores.maybeboard_writes)());
        let ticket = list.begin_load();
        spawn(async move {
            let reply = authed
                .try_run("load_maybeboard", |c, s| async move {
                    c.fetch_commander_maybeboard(&s).await
                })
                .await;
            list.land(ticket, reply.ok());
        });
    });

    // The debounced commander search feeding the result chips. Every edit
    // takes a ticket, so a reply for an older query never lands over a newer
    // one.
    let mut search_results: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut is_searching = use_signal(|| false);
    let mut searches = use_signal(Generation::default);
    use_effect(move || {
        let q = query();
        let ticket = searches.write().next_ticket();
        highlight.set(None);
        if q.trim().len() < QUICK_ADD_MIN_CHARS {
            is_searching.set(false);
            search_results.set(Vec::new());
            return;
        }
        is_searching.set(true);
        spawn(async move {
            timer::sleep(QUICK_ADD_DEBOUNCE).await;
            if !searches.peek().is_current(ticket) {
                return;
            }
            let Ok(filter) = zwipe_ui::screens::commander_maybeboard::search_query(&q).build()
            else {
                is_searching.set(false);
                return;
            };
            usage.peek().record_search();
            let found = authed
                .run_quiet("maybeboard_search", |c, s| async move {
                    c.search_commanders(&filter, &s).await
                })
                .await;
            if !searches.peek().is_current(ticket) {
                return;
            }
            if let Some(found) = found {
                search_results.set(found);
            }
            is_searching.set(false);
        });
    });

    let saved_oracles = use_memo(move || -> HashSet<Uuid> {
        entries()
            .iter()
            .filter_map(|c| c.scryfall_data.oracle_id)
            .collect()
    });
    let shown_results = use_memo(move || -> Vec<Card> {
        let saved = saved_oracles();
        search_results()
            .into_iter()
            .filter(|c| commander_maybeboard::is_addable(c.scryfall_data.oracle_id, &saved))
            .collect()
    });

    // A result chip: dropped from the chips at once, put at the top of the
    // list when the server confirms, then a silent refetch for the server's
    // order and preferred printing.
    let add_from_search = use_callback(move |card: Card| {
        let Some(oracle_id) = card.scryfall_data.oracle_id else {
            return;
        };
        search_results
            .write()
            .retain(|c| c.scryfall_data.oracle_id != Some(oracle_id));
        highlight.set(None);
        // A phone's keyboard drops with the focus, so the Added toast below it shows.
        document::eval(&format!(
            "document.getElementById('{QUICK_ADD_ID}')?.blur();"
        ));
        spawn_write(async move {
            if authed
                .run("add_commander_maybeboard", |c, s| async move {
                    c.add_commander_maybeboard_card(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                let name = card.scryfall_data.name.clone();
                if mounted.get() {
                    let already_saved = entries
                        .peek()
                        .iter()
                        .any(|c| c.scryfall_data.oracle_id == Some(oracle_id));
                    if !already_saved {
                        entries.write().insert(0, card);
                    }
                    list.refetch();
                }
                toast.info(
                    format!("Added {name}"),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });

    // The create gate: an unverified account with a deck already is at its
    // limit. Quiet; a failure leaves the gate open.
    let deck_profiles = use_resource(move || async move {
        authed
            .run_quiet("load_deck_profiles", |c, s| async move {
                c.fetch_deck_profiles(&s).await
            })
            .await
    });
    let mut swipe_open = use_signal(|| false);
    let create_with = use_callback(move |card: Card| {
        let at_limit = session.peek().as_ref().is_some_and(|s| {
            s.user.email_verified_at.is_none()
                && deck_profiles
                    .peek()
                    .as_ref()
                    .and_then(Option::as_ref)
                    .is_some_and(|p| !p.is_empty())
        });
        if at_limit {
            toast.warning(
                "Verify your email to create more than one deck".to_string(),
                ToastOptions::default().duration(TOAST_LONG),
            );
        } else {
            let mut slot = seed.0;
            slot.set(Some(card));
            navigator.push(Route::CreateDeck {});
        }
    });

    let on_remove = use_callback(move |oracle_id: Uuid| {
        spawn_write(async move {
            if authed
                .run("remove_commander_maybeboard", |c, s| async move {
                    c.remove_commander_maybeboard_card(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                if mounted.get() {
                    list.remove(oracle_id);
                }
                toast.info(
                    "Removed".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });

    let clear_maybeboard = use_callback(move |()| {
        spawn_write(async move {
            if authed
                .run("clear_commander_maybeboard", |c, s| async move {
                    c.clear_commander_maybeboard(&s).await
                })
                .await
                .is_some()
            {
                if mounted.get() {
                    list.clear();
                }
                toast.info(
                    "Maybeboard cleared".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });

    // The applied filter narrows the list, then its sort; saved order stands
    // without one. The quick add input never narrows the list.
    let filtered = use_memo(move || -> Vec<Card> {
        let applied = filter();
        let mut shown = Cards::from(entries());
        if !applied.is_empty()
            && let Ok(criteria) = applied.build_criteria()
        {
            shown = shown.matching(&criteria);
        }
        let shown = Cards::from(sort().sorted(shown.into()));
        shown.sorted(applied.sort(), applied.direction()).into()
    });
    let filtered = filtered();
    let has_entries = !entries().is_empty();
    let searching_catalog = query().trim().len() >= QUICK_ADD_MIN_CHARS;
    let results = shown_results();
    let lit = highlight();

    rsx! {
            div {
                class: "web-commanders-list",
                // One of the screen's two cursor lists, which h and l cross between.
                "data-cursor-list": "",
                div { class: "flex-col web-maybeboard",
                    if is_loading() {
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label web-ghost-chip", GhostText { chars: 9 } }
                            // An empty field ghosts empty; the space keeps a typed field's height.
                            div { class: "input-clear-wrap web-ghost-field",
                                div {
                                    class: "input input-compact input-placeholder",
                                    style: "line-height: normal;",
                                    "\u{a0}"
                                }
                            }
                        }
                        GhostShowRow {}
                        GhostChipRow { label: 4, chips: vec![12, 4, 10, 5, 9] }
                        div { class: "maybeboard-list card-group row-enter",
                            div { class: "card-group-header card-group-header-with-hint",
                                GhostText { chars: 10 }
                                GhostButton { chars: 5 }
                            }
                            for i in 0..3 {
                                div { key: "{i}", class: "skeleton-card-row web-ghost-card-row",
                                    div { class: "skeleton-bar skeleton-card-thumb" }
                                    div { class: "skeleton-bar skeleton-card-bar-row" }
                                }
                            }
                        }
                        GhostList { chars: 11, class: "danger-zone",
                            GhostActionRow { label: 16, button: 5 }
                        }
                    } else {
                        // The console's first row: Quick add.
                        div { class: "web-list-console",
                        div { class: "quick-add",
                        div { class: "chip-row",
                            label { class: "stat-chip stat-chip-p1 chip-row-label", r#for: QUICK_ADD_ID, "Quick add" }
                            ClearWrap {
                                filled: !query().is_empty(),
                                on_clear: move |()| query.set(String::new()),
                                label: "Clear the search".to_string(),
                                input {
                                    class: "input input-compact",
                                    id: QUICK_ADD_ID,
                                    r#type: "text",
                                    placeholder: "Search a commander to add",
                                    value: "{query}",
                                    autocapitalize: "none",
                                    autocorrect: "off",
                                    spellcheck: "false",
                                    oninput: move |event| query.set(event.value()),
                                    onkeydown: move |event: Event<KeyboardData>| {
                                        let count = shown_results.peek().len();
                                        match event.key() {
                                            Key::ArrowDown => {
                                                event.prevent_default();
                                                highlight.set(maybeboard::step_highlight(highlight(), count, Step::Down));
                                            }
                                            Key::ArrowUp => {
                                                event.prevent_default();
                                                highlight.set(maybeboard::step_highlight(highlight(), count, Step::Up));
                                            }
                                            Key::Enter => {
                                                let picked = highlight()
                                                    .and_then(|at| shown_results.peek().get(at).cloned());
                                                if let Some(card) = picked {
                                                    add_from_search.call(card);
                                                }
                                            }
                                            _ => {}
                                        }
                                    },
                                }
                            }
                        }
                        // The results sit under the bar in the page's flow, as the deck page's Quick
                        // add does, so the rows below move down rather than hide under them.
                        if searching_catalog {
                            div { class: "card-filter-results quick-add-results",
                                if is_searching() {
                                    div { class: "chip-unselected", "Searching…" }
                                } else if results.is_empty() {
                                    div { class: "chip-unselected", "No results" }
                                } else {
                                    for (i, card) in results.into_iter().enumerate() {
                                        button {
                                            key: "{card.scryfall_data.id}",
                                            r#type: "button",
                                            class: if lit == Some(i) { "chip-unselected card-filter-result card-filter-highlight" } else { "chip-unselected card-filter-result" },
                                            style: "animation-delay: {i.saturating_mul(40)}ms;",
                                            onmousedown: move |e| e.prevent_default(),
                                            onclick: move |_| add_from_search.call(card.clone()),
                                            "{card.scryfall_data.name}"
                                        }
                                    }
                                }
                            }
                        }
                        }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Show" }
                            // Art alone: colors come from the screen's filter.
                            Chip {
                                selected: show_row_art(),
                                onclick: move |_| show_row_art.toggle(),
                                "Art"
                            }
                        }
                        // The key chips pick, the last chip turns the order
                        // around, as on Decks.
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", {deck_list::SORT} }
                            for by in SortBy::ALL {
                                Chip {
                                    key: "{by.label()}",
                                    selected: sort().by == by,
                                    onclick: move |_| {
                                        let next = Sort { by, ..sort() };
                                        sort.set(next);
                                        MaybeboardSortStore::browser().save(next);
                                    },
                                    "{by.label()}"
                                }
                            }
                            Chip {
                                selected: false,
                                onclick: move |_| {
                                    let mut next = sort();
                                    next.direction = next.direction.flipped();
                                    sort.set(next);
                                    MaybeboardSortStore::browser().save(next);
                                },
                                "{sort().direction}"
                            }
                        }
                        // Swipe sits in the list's header, so it says
                        // what gets swiped. The panel is there whether or
                        // not anything is saved; with nothing to list, one
                        // row says why.
                        CardListFold {
                            title: LIST_TITLE,
                            actions: rsx! {
                                Button {
                                    variant: ButtonVariant::Action,
                                    onclick: move |_| swipe_open.set(true),
                                    "Swipe"
                                }
                            },
                            if load_failed() {
                                div { class: "card-row card-row-note", "Couldn't load your maybeboard" }
                            } else if !has_entries {
                                div { class: "card-row card-row-note", "No commanders" }
                            } else if filtered.is_empty() {
                                div { class: "card-row card-row-note", "No commanders match" }
                            }
                            for card in filtered {
                                MaybeboardRow {
                                    key: "{card.scryfall_data.id}",
                                    card,
                                    expanded_card,
                                    on_create: create_with,
                                    on_remove,
                                }
                            }
                        }
                        DangerZoneAction {
                            label: "Clear maybeboard",
                            button: "Clear",
                            disabled: !has_entries,
                            onclick: move |()| show_clear_dialog.set(true),
                        }
                    }
                }
            }
        AlertDialog {
            open: show_clear_dialog,
            title: "Clear maybeboard",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Clear".into(),
                    danger: true,
                    on_confirm: clear_maybeboard,
                },
            ],
            "Every commander on your maybeboard will be removed. This can't be undone."
        }
        // Commanders not saved yet: up saves one here, right starts a deck
        // with it.
        SwipeSelect {
            open: swipe_open,
            mode: SwipeMode::Commander(Format::Commander),
            exclude_oracle_ids: Some(saved_oracles.into()),
            starts_deck: true,
            on_select: move |card: Card| {
                swipe_open.set(false);
                list.refetch();
                create_with.call(card);
            },
            on_close: move |()| {
                swipe_open.set(false);
                list.refetch();
            },
        }
    }
}

/// One saved commander: the shared row with [`maybeboard_actions`] after its
/// own `Image` and `Printing`.
#[component]
fn MaybeboardRow(
    card: Card,
    expanded_card: Signal<Option<Uuid>>,
    on_create: Callback<Card>,
    on_remove: Callback<Uuid>,
) -> Element {
    let extra_actions = maybeboard_actions(card.clone(), on_create, on_remove);
    rsx! {
        CardRow {
            card,
            qty: 1,
            expanded_card,
            extra_actions,
        }
    }
}

/// A saved commander's actions: `Create deck`, then `Remove` when it has an
/// oracle id.
fn maybeboard_actions(card: Card, on_create: Callback<Card>, on_remove: Callback<Uuid>) -> Element {
    let oracle_id = card.scryfall_data.oracle_id;
    rsx! {
        button {
            class: "card-action-btn",
            onclick: move |evt| {
                evt.stop_propagation();
                on_create.call(card.clone());
            },
            "Create deck"
        }
        if let Some(oracle_id) = oracle_id {
            button {
                class: "card-action-btn",
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_remove.call(oracle_id);
                },
                "Remove"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pick_takes_focus_off_the_box_so_a_phone_s_keyboard_drops() {
        let source = include_str!("maybeboard.rs");
        let pick = source
            .split("let add_from_search = use_callback(move |card: Card| {")
            .nth(1)
            .and_then(|rest| rest.split("spawn_write(").next())
            .unwrap_or_default();
        assert!(
            pick.contains("document::eval(")
                && pick.contains("\"document.getElementById('{QUICK_ADD_ID}')?.blur();\""),
            "the pick blurs {QUICK_ADD_ID} before its add goes out: {pick}"
        );
    }
}
