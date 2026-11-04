//! Swipe select: swipe through commanders to pick one, as a full-window
//! overlay over the screen that opened it.
//!
//! A host mounts [`SwipeSelect`] once, beside its frame, and opens it with
//! its `open` signal. It stays mounted while closed, so a reopen picks up
//! where the user left off. For the commander maybeboard:
//!
//! ```ignore
//! SwipeSelect {
//!     open: swipe_open,
//!     mode: SwipeMode::Commander(Format::Commander),
//!     exclude_oracle_ids: saved_ids,     // a ReadSignal<HashSet<Uuid>>
//!     on_select: move |card: Card| { swipe_open.set(false); create_with(card); },
//!     on_close: move |()| swipe_open.set(false),
//! }
//! ```
//!
//! [`SwipeMode::Commander`] deals commanders most played first through
//! `search_commanders`, leaving out `exclude_oracle_ids`; the deck form's
//! [`SwipeMode::Partner`], [`SwipeMode::Background`] and
//! [`SwipeMode::SignatureSpell`] deal the same way for their slot, the last
//! within the `commander` prop's colors. [`SwipeMode::Suggested`]
//! deals the host's `suggested_pile` instead, read again on every open, and
//! its Filter holds the host's `suggested_filter` chips; a change to the pile
//! mid-session re-deals it behind the cards already swiped.
//!
//! Right picks (`on_select`), left skips (a week's dismissal in suggested
//! mode), up saves to the commander maybeboard (not for a background or a
//! signature spell), down undoes. The card drags,
//! the arrow keys swipe, and four buttons under the card do the same.

use std::collections::HashSet;

use components::icons::REFRESH;
use components::{
    BottomSheet, CardDetails, DockFilter, DockIcon, EyeIcon, PageHeader, TOAST_QUICK,
    use_overlay_back,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::http::contracts::suggestions::HttpCommanderSuggestion;
use zwipe_ui::deck::form::{self, Slot};
use zwipe_ui::filter;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::generation::Generation;
use zwipe_ui::hint::SwipeSelectMode;
use zwipe_ui::playstyle::empty_identity_line;
use zwipe_ui::stack::{CardStack, Direction, StackAction};
use zwipe_ui::timing::{LOAD_MORE_AT, MAYBEBOARD_TOAST, PAGE_SIZE};

use crate::domain::swipe_select::{card_key, redeal};
use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::cards::card_info::{CardDetailsSheet, CardInfoDisplay, CardSkeleton};
use crate::inbound::cards::otag::{OtagExamplesOpen, use_describe_tag};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::swipe_stack::{
    EndUndoButton, Swipe, SwipeButtons, SwipeControl, SwipeStack, use_swipe_control,
    use_swipe_keys_or_undo,
};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::back_arrow::BackArrow;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, SwipeSelectHint, open_and_record_hint};
use crate::inbound::chrome::sheet::{SheetForm, SheetFrame};
use crate::inbound::screens::playstyle::SuggestionReasons;

/// Pages of commanders the search keeps pulling when the host's exclusions
/// empty a page, before it waits for the next swipe.
const EXCLUDED_PAGE_RETRIES: usize = 4;

/// What a Swipe select deals. A host keys the overlay by its mode's inputs
/// (the format, the commander), so a change deals a fresh pile while an
/// unchanged reopen keeps the old one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeMode {
    /// Commanders legal in the format, most played first.
    Commander(Format),
    /// Partner commanders, for the deck form's Partner slot.
    Partner,
    /// Backgrounds, for the deck form's Background slot.
    Background,
    /// Signature spells within the `commander` prop's colors.
    SignatureSpell,
    /// The host's suggested commanders, a fixed ranking.
    Suggested,
}

impl SwipeMode {
    /// The deck form slot this mode fills and the format it searches in,
    /// for zwipe-ui's query; `None` for the suggested pile.
    fn slot(self) -> Option<(Slot, Option<Format>)> {
        match self {
            Self::Commander(format) => Some((Slot::Commander, Some(format))),
            Self::Partner => Some((Slot::Partner, None)),
            Self::Background => Some((Slot::Background, None)),
            Self::SignatureSpell => Some((Slot::SignatureSpell, None)),
            Self::Suggested => None,
        }
    }

    /// The word a right swipe picks, as zwipe-ui's slot names it.
    fn noun(self) -> &'static str {
        match self.slot() {
            Some((slot, format)) => slot.noun(format),
            None => "commander",
        }
    }

    /// Whether up saves to the commander maybeboard: commanders and
    /// partners, which can lead a deck.
    fn saves_up(self) -> bool {
        !matches!(self, Self::Background | Self::SignatureSpell)
    }
}

/// The ranking [`SwipeMode::Suggested`] deals, held by its host.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SuggestedPile {
    /// Best fit first, as the list shows them.
    pub suggestions: Vec<HttpCommanderSuggestion>,
    /// Decks the reasons count against.
    pub deck_count: u32,
    /// The identity the host's Show chip narrows `suggestions` to; `None` is
    /// All.
    pub identity: Option<Vec<Color>>,
}

/// A swipe that moved the pile on, so undo can reverse it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectSwipe {
    /// Left: nothing saved.
    Skip,
    /// Up: saved this oracle to the commander maybeboard.
    Maybe(Uuid),
    /// Left in suggested mode: dismissed this oracle for a week.
    Dismissed(Uuid),
}

impl StackAction for SelectSwipe {
    fn exited(&self) -> Direction {
        match self {
            Self::Skip | Self::Dismissed(_) => Direction::Left,
            Self::Maybe(_) => Direction::Up,
        }
    }
}

/// The page at `offset` for `mode`: zwipe-ui's slot query over the user's
/// `filter`, with no tokens and a page at a time. With no sort chosen the
/// server serves its most-played order.
/// A signature spell needs the commander whose colors bound it.
fn page_query(
    mode: SwipeMode,
    filter: &CardQueryBuilder,
    commander: Option<&Card>,
    offset: u32,
) -> Option<CardQuery> {
    let (slot, format) = mode.slot()?;
    if slot == Slot::SignatureSpell && commander.is_none() {
        return None;
    }
    form::swipe_query(filter, slot, format, commander, offset).ok()
}

/// Whether a card is one of the host's exclusions.
fn excluded(card: &Card, ids: &HashSet<Uuid>) -> bool {
    card.scryfall_data
        .oracle_id
        .is_some_and(|oracle_id| ids.contains(&oracle_id))
}

/// The pile and where the user is in it, kept while the overlay is closed.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Pile {
    stack: Signal<CardStack<SelectSwipe>>,
    suggestions: Signal<Vec<HttpCommanderSuggestion>>,
    deck_count: Signal<u32>,
    offset: Signal<u32>,
    exhausted: Signal<bool>,
    loading: Signal<bool>,
    loading_more: Signal<bool>,
    loads: Signal<Generation>,
}

/// The overlay. `on_select` takes the picked card; `on_close` is `Back`.
/// See the module docs for the modes and their props.
#[component]
pub fn SwipeSelect(
    open: Signal<bool>,
    mode: SwipeMode,
    on_select: EventHandler<Card>,
    on_close: EventHandler<()>,
    #[props(default)] exclude_oracle_ids: Option<ReadSignal<HashSet<Uuid>>>,
    #[props(default)] suggested_pile: Option<ReadSignal<SuggestedPile>>,
    #[props(default)] suggested_filter: Option<Element>,
    #[props(default)] commander: Option<Card>,
) -> Element {
    use_overlay_back(open);
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let authed = use_authed();
    let pile = Pile {
        stack: use_signal(CardStack::default),
        suggestions: use_signal(Vec::new),
        deck_count: use_signal(|| 0),
        offset: use_signal(|| 0),
        exhausted: use_signal(|| false),
        loading: use_signal(|| false),
        loading_more: use_signal(|| false),
        loads: use_signal(Generation::default),
    };
    let Pile {
        mut stack,
        mut suggestions,
        mut deck_count,
        mut offset,
        mut exhausted,
        mut loading,
        mut loading_more,
        mut loads,
    } = pile;
    // The card filter's applied filter for the search modes, kept while
    // closed like the pile; Apply deals again.
    let user_filter = use_signal(CardQueryBuilder::new);
    // The Refresh count the pile was last dealt at; `None` before the first.
    let mut dealt_at: Signal<Option<u32>> = use_signal(|| None);
    let mut refreshes = use_signal(|| 0u32);
    let hint_open = use_signal(|| false);
    let mut hint_fired = use_signal(|| false);
    use_effect(move || {
        if open() && !*hint_fired.peek() {
            hint_fired.set(true);
            open_and_record_hint(OneTimeHint::SwipeSelect, session, client, hint_open);
        }
    });

    let excluded_now = move || -> HashSet<Uuid> {
        exclude_oracle_ids
            .map(|ids| ids.peek().clone())
            .unwrap_or_default()
    };

    // The next page of commanders. The host's exclusions can empty a page
    // while more remain, so it keeps pulling a few pages until cards land.
    let commander_for_more = commander.clone();
    let load_more = use_callback(move |(): ()| {
        if mode == SwipeMode::Suggested || *loading_more.peek() || *exhausted.peek() {
            return;
        }
        loading_more.set(true);
        let ticket = loads.peek().current();
        let commander = commander_for_more.clone();
        spawn(async move {
            for _ in 0..EXCLUDED_PAGE_RETRIES {
                let at = *offset.peek();
                let Some(filter) = page_query(mode, &user_filter.peek(), commander.as_ref(), at)
                else {
                    break;
                };
                let reply = authed
                    .run_quiet("swipe_select_more", |c, s| async move {
                        c.search_commanders(&filter, &s).await
                    })
                    .await;
                if !loads.peek().is_current(ticket) {
                    break;
                }
                let Some(found) = reply else {
                    break;
                };
                if found.is_empty() {
                    exhausted.set(true);
                    break;
                }
                let seen: HashSet<Uuid> = stack
                    .peek()
                    .cards()
                    .iter()
                    .map(|c| c.scryfall_data.id)
                    .collect();
                let skip = excluded_now();
                let page: Vec<Card> = found
                    .into_iter()
                    .filter(|c| !seen.contains(&c.scryfall_data.id) && !excluded(c, &skip))
                    .collect();
                let Some(next) = at.checked_add(PAGE_SIZE) else {
                    exhausted.set(true);
                    stack.write().append(page);
                    break;
                };
                offset.set(next);
                if !page.is_empty() {
                    stack.write().append(page);
                    break;
                }
            }
            loading_more.set(false);
        });
    });

    let commander_for_deal = commander;
    // Deal on the first open and on Refresh. The suggested pile is read
    // again on every open, since the host's own actions change it between
    // opens; a commander pile is kept across a close.
    use_effect(move || {
        if !open() {
            return;
        }
        let refresh = refreshes();
        let suggested = mode == SwipeMode::Suggested;
        if !suggested && *dealt_at.peek() == Some(refresh) {
            return;
        }
        dealt_at.set(Some(refresh));
        let ticket = loads.write().next_ticket();
        loading.set(false);
        stack.write().reset();
        offset.set(0);
        exhausted.set(false);
        if suggested {
            let held = suggested_pile
                .map(|pile| pile.peek().clone())
                .unwrap_or_default();
            exhausted.set(true);
            deck_count.set(held.deck_count);
            stack
                .write()
                .replace(held.suggestions.iter().map(|s| s.card.clone()).collect());
            suggestions.set(held.suggestions);
            return;
        }
        let Some(filter) = page_query(mode, &user_filter.peek(), commander_for_deal.as_ref(), 0)
        else {
            return;
        };
        loading.set(true);
        spawn(async move {
            let reply = authed
                .run_quiet("swipe_select_search", |c, s| async move {
                    c.search_commanders(&filter, &s).await
                })
                .await;
            if !loads.peek().is_current(ticket) {
                return;
            }
            loading.set(false);
            let Some(found) = reply else {
                return;
            };
            let raw_empty = found.is_empty();
            let skip = excluded_now();
            let page: Vec<Card> = found.into_iter().filter(|c| !excluded(c, &skip)).collect();
            exhausted.set(raw_empty);
            offset.set(PAGE_SIZE);
            let need_more = !raw_empty && page.is_empty();
            stack.write().replace(page);
            if need_more {
                load_more.call(());
            }
        });
    });

    // The host's Filter chips changed the suggested pile mid-session: deal it
    // again behind the cards already swiped.
    use_effect(move || {
        let Some(held) = suggested_pile else {
            return;
        };
        let held = held();
        if !*open.peek() || mode != SwipeMode::Suggested {
            return;
        }
        let fresh: Vec<Card> = held.suggestions.iter().map(|s| s.card.clone()).collect();
        let (cards, index, history) = stack.peek().park_state();
        let pile = redeal(&cards, index, &fresh, card_key);
        stack.write().restore(pile, index, history);
        let mut reasons = suggestions.peek().clone();
        for suggestion in held.suggestions {
            if !reasons
                .iter()
                .any(|s| s.card.scryfall_data.id == suggestion.card.scryfall_data.id)
            {
                reasons.push(suggestion);
            }
        }
        suggestions.set(reasons);
        deck_count.set(held.deck_count);
    });

    let title = if mode == SwipeMode::Suggested {
        "Suggested commanders"
    } else {
        "Swipe select"
    };
    let hint_mode = match mode {
        SwipeMode::Suggested => SwipeSelectMode::Suggested {
            has_filter: suggested_filter.is_some(),
        },
        SwipeMode::Commander(_)
        | SwipeMode::Partner
        | SwipeMode::Background
        | SwipeMode::SignatureSpell => SwipeSelectMode::Picking(mode.noun()),
    };
    let suggested_identity = suggested_pile.and_then(|held| held.read().identity.clone());
    let pile_empty = suggested_pile.is_some_and(|held| held.read().suggestions.is_empty());

    rsx! {
        div {
            class: if open() { "screen swipe-select-screen web-swipe-select show" } else { "screen swipe-select-screen web-swipe-select" },
            components::GridCells { still: true }
            if open() {
                SwipeSelectBody {
                    pile,
                    mode,
                    title,
                    hint_open,
                    on_select,
                    on_close,
                    load_more,
                    on_refresh: move |()| {
                        let next = refreshes.peek().wrapping_add(1);
                        refreshes.set(next);
                    },
                    user_filter,
                    suggested_identity,
                    pile_empty,
                    suggested_filter,
                }
            }
            SwipeSelectHint { open: hint_open, mode: hint_mode }
        }
    }
}

/// The open overlay: the header, the stack with its buttons and keys, the
/// card's details beside it on a wide window, the toolbar and the sheets.
/// Mounted only while open, so its keys never answer under a closed overlay.
#[component]
fn SwipeSelectBody(
    pile: Pile,
    mode: SwipeMode,
    title: &'static str,
    hint_open: Signal<bool>,
    on_select: EventHandler<Card>,
    on_close: EventHandler<()>,
    load_more: Callback<()>,
    on_refresh: EventHandler<()>,
    suggested_identity: Option<Vec<Color>>,
    pile_empty: bool,
    suggested_filter: Option<Element>,
    user_filter: Signal<CardQueryBuilder>,
) -> Element {
    let toast = use_toast();
    let authed = use_authed();
    let describe_tag = use_describe_tag();
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let Pile {
        mut stack,
        suggestions,
        deck_count,
        exhausted,
        loading,
        loading_more,
        ..
    } = pile;
    let mut hint_again = hint_open;
    let mut show_details = use_signal(|| false);
    let mut printing_open = use_signal(|| false);
    let mut filter_open = use_signal(|| false);
    let suggested = mode == SwipeMode::Suggested;
    let allowed = if mode.saves_up() {
        vec![Swipe::Left, Swipe::Right, Swipe::Up, Swipe::Down]
    } else {
        vec![Swipe::Left, Swipe::Right, Swipe::Down]
    };
    let control: SwipeControl = use_swipe_control();

    let mut advance = move || {
        let near_end = {
            let mut s = stack.write();
            let total = s.len();
            s.advance() && total.saturating_sub(s.index()) <= LOAD_MORE_AT
        };
        if near_end && !*exhausted.peek() {
            load_more.call(());
        }
    };

    let undo = use_callback(move |()| {
        let action = {
            let mut s = stack.write();
            let Some(action) = s.pop_action() else {
                drop(s);
                toast.info("Nothing to undo".to_string(), ToastOptions::default());
                return;
            };
            if !s.step_back(&action) {
                s.record(action);
                return;
            }
            s.prime_entering(Direction::Left);
            action
        };
        match action {
            SelectSwipe::Maybe(oracle_id) => {
                spawn(async move {
                    if authed
                        .try_run("undo_commander_maybeboard", |c, s| async move {
                            c.remove_commander_maybeboard_card(oracle_id, &s).await
                        })
                        .await
                        .is_ok()
                    {
                        toast.info(
                            "Removed from your commander maybeboard".to_string(),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                    }
                });
            }
            SelectSwipe::Dismissed(oracle_id) => {
                spawn(async move {
                    authed
                        .run_quiet("undismiss_commander", |c, s| async move {
                            c.undismiss_commander(oracle_id, &s).await
                        })
                        .await;
                });
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
            SelectSwipe::Skip => {
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        }
    });
    use_swipe_keys_or_undo(
        control,
        allowed.clone(),
        Callback::new(move |()| stack.peek().current().is_some()),
        undo,
    );

    let on_swipe = move |(swipe, card): (Swipe, Card)| match swipe {
        Swipe::Left => {
            let action = match (suggested, card.scryfall_data.oracle_id) {
                (true, Some(oracle_id)) => {
                    // Quiet like a deck skip: a failed post only means it
                    // comes back sooner.
                    spawn(async move {
                        authed
                            .run_quiet("dismiss_commander", |c, s| async move {
                                c.dismiss_commander(oracle_id, &s).await
                            })
                            .await;
                    });
                    SelectSwipe::Dismissed(oracle_id)
                }
                _ => SelectSwipe::Skip,
            };
            stack.write().record(action);
            advance();
            toast.info(
                "Skipped".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        Swipe::Right => {
            let mut label = mode.noun().to_string();
            if let Some(first) = label.get_mut(..1) {
                first.make_ascii_uppercase();
            }
            toast.success(
                format!("{label} selected"),
                ToastOptions::default().duration(TOAST_QUICK),
            );
            on_select.call(card);
        }
        Swipe::Up => {
            let Some(oracle_id) = card.scryfall_data.oracle_id else {
                stack.write().record(SelectSwipe::Skip);
                advance();
                toast.error(
                    "This card can't be saved".to_string(),
                    ToastOptions::default(),
                );
                return;
            };
            stack.write().record(SelectSwipe::Maybe(oracle_id));
            advance();
            spawn(async move {
                if authed
                    .try_run("add_commander_maybeboard", |c, s| async move {
                        c.add_commander_maybeboard_card(oracle_id, &s).await
                    })
                    .await
                    .is_ok()
                {
                    toast.success(
                        "Added to your commander maybeboard".to_string(),
                        ToastOptions::default().duration(MAYBEBOARD_TOAST),
                    );
                }
            });
        }
        Swipe::Down => undo.call(()),
    };

    let (window, current, entering) = {
        let s = stack.read();
        (s.window().to_vec(), s.current().cloned(), s.entering())
    };
    let reasons = current.as_ref().and_then(|card| {
        suggestions
            .read()
            .iter()
            .find(|s| s.card.scryfall_data.id == card.scryfall_data.id)
            .map(|s| s.reasons.clone())
    });
    let empty_line = (suggested && current.is_none() && pile_empty)
        .then_some(suggested_identity.as_deref())
        .flatten()
        .map(empty_identity_line);
    let has_filter = suggested && suggested_filter.is_some();
    let has_card = current.is_some();

    rsx! {
        PageHeader {
            title,
            leading: rsx! {
                BackArrow { class: "page-header-support", onclick: move |()| on_close.call(()) }
            },
            on_hint: move |()| hint_again.set(true),
        }
        div { class: "screen-content card-swipe content-enter web-swipe-select-body",
            div { class: "form-container web-swipe-select-card",
                if let Some(card) = current.clone() {
                    SwipeStack {
                        cards: window,
                        allowed,
                        control,
                        entering,
                        on_entered: move |()| stack.write().cancel_entering(),
                        on_swipe,
                    }
                    SwipeButtons {
                        control,
                        buttons: [
                            (Swipe::Left, "Skip"),
                            (Swipe::Up, "Save"),
                            (Swipe::Down, "Undo"),
                            (Swipe::Right, "Pick"),
                        ]
                            .into_iter()
                            .filter(|(swipe, _)| *swipe != Swipe::Up || mode.saves_up())
                            .collect(),
                    }
                    CardInfoDisplay { card }
                } else if loading() || loading_more() {
                    CardSkeleton { is_loading: true }
                } else if let Some(line) = empty_line {
                    div { class: "message-empty",
                        span { class: "chip-note", "{line}" }
                    }
                } else {
                    CardSkeleton {}
                    EndUndoButton { label: "Undo", on_press: undo }
                }
            }
            if let Some(card) = current.clone() {
                div { class: "web-examples-details web-swipe-select-details",
                    div { class: "sheet-panel",
                        CardDetails {
                            key: "{card.scryfall_data.id}",
                            card,
                            show_name: true,
                            show_cost: true,
                            show_classification: true,
                            describe_tag: Some(describe_tag),
                        }
                        if let Some(reasons) = reasons.clone() {
                            SuggestionReasons {
                                reasons,
                                deck_count: deck_count(),
                                describe_tag,
                                on_examples,
                            }
                        }
                    }
                }
            }
        }
        // Filter, refresh and card details as the action dock. The suggested
        // ranking filters by the host's Show chips alone; a chosen identity
        // lights the dot. Back is the header's arrow.
        div { class: "action-dock action-dock-right",
            if has_filter {
                DockFilter {
                    label: "Filter",
                    dot: suggested_identity.is_some(),
                    onclick: move |()| filter_open.set(true),
                }
            } else if !suggested {
                DockFilter {
                    label: "Filter",
                    dot: filter::shows_dot(&user_filter.read()),
                    onclick: move |()| filter_open.set(true),
                }
            }
            button {
                class: "action-dock-btn",
                "aria-label": "Refresh",
                onclick: move |_| {
                    on_refresh.call(());
                    toast.info(
                        "Stack refreshed".to_string(),
                        ToastOptions::default().duration(TOAST_QUICK),
                    );
                },
                DockIcon { path: REFRESH }
            }
            span { class: "action-dock-divider" }
            button {
                class: "action-dock-btn web-dock-eye",
                "aria-label": "Card details",
                disabled: !has_card,
                onclick: move |_| show_details.toggle(),
                EyeIcon {}
            }
        }
        if let Some(card) = current {
            CardDetailsSheet {
                open: show_details,
                card: card.clone(),
                on_printings: move |()| printing_open.set(true),
                footer: reasons.map(|reasons| rsx! {
                    SuggestionReasons { reasons, deck_count: deck_count(), describe_tag, on_examples }
                }),
            }
            PrintingSheet {
                card,
                open: printing_open,
                on_save: move |picked: Card| {
                    stack.write().replace_current(picked);
                    let mut details = show_details;
                    details.set(false);
                },
            }
        }
        if !suggested {
            CardFilterPanel {
                open: filter_open,
                filter: user_filter,
                config: SheetConfig::default(),
                quiet_reset: true,
                on_apply: move |applied: CardQueryBuilder| {
                    let mut user_filter = user_filter;
                    user_filter.set(applied);
                    on_refresh.call(());
                },
            }
        }
        if let (true, Some(chips)) = (suggested, suggested_filter) {
            SheetFrame { form: SheetForm::SidePanel,
                BottomSheet { open: filter_open, title: "Filter", closable: true, {chips} }
            }
        }
    }
}
