//! The app's `More actions` sheet, as a popover under `More` on a wide
//! window and the app's bottom sheet below the breakpoint.

use std::rc::Rc;

use components::{BottomSheet, Button, ButtonVariant};
use dioxus::prelude::*;

/// Where the popover sits, in window pixels: under the `More` button's left
/// edge.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Anchor {
    top: f64,
    left: f64,
}

/// One screen's `More` menu: whether it is open, and the button it hangs
/// from. `Copy`, so the button and the menu share it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoreMenuState {
    /// Whether the menu shows.
    pub open: Signal<bool>,
    button: Signal<Option<Rc<MountedData>>>,
    anchor: Signal<Anchor>,
}

/// A closed menu for one screen.
pub fn use_more_menu() -> MoreMenuState {
    MoreMenuState {
        open: use_signal(|| false),
        button: use_signal(|| None),
        anchor: use_signal(Anchor::default),
    }
}

/// Pixels between the button's bottom edge and the popover.
const GAP_PX: f64 = 6.0;

/// The `More` button, for the screen's action bar. A click places the
/// popover under it and opens the menu.
#[component]
pub fn MoreButton(menu: MoreMenuState) -> Element {
    let MoreMenuState {
        mut open,
        mut button,
        mut anchor,
    } = menu;
    rsx! {
        span {
            class: "web-more-anchor",
            onmounted: move |event| button.set(Some(event.data())),
            Button {
                variant: ButtonVariant::Action,
                onclick: move |_| {
                    let mounted = button.peek().clone();
                    spawn(async move {
                        if let Some(mounted) = mounted
                            && let Ok(rect) = mounted.get_client_rect().await
                        {
                            anchor.set(Anchor {
                                top: rect.max_y() + GAP_PX,
                                left: rect.min_x(),
                            });
                        }
                        open.set(true);
                    });
                },
                "More"
            }
        }
    }
}

/// The menu's items in `children`, in the app's order. A wide window shows
/// them in a popover under the [`MoreButton`] that a click anywhere else
/// closes; a narrow one in the app's `More` bottom sheet with its `Close`.
/// Esc closes either. Render it beside the screen's action bar, not in it,
/// since the bar is a containing block for fixed elements.
#[component]
pub fn MoreMenu(menu: MoreMenuState, children: Element) -> Element {
    let mut open = menu.open;
    let Anchor { top, left } = (menu.anchor)();
    rsx! {
        div { class: "web-more-sheet",
            BottomSheet { open, title: "More".to_string(), {children.clone()} }
        }
        if open() {
            div { class: "web-more-popover-layer",
                div {
                    class: "web-more-catcher",
                    onclick: move |_| open.set(false),
                }
                div {
                    class: "web-more-popover",
                    role: "menu",
                    style: "top: {top}px; left: {left}px;",
                    {children}
                }
            }
        }
    }
}
