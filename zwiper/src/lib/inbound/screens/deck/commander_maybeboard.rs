//! Commander maybeboard screen — the per-user "maybe this commander" list,
//! and a direct commander-discovery surface.
//!
//! Entries arrive three ways: up-swiping during any commander Zwipe-select,
//! up-swiping in this screen's own Swipe overlay, or tapping a floating
//! result chip under the Quick add input (a debounced commander-name search,
//! hung downward since this console sits at the top of the page). The input
//! is pure quick add — narrowing the saved list belongs to the five color
//! pips alone (contains-all identity semantics, like the deck list's Show
//! row); both console rows render statically per the closed-vocabulary
//! skeleton rule.
//!
//! Right-swiping in the Swipe overlay, or an expanded row's **Create deck**,
//! seeds the create screen with that commander (capacity-gated like the deck
//! list's Create). **Remove** un-maybes an entry in place.

use super::components::swipe_select::{SwipeMode, SwipeSelect};
use crate::{
    inbound::{
        components::{
            alert_dialog::{
                AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
                AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
            },
            auth::ensure_session::EnsureFresh,
            bottom_sheet::BottomSheet,
            catalog_cache::CatalogCache,
            hint_dialog::{
                HintBullet, HintBullets, HintDialog, HintKey, HintLine, use_one_time_hint,
            },
            screen_header::ScreenHeader,
            telemetry::{
                usage_buffer::UsageBuffer,
                vocabulary::{component, screen},
            },
        },
        router::Router,
        screens::{
            deck::{
                card::{
                    components::{
                        card_row::{CardRow, OtagDescribe, OtagExamplesOpen, ShowRowArt},
                        image_preview::ImagePreview,
                        printing_sheet::PrintingSheet,
                    },
                    filter::card_filter_sheet::CardFilterSheet,
                },
                create::CreateDeckCommanderSeed,
            },
            oracle_tag_examples::OracleTagExamples,
        },
    },
    outbound::client::{
        ClientError, ZwipeClient, card::search_commanders::ClientSearchCommanders,
        deck::get_deck_profiles::ClientGetDeckList,
        user::commander_maybeboard::ClientCommanderMaybeboard,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::{collections::HashSet, time::Duration};
use tokio::time::sleep;
use uuid::Uuid;
use zwipe_components::{ActionBar, Button, ButtonVariant, Chip};
use zwipe_core::domain::{
    auth::models::session::Session,
    card::{
        Card,
        scryfall_data::{ScryfallData, colors::Color},
        search_card::{
            card_filter::{builder::CardQueryBuilder, card_sort_key::CardSortKey},
            cards::Cards,
        },
    },
    deck::{deck_profile::DeckProfile, format::Format},
    user::models::hints::HINT_COMMANDER_MAYBEBOARD,
};

/// Minimum characters before the commander search fires.
const MIN_QUERY_LEN: usize = 2;
/// Debounce so results follow typing without a request per keystroke.
const DEBOUNCE_MS: u64 = 300;
/// Result chips shown under the search bar.
const RESULT_LIMIT: u32 = 8;

/// One maybeboard entry: the shared expandable card row, with this screen's
/// Create deck / Remove pair riding the row's own action bar (`extra_actions`)
/// next to the shared Printing button.
#[component]
fn MaybeboardRow(
    card: Card,
    expanded_card: Signal<Option<Uuid>>,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
    on_create: EventHandler<Card>,
    on_remove: EventHandler<Uuid>,
    on_printing: EventHandler<Card>,
) -> Element {
    let oracle_id = card.scryfall_data.oracle_id;
    let card_for_create = card.clone();
    rsx! {
        CardRow {
            card,
            qty: 1,
            expanded_card,
            preview_card,
            preview_dismissing,
            on_printing: move |c: Card| on_printing.call(c),
            extra_actions: rsx! {
                button {
                    class: "card-action-btn",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_create.call(card_for_create.clone());
                    },
                    "Create deck"
                }
                if let Some(oid) = oracle_id {
                    button {
                        class: "card-action-btn",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_remove.call(oid);
                        },
                        "Remove"
                    }
                }
            },
        }
    }
}

/// Screen listing the user's saved "maybe" commanders with search-to-add,
/// an in-place commander Swipe overlay, and per-entry create-deck / remove.
#[component]
pub fn CommanderMaybeboard() -> Element {
    let navigator = use_navigator();
    let auth_client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let hint_open = use_one_time_hint(HINT_COMMANDER_MAYBEBOARD);
    let seed: CreateDeckCommanderSeed = use_context();
    // Art-crop thumbnails on the rows, on by default; the "Art" chip in the
    // Show row toggles it, easing the thumbs in and out (the shared row keeps
    // them mounted, same as the deck cards screen). Screen-local.
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));

    // Oracle-tag reveal inside expanded rows — the same wiring as the deck
    // cards screen: a description lookup from the shared catalog cache and an
    // examples-browse opener, handed to every CardRow via context. Without
    // these the rows keep plain, non-expandable otag chips.
    let cache: CatalogCache = use_context();
    use_effect(move || {
        cache.ensure_oracle_tags(auth_client);
    });
    let describe_tag = use_callback(move |slug: String| {
        cache.oracle_tags.cell().read().loaded().and_then(|tags| {
            tags.iter()
                .find(|t| t.slug == slug)
                .and_then(|t| t.description.clone())
        })
    });
    let mut otag_examples_open = use_signal(|| false);
    let mut otag_examples_slug = use_signal(String::new);
    let open_examples = use_callback(move |slug: String| {
        otag_examples_slug.set(slug);
        otag_examples_open.set(true);
    });
    use_context_provider(|| OtagDescribe(describe_tag));
    use_context_provider(|| OtagExamplesOpen(open_examples));

    let mut query = use_signal(String::new);
    let mut selected_colors: Signal<HashSet<Color>> = use_signal(HashSet::new);
    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    let preview_card: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    let preview_dismissing = use_signal(|| false);
    let mut swipe_open = use_signal(|| false);
    let swipe_mode = use_memo(|| Some(SwipeMode::Commander(Format::Commander)));
    // Printing sheet target: the row whose Printing button was tapped. The
    // swap is display-local (the maybeboard is oracle-keyed; a refetch goes
    // back to the preferred printing), but a Create deck after swapping
    // carries the chosen printing into the new deck's commander slot.
    let mut printing_card: Signal<Option<Card>> = use_signal(|| None);
    let mut printing_open = use_signal(|| false);

    // Local filter context for the shared CardFilterSheet — blank by default
    // and Reset returns to blank; no format filter (the list is commanders
    // only). Same wiring as the Swipe select overlay's local sheet.
    let mut filter_builder = use_signal(CardQueryBuilder::new);
    use_context_provider(|| filter_builder);
    let filter_reset_counter = use_signal(|| 0u32);
    use_context_provider(|| filter_reset_counter);
    let mut filters_overlay_open = use_signal(|| false);
    let mut show_more_sheet = use_signal(|| false);
    let mut show_clear_dialog = use_signal(|| false);

    // Entries live in a plain signal (not a Resource) so Remove and the
    // search-to-add chips can edit the list in place. `reload` refetches:
    // bumped when the Swipe overlay closes, since up-swipes in there add
    // entries server-side. A refetch with entries present is silent (no
    // skeleton flash).
    let mut entries: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut is_loading = use_signal(|| true);
    let mut load_failed = use_signal(|| false);
    let mut reload = use_signal(|| 0u32);
    use_effect(move || {
        let _tick = reload();
        spawn(async move {
            let session = match session.ensure_fresh(auth_client).await {
                Ok(session) => session,
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::COMMANDER_MAYBEBOARD,
                        component::NONE,
                        "load_maybeboard",
                        &e,
                    );
                    load_failed.set(entries.peek().is_empty());
                    is_loading.set(false);
                    return;
                }
            };
            match auth_client().get_commander_maybeboard(&session).await {
                Ok(cards) => {
                    entries.set(cards);
                    load_failed.set(false);
                    is_loading.set(false);
                }
                Err(e) => {
                    tracing::warn!("commander maybeboard load failed: {e}");
                    usage_buffer.peek().report_error(
                        screen::COMMANDER_MAYBEBOARD,
                        component::NONE,
                        "load_maybeboard",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    load_failed.set(entries.peek().is_empty());
                    is_loading.set(false);
                }
            }
        });
    });

    // Refetch when the Swipe overlay closes — its up-swipes saved entries.
    let mut swipe_was_open = use_signal(|| false);
    use_effect(move || {
        if swipe_open() {
            swipe_was_open.set(true);
        } else if *swipe_was_open.peek() {
            swipe_was_open.set(false);
            let tick = *reload.peek();
            reload.set(tick + 1);
        }
    });

    // Debounced commander-name search feeding the result chips under the bar.
    // Mirrors quick add: sleep, bail if the query moved on, explicit name sort
    // so the server's popularity ordering doesn't apply to a typed name.
    let mut search_results: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut is_searching = use_signal(|| false);
    use_effect(move || {
        let q = query();
        if q.trim().len() < MIN_QUERY_LEN {
            is_searching.set(false);
            search_results.set(Vec::new());
            return;
        }
        is_searching.set(true);
        spawn(async move {
            sleep(Duration::from_millis(DEBOUNCE_MS)).await;
            if query() != q {
                return;
            }
            let session = match session.ensure_fresh(auth_client).await {
                Ok(session) => session,
                Err(_) => {
                    is_searching.set(false);
                    return;
                }
            };
            let mut builder = CardQueryBuilder::with_name_contains(q.trim());
            builder.set_is_commander_in_format(Format::Commander);
            builder.set_is_token(false);
            builder.set_sort(CardSortKey::Name);
            builder.set_limit(RESULT_LIMIT);
            let Ok(filter) = builder.build() else {
                is_searching.set(false);
                return;
            };
            usage_buffer().record_search();
            match auth_client().search_commanders(&filter, &session).await {
                Ok(found) => {
                    search_results.set(found);
                    is_searching.set(false);
                }
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::COMMANDER_MAYBEBOARD,
                        component::NONE,
                        "maybeboard_search",
                        &e,
                    );
                    is_searching.set(false);
                }
            }
        });
    });

    // Tap a result chip → save it. Dropped from the chips at once (no double
    // add, instant feel), prepended to the list (newest save first) when the
    // server confirms.
    let mut add_from_search = move |card: Card| {
        let Some(oracle_id) = card.scryfall_data.oracle_id else {
            return;
        };
        search_results
            .write()
            .retain(|c| c.scryfall_data.oracle_id != Some(oracle_id));
        spawn(async move {
            let session = match session.ensure_fresh(auth_client).await {
                Ok(session) => session,
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::COMMANDER_MAYBEBOARD,
                        component::NONE,
                        "add_commander_maybeboard",
                        &e,
                    );
                    toast.error(e.to_user_message(), ToastOptions::default());
                    return;
                }
            };
            match auth_client()
                .add_commander_maybeboard_card(oracle_id, &session)
                .await
            {
                Ok(()) => {
                    let name = card.scryfall_data.name.clone();
                    let already_saved = entries
                        .peek()
                        .iter()
                        .any(|c| c.scryfall_data.oracle_id == Some(oracle_id));
                    if !already_saved {
                        entries.write().insert(0, card);
                    }
                    // Belt and braces: a silent refetch reconciles to server
                    // truth (order + preferred printing) behind the
                    // optimistic insert.
                    let tick = *reload.peek();
                    reload.set(tick + 1);
                    toast.info(
                        format!("Added {name}"),
                        ToastOptions::default().duration(Duration::from_millis(1500)),
                    );
                }
                Err(e) => {
                    tracing::warn!("commander maybeboard add failed: {e}");
                    usage_buffer.peek().report_error(
                        screen::COMMANDER_MAYBEBOARD,
                        component::NONE,
                        "add_commander_maybeboard",
                        &e,
                    );
                    toast.error(e.to_user_message(), ToastOptions::default());
                }
            }
        });
    };

    // Deck profiles back the same proactive create gate as the deck list's
    // Create button: unverified accounts are limited to 1 deck.
    let deck_profiles_resource: Resource<Result<Vec<DeckProfile>, ClientError>> =
        use_resource(move || async move {
            let session = session.ensure_fresh(auth_client).await?;

            auth_client().get_deck_profiles(&session).await
        });

    let at_deck_limit = move || {
        session().is_some_and(|s| {
            s.user.email_verified_at.is_none()
                && deck_profiles_resource
                    .read()
                    .as_ref()
                    .and_then(|r| r.as_ref().ok())
                    .is_some_and(|p| !p.is_empty())
        })
    };

    let create_with = move |card: Card| {
        if at_deck_limit() {
            toast.warning(
                "Verify your email to create more than 1 deck".to_string(),
                ToastOptions::default().duration(Duration::from_millis(4000)),
            );
        } else {
            let mut slot = seed.0;
            slot.set(Some(card));
            navigator.push(Router::CreateDeck);
        }
    };

    let on_create = move |card: Card| create_with(card);

    let clear_maybeboard = move || {
        spawn(async move {
            let session = match session.ensure_fresh(auth_client).await {
                Ok(session) => session,
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::COMMANDER_MAYBEBOARD,
                        component::NONE,
                        "clear_commander_maybeboard",
                        &e,
                    );
                    toast.error(e.to_user_message(), ToastOptions::default());
                    return;
                }
            };
            match auth_client().clear_commander_maybeboard(&session).await {
                Ok(()) => {
                    entries.set(Vec::new());
                    toast.info(
                        "Maybeboard cleared".to_string(),
                        ToastOptions::default().duration(Duration::from_millis(1500)),
                    );
                }
                Err(e) => {
                    tracing::warn!("commander maybeboard clear failed: {e}");
                    usage_buffer.peek().report_error(
                        screen::COMMANDER_MAYBEBOARD,
                        component::NONE,
                        "clear_commander_maybeboard",
                        &e,
                    );
                    toast.error(e.to_user_message(), ToastOptions::default());
                }
            }
        });
    };

    let on_remove = move |oracle_id: Uuid| {
        spawn(async move {
            let session = match session.ensure_fresh(auth_client).await {
                Ok(session) => session,
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::COMMANDER_MAYBEBOARD,
                        component::NONE,
                        "remove_commander_maybeboard",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    return;
                }
            };
            match auth_client()
                .remove_commander_maybeboard_card(oracle_id, &session)
                .await
            {
                Ok(()) => {
                    entries
                        .write()
                        .retain(|c| c.scryfall_data.oracle_id != Some(oracle_id));
                    toast.info(
                        "Removed".to_string(),
                        ToastOptions::default().duration(Duration::from_millis(1500)),
                    );
                }
                Err(e) => {
                    tracing::warn!("commander maybeboard remove failed: {e}");
                    usage_buffer.peek().report_error(
                        screen::COMMANDER_MAYBEBOARD,
                        component::NONE,
                        "remove_commander_maybeboard",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                }
            }
        });
    };

    // In-memory narrowing: the Show pips (contains-all identity, empty set =
    // All) compose with the filter sheet's criteria. The quick-add input
    // deliberately narrows nothing — one input driving two result sets reads
    // ambiguous. `filter_reset_counter` is the subscription (the sheet's
    // Apply bumps it); the builder itself is peeked so mid-edit sheet state
    // doesn't reshuffle the list, mirroring the deck cards screen.
    let _ = filter_reset_counter();
    let builder = filter_builder.peek().clone();
    let colors = selected_colors();
    let all_on = colors.is_empty();
    let mut filtered: Vec<Card> = entries()
        .iter()
        .filter(|c| {
            colors
                .iter()
                .all(|color| c.scryfall_data.color_identity.contains(color))
        })
        .cloned()
        .collect();
    if !builder.is_empty()
        && let Ok(criteria) = builder.build_criteria()
    {
        filtered = Cards::from(filtered).matching(&criteria).into();
    }
    // Save order (newest first) stands unless the sheet chose a sort.
    if let Some(sort) = builder.sort() {
        filtered = Cards::from(filtered)
            .sorted(sort, builder.ascending())
            .into();
    }
    let has_entries = !entries().is_empty();
    // Already-saved oracles: quick-add result chips skip them (nothing to
    // add), and the Swipe overlay excludes them from its pile (discovery
    // wants fresh commanders; deck create/edit still serve saves).
    let saved_oracles = use_memo(move || -> HashSet<Uuid> {
        entries()
            .iter()
            .filter_map(|c| c.scryfall_data.oracle_id)
            .collect()
    });
    let shown_results: Vec<Card> = search_results()
        .iter()
        .filter(|c| {
            c.scryfall_data
                .oracle_id
                .is_some_and(|o| !saved_oracles.read().contains(&o))
        })
        .cloned()
        .collect();
    let searching_catalog = query().trim().len() >= MIN_QUERY_LEN;

    rsx! {
        div { class: "screen",
            ScreenHeader { title: "Commander maybeboard", hint: hint_open }

            HintDialog {
                open: hint_open,
                title: "Commander maybeboard",
                HintLine {
                    "Commanders you save land here. Swipe "
                    HintKey { color: "--color-warning", "up" }
                    " while picking a commander to add one."
                }
                HintBullets {
                    HintBullet {
                        HintKey { color: "--accent-secondary", "Swipe" }
                        " deals commanders right here: right starts a deck, up saves it"
                    }
                    HintBullet {
                        HintKey { "Quick add" }
                        " searches commanders by name; tap a result to save it"
                    }
                    HintBullet {
                        HintKey { color: "--color-success", "Create deck" }
                        " on an entry starts a new deck with that commander"
                    }
                    HintBullet {
                        HintKey { color: "--color-error", "Remove" }
                        " takes it off the list"
                    }
                }
            }

            div { class: "screen-content",
                // Bounded like the deck cards screen: screen-content centers
                // its children, so an unbounded column would grow to content
                // width and overflow the viewport on long rows.
                div {
                    class: "flex-col",
                    style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    // Console rows render statically (closed vocabulary): the
                    // search input and all five pips are always valid.
                    div { class: "chip-row",
                        span { class: "chip-row-label", "Quick add:" }
                        input {
                            class: "input input-compact",
                            style: "flex: 1; margin-bottom: 0;",
                            id: "commander-maybeboard-search",
                            r#type: "text",
                            placeholder: "Search a commander to add",
                            value: "{query}",
                            autocapitalize: "none",
                            autocorrect: "off",
                            spellcheck: "false",
                            oninput: move |event| query.set(event.value()),
                        }
                        if !query().is_empty() {
                            button {
                                class: "clear-btn",
                                onclick: move |_| query.set(String::new()),
                                "\u{00d7}"
                            }
                        }
                    }
                    // Catalog results float below the bar, over the Show row
                    // and list — the shared search-float chips, hung downward
                    // (this console is the top of the page, so down is the
                    // only direction with room). Tap a chip to save it.
                    if searching_catalog {
                        div { class: "search-float-anchor",
                            if is_searching() {
                                div { class: "search-float-results search-float-below",
                                    div { class: "chip-unselected search-float-chip", "Searching..." }
                                }
                            } else if shown_results.is_empty() {
                                div { class: "search-float-results search-float-below",
                                    div { class: "chip-unselected search-float-chip", "No results" }
                                }
                            } else {
                                div { class: "search-float-results search-float-below",
                                    for (i, card) in shown_results.iter().cloned().enumerate() {
                                        div {
                                            key: "{card.scryfall_data.id}",
                                            class: "chip-unselected search-float-chip",
                                            style: "animation-delay: {i * 40}ms, {250 + i * 40}ms;",
                                            onclick: move |_| add_from_search(card.clone()),
                                            "{card.scryfall_data.name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "chip-row",
                        span { class: "chip-row-label", "Show:" }
                        Chip {
                            selected: all_on,
                            onclick: move |_| selected_colors.write().clear(),
                            "All"
                        }
                        for color in Color::all() {
                            Chip {
                                key: "{color.to_short_name()}",
                                selected: selected_colors().contains(&color),
                                onclick: move |_| {
                                    let mut colors = selected_colors.write();
                                    if !colors.remove(&color) {
                                        colors.insert(color);
                                    }
                                },
                                i { class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost" }
                            }
                        }
                        Chip {
                            selected: show_row_art(),
                            onclick: move |_| show_row_art.set(!show_row_art()),
                            "Art"
                        }
                    }

                    if is_loading() {
                        // The group and its header are static chrome, so they
                        // render real while only the data rows ghost (the
                        // closed-vocabulary skeleton rule).
                        div { class: "maybeboard-list card-group row-enter",
                            div { class: "card-group-header", "Commanders" }
                            for i in 0..6 {
                                div { key: "{i}", class: "skeleton-card-row",
                                    div { class: "skeleton-bar skeleton-card-thumb" }
                                    div { class: "skeleton-bar skeleton-card-bar-row" }
                                }
                            }
                        }
                    } else if load_failed() {
                        div { class: "message-empty",
                            p { "Could not load your commander maybeboard" }
                        }
                    } else if !has_entries {
                        div { class: "message-empty",
                            p { "No commanders" }
                        }
                    } else if filtered.is_empty() {
                        div { class: "message-empty",
                            p { "No commanders match" }
                        }
                    } else {
                        // Same bordered container the deck cards list wraps
                        // its rows in.
                        div { class: "maybeboard-list card-group row-enter",
                            div { class: "card-group-header", "Commanders" }
                            for card in filtered {
                                MaybeboardRow {
                                    key: "{card.scryfall_data.id}",
                                    card,
                                    expanded_card,
                                    preview_card,
                                    preview_dismissing,
                                    on_create,
                                    on_remove,
                                    on_printing: move |c: Card| {
                                        printing_card.set(Some(c));
                                        printing_open.set(true);
                                    },
                                }
                            }
                        }
                    }
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        navigator.push(Router::DeckList {});
                    },
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| filters_overlay_open.set(true),
                    "Filter"
                    if !filter_builder.read().is_empty() || filter_builder.read().sort().is_some() {
                        span { class: "filter-dot" }
                    }
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| swipe_open.set(true),
                    "Swipe"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| show_more_sheet.set(true),
                    "More"
                }
            }

            BottomSheet { open: show_more_sheet, title: "More actions",
                Button {
                    danger: true,
                    onclick: move |_| {
                        show_clear_dialog.set(true);
                    },
                    "Clear maybeboard"
                }
            }

            AlertDialogRoot {
                open: show_clear_dialog(),
                on_open_change: move |open| show_clear_dialog.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "Clear maybeboard" }
                    hr { class: "dialog-rule" }
                    AlertDialogDescription {
                        "Every commander on your maybeboard will be removed. This can't be undone."
                    }
                    hr { class: "dialog-rule" }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_clear_dialog.set(false),
                            "Cancel"
                        }
                        AlertDialogAction {
                            danger: true,
                            on_click: move |_| {
                                show_clear_dialog.set(false);
                                show_more_sheet.set(false);
                                clear_maybeboard();
                            },
                            "Clear"
                        }
                    }
                }
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: false,
                show_active_indicators: true,
                on_clear: move |_| {
                    filter_builder.write().clear();
                },
            }

            // In-place commander swiping: up-swipes save here (the shared
            // overlay's maybeboard wiring), right-swipe seeds a new deck with
            // the pick and jumps to the create screen.
            SwipeSelect {
                host_screen: screen::COMMANDER_MAYBEBOARD,
                open: swipe_open,
                mode: swipe_mode,
                exclude_oracle_ids: Some(saved_oracles.into()),
                on_select: move |card: Card| {
                    swipe_open.set(false);
                    create_with(card);
                },
                on_close: move |_| swipe_open.set(false),
            }

            // Pick a printing → swap that row's card in place and keep the
            // row expanded under its new scryfall id.
            if let Some(card) = printing_card() {
                PrintingSheet {
                    card,
                    open: printing_open,
                    host_screen: screen::COMMANDER_MAYBEBOARD,
                    on_save: move |new_card: Card| {
                        let target = new_card.scryfall_data.oracle_id;
                        if target.is_some() {
                            let mut expanded = expanded_card;
                            expanded.set(Some(new_card.scryfall_data.id));
                            if let Some(slot) = entries
                                .write()
                                .iter_mut()
                                .find(|c| c.scryfall_data.oracle_id == target)
                            {
                                *slot = new_card;
                            }
                        }
                    },
                }
            }

            if otag_examples_open() {
                OracleTagExamples { open: otag_examples_open, slug: otag_examples_slug() }
            }

            ImagePreview { card: preview_card, dismissing: preview_dismissing }
        }
    }
}
