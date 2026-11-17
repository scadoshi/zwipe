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

/// The search input every picker leads with, its `×` easing in as text is typed.
#[component]
pub fn PickerSearch(
    id: &'static str,
    placeholder: &'static str,
    mut query: Signal<String>,
) -> Element {
    rsx! {
        ClearWrap {
            filled: !query.read().is_empty(),
            on_clear: move |()| query.set(String::new()),
            input {
                class: "input",
                id,
                r#type: "text",
                placeholder,
                value: "{query}",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| query.set(event.value()),
            }
        }
    }
}
