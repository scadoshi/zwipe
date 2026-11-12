//! The sheet the card filter, the deck form's pickers and the card details
//! draw: the kit's drawer along the right edge on a wide window and its
//! bottom sheet below the breakpoint, or the drawer at every width, with a
//! title bar (×, title and an optional `?`), the content and a dock at its
//! bottom right.

use dioxus::prelude::*;

use crate::{ActionDock, Button, ButtonVariant, DockPlace, SheetDim, SheetFrame, SheetWide};

/// How long the sheet takes to slide out, in milliseconds: the kit's 0.3s
/// sheet transform.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(dead_code, reason = "only the browser times the slide out")
)]
const CLOSE_MS: u32 = 300;

/// The sheet, shown while `open`. Its title bar holds the ×, then `title`
/// unless it is empty (a sheet whose content titles itself), followed by
/// `title_trailing` when given, such as a card's mana cost. The × and a
/// backdrop tap call
/// `on_close`; `on_hint`, when given, puts a `?` in the title bar's right
/// corner. `class` joins the sheet's own classes. `children` mount while
/// open and stay through the slide out after it closes, as the theme sheet
/// keeps its rows, so the sheet never slides away empty; `dock`, when given, holds the dock's actions. With
/// `every_width` it is a drawer at every width: below the kit's breakpoint
/// it floats in from the right, inset under the header, above the docks and
/// from the right edge, over most of the window's width.
///
/// `layer` is the z-index its host's overlay registration hands back
/// (`use_overlay_back`), which the sheet draws at, holding it through the
/// slide out, so the sheet opened last is on top. `dim` says whether its
/// backdrop dims the page ([`SheetDim`]).
#[component]
pub fn DrawerSheet(
    open: ReadSignal<bool>,
    #[props(into, default)] title: String,
    #[props(default)] title_trailing: Option<Element>,
    on_close: EventHandler<()>,
    #[props(default)] on_hint: Option<EventHandler<()>>,
    #[props(default)] class: &'static str,
    #[props(default)] every_width: bool,
    #[props(default)] dock: Option<Element>,
    #[props(default)] layer: Option<u32>,
    #[props(default)] dim: SheetDim,
    children: Element,
) -> Element {
    let shown = open();
    // Whether the content is mounted: from opening until the slide out ends.
    let mut content_up = use_signal(move || *open.peek());
    use_effect(move || {
        if open() {
            content_up.set(true);
        } else if *content_up.peek() {
            close_after_slide(open, content_up);
        }
    });
    let reach = if every_width {
        "drawer-every-width"
    } else {
        ""
    };
    let mut held = use_signal(|| None::<u32>);
    use_effect(use_reactive!(|layer| {
        if layer.is_some() {
            held.set(layer);
        }
    }));
    let layer_style = held()
        .filter(|_| shown || content_up())
        .map(|z| format!("--overlay-z: {z};"));
    let dim_class = match dim {
        SheetDim::Always => "",
        SheetDim::Phone => " sheet-dim-phone",
    };
    rsx! {
        SheetFrame { wide: SheetWide::Drawer,
            div {
                class: if shown { "sheet-backdrop show {reach}{dim_class}" } else { "sheet-backdrop {reach}{dim_class}" },
                style: layer_style.clone(),
                onclick: move |_| on_close.call(()),
            }
            div {
                class: if shown { "bottom-sheet show {reach} {class}" } else { "bottom-sheet {reach} {class}" },
                style: layer_style,
                div { class: "sheet-header", style: "position: relative;",
                    button {
                        class: "sheet-close",
                        "aria-label": "Close",
                        onclick: move |_| on_close.call(()),
                        "\u{00d7}"
                    }
                    if !title.is_empty() || title_trailing.is_some() {
                        span { class: "sheet-title", "{title}" {title_trailing} }
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
                    if shown || content_up() {
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

/// Unmounts the content once the slide out has run, unless the sheet opened
/// again meanwhile.
#[cfg(target_arch = "wasm32")]
fn close_after_slide(open: ReadSignal<bool>, mut content_up: Signal<bool>) {
    spawn(async move {
        gloo_timers::future::TimeoutFuture::new(CLOSE_MS).await;
        if !*open.peek() {
            content_up.set(false);
        }
    });
}

/// Off the browser nothing is timed: the content drops at once.
#[cfg(not(target_arch = "wasm32"))]
fn close_after_slide(_open: ReadSignal<bool>, mut content_up: Signal<bool>) {
    content_up.set(false);
}
