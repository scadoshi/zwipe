//! A screen that is a page below the breakpoint and a modal over the page
//! behind it on a wide window: Import and Export over the deck view.

use components::icons::DockIcon;
use components::{BottomSheet, SheetFrame, SheetWide};
use dioxus::prelude::*;

use crate::inbound::chrome::media::use_wide;
use crate::inbound::shell::ScreenFrame;

/// One floating action: the dock's button at the page's bottom right, or
/// the modal's.
#[derive(Debug, Clone, PartialEq)]
pub struct DockButton {
    /// What the button does, for assistive tech.
    pub label: String,
    /// The icon's path, from `components::icons`.
    pub icon: &'static str,
    /// Greys the button out.
    pub disabled: bool,
    /// Fires on click.
    pub onclick: Callback<()>,
}

/// The screen: `title`, `description` and `on_hint` as [`ScreenFrame`]
/// takes them, the `children` as the body, and `dock` as the one floating
/// button. On a wide window the body sits in a centered modal with the
/// `title` in its bar, the `?` beside it, an × that calls `on_close`, and
/// the dock in its corner; `class` joins the modal's classes, for a width.
#[component]
pub fn ModalScreen(
    title: String,
    description: String,
    hint_open: Signal<bool>,
    dock: DockButton,
    on_close: Callback<()>,
    #[props(default)] class: Option<String>,
    children: Element,
) -> Element {
    let wide = use_wide();
    let open = use_signal(|| true);
    let on_hint = use_callback(move |()| {
        let mut hint_open = hint_open;
        hint_open.set(true);
    });
    let DockButton {
        label,
        icon,
        disabled,
        onclick,
    } = dock;
    let button = rsx! {
        button {
            class: "action-dock-btn",
            "aria-label": label,
            disabled,
            onclick: move |_| onclick.call(()),
            DockIcon { path: icon }
        }
    };
    if wide() {
        let class = match class {
            Some(extra) => format!("web-modal-screen {extra}"),
            None => "web-modal-screen".to_owned(),
        };
        rsx! {
            document::Title { "{title} | Zwipe" }
            div { class,
                SheetFrame { wide: SheetWide::Modal,
                    BottomSheet {
                        open,
                        title,
                        closable: true,
                        hint: hint_open,
                        on_dismiss: move |()| on_close.call(()),
                        div { class: "web-modal-body", {children} }
                        div { class: "action-dock action-dock-right sheet-dock", {button} }
                    }
                }
            }
        }
    } else {
        rsx! {
            ScreenFrame { title, description, on_hint,
                {children}
                div { class: "action-dock action-dock-right", {button} }
            }
        }
    }
}
