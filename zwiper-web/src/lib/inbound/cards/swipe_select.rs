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
//!     starts_deck: true,                 // the hint says a pick starts a deck
//!     on_select: move |card: Card| { swipe_open.set(false); create_with(card); },
//!     on_close: move |()| swipe_open.set(false),
//! }
//! ```
//!
//! [`SwipeMode::Commander`] deals commanders most played first through
//! `search_commanders`, leaving out `exclude_oracle_ids`; the deck form's
//! [`SwipeMode::Partner`], [`SwipeMode::Background`] and
//! [`SwipeMode::SignatureSpell`] deal the same way for their slot, the last
//! within the `commander` prop's colors. [`SwipeMode::Commanders`] deals
//! the merged commander ranking through `deal_commanders` instead, for its
//! door: Commanders' Suggested list, or deck creation's commander slot. It
//! starts from the host's `start_filter` on every open, and a next page
//! leaves out what it already dealt.
//!
//! Right picks (`on_select`), left skips (a week's dismissal when dealing
//! the merged ranking), up saves to the commander maybeboard (not for a background or a
//! signature spell), down undoes. The card drags,
//! the arrow keys swipe, and four buttons under the card do the same.

use std::collections::{HashMap, HashSet};

use components::icons::{DockIcon, REFRESH};
use components::keys::{use_page_keys_through, use_undo_key};
use components::swipe_stack::{
    Swipe, SwipeControl, SwipeDock, SwipeStack, use_swipe_control, use_swipe_keys_or_undo,
};
use components::{
    BackArrow, CardDetailsPanelSkeleton, CardGhost, Chip, DockEye, DockFilter, MeldSource,
    PageHeader, PrintingsFace, Surface, TOAST_QUICK, open_on_shown_face, spawn_write_in_order,
    turn_face, use_details_eye, use_mounted, use_overlay_back,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::http::contracts::suggestions::HttpCommanderSuggestion;
use zwipe_ui::commander::deal::{self, Door};
use zwipe_ui::commander::maybeboard;
use zwipe_ui::deck::form::{self, Slot};
use zwipe_ui::filter;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::generation::Generation;
use zwipe_ui::hint::{Role, SwipeSelectMode};
use zwipe_ui::screens::add_cards::{self, Source};
use zwipe_ui::screens::swipe_select::{empty_pile_line, filter_lit};
use zwipe_ui::stack::{CardStack, Direction, NO_CARDS_LEFT, StackAction};
use zwipe_ui::telemetry::vocabulary::component;
use zwipe_ui::timing::{LOAD_MORE_AT, MAYBEBOARD_TOAST, PAGE_SIZE};

use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_info::{
    CardDetailsSheet, DETAILS_DRAWER_CLASS, THROUGH_DRAWER_KEYS,
};
use crate::inbound::cards::otag::{OtagExamplesOpen, use_describe_tag};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::swipe_confirm::{SwipeScreenChips, use_swipe_confirm};
use crate::inbound::chrome::authed::{Authed, use_authed};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, SwipeSelectHint, open_and_record_hint};
use crate::inbound::chrome::shortcuts::{self, SELECT_LABELS, Shortcut, ShortcutsScreen};
use crate::inbound::chrome::shortcuts_sheet::{use_global_keys, use_offer_shortcuts};
use crate::inbound::screens::deck::cards::SwipeDetailsPanel;
use crate::inbound::screens::playstyle::SuggestionReasons;
use crate::inbound::shell::{WebFooter, use_overlay_nav};
use crate::inbound::telemetry;

/// The open sheet whose keys still reach the screen, and those keys: `v` through the details
/// drawer turns the card, as on deck Add.
const SWIPE_KEYS_THROUGH: (&str, &[&str; 2]) = (DETAILS_DRAWER_CLASS, &THROUGH_DRAWER_KEYS);

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
    /// The merged commander ranking for this door, paged by the server.
    Commanders(Door),
}

impl SwipeMode {
    /// The deck form slot this mode fills and the format it searches in,
    /// for zwipe-ui's query; `None` for the merged ranking, which isn't a search.
    fn slot(self) -> Option<(Slot, Option<Format>)> {
        match self {
            Self::Commander(format) => Some((Slot::Commander, Some(format))),
            Self::Partner => Some((Slot::Partner, None)),
            Self::Background => Some((Slot::Background, None)),
            Self::SignatureSpell => Some((Slot::SignatureSpell, None)),
            Self::Commanders(_) => None,
        }
    }

    /// The door whose merged ranking the mode deals, if it deals one.
    fn door(self) -> Option<Door> {
        match self {
            Self::Commanders(door) => Some(door),
            Self::Commander(_) | Self::Partner | Self::Background | Self::SignatureSpell => None,
        }
    }

    /// The word a right swipe picks, as zwipe-ui's slot names it.
    fn noun(self) -> &'static str {
        match (self.slot(), self.door()) {
            (Some((slot, format)), _) => slot.noun(format),
            (None, Some(Door::DeckCreation(format))) => Slot::Commander.noun(Some(format)),
            (None, Some(Door::Suggested) | None) => "commander",
        }
    }

    /// Whether up saves to the commander maybeboard: commanders and
    /// partners, which can lead a deck.
    fn saves_up(self) -> bool {
        !matches!(self, Self::Background | Self::SignatureSpell)
    }
}

/// A swipe that moved the pile on, so undo can reverse it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectSwipe {
    /// Left: nothing saved.
    Skip,
    /// Up, saving nothing: the card had no oracle id.
    UnsavedUp,
    /// Up: saved this oracle to the commander maybeboard.
    Maybe(Uuid),
    /// Left on the merged ranking: dismissed this oracle for a week.
    Dismissed(Uuid),
}

impl StackAction for SelectSwipe {
    fn exited(&self) -> Direction {
        match self {
            Self::Skip | Self::Dismissed(_) => Direction::Left,
            Self::Maybe(_) | Self::UnsavedUp => Direction::Up,
        }
    }
}

/// Takes the newest swipe off `stack` and steps back to its card, which comes back from the
/// side it left by. `None` with nothing to undo, or when the swiped card is gone, which keeps
/// the swipe.
fn undo_step(stack: &mut CardStack<SelectSwipe>) -> Option<SelectSwipe> {
    let action = stack.pop_action()?;
    if !stack.step_back(&action) {
        stack.record(action);
        return None;
    }
    stack.prime_entering(action.exited());
    Some(action)
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

/// `cards` without the ones the commander cannot pair with, on the partner pile with a commander chosen.
fn partners_only(mode: SwipeMode, commander: Option<&Card>, cards: Vec<Card>) -> Vec<Card> {
    match (mode, commander) {
        (SwipeMode::Partner, Some(commander)) => form::legal_partners(commander, cards),
        _ => cards,
    }
}

/// Whether a card is one of the host's exclusions.
fn excluded(card: &Card, ids: &HashSet<Uuid>) -> bool {
    card.scryfall_data
        .oracle_id
        .is_some_and(|oracle_id| ids.contains(&oracle_id))
}

/// The oracle ids a next page of the merged ranking leaves out: the pile's cards and the host's
/// exclusions.
fn dealt_ids(stack: &CardStack<SelectSwipe>, host: &HashSet<Uuid>) -> Vec<Uuid> {
    stack
        .cards()
        .iter()
        .filter_map(|c| c.scryfall_data.oracle_id)
        .chain(host.iter().copied())
        .collect()
}

/// One page of `door`'s merged ranking under `filter`, leaving out `dealt`: `Ok(None)` once the
/// session has dealt as many as the server reads, and `Err(())` when the request failed (logged
/// quietly; the pile just stops growing).
async fn deal_page(
    authed: Authed,
    door: Door,
    filter: &CardQueryBuilder,
    dealt: Vec<Uuid>,
) -> Result<Option<Vec<HttpCommanderSuggestion>>, ()> {
    let Some(request) = deal::next_page(door, filter, &dealt, PAGE_SIZE) else {
        return Ok(None);
    };
    authed
        .run_quiet("swipe_select_deal", |c, s| async move {
            c.deal_commanders(request, &s).await
        })
        .await
        .map(|page| Some(page.suggestions))
        .ok_or(())
}

/// `saved` narrowed to the commanders that lead a deck in `door`'s format (Commander for
/// Suggested) by the format's commander rules, are legal in it where the door has a deck format,
/// as the deal asks the server for, and that `filter` matches. A filter that doesn't build
/// narrows by the format alone.
fn maybeboard_matching(saved: Vec<Card>, filter: &CardQueryBuilder, door: Door) -> Vec<Card> {
    let format = match door {
        Door::Suggested => Format::Commander,
        Door::DeckCreation(format) => format,
    };
    let mut narrowed = deal::door_filter(door, filter);
    narrowed.set_is_commander_in_format(format);
    let criteria = narrowed.build_criteria().or_else(|_| {
        let mut format_only = deal::door_filter(door, &CardQueryBuilder::new());
        format_only.set_is_commander_in_format(format);
        format_only.build_criteria()
    });
    match criteria {
        Ok(criteria) => Cards::from(saved).matching(&criteria).into(),
        Err(_) => saved,
    }
}

/// A Maybe load: how many commanders are saved, and the ones `filter` and the door's format let
/// through. `Err(())` when it didn't load.
type MaybeReply = Result<(usize, Vec<Card>), ()>;

/// The user's commander maybeboard under `filter` for `door`, matched on the device: `Err(())`
/// when it didn't load (logged quietly; the caller says so).
async fn maybeboard_pile(authed: Authed, filter: &CardQueryBuilder, door: Door) -> MaybeReply {
    let saved = authed
        .run_quiet("swipe_select_maybeboard", |c, s| async move {
            c.fetch_commander_maybeboard(&s).await
        })
        .await
        .ok_or(())?;
    let count = saved.len();
    Ok((count, maybeboard_matching(saved, filter, door)))
}

/// The line Maybe's empty card carries, as the commander maybeboard's own list words it: the
/// failed load, nothing saved, or saved commanders none of which pass. `None` with cards to swipe.
fn maybe_note(reply: &MaybeReply) -> Option<&'static str> {
    match reply {
        Err(()) => Some(maybeboard::LOAD_FAILED),
        Ok((0, _)) => Some(maybeboard::NO_COMMANDERS),
        Ok((_, matched)) if matched.is_empty() => Some(maybeboard::NO_MATCH),
        Ok(_) => None,
    }
}

/// The swipes `mode` takes from `source`, and whether left skips a commander for a week: the
/// maybeboard's commanders are saved already, so up saves nothing there and left only passes one
/// by.
fn swipes(mode: SwipeMode, source: Option<Source>) -> (Vec<Swipe>, bool) {
    let on_maybe = source == Some(Source::Maybeboard);
    let dismisses = mode.door().is_some() && !on_maybe;
    let allowed = if mode.saves_up() && !on_maybe {
        vec![Swipe::Left, Swipe::Right, Swipe::Up, Swipe::Down]
    } else {
        vec![Swipe::Left, Swipe::Right, Swipe::Down]
    };
    (allowed, dismisses)
}

/// Lands a deal's reply on the pile: the first page replaces it, a next one joins it; a short
/// page, or none left to ask for, ends the pile.
fn land_deal(pile: &Pile, reply: Result<Option<Vec<HttpCommanderSuggestion>>, ()>, first: bool) {
    let Pile {
        mut stack,
        mut suggestions,
        mut exhausted,
        mut searched,
        ..
    } = *pile;
    let page = match reply {
        Ok(Some(page)) => page,
        Ok(None) => {
            exhausted.set(true);
            return;
        }
        Err(()) => return,
    };
    searched.set(true);
    exhausted.set(deal::is_last_page(page.len(), PAGE_SIZE));
    let cards: Vec<Card> = page.iter().map(|s| s.card.clone()).collect();
    if first {
        stack.write().replace(cards);
    } else {
        stack.write().append(cards);
    }
    suggestions.write().extend(page);
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
    /// Whether the search's first page has landed since the last deal.
    searched: Signal<bool>,
    loads: Signal<Generation>,
    /// The line Maybe's empty card carries since its last load ([`maybe_note`]).
    maybe_line: Signal<Option<&'static str>>,
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
    #[props(default)] start_filter: Option<ReadSignal<CardQueryBuilder>>,
    #[props(default)] reasons_deck_count: Option<ReadSignal<u32>>,
    #[props(default)] commander: Option<Card>,
    #[props(default)] starts_deck: bool,
) -> Element {
    let layer = use_overlay_back(open);
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let authed = use_authed().in_component(component::SWIPE_SELECT);
    let usage = telemetry::use_usage();
    let toast = use_toast();
    // The From chip's source, for a mode that deals the merged ranking; kept across a close.
    let mut source = use_signal(Source::default);
    // Each source's filter, parked while another is picked.
    let mut parked: Signal<HashMap<Source, CardQueryBuilder>> = use_signal(HashMap::new);
    let pile = Pile {
        stack: use_signal(CardStack::default),
        suggestions: use_signal(Vec::new),
        deck_count: use_signal(|| 0),
        offset: use_signal(|| 0),
        exhausted: use_signal(|| false),
        loading: use_signal(|| false),
        loading_more: use_signal(|| false),
        searched: use_signal(|| false),
        loads: use_signal(Generation::default),
        maybe_line: use_signal(|| None),
    };
    let Pile {
        mut stack,
        mut suggestions,
        mut deck_count,
        mut offset,
        mut exhausted,
        mut loading,
        mut loading_more,
        mut searched,
        mut loads,
        mut maybe_line,
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

    // The deck count a suggestion's reasons count against, as the host knows it.
    use_effect(move || {
        if let Some(count) = reasons_deck_count {
            deck_count.set(count());
        }
    });

    let excluded_now =
        move || -> HashSet<Uuid> { exclude_oracle_ids.map_or_default(|ids| ids.peek().clone()) };

    // The next page of commanders. The host's exclusions can empty a page
    // while more remain, so it keeps pulling a few pages until cards land.
    let commander_for_more = commander.clone();
    let load_more = use_callback(move |(): ()| {
        if *loading_more.peek() || *exhausted.peek() {
            return;
        }
        loading_more.set(true);
        let ticket = loads.peek().current();
        let commander = commander_for_more.clone();
        if let Some(door) = mode.door() {
            if *source.peek() == Source::Maybeboard {
                loading_more.set(false);
                return;
            }
            spawn(async move {
                let dealt = dealt_ids(&stack.peek(), &excluded_now());
                let applied = user_filter.peek().clone();
                let reply = deal_page(authed, door, &applied, dealt).await;
                if loads.peek().is_current(ticket) {
                    land_deal(&pile, reply, /* first */ false);
                    if *exhausted.peek()
                        && stack.peek().current().is_none()
                        && !stack.peek().is_empty()
                    {
                        toast.info(
                            NO_CARDS_LEFT.to_owned(),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                    }
                }
                loading_more.set(false);
            });
            return;
        }
        spawn(async move {
            for _ in 0..EXCLUDED_PAGE_RETRIES {
                let at = *offset.peek();
                let Some(filter) = page_query(mode, &user_filter.peek(), commander.as_ref(), at)
                else {
                    break;
                };
                if session.peek().is_some() {
                    usage.peek().record_search();
                }
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
                    // The cursor waits past the last card: the pile has run out.
                    let at_end = {
                        let s = stack.peek();
                        !s.is_empty() && s.current().is_none()
                    };
                    if at_end {
                        toast.info(
                            NO_CARDS_LEFT.to_owned(),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                    }
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
                let page = partners_only(mode, commander.as_ref(), page);
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
        // The host's filter is where the pile starts: a change since the last
        // open deals again from the top.
        let mut restart = false;
        if let Some(start) = start_filter
            && *source.peek() == Source::Search
        {
            let start = start.peek().clone();
            if *user_filter.peek() != start {
                let mut user_filter = user_filter;
                user_filter.set(start);
                restart = true;
            }
        }
        if !restart && *dealt_at.peek() == Some(refresh) {
            return;
        }
        dealt_at.set(Some(refresh));
        let ticket = loads.write().next_ticket();
        loading.set(false);
        stack.write().reset();
        offset.set(0);
        exhausted.set(false);
        searched.set(false);
        suggestions.set(Vec::new());
        maybe_line.set(None);
        if let Some(door) = mode.door() {
            loading.set(true);
            let at = *source.peek();
            spawn(async move {
                let applied = user_filter.peek().clone();
                if at == Source::Search {
                    let dealt = dealt_ids(&CardStack::default(), &excluded_now());
                    let reply = deal_page(authed, door, &applied, dealt).await;
                    if !loads.peek().is_current(ticket) {
                        return;
                    }
                    loading.set(false);
                    land_deal(&pile, reply, /* first */ true);
                    return;
                }
                let reply = maybeboard_pile(authed, &applied, door).await;
                if !loads.peek().is_current(ticket) {
                    return;
                }
                loading.set(false);
                let mut line = pile.maybe_line;
                line.set(maybe_note(&reply));
                if let Ok((_, saved)) = reply {
                    searched.set(true);
                    exhausted.set(true);
                    stack.write().replace(saved);
                }
            });
            return;
        }
        let Some(filter) = page_query(mode, &user_filter.peek(), commander_for_deal.as_ref(), 0)
        else {
            return;
        };
        if session.peek().is_some() {
            usage.peek().record_search();
        }
        loading.set(true);
        let commander_for_page = commander_for_deal.clone();
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
            searched.set(true);
            let raw_empty = found.is_empty();
            let skip = excluded_now();
            let page: Vec<Card> = found.into_iter().filter(|c| !excluded(c, &skip)).collect();
            let page = partners_only(mode, commander_for_page.as_ref(), page);
            exhausted.set(raw_empty);
            offset.set(PAGE_SIZE);
            let need_more = !raw_empty && page.is_empty();
            stack.write().replace(page);
            if need_more {
                load_more.call(());
            }
        });
    });

    // The From chip: parks this source's filter, takes the other's, and deals again.
    let next_source = use_callback(move |()| {
        let from = *source.peek();
        let to = from.toggled();
        let leaving = user_filter.peek().clone();
        parked.write().insert(from, leaving);
        let entering = parked.peek().get(&to).cloned().unwrap_or_default();
        let mut user_filter = user_filter;
        user_filter.set(entering);
        source.set(to);
        let next = refreshes.peek().wrapping_add(1);
        refreshes.set(next);
    });

    let title = if mode == SwipeMode::Commanders(Door::Suggested) {
        "Suggested commanders"
    } else {
        "Swipe select"
    };
    // On a wide window the overlay sits under the nav, which names it, opens
    // its hint and closes it from the back arrow.
    let mut hint_from_nav = hint_open;
    use_overlay_nav(
        open,
        title.to_owned(),
        Callback::new(move |()| hint_from_nav.set(true)),
        Callback::new(move |()| on_close.call(())),
    );
    let hint_mode = match mode {
        SwipeMode::Commanders(Door::Suggested) => SwipeSelectMode::Suggested { has_filter: true },
        _ if starts_deck => SwipeSelectMode::StartsDeck,
        SwipeMode::Commander(_)
        | SwipeMode::Commanders(Door::DeckCreation(_))
        | SwipeMode::Partner
        | SwipeMode::Background
        | SwipeMode::SignatureSpell => SwipeSelectMode::Picking(mode.noun()),
    };

    rsx! {
        div {
            class: if open() { "screen swipe-select-screen web-swipe-select web-scroll-screen show" } else { "screen swipe-select-screen web-swipe-select web-scroll-screen" },
            style: layer().map(|z| format!("--overlay-z: {z};")),
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
                    source: mode.door().map(|_| source),
                    next_source,
                }
                // It scrolls as a screen does and ends as one, with the
                // site's footer below the fold.
                WebFooter {}
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
    user_filter: Signal<CardQueryBuilder>,
    source: Option<Signal<Source>>,
    next_source: Callback<()>,
) -> Element {
    let toast = use_toast();
    let confirm = use_swipe_confirm();
    // A write lands even after the screen goes; its confirmation waits on this.
    let mounted = use_mounted();
    let stores = try_use_context::<CardStores>();
    let authed = use_authed().in_component(component::SWIPE_SELECT);
    let usage = telemetry::use_usage();
    let describe_tag = use_describe_tag();
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let Pile {
        mut stack,
        suggestions,
        deck_count,
        exhausted,
        loading,
        loading_more,
        searched,
        maybe_line,
        ..
    } = pile;
    let mut hint_again = hint_open;
    let show_details = use_signal(|| false);
    let eye = use_details_eye(show_details, Surface::Window);
    let mut printing_open = use_signal(|| false);
    // The face each card shows, the front where it has none: its stack image, its pane and its
    // drawer show it, and their flips and `v` turn it, so the image and the details turn together.
    let faces: Signal<HashMap<Uuid, usize>> = use_signal(HashMap::new);
    let handed = try_use_context::<PrintingsFace>();
    let melds = try_use_context::<MeldSource>();
    let mut filter_open = use_signal(|| false);
    let on_maybe = source.is_some_and(|s| s() == Source::Maybeboard);
    let (allowed, dismisses) = swipes(mode, source.map(|s| s()));
    let control: SwipeControl = use_swipe_control();
    // A swipe screen: Shift+H opens the shortcuts sheet here.
    let labels = if mode.saves_up() {
        SELECT_LABELS
    } else {
        SELECT_LABELS.without_up()
    };
    use_offer_shortcuts(ShortcutsScreen::Swipe(labels));

    // Moves past the swiped card, fetching the next page near the end, and
    // says whether the pile ran out under it: no card left and none to come.
    let mut advance = move || -> bool {
        let near_end = {
            let mut s = stack.write();
            let total = s.len();
            s.advance() && total.saturating_sub(s.index()) <= LOAD_MORE_AT
        };
        if near_end && !*exhausted.peek() {
            load_more.call(());
        }
        let done = *exhausted.peek();
        done && !*loading_more.peek() && stack.peek().current().is_none()
    };
    // The pile's end replaces the swipe's own line, whatever the
    // confirmation setting.
    let ran_out = move || {
        toast.info(
            NO_CARDS_LEFT.to_owned(),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    };

    let undo = use_callback(move |()| {
        let had_history = stack.peek().has_history();
        let Some(action) = undo_step(&mut stack.write()) else {
            if !had_history {
                toast.info("Nothing to undo".to_string(), ToastOptions::default());
            }
            return;
        };
        match action {
            SelectSwipe::Maybe(oracle_id) => {
                spawn_write_in_order(oracle_id, async move {
                    let removed = authed
                        .try_run("undo_commander_maybeboard", |c, s| async move {
                            c.remove_commander_maybeboard_card(oracle_id, &s).await
                        })
                        .await
                        .is_ok();
                    if removed && let Some(stores) = stores {
                        stores.maybeboard_written();
                    }
                    if removed && mounted.get() {
                        confirm.info("Removed from your commander maybeboard");
                    }
                });
            }
            SelectSwipe::Dismissed(oracle_id) => {
                spawn_write_in_order(oracle_id, async move {
                    authed
                        .run_quiet("undismiss_commander", |c, s| async move {
                            c.undismiss_commander(oracle_id, &s).await
                        })
                        .await;
                });
                confirm.info("Undid skip");
            }
            SelectSwipe::Skip | SelectSwipe::UnsavedUp => {
                confirm.info("Undid skip");
            }
        }
    });
    use_swipe_keys_or_undo(
        control,
        allowed.clone(),
        Callback::new(move |()| stack.peek().current().is_some()),
        undo,
    );
    use_undo_key(Callback::new(move |()| undo.call(())));

    // Refresh, from its dock button or `r`.
    let refresh = use_callback(move |()| {
        on_refresh.call(());
        toast.info(
            "Stack refreshed".to_string(),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    });
    let send_to = try_use_context::<components::SendTo>();
    // The swipe screen's letter keys, each the same action as its dock
    // button; the swipes come through the swipe keys above, and there is no
    // Synergy here.
    use_page_keys_through(
        SWIPE_KEYS_THROUGH.0,
        SWIPE_KEYS_THROUGH.1,
        Callback::new(
            move |key: String| match shortcuts::swipe_screen_shortcut(&key) {
                Some(Shortcut::Printings) => {
                    if let Some(card) = stack.peek().current() {
                        open_on_shown_face(handed, faces, card.scryfall_data.id, printing_open);
                    }
                }
                Some(Shortcut::SendTo) => {
                    if let (Some(send_to), Some(card)) = (send_to, stack.peek().current().cloned())
                    {
                        send_to.open_for(card);
                    }
                }
                Some(Shortcut::Details) => {
                    if stack.peek().current().is_some() {
                        eye.press();
                    }
                }
                Some(Shortcut::Filter) => filter_open.set(true),
                Some(Shortcut::FilterByName) => components::open_at_name(filter_open),
                Some(Shortcut::Flip) => {
                    if let Some(card) = stack.peek().current() {
                        turn_face(faces, card, melds);
                    }
                }
                Some(Shortcut::Refresh) => refresh.call(()),
                Some(Shortcut::Synergy | Shortcut::Swipe(_)) | None => {}
            },
        ),
    );
    // `?` opens this screen's hint, as its `?` button does.
    use_global_keys(Some(use_callback(move |()| hint_again.set(true))));

    let mut on_decision = move |(swipe, card): (Swipe, Card)| match swipe {
        Swipe::Left => {
            let action = match (dismisses, card.scryfall_data.oracle_id) {
                (true, Some(oracle_id)) => {
                    // Quiet like a deck skip: a failed post only means it
                    // comes back sooner.
                    spawn_write_in_order(oracle_id, async move {
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
            if advance() {
                confirm.quietly("Skipped");
                ran_out();
            } else {
                confirm.info("Skipped");
            }
        }
        Swipe::Right => {
            let mut label = mode.noun().to_string();
            if let Some(first) = label.get_mut(..1) {
                first.make_ascii_uppercase();
            }
            confirm.success(format!("{label} selected"));
            on_select.call(card);
        }
        Swipe::Up => {
            let Some(oracle_id) = card.scryfall_data.oracle_id else {
                stack.write().record(SelectSwipe::UnsavedUp);
                advance();
                toast.error(
                    "This card can't be saved".to_string(),
                    ToastOptions::default(),
                );
                return;
            };
            stack.write().record(SelectSwipe::Maybe(oracle_id));
            let last = advance();
            if last {
                ran_out();
            }
            spawn_write_in_order(oracle_id, async move {
                let saved = authed
                    .try_run("add_commander_maybeboard", |c, s| async move {
                        c.add_commander_maybeboard_card(oracle_id, &s).await
                    })
                    .await
                    .is_ok();
                if saved && let Some(stores) = stores {
                    stores.maybeboard_written();
                }
                if saved && mounted.get() {
                    if last {
                        confirm.quietly("Added to your commander maybeboard");
                    } else {
                        confirm.success_for("Added to your commander maybeboard", MAYBEBOARD_TOAST);
                    }
                }
            });
        }
        Swipe::Down => undo.call(()),
    };
    let on_swipe = move |(swipe, card): (Swipe, Card)| {
        usage.peek().record_swipe(swipe.into());
        if matches!(swipe, Swipe::Left | Swipe::Right) {
            usage
                .peek()
                .record_select_signal(card.scryfall_data.oracle_id, swipe.into());
        }
        on_decision((swipe, card));
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
    let dealt_nothing = searched() && !loading() && !loading_more() && stack.read().is_empty();
    let empty_line =
        dealt_nothing.then(|| empty_pile_line(mode.noun(), filter_lit(&user_filter.read())));
    let has_card = current.is_some();

    let dock_labels = if on_maybe {
        labels.without_up()
    } else {
        labels
    };
    let dock = rsx! {
        SwipeDock {
            control,
            buttons: dock_labels.buttons(),
            card_on_top: has_card,
            can_undo: stack.read().has_history(),
            on_undo: move |()| undo.call(()),
            div { class: if source.is_some() { "action-dock" } else { "action-dock action-dock-right" },
                DockFilter {
                    label: "Filter",
                    dot: filter::shows_dot(&user_filter.read()),
                    onclick: move |()| filter_open.set(true),
                }
                button {
                    class: "action-dock-btn",
                    "aria-label": "Refresh",
                    onclick: move |_| refresh.call(()),
                    DockIcon { path: REFRESH }
                }
                span { class: "action-dock-divider" }
                DockEye { eye, disabled: !has_card }
            }
        }
    };

    rsx! {
        PageHeader {
            title,
            leading: rsx! {
                BackArrow { class: "page-header-support", onclick: move |()| on_close.call(()) }
            },
            on_hint: move |()| hint_again.set(true),
        }
        div { class: "screen-content card-swipe content-enter web-swipe-select-body {eye.body_class()}",
            SwipeScreenChips { title, tone: Role::Success, confirm, at_end: !has_card }
            div { class: "form-container web-swipe-select-card",
                if current.is_some() {
                    SwipeStack {
                        cards: window,
                        allowed,
                        control,
                        entering,
                        on_entered: move |()| stack.write().cancel_entering(),
                        on_swipe,
                        faces,
                        ghost: true,
                    }
                } else if loading() || loading_more() {
                    CardGhost {}
                } else if let Some(line) = maybe_line().filter(|_| on_maybe) {
                    // Maybe's empty card, worded as the commander maybeboard's list words it.
                    CardGhost { note: line }
                } else if let Some(line) = empty_line {
                    // Why there is no card, inside the card's own box, as deck Add says it.
                    CardGhost { note: line }
                } else {
                    CardGhost {}
                }
            }
            if let Some(card) = current.clone() {
                div { class: "web-examples-details web-swipe-select-details",
                    SwipeDetailsPanel {
                        key: "{card.scryfall_data.id}",
                        card,
                        describe_tag: Some(describe_tag),
                        on_printings: move |()| printing_open.set(true),
                        faces: Some(faces),
                        body_end: reasons.clone().map(|reasons| rsx! {
                            SuggestionReasons {
                                reasons,
                                deck_count: deck_count(),
                                describe_tag,
                                on_examples,
                            }
                        }),
                    }
                }
            } else if loading() || loading_more() {
                div { class: "web-examples-details web-swipe-select-details",
                    CardDetailsPanelSkeleton {}
                }
            }
        }
        // Filter, refresh and card details as the action dock, with From's chip on its left
        // where the pile has sources (as on Add). Back is the header's arrow.
        if let Some(source) = source {
            div { class: "swipe-bottom-row",
                div { class: "swipe-bottom-chips",
                    span { class: "stat-chip stat-chip-p1 chip-row-label", "{add_cards::FROM}" }
                    Chip {
                        selected: true,
                        onclick: move |_| next_source.call(()),
                        "{source().label()}"
                    }
                }
                {dock}
            }
        } else {
            {dock}
        }
        if let Some(card) = current {
            CardDetailsSheet {
                open: show_details,
                card: card.clone(),
                on_printings: move |()| printing_open.set(true),
                face: faces.read().get(&card.scryfall_data.id).copied().unwrap_or(0),
                on_face_change: {
                    let id = card.scryfall_data.id;
                    let mut faces = faces;
                    move |next: usize| {
                        faces.write().insert(id, next);
                    }
                },
                body_end: reasons.map(|reasons| rsx! {
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
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::scryfall_data::legalities::LegalityKind;

    use super::*;

    /// A real card, as the API serves it.
    fn card() -> Card {
        serde_json::from_str(include_str!(
            "../../../../../zwipe-ui/src/test_support/card.json"
        ))
        .expect("the fixture card parses")
    }

    #[test]
    fn maybe_deals_the_maybeboard_under_the_filter() {
        let saved = commander("Legendary Creature — Human Wizard");
        let name = saved.scryfall_data.name.clone();
        let door = Door::Suggested;
        assert_eq!(
            maybeboard_matching(vec![saved.clone()], &CardQueryBuilder::new(), door).len(),
            1,
            "no filter keeps every saved commander"
        );
        let mut miss = CardQueryBuilder::new();
        miss.set_name_contains("zzzzqqq");
        assert_eq!(
            maybeboard_matching(vec![saved.clone()], &miss, door).len(),
            0,
            "a name it lacks drops it"
        );
        let mut hit = CardQueryBuilder::new();
        hit.set_name_contains(name);
        assert_eq!(maybeboard_matching(vec![saved], &hit, door).len(), 1);
    }

    #[test]
    fn maybe_deals_only_commanders_that_lead_in_the_door_s_format() {
        let vehicle = commander("Legendary Artifact — Vehicle");
        let none = CardQueryBuilder::new();
        assert_eq!(
            maybeboard_matching(vec![vehicle.clone()], &none, Door::Suggested).len(),
            1,
            "a legendary vehicle with power and toughness leads in Commander"
        );
        assert_eq!(
            maybeboard_matching(vec![vehicle], &none, Door::DeckCreation(Format::Brawl)).len(),
            0,
            "but not in Brawl"
        );
        let mut creature = commander("Legendary Creature — Human Wizard");
        creature.scryfall_data.legalities.brawl = Some(LegalityKind::Legal);
        assert_eq!(
            maybeboard_matching(vec![creature], &none, Door::DeckCreation(Format::Brawl)).len(),
            1
        );
    }

    #[test]
    fn maybe_on_a_brawl_deck_drops_a_commander_not_legal_in_brawl() {
        let mut meren = commander("Legendary Creature — Human Shaman");
        meren.scryfall_data.name = "Meren of Clan Nel Toth".to_owned();
        meren.scryfall_data.legalities.commander = Some(LegalityKind::Legal);
        meren.scryfall_data.legalities.brawl = Some(LegalityKind::NotLegal);
        meren.scryfall_data.legalities.standardbrawl = Some(LegalityKind::NotLegal);
        let none = CardQueryBuilder::new();
        assert_eq!(
            maybeboard_matching(vec![meren.clone()], &none, Door::Suggested).len(),
            1,
            "Suggested has no deck format to hold it to"
        );
        for format in [Format::Brawl, Format::StandardBrawl] {
            assert_eq!(
                maybeboard_matching(vec![meren.clone()], &none, Door::DeckCreation(format)).len(),
                0,
                "{format:?}"
            );
        }
        let mut legal = commander("Legendary Creature — Human Wizard");
        legal.scryfall_data.legalities.brawl = Some(LegalityKind::Legal);
        assert_eq!(
            maybeboard_matching(vec![legal], &none, Door::DeckCreation(Format::Brawl)).len(),
            1
        );
    }

    #[test]
    fn maybe_s_ghost_reads_as_the_commander_maybeboard_does() {
        assert_eq!(
            maybe_note(&Err(())),
            Some(maybeboard::LOAD_FAILED),
            "a failed load says so, not a grey card"
        );
        assert_eq!(
            maybe_note(&Ok((0, vec![]))),
            Some(maybeboard::NO_COMMANDERS),
            "nothing saved"
        );
        assert_eq!(
            maybe_note(&Ok((2, vec![]))),
            Some(maybeboard::NO_MATCH),
            "saved commanders, none through the filter or the format"
        );
        assert_eq!(maybe_note(&Ok((2, vec![card()]))), None, "cards to swipe");
    }

    #[test]
    fn on_maybe_up_saves_nothing_and_left_only_passes() {
        let mode = SwipeMode::Commanders(Door::Suggested);
        assert_eq!(
            swipes(mode, Some(Source::Search)),
            (
                vec![Swipe::Left, Swipe::Right, Swipe::Up, Swipe::Down],
                /* dismisses */ true
            )
        );
        assert_eq!(
            swipes(mode, Some(Source::Maybeboard)),
            (vec![Swipe::Left, Swipe::Right, Swipe::Down], false)
        );
    }

    /// The fixture card with `type_line`, for the commander rules.
    fn commander(type_line: &str) -> Card {
        let mut card = card();
        card.scryfall_data.type_line = Some(type_line.to_owned());
        card
    }

    #[test]
    fn the_merged_ranking_s_doors_deal_it_and_the_search_modes_don_t() {
        assert_eq!(
            SwipeMode::Commanders(Door::Suggested).door(),
            Some(Door::Suggested)
        );
        let deck = Door::DeckCreation(Format::Commander);
        assert_eq!(SwipeMode::Commanders(deck).door(), Some(deck));
        for mode in [
            SwipeMode::Commander(Format::Commander),
            SwipeMode::Partner,
            SwipeMode::Background,
            SwipeMode::SignatureSpell,
        ] {
            assert_eq!(mode.door(), None, "{mode:?}");
        }
        assert_eq!(
            SwipeMode::Commanders(Door::Suggested).slot(),
            None,
            "not a search"
        );
    }

    #[test]
    fn deck_creation_names_the_format_s_leader_and_suggested_a_commander() {
        assert_eq!(SwipeMode::Commanders(Door::Suggested).noun(), "commander");
        assert_eq!(
            SwipeMode::Commanders(Door::DeckCreation(Format::Commander)).noun(),
            SwipeMode::Commander(Format::Commander).noun()
        );
        assert_eq!(
            SwipeMode::Commanders(Door::DeckCreation(Format::Oathbreaker)).noun(),
            "oathbreaker"
        );
    }

    #[test]
    fn a_next_deal_leaves_out_the_pile_and_the_host_s_exclusions() {
        let mut stack: CardStack<SelectSwipe> = CardStack::default();
        let dealt = card();
        stack.replace(vec![dealt.clone()]);
        let host = Uuid::new_v4();
        let ids = dealt_ids(&stack, &HashSet::from([host]));
        assert!(ids.contains(&dealt.scryfall_data.oracle_id.unwrap()));
        assert!(ids.contains(&host));
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn undo_brings_a_card_back_from_the_side_it_left_by() {
        for (swiped, back_from) in [
            (SelectSwipe::Maybe(Uuid::nil()), Direction::Up),
            (SelectSwipe::UnsavedUp, Direction::Up),
            (SelectSwipe::Skip, Direction::Left),
            (SelectSwipe::Dismissed(Uuid::nil()), Direction::Left),
        ] {
            let mut stack: CardStack<SelectSwipe> = CardStack::default();
            stack.replace(vec![card(), card()]);
            stack.record(swiped);
            let _ = stack.advance();
            assert_eq!(undo_step(&mut stack), Some(swiped), "{swiped:?}");
            assert_eq!(stack.entering(), Some(back_from), "{swiped:?}");
            assert_eq!(stack.index(), 0, "{swiped:?}");
        }
    }

    #[test]
    fn v_reaches_the_screen_through_its_open_details_drawer() {
        use crate::inbound::cards::card_info::{DETAILS_DRAWER_CLASS, THROUGH_DRAWER_KEYS};
        assert_eq!(
            DETAILS_DRAWER_CLASS, "card-details-drawer",
            "the drawer's own class"
        );
        for key in THROUGH_DRAWER_KEYS {
            assert_eq!(
                shortcuts::swipe_screen_shortcut(key),
                Some(Shortcut::Flip),
                "{key} turns the card from the drawer"
            );
        }
        assert_eq!(
            SWIPE_KEYS_THROUGH,
            (DETAILS_DRAWER_CLASS, &THROUGH_DRAWER_KEYS),
            "the commander swipe hears v through its drawer, as deck Add does"
        );
    }

    /// The web's own sheet.
    const WEB_CSS: &str = include_str!("../../../../assets/web.css");

    #[test]
    fn a_skipped_or_picked_commander_flies_past_the_body_s_edge() {
        let selector = ".screen-content.card-swipe.web-swipe-select-body {";
        let at = WEB_CSS
            .find(selector)
            .expect("a rule for the wide swipe select body");
        let rule = WEB_CSS.get(at..).unwrap();
        let rule = rule.get(..rule.find('}').unwrap()).unwrap();
        assert!(rule.contains("overflow: visible"), "{rule}");
        let screen = WEB_CSS
            .find(".swipe-select-screen.web-swipe-select {")
            .and_then(|i| WEB_CSS.get(i..))
            .expect("a rule for the screen around it");
        let screen = screen.get(..screen.find('}').unwrap()).unwrap();
        // A scroll container clips on both axes, so the window still cuts a
        // card flying off sideways or up.
        assert!(
            screen.contains("overflow-x: hidden") && screen.contains("overflow-y: auto"),
            "the screen clips at the window and scrolls down to its footer: {screen}"
        );
    }
}
