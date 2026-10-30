//! Full-screen "Swipe select" screen: swipe to pick a command-zone card.
//!
//! Rendered as a sibling overlay above the create/edit form and toggled by
//! `open`: it stays mounted while closed so its filter and swipe position
//! persist, letting you commit a pick and reopen right where you left off. The
//! [`SwipeMode`] decides the always-on pool (commander, partner, background, or
//! signature spell); cards are served in **popularity order** through the
//! full [`CardFilterSheet`]. Swipe **left** to skip, **right** to choose,
//! **down** to undo; **up** saves to the commander maybeboard (commander and
//! partner modes only: the pile advances and selection continues).
//!
//! [`SwipeMode::Suggested`] deals a given ranking instead of a search: the
//! host's held suggestions, read through `suggested_pile` each time the overlay
//! opens. There a left swipe dismisses the commander for a week and the eyeball
//! sheet carries the reasons it was suggested. Its Filter opens a sheet holding
//! the host's `suggested_filter` chips; when they change the pile mid-session,
//! the stack re-deals through [`redeal`], keeping what was already swiped
//! behind the top card so undo still walks back through it.

use std::collections::HashSet;
use std::time::Duration;

use components::{
    ActionBar, BottomSheet, Button, ButtonVariant, HintBullet, HintBullets, HintDialog, HintKey,
    HintLine, TOAST_QUICK, use_overlay_back,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::{Color, Colors};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::domain::user::models::hints::HINT_SWIPE_SELECT;
use zwipe_core::http::contracts::suggestions::HttpCommanderSuggestion;

use crate::inbound::components::hint_dialog::{self, HintColored};
use crate::inbound::components::interactions::swipe::config::SwipeConfig;
use crate::inbound::components::interactions::swipe::direction::Direction;
use crate::inbound::components::interactions::swipe::{STACK_DEPTH, SwipeStack};
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::component;
use crate::inbound::screens::deck::card::components::card_info::{
    CardDetailsSheet, CardInfoDisplay, CardSkeleton, RulesButton, use_describe_tag,
};
use crate::inbound::screens::deck::card::components::card_row::OtagExamplesOpen;
use crate::inbound::screens::deck::card::components::printing_sheet::PrintingSheet;
use crate::inbound::screens::deck::card::filter::card_filter_sheet::CardFilterSheet;
use crate::inbound::screens::deck::suggested_commanders::{SuggestionReasons, empty_identity_line};

/// Cards fetched per page (matches the backend default).
const PAGE: u32 = 25;
/// Prefetch the next page when this many or fewer cards remain ahead of the top.
const LOAD_MORE_AT: usize = 5;

/// The ranking a [`SwipeMode::Suggested`] overlay deals, held by its host.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SuggestedPile {
    /// Best fit first, as the list shows them.
    pub suggestions: Vec<HttpCommanderSuggestion>,
    /// Decks the reasons count against.
    pub deck_count: u32,
    /// The color identity the host's Show chip narrows `suggestions` to;
    /// `None` is All.
    pub identity: Option<Vec<Color>>,
}

/// A suggested card's identity across printings: its oracle id, else its
/// scryfall id.
fn card_key(card: &Card) -> Uuid {
    card.scryfall_data
        .oracle_id
        .unwrap_or(card.scryfall_data.id)
}

/// The pile re-dealt from a new ranking mid-session. The first `swiped` of
/// `dealt` stay in front, so the top card's index and the undo history still
/// line up with them; then every item of `fresh` whose key is not among them,
/// in `fresh`'s order.
pub(crate) fn redeal<T: Clone, K: PartialEq>(
    dealt: &[T],
    swiped: usize,
    fresh: &[T],
    key: impl Fn(&T) -> K,
) -> Vec<T> {
    let mut pile: Vec<T> = dealt.iter().take(swiped).cloned().collect();
    let kept: Vec<K> = pile.iter().map(&key).collect();
    pile.extend(
        fresh
            .iter()
            .filter(|item| !kept.contains(&key(item)))
            .cloned(),
    );
    pile
}

/// Which command-zone slot a [`SwipeSelect`] is filling. Decides the always-on
/// filter constraint applied on top of whatever the user sets in the sheet.
#[derive(Clone, PartialEq)]
pub(crate) enum SwipeMode {
    /// Legal commanders for the deck's format.
    Commander(Format),
    /// Partner commanders.
    Partner,
    /// "Choose a Background" backgrounds.
    Background,
    /// Signature spells within the oathbreaker's color identity.
    SignatureSpell(Colors),
    /// The user's suggested commanders, a fixed ranking rather than a search.
    Suggested,
}

impl SwipeMode {
    /// The word used in the hint and toast ("...choose that <noun>"). The
    /// command-zone leader is an "oathbreaker" in Oathbreaker-style formats
    /// (those with a signature spell) and a "commander" everywhere else.
    fn noun(&self) -> &'static str {
        match self {
            Self::Commander(format) if format.has_signature_spell() => "oathbreaker",
            Self::Commander(_) | Self::Suggested => "commander",
            Self::Partner => "partner",
            Self::Background => "background",
            Self::SignatureSpell(_) => "signature spell",
        }
    }

    /// Applies this mode's always-on constraint to the builder. Mirrors the
    /// per-field typeahead filters in `deck_fields.rs`.
    fn apply(&self, builder: &mut CardQueryBuilder) {
        match self {
            Self::Commander(format) => {
                builder.set_is_commander_in_format(*format);
            }
            Self::Partner => {
                builder.set_is_partner(true);
            }
            Self::Background => {
                builder.set_is_background(true);
            }
            Self::SignatureSpell(colors) => {
                builder.set_is_signature_spell(true);
                builder.set_color_identity_within(colors.clone());
            }
            // The suggested pile never searches.
            Self::Suggested => {}
        }
    }
}

/// What the last pile-advancing swipe was, so a down-swipe undo can reverse
/// it faithfully: a skip just comes back; a maybeboard save also un-saves.
/// (Right-swipe commits and leaves the pile, so it never lands here.)
#[derive(Clone, Copy)]
enum SelectSwipe {
    /// Left swipe: nothing persisted.
    Skip,
    /// Up swipe: saved this oracle to the commander maybeboard.
    Maybe(Uuid),
    /// Left swipe in the suggested mode: dismissed this oracle for a week.
    Dismissed(Uuid),
}

/// Layers the mode constraint, no-tokens, and pagination onto a user-built
/// filter. No default sort is pinned: a sortless filter reaches the server as
/// sortless, so the deck-aware search serves its popularity-based, banded,
/// wildcard ordering (`context/archive/commander_select_ordering.md`). An explicit
/// user sort still flows through untouched.
fn mode_filter(mode: &SwipeMode, mut builder: CardQueryBuilder, offset: u32) -> CardQueryBuilder {
    mode.apply(&mut builder);
    builder.set_is_token(false);
    builder.set_limit(PAGE);
    builder.set_offset(offset);
    builder
}

/// In-place swipe picker for a command-zone slot. Toggled by `open`; `mode` is
/// the (reactive) slot to fill. `on_select` fires with the chosen card;
/// `on_close` cancels. Reads `Session` + `ZwipeClient` from context.
#[component]
pub(crate) fn SwipeSelect(
    open: Signal<bool>,
    mode: ReadSignal<Option<SwipeMode>>,
    on_select: EventHandler<Card>,
    on_close: EventHandler<()>,
    /// Host screen const for error-report breadcrumbs, passed through to the
    /// printing sheet (this overlay serves deck edit and deck create).
    host_screen: &'static str,
    /// Oracle ids kept out of the pile. The commander maybeboard passes its
    /// own saves here (its Swipe is discovery: a saved commander is a wasted
    /// deal); deck create/edit pass nothing, so saves still serve where the
    /// user goes to actually pick one.
    #[props(default)]
    exclude_oracle_ids: Option<ReadSignal<HashSet<Uuid>>>,
    /// The ranking the suggested mode deals, snapshotted on each open so the
    /// pile follows the host's own removes and filters. Without it the
    /// suggested pile is empty.
    #[props(default)]
    suggested_pile: Option<ReadSignal<SuggestedPile>>,
    /// The chips the suggested mode's Filter sheet holds, bound to the same
    /// state that narrows `suggested_pile`. Without it there is no Filter.
    #[props(default)]
    suggested_filter: Option<Element>,
) -> Element {
    // OS back gesture closes this overlay before touching the router.
    use_overlay_back(open);
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let toast = use_toast();

    // Local filter context (its own builder) so the shared CardFilterSheet
    // operates here without clobbering the app-wide add-screen filter. Starts
    // sortless: the server's popularity ordering is the default
    // (context/archive/commander_select_ordering.md). Persists across open/close
    // because this component stays mounted.
    let mut filter_builder = use_signal(CardQueryBuilder::new);
    use_context_provider(|| filter_builder);
    let mut filter_reset_counter = use_signal(|| 0u32);
    use_context_provider(|| filter_reset_counter);

    let mut cards = use_signal(Vec::<Card>::new);
    // Suggested mode only: the reasons behind each card in the pile, keyed by
    // scryfall id for the eyeball sheet, and the deck count they count against.
    let mut suggestions = use_signal(Vec::<HttpCommanderSuggestion>::new);
    let mut deck_count = use_signal(|| 0u32);
    let mut current_index = use_signal(|| 0usize);
    let mut offset = use_signal(|| 0u32);
    let mut exhausted = use_signal(|| false);
    // Counter value + mode that produced the cards currently in the stack, lets
    // us tell a reopen (keep state) from a real filter change or a mode change
    // (e.g. a new commander shifts the signature-spell color identity → refetch).
    let mut last_searched = use_signal(|| u32::MAX);
    let mut last_mode = use_signal(|| Option::<SwipeMode>::None);
    let mut is_loading_cards = use_signal(|| false);
    let mut is_loading_more = use_signal(|| false);
    let mut entering = use_signal(|| Option::<Direction>::None);
    // One entry per pile-advancing swipe (left/up), popped by the down-swipe
    // undo so it knows whether to also un-save a maybeboard entry.
    let mut swipe_history: Signal<Vec<SelectSwipe>> = use_signal(Vec::new);
    let mut filters_overlay_open = use_signal(|| false);
    // The suggested mode's Filter sheet, holding the host's chips.
    let mut suggested_filter_open = use_signal(|| false);
    // Swipe-vocabulary hint: the "?" button reopens it; it also auto-opens once
    // per account the first time the screen is opened (gated, since this stays
    // mounted while closed; a plain mount-time one-time hint would misfire).
    let hint_open = use_signal(|| false);
    let mut show_details = use_signal(|| false);
    let mut printing_open = use_signal(|| false);
    let mut hint_fired = use_signal(|| false);
    use_effect(move || {
        if open() && !*hint_fired.peek() {
            hint_fired.set(true);
            hint_dialog::open_and_record_hint(HINT_SWIPE_SELECT, session, client, hint_open);
        }
    });

    // The host's exclusion set, snapshotted at fetch time (peek: the pile
    // shouldn't reshuffle mid-session as saves change).
    let excluded_now = move || -> HashSet<Uuid> {
        exclude_oracle_ids
            .map(|ids| ids.peek().clone())
            .unwrap_or_default()
    };

    // Append the next page (left-swipe prefetch). Dedup and the host's
    // exclusions can filter a served page to nothing while the pool isn't
    // actually dry, so keep pulling (bounded; exclusions are capped small)
    // until cards land or the server truly runs out.
    let mut load_more = move || {
        if *is_loading_more.peek() || *exhausted.peek() {
            return;
        }
        let Some(mode) = mode.peek().clone() else {
            return;
        };
        if mode == SwipeMode::Suggested {
            return;
        }
        is_loading_more.set(true);
        spawn(async move {
            for _ in 0..4 {
                let off = *offset.peek();
                let builder = mode_filter(&mode, filter_builder.peek().clone(), off);
                let (Some(session), Ok(filter)) = (session.peek().clone(), builder.build()) else {
                    break;
                };
                match client().search_commanders(&filter, &session).await {
                    Ok(found) => {
                        let raw_empty = found.is_empty();
                        let seen: HashSet<Uuid> =
                            cards.peek().iter().map(|c| c.scryfall_data.id).collect();
                        let excluded = excluded_now();
                        let mut page: Vec<Card> = found
                            .into_iter()
                            .filter(|c| !seen.contains(&c.scryfall_data.id))
                            .filter(|c| {
                                !c.scryfall_data
                                    .oracle_id
                                    .is_some_and(|o| excluded.contains(&o))
                            })
                            .collect();
                        if raw_empty {
                            exhausted.set(true);
                            break;
                        }
                        offset.set(off + PAGE);
                        if !page.is_empty() {
                            cards.write().append(&mut page);
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("zwipe select load-more failed: {e}");
                        break;
                    }
                }
            }
            is_loading_more.set(false);
        });
    };

    // Fetch fresh results: on first open, and whenever the filter sheet changes
    // the filter. A reopen with an unchanged filter keeps the persisted stack.
    use_effect(move || {
        if !open() {
            return;
        }
        let counter = filter_reset_counter();
        let Some(current_mode) = mode.peek().clone() else {
            return;
        };
        let mode_changed = last_mode.peek().as_ref() != Some(&current_mode);
        let suggested = current_mode == SwipeMode::Suggested;
        // The suggested pile is read again on every open: the host's own
        // actions remove commanders between opens, so a kept pile goes stale.
        if !suggested
            && counter == *last_searched.peek()
            && !mode_changed
            && !cards.peek().is_empty()
        {
            return;
        }
        last_searched.set(counter);
        last_mode.set(Some(current_mode.clone()));
        cards.set(Vec::new());
        current_index.set(0);
        offset.set(0);
        exhausted.set(false);
        swipe_history.set(Vec::new());
        if suggested {
            let pile = suggested_pile
                .map(|pile| pile.peek().clone())
                .unwrap_or_default();
            exhausted.set(true);
            deck_count.set(pile.deck_count);
            cards.set(pile.suggestions.iter().map(|s| s.card.clone()).collect());
            suggestions.set(pile.suggestions);
            return;
        }
        let builder = mode_filter(&current_mode, filter_builder.peek().clone(), 0);
        let (Some(session), Ok(filter)) = (session.peek().clone(), builder.build()) else {
            return;
        };
        is_loading_cards.set(true);
        usage_buffer().record_search();
        spawn(async move {
            match client().search_commanders(&filter, &session).await {
                Ok(found) => {
                    let raw_empty = found.is_empty();
                    let excluded = excluded_now();
                    let page: Vec<Card> = found
                        .into_iter()
                        .filter(|c| {
                            !c.scryfall_data
                                .oracle_id
                                .is_some_and(|o| excluded.contains(&o))
                        })
                        .collect();
                    if raw_empty {
                        exhausted.set(true);
                    }
                    offset.set(PAGE);
                    let need_more = !raw_empty && page.is_empty();
                    cards.set(page);
                    is_loading_cards.set(false);
                    // The whole first page was excluded: pull ahead so the
                    // pile doesn't open onto an empty skeleton.
                    if need_more {
                        load_more();
                    }
                }
                Err(e) => {
                    tracing::warn!("zwipe select search failed: {e}");
                    is_loading_cards.set(false);
                }
            }
        });
    });

    // The host's Filter chips changed the suggested pile mid-session: re-deal
    // it under the cards already swiped, so the top card is the first of the
    // new pile not yet swiped and undo still walks back through the old ones.
    // A closed overlay waits for the open above, which deals afresh.
    use_effect(move || {
        let Some(pile) = suggested_pile else {
            return;
        };
        let pile = pile();
        if !*open.peek() || *mode.peek() != Some(SwipeMode::Suggested) {
            return;
        }
        let fresh: Vec<Card> = pile.suggestions.iter().map(|s| s.card.clone()).collect();
        let dealt = redeal(&cards.peek(), *current_index.peek(), &fresh, card_key);
        cards.set(dealt);
        // Keep the reasons of the cards behind the top for undo, and add the
        // new pile's.
        let mut reasons = suggestions.peek().clone();
        for suggestion in pile.suggestions {
            if !reasons
                .iter()
                .any(|s| s.card.scryfall_data.id == suggestion.card.scryfall_data.id)
            {
                reasons.push(suggestion);
            }
        }
        suggestions.set(reasons);
        deck_count.set(pile.deck_count);
    });

    // Left = skip: advance, prefetch when running low.
    let mut advance = move || {
        let total = cards.peek().len();
        let next = current_index() + 1;
        current_index.set(next);
        if !exhausted() && total.saturating_sub(next) <= LOAD_MORE_AT {
            load_more();
        }
    };

    // Down = undo the last swipe: bring the previous card back in, and if
    // that swipe was an up (maybeboard save), un-save it too; a suggested
    // mode dismissal is lifted the same way.
    let mut undo = move || {
        if current_index() == 0 {
            toast.info("Nothing to undo".to_string(), ToastOptions::default());
            return;
        }
        let last = swipe_history.write().pop();
        current_index.set(current_index() - 1);
        entering.set(Some(Direction::Left));
        match last {
            Some(SelectSwipe::Maybe(oracle_id)) => {
                spawn(async move {
                    let Some(session) = session.peek().clone() else {
                        return;
                    };
                    match client()
                        .remove_commander_maybeboard_card(oracle_id, &session)
                        .await
                    {
                        Ok(()) => {
                            toast.info(
                                "Removed from your commander maybeboard".to_string(),
                                ToastOptions::default().duration(TOAST_QUICK),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo commander maybeboard save failed: {e}");
                            usage_buffer.peek().report_error(
                                host_screen,
                                component::SWIPE_SELECT,
                                "undo_commander_maybeboard",
                                &e,
                            );
                            toast.error(e.to_user_message(), ToastOptions::default());
                        }
                    }
                });
            }
            Some(SelectSwipe::Dismissed(oracle_id)) => {
                // Quiet like the dismissal itself: the card is already back.
                spawn(async move {
                    let Some(session) = session.peek().clone() else {
                        return;
                    };
                    if let Err(e) = client().undismiss_commander(oracle_id, &session).await {
                        tracing::warn!("undo commander dismissal failed: {e}");
                        usage_buffer.peek().report_error(
                            host_screen,
                            component::SWIPE_SELECT,
                            "undismiss_commander",
                            &e,
                        );
                    }
                });
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
            _ => {
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        }
    };

    // Stage every user-set filter back to the default (all colors, the
    // backend's popularity order); the sheet's Apply commits and re-searches.
    // The mode constraint is re-applied on the next fetch.
    let mut clear_filter = move || {
        filter_builder.write().clear();
    };

    // Re-run the search with the current filter, back to the top of the stack.
    let mut refresh = move || {
        filter_reset_counter.set(filter_reset_counter() + 1);
        toast.info(
            "Stack refreshed".to_string(),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    };

    // The screen stays mounted and fades via CSS (`.show`). The inner content is
    // unmounted whenever closed so a just-committed card can't snap back to
    // center mid-fade; the solid screen simply fades out, easing the form in.
    let screen_class = if open() {
        "screen swipe-select-screen show"
    } else {
        "screen swipe-select-screen"
    };

    let (window, current_card, has_cards) = if open() {
        let idx = current_index();
        let w: Vec<Card> = cards().into_iter().skip(idx).take(STACK_DEPTH).collect();
        let cc = cards().get(idx).cloned();
        let hc = !w.is_empty();
        (w, cc, hc)
    } else {
        (Vec::new(), None, false)
    };
    let noun = mode().as_ref().map_or("card", SwipeMode::noun);
    let suggested = mode() == Some(SwipeMode::Suggested);
    // Up-swipe = "maybe this commander" (per-user save), so it's only live in
    // the modes whose pool is commanders. Background/signature-spell picks
    // have no maybeboard home.
    let maybe_enabled = matches!(
        mode().as_ref(),
        Some(SwipeMode::Commander(_) | SwipeMode::Partner | SwipeMode::Suggested)
    );
    // The reasons behind the card on top, for the details sheet's footer. Its
    // shared-tag chips take the sheet's own description lookup and the host's
    // examples opener as props: the suggested commanders screen provides the
    // opener, and its examples overlay draws over this screen and the sheet.
    let describe_tag = use_describe_tag();
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let current_reasons = current_card.as_ref().and_then(|card| {
        suggestions
            .read()
            .iter()
            .find(|s| s.card.scryfall_data.id == card.scryfall_data.id)
            .map(|s| s.reasons.clone())
    });
    let title = if suggested {
        "Suggested commanders"
    } else {
        "Swipe select"
    };
    // The suggested Filter: the chosen identity lights the button's dot, and
    // one that matches nothing says so in the screen's own words.
    let has_suggested_filter = suggested && suggested_filter.is_some();
    let suggested_identity = suggested_pile.and_then(|pile| pile.read().identity.clone());
    let suggested_empty_line = suggested_pile
        .filter(|pile| suggested && pile.read().suggestions.is_empty())
        .and(suggested_identity.as_deref())
        .map(empty_identity_line);
    let mut directions = vec![Direction::Left, Direction::Right, Direction::Down];
    if maybe_enabled {
        directions.push(Direction::Up);
    }
    let swipe_config = SwipeConfig::new(directions, 90.0, 2.5);

    rsx! {
        div { class: "{screen_class}",
            // A screen over another paints its own backdrop, since .screen is
            // clear and the one beneath would show through.
            ::components::GridCells { still: true }
            if open() {
                ScreenHeader { title, hint: hint_open }

                div { class: "screen-content card-swipe content-enter",
                    if has_cards {
                        SwipeStack {
                            cards: window,
                            config: swipe_config,
                            entering,
                            on_swipe_left: move |card: Card| {
                                usage_buffer().record_swipe(Direction::Left);
                                usage_buffer()
                                    .record_select_signal(card.scryfall_data.oracle_id, Direction::Left);
                                advance();
                                // Suggested mode: a left swipe hides the
                                // commander for a week. Quiet like a deck
                                // skip: the pile already moved on, and a
                                // failed post only means it comes back sooner.
                                if let (true, Some(oracle_id)) = (suggested, card.scryfall_data.oracle_id) {
                                    swipe_history.write().push(SelectSwipe::Dismissed(oracle_id));
                                    if let Some(session) = session.peek().clone() {
                                        spawn(async move {
                                            if let Err(e) = client().dismiss_commander(oracle_id, &session).await {
                                                tracing::warn!("dismiss commander failed: {e}");
                                            }
                                        });
                                    }
                                    toast.info(
                                        "Skipped".to_string(),
                                        ToastOptions::default().duration(TOAST_QUICK),
                                    );
                                    return;
                                }
                                swipe_history.write().push(SelectSwipe::Skip);
                                toast.info(
                                    "Skipped".to_string(),
                                    ToastOptions::default().duration(TOAST_QUICK),
                                );
                            },
                            on_swipe_right: move |card: Card| {
                                usage_buffer().record_swipe(Direction::Right);
                                usage_buffer()
                                    .record_select_signal(card.scryfall_data.oracle_id, Direction::Right);
                                let mut label = noun.to_string();
                                if let Some(first) = label.get_mut(..1) {
                                    first.make_ascii_uppercase();
                                }
                                toast.success(
                                    format!("{label} selected"),
                                    ToastOptions::default().duration(TOAST_QUICK),
                                );
                                on_select.call(card);
                            },
                            on_swipe_up: move |card: Card| {
                                // Save is non-committal: the pile advances and
                                // selection continues, exactly like a skip.
                                usage_buffer().record_swipe(Direction::Up);
                                advance();
                                let Some(oracle_id) = card.scryfall_data.oracle_id else {
                                    // Nothing was saved, so undo treats it
                                    // like a plain skip.
                                    swipe_history.write().push(SelectSwipe::Skip);
                                    toast.error(
                                        "This card can't be saved".to_string(),
                                        ToastOptions::default(),
                                    );
                                    return;
                                };
                                swipe_history.write().push(SelectSwipe::Maybe(oracle_id));
                                let Some(session) = session.peek().clone() else {
                                    return;
                                };
                                spawn(async move {
                                    match client()
                                        .add_commander_maybeboard_card(oracle_id, &session)
                                        .await
                                    {
                                        Ok(()) => {
                                            toast.success(
                                                "Added to your commander maybeboard".to_string(),
                                                ToastOptions::default()
                                                    .duration(Duration::from_millis(1500)),
                                            );
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "add to commander maybeboard failed: {e}"
                                            );
                                            usage_buffer.peek().report_error(
                                                host_screen,
                                                component::SWIPE_SELECT,
                                                "add_commander_maybeboard",
                                                &e,
                                            );
                                            toast.error(
                                                e.to_user_message(),
                                                ToastOptions::default(),
                                            );
                                        }
                                    }
                                });
                            },
                            on_swipe_down: move |_card: Card| {
                                usage_buffer().record_swipe(Direction::Down);
                                undo();
                            },
                        }
                        if let Some(card) = current_card.clone() {
                            CardInfoDisplay { card }
                        }
                    } else if is_loading_cards() || is_loading_more() {
                        CardSkeleton { is_loading: true }
                    } else if let Some(line) = suggested_empty_line {
                        div { class: "message-empty",
                            span { class: "chip-note", "{line}" }
                        }
                    } else {
                        CardSkeleton {}
                    }
                }

                ActionBar {
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| on_close.call(()),
                        "Back"
                    }
                    // The suggested ranking filters by the host's Show chips
                    // alone; a chosen identity lights the dot.
                    if has_suggested_filter {
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| suggested_filter_open.set(true),
                            "Filter"
                            if suggested_identity.is_some() {
                                span { class: "filter-dot" }
                            }
                        }
                    } else if !suggested {
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| filters_overlay_open.set(true),
                            "Filter"
                            // Popularity order is this screen's default (no sort),
                            // so any user filter or any explicit sort lights the dot.
                            if !filter_builder.read().is_empty() || filter_builder.read().sort().is_some() {
                                span { class: "filter-dot" }
                            }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| refresh(),
                        "Refresh"
                    }
                    if has_cards {
                        RulesButton { open: show_details }
                    }
                }

                // Root-mounted (siblings of the ActionBar) so the bottom sheets
                // cover the footer, the printing sheet after the details sheet
                // so it draws over it.
                if let Some(card) = current_card {
                    CardDetailsSheet {
                        open: show_details,
                        card,
                        on_printings: move |()| printing_open.set(true),
                        footer: current_reasons.map(|reasons| rsx! {
                            SuggestionReasons {
                                reasons,
                                deck_count: deck_count(),
                                describe_tag,
                                on_examples,
                            }
                        }),
                    }
                }

                // Pick a printing → swap the focused card and drop both sheets;
                // swipe-right selects that printing (persisted as commander_id).
                if let Some(card) = cards().get(current_index()).cloned() {
                    PrintingSheet {
                        card,
                        open: printing_open,
                        host_screen,
                        on_save: move |new_card: Card| {
                            let idx = current_index();
                            if let Some(slot) = cards.write().get_mut(idx) {
                                *slot = new_card;
                            }
                            show_details.set(false);
                        },
                    }
                }

                CardFilterSheet {
                    open: filters_overlay_open,
                    show_format_filter: false,
                    show_active_indicators: true,
                    on_clear: move |()| clear_filter(),
                }

                // A chip applies as it is tapped and the stack re-deals behind
                // the sheet, so Back is its one button.
                if let (true, Some(chips)) = (suggested, suggested_filter) {
                    BottomSheet {
                        open: suggested_filter_open,
                        title: "Filter",
                        footer: rsx! {
                            Button {
                                variant: ButtonVariant::Action,
                                onclick: move |_| suggested_filter_open.set(false),
                                "Back"
                            }
                        },
                        {chips}
                    }
                }

                HintDialog {
                    open: hint_open,
                    title,
                    HintBullets {
                        if suggested {
                            HintBullet { "Swipe " HintColored { color: "--color-success", "right" } " starts a deck with that commander" }
                            HintBullet { "Swipe " HintColored { color: "--color-error", "left" } " skips it for a week" }
                        } else {
                            HintBullet { "Swipe " HintColored { color: "--color-success", "right" } " picks that {noun}" }
                            HintBullet { "Swipe " HintColored { color: "--color-error", "left" } " skips it" }
                        }
                        HintBullet { "Swipe " HintColored { color: "--color-warning", "up" } " saves it to your commander maybeboard" }
                        HintBullet { "Swipe " HintColored { color: "--accent-tertiary", "down" } " undoes the last swipe" }
                    }
                    if suggested {
                        HintLine { "Best fit first, 100 deep. The eye shows why each one was suggested" }
                        if has_suggested_filter {
                            HintLine { HintKey { "Filter" } " narrows the stack to one color identity" }
                        }
                    } else {
                        HintLine { "Most-played first. " HintKey { "Filter" } " narrows by color or anything else" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(name: &&'static str) -> &'static str {
        name
    }

    #[test]
    fn redeal_keeps_the_swiped_in_front_then_the_new_pile_without_them() {
        let dealt = ["Krenko", "Narset", "Zada", "Ghired"];
        let fresh = ["Zada", "Krenko", "Talrand"];
        assert_eq!(
            redeal(&dealt, 2, &fresh, key),
            vec!["Krenko", "Narset", "Zada", "Talrand"],
            "Krenko was swiped, so the new pile deals Zada then Talrand after the two"
        );
    }

    #[test]
    fn redeal_with_nothing_swiped_starts_at_the_top_of_the_new_pile() {
        let dealt = ["Krenko", "Narset"];
        let fresh = ["Talrand", "Narset"];
        assert_eq!(redeal(&dealt, 0, &fresh, key), vec!["Talrand", "Narset"]);
    }

    #[test]
    fn redeal_to_an_empty_pile_keeps_only_the_swiped() {
        let dealt = ["Krenko", "Narset", "Zada"];
        assert_eq!(redeal(&dealt, 1, &[], key), vec!["Krenko"]);
        assert_eq!(
            redeal(&dealt, 9, &["Talrand"], key),
            vec!["Krenko", "Narset", "Zada", "Talrand"],
            "a swiped count past the pile keeps the whole pile"
        );
    }

    #[test]
    fn redeal_after_an_undo_can_deal_the_undone_card_again() {
        // Three swiped and one undone puts the index back at 2, so the third
        // is no longer swiped and the new pile deals it.
        let dealt = ["Krenko", "Narset", "Zada"];
        let fresh = ["Zada", "Talrand"];
        assert_eq!(
            redeal(&dealt, 2, &fresh, key),
            vec!["Krenko", "Narset", "Zada", "Talrand"]
        );
    }
}
