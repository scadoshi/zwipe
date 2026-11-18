//! Collection: importing a ManaBox or Archidekt collection export, adding it to the collection or
//! replacing the collection with it, then what landed and the rows that didn't. Web only.
//!
//! Laid out as deck Import is: the Mode chips and their line on top, the file field in a panel,
//! the results beside it on a wide window, and the import as the dock's checkmark. A replace asks
//! first.

use components::icons::{CHECK, DockIcon};
use components::{AlertDialog, Chip, DialogAction};
use dioxus::prelude::*;
use dioxus_primitives::toast::{Toasts, use_toast};
use zwipe_core::domain::collection::limits::MAX_BODY_BYTES;
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::http::contracts::collection::HttpImportCollection;
use zwipe_core::http::endpoints::collection::ImportCollection;
use zwipe_ui::screens::collection_import::{self, Action, Effect, State};
use zwipe_ui::telemetry::vocabulary::Screen;

use crate::inbound::chrome::authed::{Authed, use_authed};
use crate::inbound::chrome::toast::raise;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

/// The file input's id, which both labels point at.
const FILE_ID: &str = "collection-file";

/// The model and what its effects reach, `Copy` so handlers and tasks share it.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    confirm_open: Signal<bool>,
    authed: Authed,
    toast: Toasts,
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
            Effect::ImportCollection { ticket, csv, mode } => {
                spawn(async move {
                    let request = HttpImportCollection { csv, mode };
                    let reply = self
                        .authed
                        .try_run("import_collection", |c, s| async move {
                            c.call(ImportCollection(request), Some(&s)).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Ok(summary) => Action::Imported { ticket, summary },
                        Err(_) => Action::Failed { ticket },
                    });
                });
            }
            Effect::ConfirmReplace => {
                let mut open = self.confirm_open;
                open.set(true);
            }
            Effect::Toast(toast) => raise(self.toast, &toast),
        }
    }
}

/// The Collection screen.
#[component]
pub fn Collection() -> Element {
    telemetry::use_telemetry_screen(Screen::Collection);
    let driver = Driver {
        model: use_signal(State::new),
        confirm_open: use_signal(|| false),
        authed: use_authed(),
        toast: use_toast(),
    };
    let confirm = use_callback(move |()| driver.dispatch(Action::ConfirmReplace));

    let model = driver.model.read();
    let mode = model.mode();
    let file_name = model.file_name().map(str::to_owned);
    let can_submit = model.can_submit();
    let submit_label = model.submit_label();
    let explainer = model.explainer();
    let summary = model.summary().cloned();
    drop(model);

    rsx! {
        ScreenFrame { title: collection_import::TITLE,
            div { class: "screen-content content-enter web-import",
                div { class: "import-controls",
                    div { class: "chip-row",
                        span { class: "stat-chip stat-chip-p1 chip-row-label", {collection_import::MODE} }
                        for (label, value) in [("Add", ImportMode::Add), ("Replace", ImportMode::Replace)] {
                            Chip {
                                key: "{label}",
                                selected: mode == value,
                                onclick: move |_| driver.dispatch(Action::PickMode(value)),
                                "{label}"
                            }
                        }
                    }
                    p { class: "text-muted text-sm import-hint", "{explainer}" }
                }
                div { class: "import-scroll",
                    div { class: "container-sm web-import-body",
                        div { class: "web-form-panel web-import-input",
                            div { class: "web-form-fields",
                                div { class: "chip-row collection-file",
                                    label { class: "stat-chip stat-chip-p1 chip-row-label", r#for: FILE_ID, {collection_import::FILE} }
                                    input {
                                        id: FILE_ID,
                                        class: "sr-only",
                                        r#type: "file",
                                        accept: collection_import::ACCEPT,
                                        onchange: move |event: FormEvent| async move {
                                            let Some(file) = event.files().into_iter().next() else {
                                                return;
                                            };
                                            if u64::try_from(MAX_BODY_BYTES).is_ok_and(|cap| file.size() > cap) {
                                                driver.dispatch(Action::PickedTooBig);
                                                return;
                                            }
                                            match file.read_string().await {
                                                Ok(text) => driver.dispatch(Action::PickFile { name: file.name(), text }),
                                                Err(_) => driver.dispatch(Action::Unreadable),
                                            }
                                        },
                                    }
                                    if let Some(name) = file_name {
                                        label { class: "input input-compact input-tappable", r#for: FILE_ID, "{name}" }
                                    } else {
                                        label { class: "input input-compact input-placeholder input-tappable", r#for: FILE_ID, {collection_import::FILE_PLACEHOLDER} }
                                    }
                                }
                                p { class: "text-muted text-sm", {collection_import::SOURCES} }
                            }
                        }
                        div { class: "web-import-results",
                            if let Some(summary) = summary {
                                div { class: "web-form-panel",
                                    div { class: "settings-list-header web-form-head",
                                        span { class: "settings-list-title", {collection_import::IMPORTED} }
                                    }
                                    div { class: "web-form-fields web-import-cards",
                                        for (value, what) in collection_import::count_rows(&summary) {
                                            div { class: "chip-bubble",
                                                span { class: "font-light", "{value}" }
                                                span { class: "font-light opacity-50", "{what}" }
                                            }
                                        }
                                    }
                                }
                                if !summary.unmatched.is_empty() {
                                    div { class: "web-form-panel",
                                        div { class: "settings-list-header web-form-head",
                                            span { class: "settings-list-title", {collection_import::NOT_MATCHED} }
                                        }
                                        div { class: "web-form-fields web-import-cards import-output",
                                            for (row, why) in collection_import::unmatched_rows(&summary) {
                                                div { class: "chip-bubble-error",
                                                    span { class: "font-light", "{row}" }
                                                    span { class: "font-light opacity-50", "{why}" }
                                                }
                                            }
                                            if let Some(more) = collection_import::more_unmatched(&summary) {
                                                p { class: "text-muted text-sm", "{more}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "action-dock action-dock-right",
                button {
                    class: "action-dock-btn",
                    "aria-label": submit_label,
                    disabled: !can_submit,
                    onclick: move |_| driver.dispatch(Action::Submit),
                    DockIcon { path: CHECK }
                }
            }
        }
        AlertDialog {
            open: driver.confirm_open,
            title: collection_import::REPLACE_TITLE.to_owned(),
            actions: vec![
                DialogAction::Dismiss(collection_import::CANCEL.into()),
                DialogAction::Confirm {
                    label: collection_import::REPLACE.into(),
                    danger: true,
                    on_confirm: confirm,
                },
            ],
            {collection_import::REPLACE_BODY}
        }
    }
}
