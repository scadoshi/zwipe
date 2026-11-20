//! The suggested commanders, which the Commanders screen lists:
//! [`use_suggestions`] deals the list a page at a time from the merged
//! commander ranking under the screen's filter and edits it, [`SuggestedFold`]
//! draws it with its Swipe and its rows, and [`SuggestedOverlays`] holds its
//! Clear skips dialog and its Swipe select. [`use_held`] loads the builder
//! profile Playstyle draws and the deck count the reasons count against.

use components::{
    AlertDialog, Button, ButtonVariant, CardListFold, DialogAction, TOAST_LONG, TOAST_QUICK,
    spawn_write, use_mounted,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::http::contracts::suggestions::{
    HttpBuilderProfile, HttpCommanderSuggestion, RankedBy,
};
use zwipe_ui::commander::deal::{self, Door, LIST_PAGE};
use zwipe_ui::generation::Generation;

use super::rows::SuggestionRow;
use crate::inbound::caches::CreateDeckSeed;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::swipe_select::{SwipeMode, SwipeSelect};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::router::Route;

/// What [`use_held`] asks the suggestions route for: no list, only the profile and deck count,
/// since the list deals from the merged ranking.
const HELD_LIMIT: u32 = 0;

/// The suggested commanders list: its load state, its Show chip, what it shows and what its
/// rows, Swipe and Clear skips do. Cheap to copy; every field is a signal or a callback.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Suggestions {
    /// The rows dealt so far, best fit first: the first page, then each Show more's.
    pub(crate) entries: Signal<Vec<HttpCommanderSuggestion>>,
    /// What the ranking is built from, once a page has landed.
    pub(crate) ranked_by: Signal<Option<RankedBy>>,
    /// Whether the ranking has no more rows to deal.
    pub(crate) exhausted: Signal<bool>,
    /// Whether Show more has dealt its page since the last fresh deal.
    more_used: Signal<bool>,
    /// The user's deck count, which the reasons count against; 0 before it loads.
    pub(crate) deck_count: Signal<u32>,
    /// Whether the first page is still out.
    pub(crate) is_loading: Signal<bool>,
    /// Whether a page failed with nothing to show.
    pub(crate) load_failed: Signal<bool>,
    /// The user's decks, for the create gate; `None` until loaded.
    pub(crate) deck_profiles: Signal<Option<Vec<DeckProfile>>>,
    filter: Signal<CardQueryBuilder>,
    swipe_open: Signal<bool>,
    clear_open: Signal<bool>,
    show_more: Callback<()>,
    create_with: Callback<Card>,
    on_maybe: Callback<Uuid>,
    on_dismiss: Callback<Uuid>,
    clear_dismissals: Callback<()>,
}

impl Suggestions {
    /// Opens the Clear skips dialog.
    pub(crate) fn ask_clear(mut self) {
        self.clear_open.set(true);
    }
}

/// The held ranking's load: what it holds and the tickets that keep an old reply from landing.
#[derive(Clone, Copy)]
pub(crate) struct Held {
    /// The user's deck count; 0 before the first load.
    pub(crate) deck_count: Signal<u32>,
    /// The builder profile the load carries.
    pub(crate) profile: Signal<Option<HttpBuilderProfile>>,
    /// Whether the first load is still out.
    pub(crate) is_loading: Signal<bool>,
    /// Whether a load failed with nothing to show.
    pub(crate) load_failed: Signal<bool>,
    reload: Signal<u32>,
    loads: Signal<Generation>,
    /// Whether the newest load's reply is still out.
    load_out: Signal<bool>,
}

/// Loads the builder profile and deck count from the suggestions route, again on every
/// [`Held::refetch`]. Each load takes a ticket and only the newest one's reply lands.
pub(crate) fn use_held() -> Held {
    let authed = use_authed();
    let load = Held {
        deck_count: use_signal(|| 0u32),
        profile: use_signal(|| None),
        is_loading: use_signal(|| true),
        load_failed: use_signal(|| false),
        reload: use_signal(|| 0u32),
        loads: use_signal(Generation::default),
        load_out: use_signal(|| false),
    };
    let Held {
        mut deck_count,
        mut profile,
        mut is_loading,
        mut load_failed,
        reload,
        mut loads,
        mut load_out,
    } = load;
    use_effect(move || {
        let _tick = reload();
        load_out.set(true);
        let ticket = loads.write().next_ticket();
        spawn(async move {
            let reply = authed
                .try_run("load_suggestions", |c, s| async move {
                    c.commander_suggestions(HELD_LIMIT, &s).await
                })
                .await;
            if !loads.peek().is_current(ticket) {
                return;
            }
            load_out.set(false);
            match reply {
                Ok(found) => {
                    deck_count.set(found.deck_count);
                    profile.set(found.profile);
                    load_failed.set(false);
                }
                Err(_) => load_failed.set(profile.peek().is_none()),
            }
            is_loading.set(false);
        });
    });
    load
}

/// The user's decks, `None` until they load: quiet, so a failure leaves them unknown.
pub(crate) fn use_deck_profiles() -> Signal<Option<Vec<DeckProfile>>> {
    let authed = use_authed();
    let mut deck_profiles: Signal<Option<Vec<DeckProfile>>> = use_signal(|| None);
    use_hook(move || {
        spawn(async move {
            let found = authed
                .run_quiet("load_deck_profiles", |c, s| async move {
                    c.fetch_deck_profiles(&s).await
                })
                .await;
            deck_profiles.set(found);
        });
    });
    deck_profiles
}

/// The user's decks and Create deck from a suggestion: the create gate the deck list's Create
/// shares, an unverified account having one deck at most.
fn use_create_gate() -> (Signal<Option<Vec<DeckProfile>>>, Callback<Card>) {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let seed: CreateDeckSeed = use_context();
    let deck_profiles = use_deck_profiles();
    let at_deck_limit = move || {
        session().is_some_and(|s| {
            s.user.email_verified_at.is_none()
                && deck_profiles.read().as_ref().is_some_and(|p| !p.is_empty())
        })
    };
    let create_with = use_callback(move |card: Card| {
        if at_deck_limit() {
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
    (deck_profiles, create_with)
}

/// A row's Maybeboard and Remove, which drop it through `drop_row`, and Clear skips, which
/// deals the list again through `redeal`.
fn use_writes(
    drop_row: Callback<Uuid>,
    redeal: Callback<()>,
) -> (Callback<Uuid>, Callback<Uuid>, Callback<()>) {
    let toast = use_toast();
    // A save or skip lands even after the screen goes; its row update waits on this.
    let mounted = use_mounted();
    let stores = try_use_context::<CardStores>();
    let authed = use_authed();
    let on_maybe = use_callback(move |oracle_id: Uuid| {
        spawn_write(async move {
            if authed
                .run("add_commander_maybeboard", |c, s| async move {
                    c.add_commander_maybeboard_card(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                if let Some(stores) = stores {
                    stores.maybeboard_written();
                }
                if mounted.get() {
                    drop_row.call(oracle_id);
                }
                toast.success(
                    "Added to your commander maybeboard".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });
    let on_dismiss = use_callback(move |oracle_id: Uuid| {
        spawn_write(async move {
            if authed
                .run("dismiss_commander", |c, s| async move {
                    c.dismiss_commander(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                if mounted.get() {
                    drop_row.call(oracle_id);
                }
                toast.info(
                    "Skipped for a week".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });
    let clear_dismissals = use_callback(move |()| {
        spawn_write(async move {
            if authed
                .run("clear_commander_dismissals", |c, s| async move {
                    c.clear_commander_dismissals(&s).await
                })
                .await
                .is_some()
            {
                toast.info(
                    "Skips cleared".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                if mounted.get() {
                    redeal.call(());
                }
            }
        });
    });
    (on_maybe, on_dismiss, clear_dismissals)
}

/// The Suggested list's pages: what [`use_deal_pages`] holds and what deals more.
#[derive(Clone, Copy)]
struct DealPages {
    entries: Signal<Vec<HttpCommanderSuggestion>>,
    ranked_by: Signal<Option<RankedBy>>,
    exhausted: Signal<bool>,
    more_used: Signal<bool>,
    is_loading: Signal<bool>,
    load_failed: Signal<bool>,
    redeal: Callback<()>,
    show_more: Callback<()>,
}

/// Deals the first page under `filter` on mount, on a filter change and on every redeal, and the
/// next page on Show more, leaving out what the list already holds.
fn use_deal_pages(filter: Signal<CardQueryBuilder>) -> DealPages {
    let authed = use_authed();
    let mut entries: Signal<Vec<HttpCommanderSuggestion>> = use_signal(Vec::new);
    let mut ranked_by: Signal<Option<RankedBy>> = use_signal(|| None);
    let mut exhausted = use_signal(|| false);
    let mut is_loading = use_signal(|| true);
    let mut load_failed = use_signal(|| false);
    let mut more_out = use_signal(|| false);
    let mut more_used = use_signal(|| false);
    let mut tickets = use_signal(Generation::default);
    let mut redeals = use_signal(|| 0u32);

    // The first page, on mount, on a filter change and on every redeal.
    use_effect(move || {
        let applied = filter();
        let _tick = redeals();
        let ticket = tickets.write().next_ticket();
        more_out.set(false);
        more_used.set(false);
        spawn(async move {
            let Some(request) = deal::next_page(Door::Suggested, &applied, &[], LIST_PAGE) else {
                return;
            };
            let reply = authed
                .try_run("deal_suggestions", |c, s| async move {
                    c.deal_commanders(request, &s).await
                })
                .await;
            if !tickets.peek().is_current(ticket) {
                return;
            }
            match reply {
                Ok(page) => {
                    exhausted.set(deal::is_last_page(page.suggestions.len(), LIST_PAGE));
                    ranked_by.set(Some(page.ranked_by));
                    entries.set(page.suggestions);
                    load_failed.set(false);
                }
                Err(_) => load_failed.set(entries.peek().is_empty()),
            }
            is_loading.set(false);
        });
    });
    let redeal = use_callback(move |()| {
        let tick = *redeals.peek();
        redeals.set(tick.wrapping_add(1));
    });

    // The next page, leaving out what the list already holds.
    let show_more = use_callback(move |()| {
        if *more_out.peek() || *exhausted.peek() {
            return;
        }
        let held_ids = deal::oracle_ids(&entries.peek());
        let Some(request) = deal::next_page(Door::Suggested, &filter.peek(), &held_ids, LIST_PAGE)
        else {
            exhausted.set(true);
            return;
        };
        more_out.set(true);
        let ticket = tickets.peek().current();
        spawn(async move {
            let reply = authed
                .try_run("deal_more_suggestions", |c, s| async move {
                    c.deal_commanders(request, &s).await
                })
                .await;
            if !tickets.peek().is_current(ticket) {
                return;
            }
            more_out.set(false);
            if let Ok(page) = reply {
                exhausted.set(deal::is_last_page(page.suggestions.len(), LIST_PAGE));
                entries.write().extend(page.suggestions);
            }
        });
    });

    DealPages {
        entries,
        ranked_by,
        exhausted,
        more_used,
        is_loading,
        load_failed,
        redeal,
        show_more,
    }
}

/// Deals the suggested commanders a page at a time from the merged ranking under the screen's
/// `filter`, and holds what their list does. A filter change, Swipe select closing and Clear
/// skips deal again from the top; Show more deals the next page, leaving out the rows already
/// dealt; a dropped row tops the list back up to a page. Each deal takes a ticket and only the
/// newest one's reply lands.
pub(crate) fn use_suggestions(filter: Signal<CardQueryBuilder>) -> Suggestions {
    let swipe_open = use_signal(|| false);
    let clear_open = use_signal(|| false);
    let held = use_held();
    let DealPages {
        mut entries,
        ranked_by,
        exhausted,
        more_used,
        is_loading,
        load_failed,
        redeal,
        show_more,
    } = use_deal_pages(filter);

    // The overlay's swipes dismissed and saved commanders the list must not
    // show any more, so closing it deals again.
    let mut swipe_was_open = use_signal(|| false);
    use_effect(move || {
        if swipe_open() {
            swipe_was_open.set(true);
        } else if *swipe_was_open.peek() {
            swipe_was_open.set(false);
            redeal.call(());
        }
    });

    // Drops a row, topping the list back up to a page when it runs short.
    let drop_row = use_callback(move |oracle_id: Uuid| {
        entries
            .write()
            .retain(|s| s.card.scryfall_data.oracle_id != Some(oracle_id));
        let short = u32::try_from(entries.peek().len()).is_ok_and(|n| n < LIST_PAGE);
        if short {
            show_more.call(());
        }
    });

    let (deck_profiles, create_with) = use_create_gate();
    let (on_maybe, on_dismiss, clear_dismissals) = use_writes(drop_row, redeal);

    Suggestions {
        entries,
        ranked_by,
        exhausted,
        more_used,
        deck_count: held.deck_count,
        is_loading,
        load_failed,
        deck_profiles,
        filter,
        swipe_open,
        clear_open,
        show_more,
        create_with,
        on_maybe,
        on_dismiss,
        clear_dismissals,
    }
}

/// The list's fold, titled `title`: Swipe and the list's `?` in its header, the empty lines and
/// the first ten rows, each expanding into `expanded_card`. Its controls sit above it, in the
/// screen's console.
#[component]
pub(crate) fn SuggestedFold(
    list: Suggestions,
    title: &'static str,
    expanded_card: Signal<Option<Uuid>>,
) -> Element {
    let Suggestions {
        entries,
        ranked_by,
        exhausted,
        mut more_used,
        deck_count,
        filter,
        mut swipe_open,
        show_more,
        create_with,
        on_maybe,
        on_dismiss,
        ..
    } = list;
    // The fold's state, so Show more under the panel hides while it is shut.
    let open = use_signal(|| true);
    let rows = entries();
    let more = open() && offers_more(rows.len(), exhausted(), more_used());
    let line = ranked_by().and_then(deal::ranked_line);
    rsx! {
        // Swipe sits in the list's header, so it says
        // what gets swiped.
        CardListFold {
            title,
            open,
            hint: rsx! {
                InfoButton { topic: HintTopic::SuggestionList }
            },
            actions: rsx! {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| swipe_open.set(true),
                    "Swipe"
                }
            },
            if let Some(line) = line {
                div { class: "activity-total activity-total-group",
                    span { class: "chip-note", {line} }
                }
            }
            if rows.is_empty() {
                div { class: "activity-total activity-total-group",
                    span { class: "chip-note",
                        if filter.read().is_empty() {
                            "No suggestions right now"
                        } else {
                            "No commanders match"
                        }
                    }
                }
            }
            for suggestion in rows {
                SuggestionRow {
                    key: "{suggestion.card.scryfall_data.id}",
                    suggestion,
                    deck_count: deck_count(),
                    expanded_card,
                    on_start: create_with,
                    on_maybe,
                    on_dismiss,
                }
            }
        }
        // Show more sits under the panel, as a chip row sits over one, and deals one page.
        if more {
            button {
                r#type: "button",
                class: "chip card-list-more",
                onclick: move |_| {
                    more_used.set(true);
                    show_more.call(());
                },
                "Show more"
            }
        }
    }
}

/// Whether the list offers Show more: it holds rows, the ranking has more to deal, and Show more
/// has not dealt its page since the last fresh deal.
fn offers_more(rows: usize, exhausted: bool, more_used: bool) -> bool {
    rows > 0 && !exhausted && !more_used
}

/// The list's Clear skips dialog and its Swipe select, which starts from the screen's filter.
#[component]
pub(crate) fn SuggestedOverlays(list: Suggestions) -> Element {
    let Suggestions {
        filter,
        deck_count,
        mut swipe_open,
        clear_open,
        create_with,
        clear_dismissals,
        ..
    } = list;
    rsx! {
        AlertDialog {
            open: clear_open,
            title: "Clear skips",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Clear".into(),
                    danger: true,
                    on_confirm: clear_dismissals,
                },
            ],
            "Commanders you skipped will be suggested again. This can't be undone."
        }
        SwipeSelect {
            open: swipe_open,
            mode: SwipeMode::Commanders(Door::Suggested),
            start_filter: Some(filter.into()),
            reasons_deck_count: Some(deck_count.into()),
            on_select: move |card: Card| {
                swipe_open.set(false);
                create_with.call(card);
            },
            on_close: move |()| swipe_open.set(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_more_is_offered_once_per_deal_while_the_ranking_holds_more() {
        assert!(
            offers_more(10, false, false),
            "a first page with more to deal"
        );
        assert!(!offers_more(10, false, true), "used once, it goes away");
        assert!(!offers_more(10, true, false), "nothing left to deal");
        assert!(
            !offers_more(0, false, false),
            "no rows, nothing to show more of"
        );
    }
}
