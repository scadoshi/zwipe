//! The sheet the card filter, the deck form's pickers and the card details
//! draw: the kit's drawer along the right edge on a wide window and its
//! bottom sheet below the breakpoint, or the drawer at every width, with a
//! title bar (×, title and an optional `?`), the content and a dock at its
//! bottom right.

use dioxus::prelude::*;

use crate::{ActionDock, Button, ButtonVariant, DockPlace, SheetFrame, SheetWide};

/// The sheet, shown while `open`. Its title bar holds the ×, then `title`
/// unless it is empty (a sheet whose content titles itself). The × and a
/// backdrop tap call
/// `on_close`; `on_hint`, when given, puts a `?` in the title bar's right
/// corner. `class` joins the sheet's own classes. `children` mount only
/// while open; `dock`, when given, holds the dock's actions. With
/// `every_width` it is a drawer at every width: below the kit's breakpoint
/// it floats in from the right, inset under the header, above the docks and
/// from the right edge, over most of the window's width.
#[component]
pub fn DrawerSheet(
    open: ReadSignal<bool>,
    #[props(into, default)] title: String,
    on_close: EventHandler<()>,
    #[props(default)] on_hint: Option<EventHandler<()>>,
    #[props(default)] class: &'static str,
    #[props(default)] every_width: bool,
    #[props(default)] dock: Option<Element>,
    children: Element,
) -> Element {
    let shown = open();
    let reach = if every_width {
        "drawer-every-width"
    } else {
        ""
    };
    rsx! {
        SheetFrame { wide: SheetWide::Drawer,
            div {
                class: if shown { "sheet-backdrop show {reach}" } else { "sheet-backdrop {reach}" },
                onclick: move |_| on_close.call(()),
            }
            div { class: if shown { "bottom-sheet show {reach} {class}" } else { "bottom-sheet {reach} {class}" },
                div { class: "sheet-header", style: "position: relative;",
                    button {
                        class: "sheet-close",
                        "aria-label": "Close",
                        onclick: move |_| on_close.call(()),
                        "\u{00d7}"
                    }
                    if !title.is_empty() {
                        span { class: "sheet-title", "{title}" }
                    }
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
                if let Some(dock) = dock {
                    ActionDock { place: DockPlace::Sheet, {dock} }
                }
            }
        }
    }
}
