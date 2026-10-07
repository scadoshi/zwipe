//! Slide-up bottom sheet, for the apps. Styled by `assets/app.css`.

use dioxus::prelude::*;

use crate::{
    ActionBar, Button, ButtonVariant, entrance::sleep_ms, overlay_stack::use_overlay_back_action,
};

/// A bottom sheet with backdrop, title, content slot, and footer.
///
/// `footer` overrides the default single "Close" button (e.g. a Back/Save
/// pair). `on_dismiss` fires when the backdrop is tapped or the OS back
/// gesture lands, and owns the close: a sheet that sets it closes itself,
/// which lets the theme sheet hold still through the wipe that restores a
/// discarded pick. `hidden` drops the sheet and backdrop with no slide, for a
/// sheet that leaves inside that wipe so the new snapshot is taken without it.
///
/// Registers with the [`OverlayBackStack`](crate::OverlayBackStack), so the
/// host must provide one above it.
#[component]
pub fn BottomSheet(
    mut open: Signal<bool>,
    title: String,
    children: Element,
    footer: Option<Element>,
    on_dismiss: Option<EventHandler<()>>,
    #[props(default)] hidden: bool,
    /// A hint's open signal. Given one, the sheet's header carries the same
    /// "?" the page headers do; without one, the corner stays empty.
    hint: Option<Signal<bool>>,
) -> Element {
    // The OS back gesture closes the sheet the way a backdrop tap does.
    let dismiss = use_callback(move |()| {
        let mut open = open;
        match on_dismiss {
            Some(h) => h.call(()),
            None => open.set(false),
        }
    });
    use_overlay_back_action(open.into(), dismiss);

    // The first render carries `transition: none` through the premount class,
    // dropped once mounted. Without it iOS WebKit replays the transform
    // transition on insert and a freshly mounted sheet visibly slides away.
    // The flag has to flip after WebKit's first post-insert paint, so it
    // waits a couple of frames. It is a class rather than an inline style
    // because clearing an inline `transition: none` does not reliably take in
    // that WebView, and the rule would linger and kill every slide.
    let mut mounted = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            sleep_ms(50).await;
            mounted.set(true);
        });
    });

    rsx! {
        div {
            class: if hidden {
                "modal-backdrop snap"
            } else if open() {
                "modal-backdrop show"
            } else {
                "modal-backdrop"
            },
            onclick: move |_| dismiss.call(()),
        }
        div {
            class: if hidden {
                "bottom-sheet snap"
            } else if open() {
                "bottom-sheet show"
            } else if mounted() {
                "bottom-sheet"
            } else {
                "bottom-sheet bottom-sheet-premount"
            },
            div { class: "modal-header",
                span { class: "modal-title", "{title}" }
                if let Some(mut hint) = hint {
                    Button {
                        variant: ButtonVariant::Util,
                        class: "page-header-corner",
                        onclick: move |_| hint.set(true),
                        "?"
                    }
                }
            }
            div { class: "modal-content",
                div { class: "flex-col", style: "gap: 0.5rem;", {children} }
            }
            ActionBar {
                if let Some(f) = footer {
                    {f}
                } else {
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| open.set(false),
                        "Close"
                    }
                }
            }
        }
    }
}
