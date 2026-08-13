//! Full-screen "Swipe select" screen — swipe to pick a command-zone card.
//!
//! Rendered as a sibling overlay above the create/edit form and toggled by
//! `open` — it stays mounted while closed so its filter and swipe position
//! persist, letting you commit a pick and reopen right where you left off. The
//! [`SwipeMode`] decides the always-on pool (commander, partner, background, or
//! signature spell); cards are served in **EDHREC-popularity order** through the
//! full [`CardFilterSheet`]. Swipe **left** to skip, **right** to choose,
//! **down** to undo; **up** saves to the commander maybeboard (commander and
//! partner modes only — the pile advances and selection continues).

use crate::{
    inbound::{
        components::{
            hint_dialog::{
                HintBullet, HintBullets, HintColored, HintDialog, HintKey, HintLine,
                open_and_record_hint,
            },
            interactions::swipe::{
                STACK_DEPTH, SwipeStack, config::SwipeConfig, direction::Direction,
            },
            navigation::overlay_stack::use_overlay_back,
            screen_header::ScreenHeader,
            telemetry::{usage_buffer::UsageBuffer, vocabulary::component},
        },
        screens::deck::card::{
            components::{
                card_info::{CardDetailsDialog, CardInfoDisplay, CardSkeleton, RulesButton},
                printing_sheet::PrintingSheet,
            },
            filter::card_filter_sheet::CardFilterSheet,
        },
    },
    outbound::client::{
        ZwipeClient, card::search_commanders::ClientSearchCommanders,
        user::commander_maybeboard::ClientCommanderMaybeboard,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::{collections::HashSet, time::Duration};
use uuid::Uuid;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::domain::{
    auth::models::session::Session,
    card::{
        Card, scryfall_data::colors::Colors, search_card::card_filter::builder::CardQueryBuilder,
    },
    deck::format::Format,
    user::models::hints::HINT_SWIPE_SELECT,
};

/// Cards fetched per page (matches the backend default).
const PAGE: u32 = 25;
/// Prefetch the next page when this many or fewer cards remain ahead of the top.
const LOAD_MORE_AT: usize = 5;

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
}

impl SwipeMode {
    /// The word used in the hint and toast ("...choose that <noun>"). The
    /// command-zone leader is an "oathbreaker" in Oathbreaker-style formats
    /// (those with a signature spell) and a "commander" everywhere else.
    fn noun(&self) -> &'static str {
        match self {
            Self::Commander(format) if format.has_signature_spell() => "oathbreaker",
            Self::Commander(_) => "commander",
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
        }
    }
}

/// What the last pile-advancing swipe was, so a down-swipe undo can reverse
/// it faithfully: a skip just comes back; a maybeboard save also un-saves.
/// (Right-swipe commits and leaves the pile, so it never lands here.)
#[derive(Clone, Copy)]
enum SelectSwipe {
    /// Left swipe — nothing persisted.
    Skip,
    /// Up swipe — saved this oracle to the commander maybeboard.
    Maybe(Uuid),
}

/// Layers the mode constraint, no-tokens, and pagination onto a user-built
/// filter. No default sort is pinned: a sortless filter reaches the server as
/// sortless, so the deck-aware search serves its popularity-based, banded,
/// wildcard ordering (context/archive/commander_select_ordering.md). An explicit
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
    /// own saves here (its Swipe is discovery — a saved commander is a wasted
    /// deal); deck create/edit pass nothing, so saves still serve where the
    /// user goes to actually pick one.
    #[props(default)]
    exclude_oracle_ids: Option<ReadSignal<HashSet<Uuid>>>,
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
    let mut current_index = use_signal(|| 0usize);
    let mut offset = use_signal(|| 0u32);
    let mut exhausted = use_signal(|| false);
    // Counter value + mode that produced the cards currently in the stack — lets
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
    // Swipe-vocabulary hint: the "?" button reopens it; it also auto-opens once
    // per account the first time the screen is opened (gated, since this stays
    // mounted while closed — a plain mount-time one-time hint would misfire).
    let hint_open = use_signal(|| false);
    let show_details = use_signal(|| false);
    let mut printing_open = use_signal(|| false);
    let mut hint_fired = use_signal(|| false);
    use_effect(move || {
        if open() && !*hint_fired.peek() {
            hint_fired.set(true);
            open_and_record_hint(HINT_SWIPE_SELECT, session, client, hint_open);
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
    // actually dry, so keep pulling (bounded — exclusions are capped small)
    // until cards land or the server truly runs out.
    let mut load_more = move || {
        if *is_loading_more.peek() || *exhausted.peek() {
            return;
        }
        let Some(mode) = mode.peek().clone() else {
            return;
        };
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
        if counter == *last_searched.peek() && !mode_changed && !cards.peek().is_empty() {
            return;
        }
        last_searched.set(counter);
        last_mode.set(Some(current_mode.clone()));
        cards.set(Vec::new());
        current_index.set(0);
        offset.set(0);
        exhausted.set(false);
        swipe_history.set(Vec::new());
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
                    // The whole first page was excluded — pull ahead so the
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
    // that swipe was an up (maybeboard save), un-save it too.
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
                                ToastOptions::default().duration(Duration::from_millis(1500)),
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
            _ => {
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
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
            ToastOptions::default().duration(Duration::from_millis(1500)),
        );
    };

    // The screen stays mounted and fades via CSS (`.show`). The inner content is
    // unmounted whenever closed so a just-committed card can't snap back to
    // center mid-fade — the solid screen simply fades out, easing the form in.
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
    let noun = mode().as_ref().map(SwipeMode::noun).unwrap_or("card");
    // Up-swipe = "maybe this commander" (per-user save), so it's only live in
    // the modes whose pool is commanders. Background/signature-spell picks
    // have no maybeboard home.
    let maybe_enabled = matches!(
        mode().as_ref(),
        Some(SwipeMode::Commander(_) | SwipeMode::Partner)
    );
    let mut directions = vec![Direction::Left, Direction::Right, Direction::Down];
    if maybe_enabled {
        directions.push(Direction::Up);
    }
    let swipe_config = SwipeConfig::new(directions, 90.0, 2.5);

    rsx! {
        div { class: "{screen_class}",
            if open() {
                ScreenHeader { title: "Swipe select", hint: hint_open }

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
                                swipe_history.write().push(SelectSwipe::Skip);
                                advance();
                                toast.info(
                                    "Skipped".to_string(),
                                    ToastOptions::default().duration(Duration::from_millis(1500)),
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
                                    ToastOptions::default().duration(Duration::from_millis(1500)),
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
                        if let Some(card) = current_card {
                            CardInfoDisplay { card: card.clone() }
                            CardDetailsDialog {
                                open: show_details,
                                card,
                                on_printings: move |_| printing_open.set(true),
                            }
                        }
                    } else if is_loading_cards() || is_loading_more() {
                        CardSkeleton { is_loading: true }
                    } else {
                        CardSkeleton {}
                    }
                }

                ActionBar {
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| on_close.call(()),
                        "Back"
                    }
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| filters_overlay_open.set(true),
                        "Filter"
                        // Popularity order is this screen's default (no sort),
                        // so any user filter or any explicit sort lights the dot.
                        if !filter_builder.read().is_empty() || filter_builder.read().sort().is_some() {
                            span { class: "filter-dot" }
                        }
                    }
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| refresh(),
                        "Refresh"
                    }
                    if has_cards {
                        RulesButton { open: show_details }
                    }
                }

                // Root-mounted (sibling of the ActionBar) so the bottom sheet
                // covers the footer. Pick a printing → swap the focused card;
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
                        },
                    }
                }

                CardFilterSheet {
                    open: filters_overlay_open,
                    show_format_filter: false,
                    show_active_indicators: true,
                    on_clear: move |_| clear_filter(),
                }

                HintDialog {
                    open: hint_open,
                    title: "Swipe select",
                    HintBullets {
                        HintBullet {
                            "Swipe "
                            HintColored { color: "--color-success", "right" }
                            " to choose that {noun}."
                        }
                        HintBullet {
                            "Swipe "
                            HintColored { color: "--color-error", "left" }
                            " to skip it."
                        }
                        if maybe_enabled {
                            HintBullet {
                                "Swipe "
                                HintColored { color: "--color-warning", "up" }
                                " to save it to your commander maybeboard."
                            }
                        }
                        HintBullet {
                            "Swipe "
                            HintColored { color: "--accent-tertiary", "down" }
                            " to undo your last swipe."
                        }
                    }
                    HintLine {
                        "Most-played cards come first. Tap "
                        HintKey { color: "--accent-secondary", "Filter" }
                        " to narrow by color or anything else."
                    }
                }
            }
        }
    }
}
