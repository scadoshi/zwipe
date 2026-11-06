//! The sheet the card filter and the deck form's pickers draw: the kit's
//! drawer along the right edge on a wide window and its bottom sheet below
//! the breakpoint, with a title bar (×, title and an optional `?`), the
//! content and a dock at its bottom right.

use dioxus::prelude::*;

use crate::{ActionDock, Button, ButtonVariant, DockPlace, SheetFrame, SheetWide};

/// The sheet, shown while `open`. The title bar's × and a backdrop tap call
/// `on_close`; `on_hint`, when given, puts a `?` in the title bar's right
/// corner. `class` joins the sheet's own classes. `children` mount only
/// while open; `dock` holds the dock's actions.
#[component]
pub fn DrawerSheet(
    open: ReadSignal<bool>,
    title: &'static str,
    on_close: EventHandler<()>,
    #[props(default)] on_hint: Option<EventHandler<()>>,
    #[props(default)] class: &'static str,
    dock: Element,
    children: Element,
) -> Element {
    let shown = open();
    rsx! {
        SheetFrame { wide: SheetWide::Drawer,
            div {
                class: if shown { "sheet-backdrop show" } else { "sheet-backdrop" },
                onclick: move |_| on_close.call(()),
            }
            div { class: if shown { "bottom-sheet show {class}" } else { "bottom-sheet {class}" },
                div { class: "sheet-header", style: "position: relative;",
                    button {
                        class: "sheet-close",
                        "aria-label": "Close",
                        onclick: move |_| on_close.call(()),
                        "\u{00d7}"
                    }
                    span { class: "sheet-title", "{title}" }
                    if let Some(on_hint) = on_hint {
                        Button {
                            variant: ButtonVariant::Action,
                            class: "drawer-sheet-hint",
                            onclick: move |_| on_hint.call(()),
                            "?"
                        }
                    }
                }
                div { class: "sheet-content",
                    if shown {
                        {children}
                    }
                }
                ActionDock { place: DockPlace::Sheet, {dock} }
            }
        }
    }
}
