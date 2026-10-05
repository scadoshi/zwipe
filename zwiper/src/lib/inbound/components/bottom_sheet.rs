//! Reusable bottom sheet overlay component.

use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use zwipe_components::{ActionBar, Button, ButtonVariant};

use crate::inbound::components::navigation::overlay_stack::use_overlay_back_action;

/// A slide-up bottom sheet with backdrop, title, content slot, and footer.
///
/// `footer` overrides the default single "Close" button (e.g. a Back/Save pair).
/// `on_dismiss` fires when the backdrop is tapped or the OS back gesture lands,
/// and owns the close: a sheet that sets it closes itself, which lets the
/// Themes sheet hold still through the wipe that restores a discarded pick.
/// `hidden` drops the sheet and backdrop with no slide, for a sheet that leaves
/// inside that wipe so the new snapshot is taken without it.
/// `hint` renders a grayed "?" at the header's right edge that opens the given
/// dialog signal, mirroring `ScreenHeader`'s affordance.
#[component]
pub fn BottomSheet(
    mut open: Signal<bool>,
    title: String,
    children: Element,
    footer: Option<Element>,
    on_dismiss: Option<EventHandler<()>>,
    #[props(default)] hidden: bool,
    hint: Option<Signal<bool>>,
) -> Element {
    // The OS back gesture closes the sheet before the router sees it, exactly
    // as tapping the backdrop does. Registered here rather than per-screen so
    // every sheet in the app inherits it; a sheet that has to remember its own
    // hook is a sheet that eventually forgets.
    let dismiss = use_callback(move |_: ()| {
        let mut open = open;
        match on_dismiss {
            Some(h) => h.call(()),
            None => open.set(false),
        }
    });
    use_overlay_back_action(open.into(), dismiss);

    // On the first render the sheet carries `transition: none` (via the
    // `bottom-sheet-premount` class), then drops it once mounted. Without this,
    // iOS WebKit replays the transform transition on insert, animating the
    // sheet from its default position down to translateY(100%), so a screen
    // that mounts a sheet on startup (e.g. Home with the support button on an
    // authenticated launch) flashes it sliding away. The flag must flip *after*
    // WebKit's first post-insert paint: a synchronous `use_effect` re-enables
    // the transition before that paint and the replay still shows, so we defer
    // a couple frames. This is a class, not an inline style; clearing an inline
    // `transition: none` back to empty doesn't reliably take in this WebView, so
    // it would linger and kill every sheet's open/close animation.
    let mut mounted = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            sleep(Duration::from_millis(50)).await;
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
            // Before mount, add `bottom-sheet-premount` (CSS `transition: none`)
            // so WebKit can't replay the slide on insert (see note above); the
            // class drops after mount so the normal open/close slide animates.
            // This is a class, not an inline style, because clearing an inline
            // `transition: none` back to empty doesn't reliably take in this
            // WebView: the rule lingers and kills every sheet's animation.
            class: if hidden {
                "bottom-sheet snap"
            } else if open() {
                "bottom-sheet show"
            } else if mounted() {
                "bottom-sheet"
            } else {
                "bottom-sheet bottom-sheet-premount"
            },
            div { class: "modal-header", style: "position: relative;",
                span { style: "font-size: 1rem; color: var(--accent-tertiary);", "{title}" }
                if let Some(mut hint_open) = hint {
                    Button {
                        variant: ButtonVariant::Util,
                        style: "position: absolute; right: 1rem; top: 50%; transform: translateY(-50%); opacity: 0.55; padding: 0.2rem 0.6rem;",
                        onclick: move |_| hint_open.set(true),
                        "?"
                    }
                }
            }
            div { class: "modal-content",
                div { class: "flex-col", style: "gap: 0.5rem;",
                    {children}
                }
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
