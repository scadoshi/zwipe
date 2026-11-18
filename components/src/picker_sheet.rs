//! The picker: a sheet of choices with a search field, a drawer beside the page on a wide
//! window and a screen over it on a narrow one or the phone. The deck form's Format, Tags and
//! Oracle tags pickers and Send to are built on it.

use dioxus::prelude::*;

use crate::icons::CHECK;
use crate::keys::{SHEET_ENTER_OR_CLOSE, focus_by_id, use_page_keys, use_sheet_focus};
use crate::{
    ActionDock, BackArrow, ClearWrap, DockAction, DrawerSheet, GridCells, PageHeader, use_min_width,
};

/// The width, in pixels, from which a picker is a drawer beside the page; narrower it is a
/// screen over it.
pub const PICKER_DRAWER_PX: u32 = 600;

/// What takes focus as a picker opens as a screen: its back arrow, so no field raises a phone's
/// keyboard.
const SCREEN_ENTRY: &str = ".picker-screen.show .page-header-support";

/// The picker, open on `open`. From [`PICKER_DRAWER_PX`] it is a [`DrawerSheet`] at every
/// width, sliding in from the right, its title bar's × and a backdrop tap calling `on_back`.
/// Narrower it is a screen over the page: it fades in with no slide, and its header's back arrow
/// calls `on_back`. With `on_save`, the dock's checkmark calls it and stays off while
/// `unchanged`; without it the picker has no dock, for a picker whose choice acts on the tap.
/// With `hint_open`, the title bar's `?` sets it. The body mounts while open (and, as a drawer,
/// through the slide out), so its keys stop answering once the picker is gone; with
/// `search_key`, `/` focuses the search field `search_id`. As a screen its back arrow takes
/// focus as it opens and the opener gets it back as it closes; as a drawer, its ×.
#[component]
pub fn PickerSheet(
    open: Signal<bool>,
    #[props(into)] title: String,
    search_id: &'static str,
    #[props(default)] hint_open: Option<Signal<bool>>,
    #[props(default)] unchanged: bool,
    on_back: EventHandler<()>,
    #[props(default)] on_save: Option<EventHandler<()>>,
    #[props(default = true)] search_key: bool,
    #[props(default)] layer: Option<u32>,
    children: Element,
) -> Element {
    let wide = use_min_width(PICKER_DRAWER_PX);
    let as_screen = use_memo(move || open() && !wide());
    use_sheet_focus(as_screen.into(), SCREEN_ENTRY);
    let save = on_save.map(|on_save| {
        rsx! {
            DockAction {
                label: "Save",
                icon: CHECK,
                disabled: unchanged,
                class: SHEET_ENTER_OR_CLOSE.to_owned(),
                onclick: move |()| on_save.call(()),
            }
        }
    });
    let on_hint = hint_open.map(|mut hint_open| Callback::new(move |()| hint_open.set(true)));
    if !wide() {
        let shown = open();
        return rsx! {
            div {
                class: if shown { "screen swipe-select-screen picker-screen show" } else { "screen swipe-select-screen picker-screen" },
                style: layer.map(|z| format!("--overlay-z: {z};")),
                GridCells { still: true }
                if shown {
                    PageHeader {
                        title,
                        leading: rsx! {
                            BackArrow { class: "page-header-support", onclick: move |()| on_back.call(()) }
                        },
                        on_hint,
                    }
                    div { class: "screen-content content-enter",
                        PickerBody { search_id, search_key, {children} }
                    }
                    if let Some(save) = save {
                        ActionDock { {save} }
                    }
                }
            }
        };
    }
    rsx! {
        DrawerSheet {
            open,
            layer,
            title,
            on_close: move |()| on_back.call(()),
            on_hint: on_hint.map(|hint| EventHandler::new(move |()| hint.call(()))),
            class: "picker-sheet",
            every_width: true,
            dock: save,
            PickerBody { search_id, search_key, {children} }
        }
    }
}

/// The open picker's body, with `/` focusing its search while `search_key`.
#[component]
fn PickerBody(search_id: &'static str, search_key: bool, children: Element) -> Element {
    use_page_keys(Callback::new(move |key: String| {
        if search_key && key == "/" {
            focus_by_id(search_id);
        }
    }));
    rsx! {
        div { class: "picker-body", {children} }
    }
}

/// Sends the text in the field with id `id` as typed, or nothing when the field is gone.
fn field_value_js(id: &str) -> String {
    format!("dioxus.send(document.getElementById({id:?})?.value ?? null);")
}

/// The search every picker leads with: its `Search` label chip on the left and the filter's
/// compact field on the same line, as Quick add's row, the field's `×` easing in as text is
/// typed. Enter in the field calls `on_enter` with the field's text as it is at that moment,
/// which the query takes on first: Go can land before the last keystroke reaches the query, so
/// the picker reads its top match from that text, not the grid last drawn.
#[component]
pub fn PickerSearch(
    id: &'static str,
    placeholder: &'static str,
    mut query: Signal<String>,
    #[props(default)] on_enter: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        div { class: "chip-row",
            label { class: "stat-chip stat-chip-p1 chip-row-label", r#for: id, "Search" }
            ClearWrap {
                filled: !query.read().is_empty(),
                on_clear: move |()| query.set(String::new()),
                input {
                    class: "input input-compact",
                    id,
                    r#type: "text",
                    placeholder,
                    value: "{query}",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    oninput: move |event| query.set(event.value()),
                    onkeydown: move |event| {
                        if event.key() == Key::Enter && !event.is_composing()
                            && let Some(on_enter) = on_enter
                        {
                            event.prevent_default();
                            spawn(async move {
                                let mut field = document::eval(&field_value_js(id));
                                let typed = match field.recv::<Option<String>>().await {
                                    Ok(Some(typed)) => typed,
                                    _ => query.peek().clone(),
                                };
                                if *query.peek() != typed {
                                    query.set(typed.clone());
                                }
                                on_enter.call(typed);
                            });
                        }
                    },
                }
            }
        }
    }
}

/// One choice in a picker's grid, marked as the one Enter acts on while `target`, which the
/// stylesheet shows while the search has focus. The wrapper takes no box of its own.
#[component]
pub fn PickerChoice(target: bool, children: Element) -> Element {
    rsx! {
        span { class: if target { "picker-choice enter-target" } else { "picker-choice" }, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Go can land in the same instant as the typing, before the field's text reaches the
    /// query: Enter reads the field itself.
    #[test]
    fn enter_reads_the_field_s_text_as_typed() {
        let js = field_value_js("send-to-search");
        assert!(
            js.contains(r#"document.getElementById("send-to-search")"#),
            "{js}"
        );
        assert!(js.contains(".value"), "{js}");
    }
}
