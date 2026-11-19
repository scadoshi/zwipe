//! The picker: a sheet of choices with a search field, a drawer beside the page on a wide
//! window and a screen over it on a narrow one or the phone, or a drawer at every width. The
//! deck form's Format, Tags and Oracle tags pickers and Send to are built on it.

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

/// The picker, open on `open`. From [`PICKER_DRAWER_PX`], or at any width with `every_width`,
/// it is a [`DrawerSheet`] sliding in from the right, its title bar's × and a backdrop tap
/// calling `on_back`; below the kit's breakpoint that drawer floats along the right edge under
/// the header, as the card details do. Narrower without `every_width` it is a screen over the
/// page: it fades in with no slide, and its header's back arrow calls `on_back`. With
/// `on_save`, the dock's checkmark calls it and stays off while `unchanged`; without it the
/// picker has no dock, for a picker whose choice acts on the tap. With `hint_open`, the title
/// bar's `?` sets it. The body mounts while open (and, as a drawer, through the slide out), so
/// its keys stop answering once the picker is gone; with `search_key`, `/` focuses the search
/// field `search_id`. As a screen its back arrow takes focus as it opens and the opener gets it
/// back as it closes; as a drawer, its ×.
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
    #[props(default)] every_width: bool,
    children: Element,
) -> Element {
    let min_width = use_min_width(PICKER_DRAWER_PX);
    let drawer = every_width || min_width();
    let as_screen = use_memo(use_reactive!(
        |every_width| open() && !(every_width || min_width())
    ));
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
    if !drawer {
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

/// Records on a text field the text it holds as Enter goes down, in the capture phase, ahead of
/// every handler: in the app the field's events reach the app one at a time, and a draw of an
/// older query can land in the field between the key and the read. Installed once per page.
const ENTER_TEXT_JS: &str = "if (!window.__zwipeEnterText) { window.__zwipeEnterText = true; document.addEventListener('keydown', (e) => { if (e.key === 'Enter' && e.target instanceof HTMLInputElement) { e.target.dataset.enterText = e.target.value; } }, true); }";

/// Sends the text the field with id `id` held as Enter went down ([`ENTER_TEXT_JS`]), once, else
/// its text now, or nothing when the field is gone.
fn field_value_js(id: &str) -> String {
    format!(
        "(() => {{ const el = document.getElementById({id:?}); if (!el) {{ dioxus.send(null); return; }} const text = el.dataset.enterText ?? el.value; delete el.dataset.enterText; dioxus.send(text); }})();"
    )
}

/// Sets the text of the field with id `id` to the text the app sends next, unless it holds it.
/// It waits at the script's top level: the app closes an eval's channel once the script returns.
fn write_field_js(id: &str) -> String {
    format!(
        "const text = await dioxus.recv(); const el = document.getElementById({id:?}); if (el && el.value !== text) el.value = text;"
    )
}

/// The search every picker leads with: its `Search` label chip on the left and the filter's
/// compact field on the same line, as Quick add's row, the field's `×` easing in as text is
/// typed. The field's text is the DOM's: typing changes the query, and no draw writes the field
/// back, so a draw of an older query can't take letters out of it. The app writes the field only
/// when the query changes another way (a clear, or an opening that starts it with text). Enter
/// in the field calls `on_enter` with the field's text as it was when the key went down, which
/// the query takes on first: Go can land before the last keystroke reaches the query, so the
/// picker reads its top match from that text, not the grid last drawn.
#[component]
pub fn PickerSearch(
    id: &'static str,
    placeholder: &'static str,
    mut query: Signal<String>,
    #[props(default)] on_enter: Option<EventHandler<String>>,
) -> Element {
    use_hook(|| {
        document::eval(ENTER_TEXT_JS);
    });
    // The text the field holds as far as the app knows: what it typed, or what the app last
    // wrote. The field starts empty.
    let mut shown = use_signal(String::new);
    use_effect(move || {
        let text = query();
        if *shown.peek() != text {
            shown.set(text.clone());
            let field = document::eval(&write_field_js(id));
            let _ = field.send(text);
        }
    });
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
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    oninput: move |event| {
                        let typed = event.value();
                        shown.set(typed.clone());
                        query.set(typed);
                    },
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
                                    shown.set(typed.clone());
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

    #[component]
    fn Typed() -> Element {
        let query = use_signal(|| "draw".to_owned());
        rsx! {
            PickerSearch { id: "test-search", placeholder: "Search", query }
        }
    }

    /// The field's text is the DOM's: no draw writes it, so a draw of an older query that lands
    /// while typing can't take letters back out of the field.
    #[test]
    fn no_draw_writes_the_field_s_text() {
        let mut dom = VirtualDom::new(Typed);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("id=\"test-search\""), "{html}");
        assert!(
            !html.contains("value="),
            "the field carries no value attribute: {html}"
        );
    }

    /// The app runs an eval's script as an async function and closes its channel when that
    /// function returns, so a clear sent to a script that waits inside a nested function finds
    /// the channel closed and the field keeps its text. The write waits at the script's top level.
    #[test]
    fn the_write_waits_for_the_app_s_text_at_the_top_level() {
        let js = write_field_js("oracle-tag-search");
        assert!(js.starts_with("const text = await dioxus.recv();"), "{js}");
        assert!(
            !js.contains("=>") && !js.contains("function"),
            "no nested function the eval returns ahead of: {js}"
        );
        assert!(
            js.contains(r#"document.getElementById("oracle-tag-search")"#),
            "{js}"
        );
    }

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

    /// In the app the field can be drawn back to an older query between the key and the read:
    /// Enter takes the text the field held as the key went down, recorded before any handler.
    #[test]
    fn enter_takes_the_text_recorded_as_the_key_went_down() {
        let record = ENTER_TEXT_JS;
        assert!(record.contains("addEventListener('keydown'"), "{record}");
        assert!(
            record.contains("}, true);"),
            "a capture listener, ahead of the app's: {record}"
        );
        assert!(
            record.contains("dataset.enterText = e.target.value"),
            "{record}"
        );
        let read = field_value_js("format-search");
        let recorded = read
            .find("dataset.enterText")
            .expect("reads the recorded text");
        let live = read.find(".value").expect("falls back to the field");
        assert!(recorded < live, "the recorded text comes first: {read}");
        assert!(read.contains("delete"), "a recording is read once: {read}");
    }
}
