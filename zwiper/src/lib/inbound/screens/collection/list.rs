//! Collection's list, under the import: the collection a page at a time, searched by card name,
//! each row a card row whose stepper sets its copies, and Export in the list's header, which hands
//! the file to the share sheet.
//!
//! Rows open the details deck rows do, Send to included (it adds the row's printing to the deck
//! picked). One printing can hold several rows (finish, condition, language), so each row is keyed
//! by its own id.
//!
//! The View chips switch the list to the deck page's stacks, one per card type with lands last,
//! and back; Collection opens on the stacks until the user picks, and remembers the pick apart
//! from the deck page's. A stack shows a printing once with its rows' copies summed, and its
//! drawer holds a stepper for each of those rows.

use std::collections::{HashMap, HashSet};

use components::{
    Button, ButtonVariant, CardEditButtons, CardListFold, CardsViewSwitch, ClearWrap,
    DRAWER_CLASSES, DeckSection, DeckStacks, DeckStacksGhost, DrawerEdge, GroupBuy, GroupBuyDialog,
    LoadFailed, StackDetails, Toasts, spawn_in_order, use_group_buy, use_stack_details,
    use_stack_details_sync, use_toast,
};
use dioxus::prelude::*;
use tokio::time;
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::collection::CollectionPageSize;
use zwipe_ui::deck_cards::columns::Side;
use zwipe_ui::deck_cards::keys::{RowInfo, RowKind};
use zwipe_ui::deck_cards::stacks::{
    CardsView, DEAL_DURATION, DEAL_STAGGER_CAP, DealCue, Reveal, deal_delay,
};
use zwipe_ui::screens::collection_list::stacks::{self, Stack, StackCard};
use zwipe_ui::screens::collection_list::{
    self, Action, Effect, RowKey, State, Status, row_label, row_tags,
};
use zwipe_ui::toast::{Toast, ToastKind};

use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::collection::CollectionCache;
use crate::inbound::components::telemetry::vocabulary::Screen;
use crate::inbound::components::ui_toast;
use crate::inbound::screens::deck::card::components::card_row::{
    CardRow, DeckFaces, OtagDescribe, OtagExamplesOpen,
};
use crate::outbound::{cards_view_store, share};

/// The list's header.
const LIST_TITLE: &str = "Cards";

/// The search field's id, which its label points at.
const SEARCH_ID: &str = "collection-search";

/// The toast when the share sheet couldn't open.
const SHARE_FAILED: &str = "Couldn't open the share sheet";

/// The model and what its effects reach, `Copy` so handlers and tasks share it.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    authed: Authed,
    toast: Toasts,
    /// What the root keeps of the collection, dropped when a row's copies change.
    collection: CollectionCache,
}

impl Driver {
    /// Hands `action` to the model and performs what it asks for.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    fn perform(self, effect: Effect) {
        match effect {
            Effect::FetchPage {
                ticket,
                search,
                after,
            } => {
                spawn(async move {
                    let page = self
                        .authed
                        .try_run_quiet("collection_page", |c, s| async move {
                            c.fetch_collection_page(
                                &s,
                                &search,
                                after.as_ref(),
                                CollectionPageSize::DEFAULT,
                            )
                            .await
                        })
                        .await;
                    self.dispatch(match page {
                        Ok(page) => Action::PageLoaded {
                            ticket,
                            page: Box::new(page),
                        },
                        Err(_) => Action::PageFailed { ticket },
                    });
                });
            }
            Effect::SearchAfter { ticket, wait } => {
                spawn(async move {
                    time::sleep(wait).await;
                    self.dispatch(Action::Search(ticket));
                });
            }
            Effect::FlushAfter { ticket, wait } => {
                spawn(async move {
                    time::sleep(wait).await;
                    self.dispatch(Action::Flush(ticket));
                });
            }
            Effect::SetCopies { ticket, set } => {
                // One printing's sets reach the server in the order they were made, each the
                // row's whole count.
                spawn_in_order(set.scryfall_data_id, async move {
                    let reply = self
                        .authed
                        .try_run_quiet("set_collection_card", |c, s| async move {
                            c.set_collection_card(&s, &set).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Ok(reply) => Action::Saved {
                            ticket,
                            reply: Box::new(reply),
                        },
                        Err(_) => Action::SaveFailed { ticket },
                    });
                });
            }
            Effect::CollectionChanged => {
                let CollectionCache(mut cache) = self.collection;
                cache.set(None);
            }
            Effect::Export { ticket } => {
                spawn(async move {
                    let export = self
                        .authed
                        .try_run_quiet("export_collection", |c, s| async move {
                            c.export_collection(&s).await
                        })
                        .await;
                    self.dispatch(match export {
                        Ok(export) => Action::Exported { ticket, export },
                        Err(_) => Action::ExportFailed { ticket },
                    });
                });
            }
            Effect::Download { file_name, text } => {
                if let Err(e) = share::share_file(&file_name, &text) {
                    tracing::error!("collection export not shared: {e}");
                    ui_toast::raise(self.toast, Toast::normal(ToastKind::Error, SHARE_FAILED));
                }
            }
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
        }
    }
}

/// What the stacks share with the overlays they open, which sit at the screen's root: inside
/// the list's scroll they would sit under the pinned controls.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct StackOverlays {
    /// The stacks' reveal and open card.
    details: StackDetails,
    /// Every card's shown face, shared by its row, its stack card and its drawer.
    faces: Signal<HashMap<Uuid, usize>>,
    /// The stack headers' group buy.
    group_buy: GroupBuy,
    /// The stacks' cards, as the list last laid them out.
    cards: Signal<Vec<StackCard>>,
    /// Sets a row's copies, once the list is up.
    step: Signal<Option<EventHandler<(RowKey, i32)>>>,
}

/// The screen's [`StackOverlays`], and the shared faces every row on it reads.
pub(super) fn use_stack_overlays() -> StackOverlays {
    let faces: Signal<HashMap<Uuid, usize>> = use_signal(HashMap::new);
    use_context_provider(|| DeckFaces(faces));
    StackOverlays {
        details: use_stack_details(use_signal(Reveal::default), use_signal(|| false)),
        faces,
        group_buy: use_group_buy(),
        cards: use_signal(Vec::new),
        step: use_signal(|| None),
    }
}

/// The overlays the stacks open: the group buy and the open card's drawer.
#[component]
pub(super) fn StackOverlaysHost(
    overlays: StackOverlays,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
) -> Element {
    let StackOverlays {
        details,
        faces,
        group_buy,
        cards,
        step,
    } = overlays;
    rsx! {
        GroupBuyDialog { group_buy }
        if let Some(on_step) = step() {
            CollectionStackDrawer { cards: cards(), details, faces, preview_card, preview_dismissing, on_step }
        }
    }
}

/// Collection's list: the search row, then the rows under the `Cards` header with Export.
///
/// `imports` counts the imports that landed on this visit; each one reads the list again from
/// its first page. `overlays` holds what the stacks share with the drawer they open.
#[component]
pub(crate) fn CollectionList(
    imports: ReadSignal<u32>,
    overlays: StackOverlays,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
) -> Element {
    // Opening asks for the first page; the effects run once the model is in place.
    let mut opening = Vec::new();
    let model = use_signal(|| {
        let mut state = State::new();
        opening = state.update(Action::Open);
        state
    });
    let driver = Driver {
        model,
        authed: use_authed(Screen::Collection),
        toast: use_toast(),
        collection: use_context(),
    };
    use_hook(move || {
        for effect in opening {
            driver.perform(effect);
        }
    });
    let mut imports_seen = use_signal(|| imports.peek().to_owned());
    use_effect(move || {
        let landed = imports();
        if landed != *imports_seen.peek() {
            imports_seen.set(landed);
            driver.dispatch(Action::Imported);
        }
    });
    let mut expanded_card: Signal<Option<Uuid>> = use_signal(|| None);

    // List or Stacks, as last left here.
    let view = use_signal(cards_view_store::restored_collection);
    let StackOverlays {
        details,
        faces,
        group_buy,
        mut cards,
        mut step,
    } = overlays;
    let collapsed = use_signal(HashSet::new);
    let shown_stacks = use_memo(move || stacks::stacks(driver.model.read().rows()));
    use_effect(move || cards.set(shown_stacks().into_iter().flat_map(|s| s.cards).collect()));
    use_hook(move || {
        step.set(Some(EventHandler::new(
            move |(key, delta): (RowKey, i32)| {
                driver.dispatch(Action::Step { key, delta });
            },
        )));
    });
    let listed = use_memo(move || -> Vec<Uuid> {
        shown_stacks()
            .iter()
            .flat_map(|s| s.cards.iter().map(|c| c.card.scryfall_data.id))
            .collect()
    });
    use_stack_details_sync(details.reveal, details.details_open, listed);
    // The stacks deal in on entering Stacks once the first page is in, and again after an
    // import, for as long as the longest stack takes.
    let mut deal_cue = use_signal(DealCue::default);
    let mut dealing = use_signal(|| false);
    use_effect(move || {
        let shown = view();
        let ready = driver.model.read().status() != Status::Loading;
        let _landed = imports();
        if deal_cue.write().on(shown, ready) {
            dealing.set(true);
            spawn(async move {
                time::sleep(deal_delay(DEAL_STAGGER_CAP).saturating_add(DEAL_DURATION)).await;
                dealing.set(false);
            });
        }
    });

    let model = driver.model.read();
    let status = model.status();
    let rows = model.rows().to_vec();
    let text = model.text().to_owned();
    let exporting = model.exporting();
    let more = status == Status::List && model.has_more();
    drop(model);
    let stacks_on = view() == CardsView::Stacks;

    rsx! {
        div { class: "collection-list",
            div { class: "chip-row",
                label { class: "stat-chip stat-chip-p1 chip-row-label", r#for: SEARCH_ID, "Search" }
                ClearWrap {
                    filled: !text.is_empty(),
                    on_clear: move |()| driver.dispatch(Action::Type(String::new())),
                    label: "Clear the search".to_string(),
                    input {
                        class: "input input-compact",
                        id: SEARCH_ID,
                        r#type: "text",
                        placeholder: "Search your collection",
                        value: "{text}",
                        autocapitalize: "none",
                        autocorrect: "off",
                        spellcheck: "false",
                        oninput: move |event| driver.dispatch(Action::Type(event.value())),
                    }
                }
            }
            CardsViewSwitch {
                view,
                on_change: move |choice: CardsView| {
                    cards_view_store::infallible_save_collection(choice);
                    details.dismiss();
                    expanded_card.set(None);
                },
            }
            if status == Status::Loading && stacks_on {
                DeckStacksGhost { touch: true }
            } else if status == Status::Loading {
                div { class: "maybeboard-list card-group row-enter",
                    div { class: "card-group-header",
                        div { class: "skeleton-bar skeleton-card-bar-header" }
                    }
                    for i in 0..3 {
                        div { key: "{i}", class: "skeleton-card-row",
                            div { class: "skeleton-bar skeleton-card-thumb" }
                            div { class: "skeleton-bar skeleton-card-bar-row" }
                        }
                    }
                }
            } else if status == Status::Failed {
                LoadFailed {
                    message: collection_list::FAILED,
                    on_retry: move |()| driver.dispatch(Action::Retry),
                }
            } else {
                CardListFold {
                    title: LIST_TITLE,
                    actions: rsx! {
                        Button {
                            variant: ButtonVariant::Action,
                            disabled: exporting,
                            onclick: move |_| driver.dispatch(Action::Export),
                            "Export"
                        }
                    },
                    if status == Status::Empty {
                        div { class: "card-row card-row-note", {collection_list::EMPTY} }
                    } else if status == Status::NoMatch {
                        div { class: "card-row card-row-note", {collection_list::NO_MATCH} }
                    }
                    if stacks_on && !rows.is_empty() {
                        div { class: "collection-stacks", "data-settle-scope": "",
                            DeckStacks {
                                sections: sections(&shown_stacks()),
                                currency: PriceCurrency::default(),
                                collapsed,
                                details,
                                faces,
                                group_buy,
                                dealing: dealing(),
                                touch: true,
                            }
                        }
                    }
                    for row in rows.into_iter().filter(|_| !stacks_on) {
                        CardRow {
                            key: "{row.key.id()}",
                            card: row.card.card.clone(),
                            qty: row.copies,
                            expanded_card,
                            preview_card,
                            preview_dismissing,
                            row_key: row.key.id(),
                            qty_tags: row_tags(&row),
                            on_qty_change: row.editable.then(|| {
                                let key = row.key;
                                EventHandler::new(move |delta: i32| {
                                    driver.dispatch(Action::Step { key, delta });
                                })
                            }),
                        }
                    }
                    if more {
                        button {
                            class: "chip changelog-more",
                            onclick: move |_| driver.dispatch(Action::NearEnd),
                            "Show more"
                        }
                    }
                }
            }
        }
    }
}

/// The deck page's sections for `stacks`, each card's row carrying its copies.
fn sections(stacks: &[Stack]) -> Vec<(DeckSection, Vec<RowInfo>)> {
    stacks
        .iter()
        .map(|stack| {
            let section = DeckSection {
                key: stack.key.clone(),
                title: stack.title.clone(),
                pips: None,
                cards: stack.cards.iter().map(|c| c.card.clone()).collect(),
            };
            let rows = stack
                .cards
                .iter()
                .map(|c| RowInfo {
                    card: c.card.clone(),
                    qty: c.copies,
                    kind: RowKind::Main { mvp: false },
                })
                .collect();
            (section, rows)
        })
        .collect()
}

/// The open stack card's drawer: the card's details with Image and Send to in its foot, above
/// them a line for each of its rows with that row's label and its copies stepper. Mounted,
/// closed, on the first card before any tap, so it slides in and out; nothing while no card is
/// listed.
#[component]
fn CollectionStackDrawer(
    cards: Vec<StackCard>,
    details: StackDetails,
    faces: Signal<HashMap<Uuid, usize>>,
    mut preview_card: Signal<Option<(ScryfallData, usize)>>,
    mut preview_dismissing: Signal<bool>,
    on_step: EventHandler<(RowKey, i32)>,
) -> Element {
    let StackDetails {
        reveal,
        details_open,
        side,
        ..
    } = details;
    let drawn = reveal().drawn();
    let Some(shown) = cards
        .iter()
        .find(|c| Some(c.card.scryfall_data.id) == drawn)
        .or_else(|| cards.first())
        .cloned()
    else {
        return rsx! {};
    };
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let card = shown.card;
    let card_id = card.scryfall_data.id;
    let shown_face = faces.read().get(&card_id).copied().unwrap_or(0);
    let image_of = card.scryfall_data.clone();
    let footer = rsx! {
        for row in shown.rows {
            div { key: "{row.key.id()}", class: "card-action-row",
                span { class: "card-row-stat", {row_label(&row)} }
                CardEditButtons {
                    qty: row.copies,
                    on_qty_change: row.editable.then(|| {
                        let key = row.key;
                        EventHandler::new(move |delta: i32| on_step.call((key, delta)))
                    }),
                }
            }
        }
    };
    rsx! {
        components::CardDetailsSheet {
            open: details_open,
            card,
            on_image: move |face: usize| {
                preview_card.set(Some((image_of.clone(), face)));
                preview_dismissing.set(false);
            },
            footer,
            class: DRAWER_CLASSES,
            face: Some(shown_face),
            on_face_change: move |face: usize| {
                let mut faces = faces;
                faces.write().insert(card_id, face);
            },
            describe_tag,
            on_examples,
            edge: match side() {
                Side::Left => DrawerEdge::Left,
                Side::Right => DrawerEdge::Right,
            },
        }
    }
}
