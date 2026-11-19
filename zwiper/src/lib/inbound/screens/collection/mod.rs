//! Collection: importing a ManaBox or Archidekt collection export, from a picked file or pasted
//! text, adding it to the collection or replacing the collection with it, then what landed and the
//! rows that didn't. The collection's own list follows, [`CollectionList`].
//!
//! Laid out as deck Import is: the Mode chips and their line pinned on top, the File and Paste
//! panel, the results under it, and the import as the dock's checkmark. A replace asks first.
//!
//! What the screen knows and does is `zwipe_ui::screens::collection_import`; this file draws its
//! state and performs its effects.
//!
//! The webview's own file input opens the picker: Dioxus answers a file input it drives with no
//! files on iOS and Android, so the input lives outside its tree, a tap on the File field opens it
//! from the same gesture, and the page reads the file and sends its name and text back.

/// The list under the import.
mod list;

use components::icons::{CHECK, DockIcon};
use components::{AlertDialog, Chip, CopyCap, DialogAction, Toasts, use_toast};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use serde_json::Value;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::collection::CollectionQuantity;
use zwipe_core::domain::collection::limits::MAX_BODY_BYTES;
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::http::contracts::collection::HttpImportCollection;
use zwipe_core::http::endpoints::collection::ImportCollection;
use zwipe_ui::screens::collection_import::{self, Action, Effect, State};

use self::list::CollectionList;
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::collection::CollectionCache;
use crate::inbound::components::navigation::back_handler;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::Screen;
use crate::inbound::components::ui_toast;
use crate::inbound::router::Router;
use crate::inbound::screens::deck::card::components::image_preview::ImagePreview;

/// The File field's id, which the page's tap listener looks for.
const FILE_ID: &str = "collection-file";

/// The paste field's id, which its label points at.
const PASTE_ID: &str = "collection-paste";

/// The page script behind the File field: one hidden file input on the body, outside Dioxus's
/// tree, opened by a tap on the field and emptied after each pick so the same file reads fresh.
/// A file past `cap` bytes is turned down unread. Each mount points the script at its own channel.
fn pick_script(cap: usize) -> String {
    format!(
        r##"
window.__zwipeCollectionPick = (message) => dioxus.send(message);
if (!window.__zwipeCollectionPicker) {{
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "{accept}";
    input.style.display = "none";
    document.body.appendChild(input);
    input.addEventListener("change", () => {{
        const file = input.files && input.files[0];
        input.value = "";
        if (!file) return;
        const send = (message) => window.__zwipeCollectionPick(message);
        if (file.size > {cap}) {{
            send({{ kind: "too_big" }});
            return;
        }}
        const reader = new FileReader();
        reader.onload = () => send({{ kind: "file", name: file.name, text: String(reader.result) }});
        reader.onerror = () => send({{ kind: "unreadable" }});
        reader.readAsText(file);
    }});
    document.addEventListener("click", (event) => {{
        if (event.target instanceof Element && event.target.closest("#{FILE_ID}")) input.click();
    }});
    window.__zwipeCollectionPicker = input;
}}
"##,
        accept = collection_import::ACCEPT,
    )
}

/// The action a message from [`pick_script`] stands for; `None` for one it never sends.
fn picked(message: &Value) -> Option<Action> {
    match message.get("kind")?.as_str()? {
        "file" => Some(Action::PickFile {
            name: message.get("name")?.as_str()?.to_owned(),
            text: message.get("text")?.as_str()?.to_owned(),
        }),
        "too_big" => Some(Action::PickedTooBig),
        "unreadable" => Some(Action::Unreadable),
        _ => None,
    }
}

/// The model and what its effects reach, `Copy` so handlers and tasks share it.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    confirm_open: Signal<bool>,
    authed: Authed,
    toast: Toasts,
    navigator: Navigator,
    /// What the root keeps of the collection, dropped when an import lands.
    collection: CollectionCache,
    /// How many imports have landed on this visit; the list reads its rows again on each.
    imports: Signal<u32>,
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
            Effect::ImportCollection {
                ticket,
                csv,
                mode,
                request_id,
            } => {
                spawn(async move {
                    let request = HttpImportCollection {
                        csv,
                        mode,
                        request_id: Some(request_id),
                    };
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
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
            Effect::CollectionChanged => {
                let CollectionCache(mut cache) = self.collection;
                cache.set(None);
                // The list only compares counts, so wrapping keeps it reading a change.
                let mut imports = self.imports;
                let next = imports.peek().wrapping_add(1);
                imports.set(next);
            }
        }
    }
}

/// The Collection screen.
#[component]
pub fn Collection() -> Element {
    let driver = Driver {
        model: use_signal(State::new),
        confirm_open: use_signal(|| false),
        authed: use_authed(Screen::Collection),
        toast: use_toast(),
        navigator: use_navigator(),
        collection: use_context(),
        imports: use_signal(|| 0),
    };
    use_future(move || async move {
        let mut eval = document::eval(&pick_script(MAX_BODY_BYTES));
        while let Ok(message) = eval.recv::<Value>().await {
            if let Some(action) = picked(&message) {
                driver.dispatch(action);
            }
        }
    });
    let confirm = use_callback(move |()| driver.dispatch(Action::ConfirmReplace));
    // The collection holds up to 9,999 copies of a printing; a deck's 99 doesn't apply.
    use_context_provider(|| CopyCap(CollectionQuantity::MAX));
    let preview_card: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    let preview_dismissing = use_signal(|| false);

    let model = driver.model.read();
    let mode = model.mode();
    let file_name = model.file_name().map(str::to_owned);
    let pasted = model.pasted().to_owned();
    let can_submit = model.can_submit();
    let submit_label = model.submit_label();
    let explainer = model.explainer();
    let summary = model.summary().cloned();
    drop(model);

    rsx! {
        div { class: "screen",
            ScreenHeader {
                title: collection_import::TITLE,
                back: move |()| back_handler::back_or(driver.navigator, Router::Home {}),
            }
            div { class: "screen-content content-enter",
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
                    div { class: "container-sm import-panels collection-import",
                        div { class: "form-panel",
                            div { class: "form-panel-fields",
                                div { class: "chip-row collection-file",
                                    span { class: "stat-chip stat-chip-p1 chip-row-label", {collection_import::FILE} }
                                    button {
                                        id: FILE_ID,
                                        r#type: "button",
                                        class: if file_name.is_some() { "input input-compact input-tappable" } else { "input input-compact input-placeholder input-tappable" },
                                        {file_name.as_deref().unwrap_or(collection_import::FILE_PLACEHOLDER)}
                                    }
                                }
                                div { class: "chip-row collection-paste",
                                    label { class: "stat-chip stat-chip-p1 chip-row-label", r#for: PASTE_ID, {collection_import::PASTE} }
                                }
                                textarea {
                                    id: PASTE_ID,
                                    class: "input",
                                    style: "width:100%;min-height:8rem;resize:vertical;",
                                    placeholder: collection_import::PASTE_PLACEHOLDER,
                                    value: "{pasted}",
                                    autocapitalize: "none",
                                    autocorrect: "off",
                                    spellcheck: "false",
                                    oninput: move |e| driver.dispatch(Action::Paste(e.value())),
                                }
                                p { class: "text-muted text-sm", {collection_import::SOURCES} }
                            }
                        }
                        if let Some(summary) = summary {
                            div { class: "form-panel",
                                div { class: "settings-list-header",
                                    span { class: "settings-list-title", {collection_import::IMPORTED} }
                                }
                                div { class: "form-panel-fields import-cards",
                                    for (value, what) in collection_import::count_rows(&summary) {
                                        div { class: "chip-bubble",
                                            span { class: "font-light", "{value}" }
                                            span { class: "font-light opacity-50", "{what}" }
                                        }
                                    }
                                }
                            }
                            if !summary.unmatched.is_empty() {
                                div { class: "form-panel",
                                    div { class: "settings-list-header",
                                        span { class: "settings-list-title", {collection_import::NOT_MATCHED} }
                                    }
                                    div { class: "form-panel-fields import-cards import-output",
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
                    CollectionList { imports: driver.imports, preview_card, preview_dismissing }
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
            ImagePreview { card: preview_card, dismissing: preview_dismissing }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_page_sends_each_pick_as_its_action() {
        assert!(matches!(
            picked(&json!({"kind": "file", "name": "manabox.csv", "text": "Name\n"})),
            Some(Action::PickFile { name, text }) if name == "manabox.csv" && text == "Name\n"
        ));
        assert!(matches!(
            picked(&json!({"kind": "too_big"})),
            Some(Action::PickedTooBig)
        ));
        assert!(matches!(
            picked(&json!({"kind": "unreadable"})),
            Some(Action::Unreadable)
        ));
        assert!(
            picked(&json!({"kind": "file", "name": "x.csv"})).is_none(),
            "no text"
        );
        assert!(picked(&json!({"kind": "other"})).is_none());
    }

    #[test]
    fn the_script_opens_from_the_file_field_and_caps_the_read() {
        let script = pick_script(1234);
        assert!(script.contains(&format!("closest(\"#{FILE_ID}\")")));
        assert!(script.contains("file.size > 1234"));
        assert!(script.contains(collection_import::ACCEPT));
    }
}
