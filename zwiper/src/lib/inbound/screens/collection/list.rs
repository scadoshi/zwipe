//! Collection's list, under the import: the collection a page at a time, searched by card name,
//! each row a card row whose stepper sets its copies, and Export in the list's header, which hands
//! the file to the share sheet.
//!
//! Rows open the details deck rows do, Send to included (it adds the row's printing to the deck
//! picked). One printing can hold several rows (finish, condition, language), so each row is keyed
//! by its own id.

use components::{
    Button, ButtonVariant, CardListFold, ClearWrap, LoadFailed, Toasts, spawn_write, use_toast,
};
use dioxus::prelude::*;
use tokio::time;
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::collection::CollectionPageSize;
use zwipe_ui::screens::collection_list::{self, Action, Effect, State, Status, row_tags};
use zwipe_ui::toast::{Toast, ToastKind};

use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::collection::CollectionCache;
use crate::inbound::components::telemetry::vocabulary::Screen;
use crate::inbound::components::ui_toast;
use crate::inbound::screens::deck::card::components::card_row::CardRow;
use crate::outbound::share::{self, ShareError};

/// The list's header.
const LIST_TITLE: &str = "Cards";

/// The search field's id, which its label points at.
const SEARCH_ID: &str = "collection-search";

/// The toast when the share sheet couldn't open.
const SHARE_FAILED: &str = "Couldn't open the share sheet";

/// The toast when the export is past what Android's share carries as text.
const SHARE_TOO_BIG: &str = "Your collection is too big to share from here";

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
            Effect::Download { file_name, text } => {
                if let Err(e) = share::share_file(&file_name, &text) {
                    tracing::error!("collection export not shared: {e}");
                    let words = match e {
                        ShareError::TooBig => SHARE_TOO_BIG,
                        ShareError::Unsupported
                        | ShareError::NotMainThread
                        | ShareError::Platform(_) => SHARE_FAILED,
                    };
                    ui_toast::raise(self.toast, Toast::normal(ToastKind::Error, words));
                }
            }
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
        }
    }
}

/// Collection's list: the search row, then the rows under the `Cards` header with Export.
///
/// `imports` counts the imports that landed on this visit; each one reads the list again from
/// its first page. `preview_card` and `preview_dismissing` drive the screen's full-size image.
#[component]
pub(crate) fn CollectionList(
    imports: ReadSignal<u32>,
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
    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);

    let model = driver.model.read();
    let status = model.status();
    let rows = model.rows().to_vec();
    let text = model.text().to_owned();
    let exporting = model.exporting();
    let more = status == Status::List && model.has_more();
    drop(model);

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
            if status == Status::Loading {
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
                    for row in rows {
                        CardRow {
                            key: "{row.key.id()}",
                            card: row.card.card.clone(),
                            qty: row.copies,
                            expanded_card,
                            preview_card,
                            preview_dismissing,
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
        }
    }
}
