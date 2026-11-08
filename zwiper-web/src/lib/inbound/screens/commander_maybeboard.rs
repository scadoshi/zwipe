//! Commander maybeboard: the commanders a user saved for later, with Quick
//! add by name, the Show row's color pips, and Create deck or Remove on each.
//!
//! On a wide window the Quick add, Show and list column sits on the left,
//! `/` focuses Quick add, and ↑ and ↓ move through its result chips with
//! Enter adding the highlighted one. Below the breakpoint it is the app's
//! screen. Swipe sits in the list's header, Filter floats in the dock and
//! Clear maybeboard is the Danger zone panel under the list.

use std::collections::HashSet;

use components::{
    ActionDock, AlertDialog, Button, ButtonVariant, CardListFold, Chip, ClearWrap,
    DangerZoneAction, DialogAction, DockFilter, HoverStackHost, TOAST_LONG, TOAST_QUICK,
    use_hover_stack,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::format::Format;
use zwipe_ui::commander::maybeboard::{self, Step};
use zwipe_ui::filter;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::generation::{Generation, Ticket};
use zwipe_ui::timing::{MAYBEBOARD_QUICK_ADD_LIMIT, QUICK_ADD_DEBOUNCE, QUICK_ADD_MIN_CHARS};

use crate::domain::commander_maybeboard;
use crate::inbound::caches::CreateDeckSeed;
use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::cards::card_row::{CardOverlaysHost, CardRow, ShowRowArt, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::cards::swipe_select::{SwipeMode, SwipeSelect};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::ghost::{
    GhostActionRow, GhostButton, GhostList, GhostShowRow, GhostText,
};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::chrome::page_keys::{focus_by_id, use_page_keys};
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;
use crate::outbound::timer;

/// The Quick add input's id, for `/`.
const QUICK_ADD_ID: &str = "commander-maybeboard-search";

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

/// The commander maybeboard screen.
#[component]
pub fn CommanderMaybeboard() -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let seed: CreateDeckSeed = use_context();
    let toast = use_toast();
    let authed = use_authed();
    let hint_open = use_one_time_hint(OneTimeHint::CommanderMaybeboard);
    let mut show_clear_dialog = use_signal(|| false);

    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let hover_stack = use_hover_stack();
    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);

    let mut query = use_signal(String::new);
    let mut selected_colors: Signal<HashSet<Color>> = use_signal(HashSet::new);
    let mut filter = use_signal(CardQueryBuilder::new);
    let mut filter_open = use_signal(|| false);
    let mut highlight: Signal<Option<usize>> = use_signal(|| None);

    let mut list = SavedList {
        entries: use_signal(Vec::new),
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
    use_effect(move || {
        let _tick = reload();
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
            let mut builder = CardQueryBuilder::with_name_contains(q.trim());
            builder
                .set_is_commander_in_format(Format::Commander)
                .set_is_token(false)
                .set_sort(CardSortKey::Name)
                .set_limit(MAYBEBOARD_QUICK_ADD_LIMIT);
            let Ok(filter) = builder.build() else {
                is_searching.set(false);
                return;
            };
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
        spawn(async move {
            if authed
                .run("add_commander_maybeboard", |c, s| async move {
                    c.add_commander_maybeboard_card(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                let name = card.scryfall_data.name.clone();
                let already_saved = entries
                    .peek()
                    .iter()
                    .any(|c| c.scryfall_data.oracle_id == Some(oracle_id));
                if !already_saved {
                    entries.write().insert(0, card);
                }
                list.refetch();
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
        spawn(async move {
            if authed
                .run("remove_commander_maybeboard", |c, s| async move {
                    c.remove_commander_maybeboard_card(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                list.remove(oracle_id);
                toast.info(
                    "Removed".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });

    let clear_maybeboard = use_callback(move |()| {
        spawn(async move {
            if authed
                .run("clear_commander_maybeboard", |c, s| async move {
                    c.clear_commander_maybeboard(&s).await
                })
                .await
                .is_some()
            {
                list.clear();
                toast.info(
                    "Maybeboard cleared".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });

    use_page_keys(Callback::new(move |key: String| {
        if key == "/" {
            focus_by_id(QUICK_ADD_ID);
        }
    }));

    // A printing swap is display only: the maybeboard is keyed by oracle, and
    // a refetch returns the preferred printing. A Create deck after the swap
    // carries the picked printing.
    let mut expanded_for_save = expanded_card;
    let on_printing_save = move |new_card: Card| {
        let target = new_card.scryfall_data.oracle_id;
        if target.is_some() {
            expanded_for_save.set(Some(new_card.scryfall_data.id));
            if let Some(slot) = entries
                .write()
                .iter_mut()
                .find(|c| c.scryfall_data.oracle_id == target)
            {
                *slot = new_card;
            }
        }
    };

    // The Show pips narrow in memory, every picked color in the identity,
    // then the applied filter, then its sort; saved order stands without
    // one. The quick add input never narrows the list.
    let filtered = use_memo(move || -> Vec<Card> {
        let colors = selected_colors();
        let shown: Vec<Card> = entries()
            .into_iter()
            .filter(|c| commander_maybeboard::passes_pips(&c.scryfall_data.color_identity, &colors))
            .collect();
        let applied = filter();
        let mut shown = Cards::from(shown);
        if !applied.is_empty()
            && let Ok(criteria) = applied.build_criteria()
        {
            shown = shown.matching(&criteria);
        }
        shown.sorted(applied.sort(), applied.direction()).into()
    });
    let filtered = filtered();
    let colors = selected_colors();
    let has_entries = !entries().is_empty();
    let searching_catalog = query().trim().len() >= QUICK_ADD_MIN_CHARS;
    let results = shown_results();
    let lit = highlight();

    rsx! {
        ScreenFrame {
            title: "Commander maybeboard",
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div {
                class: "screen-content",
                onmousemove: move |e| hover_stack.track(e.client_coordinates().x),
                div { class: "flex-col web-maybeboard",
                    if is_loading() {
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label web-ghost-chip", GhostText { chars: 9 } }
                            div { class: "input-clear-wrap web-ghost-field",
                                div {
                                    class: "input input-compact input-placeholder",
                                    style: "line-height: normal;",
                                    GhostText { chars: 25 }
                                }
                            }
                        }
                        GhostShowRow {}
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
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Quick add" }
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
                        if searching_catalog {
                            div { class: "search-float-anchor",
                                div { class: "search-float-results search-float-below",
                                    if is_searching() {
                                        div { class: "chip-unselected search-float-chip", "Searching…" }
                                    } else if results.is_empty() {
                                        div { class: "chip-unselected search-float-chip", "No results" }
                                    } else {
                                        for (i, card) in results.into_iter().enumerate() {
                                            div {
                                                key: "{card.scryfall_data.id}",
                                                class: if lit == Some(i) { "chip-unselected search-float-chip web-float-highlight" } else { "chip-unselected search-float-chip" },
                                                style: "animation-delay: {i.saturating_mul(40)}ms, {i.saturating_mul(40).saturating_add(250)}ms;",
                                                onclick: move |_| add_from_search.call(card.clone()),
                                                "{card.scryfall_data.name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Show" }
                            Chip {
                                selected: colors.is_empty(),
                                onclick: move |_| selected_colors.write().clear(),
                                "All"
                            }
                            for color in Color::all() {
                                Chip {
                                    key: "{color.to_short_name()}",
                                    selected: colors.contains(&color),
                                    onclick: move |_| {
                                        let mut picked = selected_colors.write();
                                        if !picked.remove(&color) {
                                            picked.insert(color);
                                        }
                                    },
                                    i {
                                        class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                                        "aria-hidden": "true",
                                    }
                                    span { class: "sr-only", "{color.to_long_name()}" }
                                }
                            }
                            Chip {
                                selected: show_row_art(),
                                onclick: move |_| show_row_art.toggle(),
                                "Art"
                            }
                        }
                        // Swipe sits in the list's header, so it says
                        // what gets swiped. The panel is there whether or
                        // not anything is saved; with nothing to list, one
                        // row says why.
                        CardListFold {
                            title: "Commanders",
                            actions: rsx! {
                                Button {
                                    variant: ButtonVariant::Action,
                                    onclick: move |_| swipe_open.set(true),
                                    "Swipe"
                                }
                            },
                            if load_failed() {
                                div { class: "card-row card-row-note", "Couldn't load your commander maybeboard" }
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
                            onclick: move |()| show_clear_dialog.set(true),
                        }
                    }
                }
            }
            // Filter, the screen's action, floats at the bottom right as the
            // dock's funnel, its dot on while a filter is set.
            ActionDock {
                DockFilter {
                    label: "Filter",
                    dot: filter::shows_dot(&filter.read()),
                    onclick: move |()| filter_open.set(true),
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::CommanderMaybeboard, open: hint_open }
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
        HoverStackHost { stack: hover_stack }
        CardOverlaysHost { overlays, on_printing_save }
        OtagExamplesHost { examples }
        CardFilterPanel {
            open: filter_open,
            filter,
            config: SheetConfig::default(),
            quiet_reset: true,
            on_apply: move |applied: CardQueryBuilder| filter.set(applied),
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

/// One saved commander: the shared row with `Create deck` and, when it has
/// an oracle id, `Remove` after its own `Image` and `Printing`.
#[component]
fn MaybeboardRow(
    card: Card,
    expanded_card: Signal<Option<Uuid>>,
    on_create: Callback<Card>,
    on_remove: Callback<Uuid>,
) -> Element {
    let oracle_id = card.scryfall_data.oracle_id;
    let card_for_create = card.clone();
    rsx! {
        CardRow {
            card,
            qty: 1,
            expanded_card,
            extra_actions: rsx! {
                button {
                    class: "card-action-btn",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_create.call(card_for_create.clone());
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
            },
        }
    }
}
