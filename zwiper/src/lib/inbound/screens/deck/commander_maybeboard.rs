//! Commander maybeboard screen: the per-user "maybe this commander" list,
//! and a direct commander-discovery surface.
//!
//! Entries arrive three ways: up-swiping during any commander Zwipe-select,
//! up-swiping in this screen's own Swipe overlay, or tapping a floating
//! result chip under the Quick add input (a debounced commander-name search,
//! hung downward since this console sits at the top of the page). The input
//! is pure quick add: narrowing the saved list belongs to the five color
//! pips alone (contains-all identity semantics, like the deck list's Show
//! row); both console rows render statically per the closed-vocabulary
//! skeleton rule.
//!
//! Right-swiping in the Swipe overlay, or an expanded row's **Create deck**,
//! seeds the create screen with that commander (capacity-gated like the deck
//! list's Create). **Remove** un-maybes an entry in place.
//!
//! What the screen knows and does is
//! `zwipe_ui::screens::commander_maybeboard`; this file draws its state and
//! performs its effects.

use std::collections::HashSet;
use std::iter;

use components::{
    AlertDialog, Button, ButtonVariant, CardListFold, Chip, ClearWrap, DialogAction, Toasts,
    use_toast,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use tokio::time;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::commander::maybeboard::{Sort, SortBy};
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::commander_maybeboard::{
    self as model, Action, Destination, Effect, QuickAddResults, State, Status,
};
use zwipe_ui::screens::deck_list;

use super::components::danger_zone::DangerZoneAction;
use super::components::skeletons::{ChipGhost, ChipRowGhost, QuickAddSkeleton};
use super::components::swipe_select::{SwipeMode, SwipeSelect};
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::hint_dialog::{self, OneTimeHintDialog};
use crate::inbound::components::navigation::back_handler;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::{Screen, screen};
use crate::inbound::components::ui_toast;
use crate::inbound::router::Router;
use crate::inbound::screens::deck::card::components::card_row::{
    CardRow, OtagDescribe, OtagExamplesOpen, ShowRowArt,
};
use crate::inbound::screens::deck::card::components::image_preview::ImagePreview;
use crate::inbound::screens::deck::card::components::printing_sheet::PrintingSheet;
use crate::inbound::screens::deck::card::filter::card_filter_sheet::CardFilterSheet;
use crate::inbound::screens::deck::create::CreateDeckCommanderSeed;
use crate::inbound::screens::oracle_tag_examples::OracleTagExamples;
use crate::outbound::maybeboard_sort_store;

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
                    {model::CREATE_DECK}
                }
                if let Some(oid) = oracle_id {
                    button {
                        class: "card-action-btn",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_remove.call(oid);
                        },
                        {model::REMOVE}
                    }
                }
            },
        }
    }
}

/// The maybeboard's model with the handles that perform its effects. Copy,
/// so every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    hint_open: Signal<bool>,
    seed: CreateDeckCommanderSeed,
    usage_buffer: Signal<UsageBuffer>,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        let authed = self.authed;
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(
                    hint.key(),
                    self.session,
                    self.client,
                    self.hint_open,
                );
            }
            Effect::FetchMaybeboard(ticket) => {
                spawn(async move {
                    // try_run reports a failure and raises its error toast.
                    let reply = authed
                        .try_run("load_maybeboard", |c, s| async move {
                            c.fetch_commander_maybeboard(&s).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Ok(cards) => Action::Loaded { ticket, cards },
                        Err(_) => Action::Failed { ticket },
                    });
                });
            }
            Effect::FetchDeckProfiles => {
                spawn(async move {
                    // Quiet: backs the create gate only; a failure leaves the
                    // gate open and telemetry records it.
                    let found = authed
                        .run_quiet("load_deck_profiles", |c, s| async move {
                            c.fetch_deck_profiles(&s).await
                        })
                        .await;
                    if let Some(profiles) = found {
                        self.dispatch(Action::DeckProfilesLoaded {
                            count: profiles.len(),
                        });
                    }
                });
            }
            Effect::Debounce { ticket, wait } => {
                spawn(async move {
                    time::sleep(wait).await;
                    self.dispatch(Action::Debounced { ticket });
                });
            }
            Effect::SearchCommanders { ticket, query } => {
                let usage_buffer = self.usage_buffer;
                usage_buffer().record_search();
                spawn(async move {
                    // Quiet: fires per debounced keystroke; a toast per failed
                    // keypress would bury the screen, and the results simply
                    // stay stale.
                    let found = authed
                        .run_quiet("maybeboard_search", |c, s| async move {
                            c.search_commanders(&query, &s).await
                        })
                        .await;
                    self.dispatch(match found {
                        Some(cards) => Action::Searched { ticket, cards },
                        None => Action::Failed { ticket },
                    });
                });
            }
            Effect::AddCommander { ticket, oracle_id } => self.edit(
                "add_commander_maybeboard",
                ticket,
                Action::Added { ticket },
                move |c, s| async move { c.add_commander_maybeboard_card(oracle_id, &s).await },
            ),
            Effect::RemoveCommander { ticket, oracle_id } => self.edit(
                "remove_commander_maybeboard",
                ticket,
                Action::Removed { ticket },
                move |c, s| async move { c.remove_commander_maybeboard_card(oracle_id, &s).await },
            ),
            Effect::ClearMaybeboard(ticket) => self.edit(
                "clear_commander_maybeboard",
                ticket,
                Action::Cleared { ticket },
                |c, s| async move { c.clear_commander_maybeboard(&s).await },
            ),
            Effect::SeedCommander(card) => {
                let mut slot = self.seed.0;
                slot.set(Some(*card));
            }
            Effect::Navigate(Destination::DeckList) => {
                back_handler::back_or(self.navigator, Router::DeckList {});
            }
            Effect::Navigate(Destination::CreateDeck) => {
                self.navigator.push(Router::CreateDeck);
            }
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
            Effect::RememberSort(sort) => maybeboard_sort_store::infallible_save(sort),
        }
    }

    /// Runs the list edit `call`, reported under `op`, and answers with
    /// `done`, or with [`Action::Failed`] carrying `ticket` once the runner
    /// has raised its error toast.
    fn edit<Fut>(
        self,
        op: &'static str,
        ticket: Ticket,
        done: Action,
        call: impl FnOnce(ZwipeClient, Session) -> Fut + 'static,
    ) where
        Fut: Future<Output = Result<(), ClientError>> + 'static,
    {
        let authed = self.authed;
        spawn(async move {
            let reply = authed.run(op, call).await;
            self.dispatch(match reply {
                Some(()) => done,
                None => Action::Failed { ticket },
            });
        });
    }

    /// Starts a deck with `card`, through the create gate.
    fn create(self, card: Card) {
        let verified = self
            .session
            .peek()
            .as_ref()
            .is_none_or(|s| s.user.email_verified_at.is_some());
        self.dispatch(Action::Create {
            card: Box::new(card),
            verified,
        });
    }
}

/// Screen listing the user's saved "maybe" commanders with search-to-add,
/// an in-place commander Swipe overlay, and per-entry create-deck / remove.
#[component]
pub fn CommanderMaybeboard() -> Element {
    let driver = Driver {
        model: use_signal(|| State::with_sort(maybeboard_sort_store::restored())),
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(Screen::CommanderMaybeboard),
        session: use_context(),
        client: use_context(),
        hint_open: use_signal(|| false),
        seed: use_context(),
        usage_buffer: use_context(),
    };
    let auth_client = driver.client;
    let hint_open = driver.hint_open;
    // Offers the hint and fetches the list and the decks once per mount,
    // while the screen first draws, so the hint opens with the screen.
    use_hook(move || driver.dispatch(Action::Load));
    // Art-crop thumbnails on the rows, on by default; the "Art" chip in the
    // Show row toggles it, easing the thumbs in and out (the shared row keeps
    // them mounted, same as the deck cards screen). Screen-local.
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));

    // Oracle-tag reveal inside expanded rows: the same wiring as the deck
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

    // Local filter context for the shared CardFilterSheet, blank by default
    // and Reset returns to blank; no format filter (the list is commanders
    // only). Same wiring as the Swipe select overlay's local sheet.
    let mut filter_builder = use_signal(CardQueryBuilder::new);
    use_context_provider(|| filter_builder);
    let filter_reset_counter = use_signal(|| 0u32);
    use_context_provider(|| filter_reset_counter);
    let mut filters_overlay_open = use_signal(|| false);
    let mut show_clear_dialog = use_signal(|| false);

    // Tells the model when the Swipe overlay closes: its up swipes saved
    // entries, so the list fetches again.
    let mut swipe_was_open = use_signal(|| false);
    use_effect(move || {
        if swipe_open() {
            swipe_was_open.set(true);
        } else if *swipe_was_open.peek() {
            swipe_was_open.set(false);
            driver.dispatch(Action::SwipeClosed);
        }
    });

    let on_create = move |card: Card| driver.create(card);

    let clear_maybeboard = use_callback(move |()| {
        driver.dispatch(Action::Clear);
    });

    let on_remove = move |oracle_id: Uuid| driver.dispatch(Action::Remove(oracle_id));

    // Already-saved oracles: the Swipe overlay excludes them from its pile
    // (discovery wants fresh commanders; deck create/edit still serve saves).
    let saved_oracles = use_memo(move || -> HashSet<Uuid> { driver.model.read().saved_oracles() });

    // The card filter sheet narrows after the Show pips. `filter_reset_counter`
    // is the subscription (the sheet's Apply bumps it); the builder itself is
    // peeked so mid-edit sheet state doesn't reshuffle the list, mirroring the
    // deck cards screen.
    let _ = filter_reset_counter();
    let builder = filter_builder.peek().clone();
    let view = driver.model.read();
    let is_loading = view.status() == Status::Loading;
    let sort = view.sort();
    let nothing_saved = view.status() == Status::Empty;
    let query = view.query().to_owned();
    let quick_add = view.quick_add();
    let all_on = view.shows_all();
    let pips = Color::all().map(|color| (color, view.is_picked(color)));
    let message = view.message(&builder);
    let filtered = view.shown(&builder);
    drop(view);

    rsx! {
        div { class: "screen",
            ScreenHeader { title: model::TITLE, hint: hint_open, back: move |()| driver.dispatch(Action::Back) }

            OneTimeHintDialog { open: hint_open, hint: OneTimeHint::CommanderMaybeboard }

            div { class: "screen-content",
                // Bounded like the deck cards screen: screen-content centers
                // its children, so an unbounded column would grow to content
                // width and overflow the viewport on long rows.
                div {
                    class: "flex-col",
                    style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    // The whole page ghosts on the first load: the Quick add and Show
                    // rows as their shapes, the list with its header title as a bar.
                    if is_loading {
                        QuickAddSkeleton { label: model::QUICK_ADD }
                        ChipRowGhost {
                            label: model::SHOW,
                            chips: iter::once(ChipGhost::Text(model::SHOW_ALL.to_string()))
                                .chain(Color::all().into_iter().map(ChipGhost::Pip))
                                .chain(iter::once(ChipGhost::Text(model::ART.to_string())))
                                .collect::<Vec<_>>(),
                        }
                        ChipRowGhost {
                            label: deck_list::SORT,
                            chips: SortBy::ALL
                                .into_iter()
                                .map(|by| ChipGhost::Text(by.label().to_string()))
                                .chain(iter::once(ChipGhost::Text(sort.direction.to_string())))
                                .collect::<Vec<_>>(),
                        }
                        div { class: "maybeboard-list card-group row-enter",
                            div { class: "card-group-header",
                                div { class: "skeleton-bar skeleton-card-bar-header" }
                            }
                            for i in 0..6 {
                                div { key: "{i}", class: "skeleton-card-row",
                                    div { class: "skeleton-bar skeleton-card-thumb" }
                                    div { class: "skeleton-bar skeleton-card-bar-row" }
                                }
                            }
                        }
                    } else {
                        div { class: "chip-row",
                            label {
                                class: "stat-chip stat-chip-p1 chip-row-label",
                                r#for: "commander-maybeboard-search",
                                {model::QUICK_ADD}
                            }
                            ClearWrap {
                                filled: !query.is_empty(),
                                on_clear: move |()| driver.dispatch(Action::Type(String::new())),
                                label: "Clear the search".to_string(),
                                input {
                                    class: "input input-compact",
                                    id: "commander-maybeboard-search",
                                    r#type: "text",
                                    placeholder: model::QUICK_ADD_PLACEHOLDER,
                                    value: "{query}",
                                    autocapitalize: "none",
                                    autocorrect: "off",
                                    spellcheck: "false",
                                    oninput: move |event| driver.dispatch(Action::Type(event.value())),
                                }
                            }
                        }
                        // Catalog results float below the bar, over the Show row
                        // and list: the shared search-float chips, hung downward
                        // (this console is the top of the page, so down is the
                        // only direction with room). Tap a chip to save it.
                        if let Some(results) = quick_add {
                            div { class: "search-float-anchor",
                                match results {
                                    QuickAddResults::Searching => rsx! {
                                        div { class: "search-float-results search-float-below",
                                            div { class: "chip-unselected search-float-chip", {model::SEARCHING} }
                                        }
                                    },
                                    QuickAddResults::Empty => rsx! {
                                        div { class: "search-float-results search-float-below",
                                            div { class: "chip-unselected search-float-chip", {model::NO_RESULTS} }
                                        }
                                    },
                                    QuickAddResults::Found(found) => rsx! {
                                        div { class: "search-float-results search-float-below",
                                            for (i, card) in found.into_iter().enumerate() {
                                                div {
                                                    key: "{card.scryfall_data.id}",
                                                    class: "chip-unselected search-float-chip",
                                                    style: "animation-delay: {i * 40}ms, {250 + i * 40}ms;",
                                                    onclick: move |_| {
                                                        if let Some(oracle_id) = card.scryfall_data.oracle_id {
                                                            driver.dispatch(Action::Add(oracle_id));
                                                        }
                                                    },
                                                    "{card.scryfall_data.name}"
                                                }
                                            }
                                        }
                                    },
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", {model::SHOW} }
                            Chip {
                                selected: all_on,
                                onclick: move |_| driver.dispatch(Action::ShowAll),
                                {model::SHOW_ALL}
                            }
                            for (color, picked) in pips {
                                Chip {
                                    key: "{color.to_short_name()}",
                                    selected: picked,
                                    onclick: move |_| driver.dispatch(Action::ToggleColor(color)),
                                    i {
                                        class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                                        "aria-hidden": "true",
                                    }
                                    span { class: "sr-only", "{color.to_long_name()}" }
                                }
                            }
                            Chip {
                                selected: show_row_art(),
                                onclick: move |_| show_row_art.set(!show_row_art()),
                                {model::ART}
                            }
                        }
                        // The key chips pick, the last chip turns the order
                        // around, as on Decks.
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", {deck_list::SORT} }
                            for by in SortBy::ALL {
                                Chip {
                                    key: "{by.label()}",
                                    selected: sort.by == by,
                                    onclick: move |_| driver.dispatch(Action::Sort(Sort { by, ..sort })),
                                    "{by.label()}"
                                }
                            }
                            Chip {
                                selected: false,
                                onclick: move |_| driver.dispatch(Action::Sort(Sort { direction: sort.direction.flipped(), ..sort })),
                                "{sort.direction}"
                            }
                        }

                        // Same bordered container the deck cards list wraps
                        // its rows in, there whether or not anything is saved.
                        // Swipe sits in the list's header, so it says what gets
                        // swiped; with nothing to list, one row says why.
                        CardListFold {
                            title: model::LIST_TITLE,
                            actions: rsx! {
                                Button {
                                    variant: ButtonVariant::Action,
                                    onclick: move |_| swipe_open.set(true),
                                    {model::SWIPE}
                                }
                            },
                            if let Some(message) = message {
                                div { class: "card-row card-row-note", {message} }
                            }
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
                        DangerZoneAction {
                            label: model::CLEAR_MAYBEBOARD,
                            button: model::CLEAR,
                            disabled: nothing_saved,
                            onclick: move |()| show_clear_dialog.set(true),
                        }
                    }
                }
            }

            // Filter, the screen's action, floats at the bottom right.
            div { class: "action-dock action-dock-right",
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| filters_overlay_open.set(true),
                    {model::FILTER}
                    if !filter_builder.read().is_empty() || filter_builder.read().sort().is_some() {
                        span { class: "filter-dot" }
                    }
                }
            }

            AlertDialog {
                open: show_clear_dialog,
                title: model::CLEAR_MAYBEBOARD,
                actions: vec![
                    DialogAction::Dismiss(model::CANCEL.into()),
                    DialogAction::Confirm {
                        label: model::CLEAR.into(),
                        danger: true,
                        on_confirm: clear_maybeboard,
                    },
                ],
                {model::CLEAR_BODY}
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: false,
                show_active_indicators: true,
                on_clear: move |()| {
                    filter_builder.write().clear();
                },
            }

            // Commanders not saved yet: up saves one here, right starts a
            // deck with it and goes to the create screen.
            SwipeSelect {
                host_screen: screen::COMMANDER_MAYBEBOARD,
                open: swipe_open,
                mode: swipe_mode,
                exclude_oracle_ids: Some(saved_oracles.into()),
                starts_deck: true,
                on_select: move |card: Card| {
                    swipe_open.set(false);
                    driver.create(card);
                },
                on_close: move |()| swipe_open.set(false),
            }

            // Pick a printing → swap that row's card in place and keep the
            // row expanded under its new scryfall id.
            if let Some(card) = printing_card() {
                PrintingSheet {
                    card,
                    open: printing_open,
                    host_screen: screen::COMMANDER_MAYBEBOARD,
                    on_save: move |new_card: Card| {
                        if new_card.scryfall_data.oracle_id.is_some() {
                            let mut expanded = expanded_card;
                            expanded.set(Some(new_card.scryfall_data.id));
                        }
                        driver.dispatch(Action::PickPrinting(Box::new(new_card)));
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
