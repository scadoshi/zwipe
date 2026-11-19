//! Collection's list, under the import: the collection a page at a time, searched by card name,
//! each row a card row whose stepper sets its copies, and Export in the list's header.
//!
//! Rows open the details deck rows do, Send to included (it adds the row's printing to the deck
//! picked), through the screen's overlays and hover preview. One printing can hold several rows
//! (finish, condition, language), so each row is keyed by its own id, and the row keys act on the
//! row that id names.
//!
//! The View chips switch the list to the deck page's stacks, one per card type with lands last,
//! and back; Collection opens on the stacks until the user picks, and remembers the pick apart
//! from the deck page's. A stack shows a printing once with its rows' copies summed, and its
//! drawer holds a stepper for each of those rows.

use std::collections::{HashMap, HashSet};

use components::{
    Button, ButtonVariant, CardDetailsSheet, CardEditButtons, CardListFold, CardsViewSwitch,
    ClearWrap, DRAWER_CLASSES, DeckSection, DeckStacks, DeckStacksGhost, DrawerEdge,
    GroupBuyDialog, HoverStack, LoadFailed, StackDetails, spawn_write, use_group_buy, use_pointer,
    use_stack_details, use_stack_details_sync,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{Toasts, use_toast};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::collection::CollectionPageSize;
use zwipe_ui::deck_cards::columns::Side;
use zwipe_ui::deck_cards::keys::{RowInfo, RowKind};
use zwipe_ui::deck_cards::stacks::{CardsView, Reveal};
use zwipe_ui::screens::collection_list::stacks::{self, Stack, StackCard};
use zwipe_ui::screens::collection_list::{self, Action, Effect, RowKey, State, Status, row_tags};
use zwipe_ui::shortcuts::ListLayout;

use crate::inbound::cards::card_row::{CardOverlays, CardRow};
use crate::inbound::cards::otag::{OtagDescribe, OtagExamplesOpen};
use crate::inbound::cards::row_keys::use_keyed_row_keys;
use crate::inbound::cards::stack_view::{RowFaces, use_stacks_dealing};
use crate::inbound::chrome::authed::{Authed, use_authed};
use crate::inbound::chrome::ghost::{GhostButton, GhostText};
use crate::inbound::chrome::shortcuts::ShortcutsScreen;
use crate::inbound::chrome::toast::raise;
use crate::inbound::collection::CollectionCache;
use crate::outbound::cards_view_store::CardsViewStore;
use crate::outbound::{browser, timer};

/// The list's header.
const LIST_TITLE: &str = "Cards";

/// The search field's id, which its label points at.
const SEARCH_ID: &str = "collection-search";

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
                    timer::sleep(wait).await;
                    self.dispatch(Action::Search(ticket));
                });
            }
            Effect::FlushAfter { ticket, wait } => {
                spawn(async move {
                    timer::sleep(wait).await;
                    self.dispatch(Action::Flush(ticket));
                });
            }
            Effect::SetCopies { ticket, set } => {
                // A write outlives the screen, so leaving mid-save never drops it.
                spawn_write(async move {
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
            Effect::Download { file_name, text } => browser::download(&file_name, &text),
            Effect::Toast(toast) => raise(self.toast, &toast),
        }
    }
}

/// Collection's list: the search row, then the rows under the `Cards` header with Export.
///
/// `imports` counts the imports that landed on this visit; each one reads the list again from
/// its first page.
#[component]
pub fn CollectionList(imports: ReadSignal<u32>) -> Element {
    // Opening asks for the first page; the effects run once the model is in place.
    let mut opening = Vec::new();
    let model = use_signal(|| {
        let mut state = State::new();
        opening = state.update(Action::Open);
        state
    });
    let driver = Driver {
        model,
        authed: use_authed(),
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
    // The screen holds the overlays, the hover preview and the copy cap, outside its content.
    let hover_stack: HoverStack = use_context();
    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    let keyed = use_memo(move || -> Vec<(Uuid, Card)> {
        driver
            .model
            .read()
            .rows()
            .iter()
            .map(|row| (row.key.id(), row.card.card.clone()))
            .collect()
    });
    use_keyed_row_keys(
        keyed,
        expanded_card,
        ShortcutsScreen::CardRows(ListLayout::One),
        None,
    );

    // List or Stacks, as last left here.
    let view = use_signal(|| CardsViewStore::collection().load());
    let pointer = use_pointer();
    let touch = !pointer();
    let RowFaces(faces) = use_context();
    let group_buy = use_group_buy();
    let collapsed = use_signal(HashSet::new);
    let details = use_stack_details(use_signal(Reveal::default), use_signal(|| false));
    let shown_stacks = use_memo(move || stacks::stacks(driver.model.read().rows()));
    let listed = use_memo(move || -> Vec<Uuid> {
        shown_stacks()
            .iter()
            .flat_map(|s| s.cards.iter().map(|c| c.card.scryfall_data.id))
            .collect()
    });
    use_stack_details_sync(details.reveal, details.details_open, listed);
    let loaded = driver.model.read().status() != Status::Loading;
    let dealing = use_stacks_dealing(view, loaded, imports());
    let on_step = EventHandler::new(move |(key, delta): (RowKey, i32)| {
        driver.dispatch(Action::Step { key, delta });
    });

    let model = driver.model.read();
    let status = model.status();
    let rows = model.rows().to_vec();
    let text = model.text().to_owned();
    let exporting = model.exporting();
    let more = status == Status::List && model.has_more();
    drop(model);
    let stacks_on = view() == CardsView::Stacks;
    let stack_cards: Vec<StackCard> = shown_stacks().into_iter().flat_map(|s| s.cards).collect();

    rsx! {
        div {
            class: "collection-list",
            onmousemove: move |e| hover_stack.track(e.client_coordinates().x),
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
                    CardsViewStore::collection().save(choice);
                    details.dismiss();
                },
            }
            if status == Status::Loading && stacks_on {
                DeckStacksGhost { key: "{touch}", touch }
            } else if status == Status::Loading {
                div { class: "maybeboard-list card-group row-enter",
                    div { class: "card-group-header card-group-header-with-hint",
                        GhostText { chars: 5 }
                        GhostButton { chars: 6 }
                    }
                    for i in 0..3 {
                        div { key: "{i}", class: "skeleton-card-row web-ghost-card-row",
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
                        div {
                            class: "collection-stacks",
                            "data-settle-scope": "",
                            "data-settle": touch.then_some(""),
                            // A tap off every card closes the reveal; a card's own tap stops at the card.
                            onclick: move |_| {
                                if touch {
                                    details.dismiss();
                                }
                            },
                            DeckStacks {
                                // A change of input lays the stacks out again for it.
                                key: "{touch}",
                                sections: sections(&shown_stacks()),
                                currency: PriceCurrency::default(),
                                collapsed,
                                details,
                                faces,
                                group_buy,
                                dealing: dealing(),
                                touch,
                            }
                        }
                    }
                    for row in rows.into_iter().filter(|_| !stacks_on) {
                        CardRow {
                            key: "{row.key.id()}",
                            card: row.card.card.clone(),
                            qty: row.copies,
                            expanded_card,
                            row_key: row.key.id(),
                            qty_tags: row_tags(&row),
                            send_to: true,
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
            GroupBuyDialog { group_buy }
            CollectionStackDrawer { cards: stack_cards, details, faces, on_step }
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

/// The open stack card's drawer, the deck page's: the card's details with Image, Printings and
/// Send to in its foot, above them a line for each of its rows with that row's finish, condition and language and
/// its copies stepper. Mounted, closed, on the first card before any click, so it slides in and
/// out; nothing while no card is listed.
#[component]
fn CollectionStackDrawer(
    cards: Vec<StackCard>,
    details: StackDetails,
    faces: Signal<HashMap<Uuid, usize>>,
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
    let overlays = try_use_context::<CardOverlays>();
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let card = shown.card;
    let card_id = card.scryfall_data.id;
    let shown_face = faces.read().get(&card_id).copied().unwrap_or(0);
    let on_image = overlays.map(|CardOverlays { mut preview, .. }| {
        let image_of = card.scryfall_data.clone();
        EventHandler::new(move |face: usize| preview.set(Some((image_of.clone(), face))))
    });
    let on_printings = overlays.map(|CardOverlays { mut printing, .. }| {
        let print_card = card.clone();
        EventHandler::new(move |()| printing.set(Some(print_card.clone())))
    });
    let footer = rsx! {
        for row in shown.rows {
            div { key: "{row.key.id()}", class: "card-action-row",
                for tag in row_tags(&row).into_iter().skip(1) {
                    span { class: "card-row-stat", "{tag}" }
                }
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
        CardDetailsSheet {
            open: details_open,
            card,
            on_image,
            on_printings,
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
