//! Reusable bottom sheet overlay component.

use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use zwipe_components::{ActionBar, Button, ButtonVariant};

/// A slide-up bottom sheet with backdrop, title, content slot, and footer.
///
/// `footer` overrides the default single "Close" button (e.g. a Back/Save pair).
/// `on_dismiss` fires when the backdrop is tapped, before the sheet closes.
#[component]
pub fn BottomSheet(
    mut open: Signal<bool>,
    title: String,
    children: Element,
    footer: Option<Element>,
    on_dismiss: Option<EventHandler<()>>,
) -> Element {
    // On the first render the sheet carries `transition: none` (via the
    // `bottom-sheet-premount` class), then drops it once mounted. Without this,
    // iOS WebKit replays the transform transition on insert — animating the
    // sheet from its default position down to translateY(100%) — so a screen
    // that mounts a sheet on startup (e.g. Home with the support button on an
    // authenticated launch) flashes it sliding away. The flag must flip *after*
    // WebKit's first post-insert paint: a synchronous `use_effect` re-enables
    // the transition before that paint and the replay still shows, so we defer
    // a couple frames. This is a class, not an inline style — clearing an inline
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
            class: if open() { "modal-backdrop show" } else { "modal-backdrop" },
            onclick: move |_| {
                if let Some(h) = on_dismiss { h.call(()); }
                open.set(false);
            },
        }
        div {
            // Before mount, add `bottom-sheet-premount` (CSS `transition: none`)
            // so WebKit can't replay the slide on insert (see note above); the
            // class drops after mount so the normal open/close slide animates.
            // This is a class, not an inline style, because clearing an inline
            // `transition: none` back to empty doesn't reliably take in this
            // WebView — the rule lingers and kills every sheet's animation.
            class: if open() {
                "bottom-sheet show"
            } else if mounted() {
                "bottom-sheet"
            } else {
                "bottom-sheet bottom-sheet-premount"
            },
            div { class: "modal-header",
                span { style: "font-size: 1rem; color: var(--accent-tertiary);", "{title}" }
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
