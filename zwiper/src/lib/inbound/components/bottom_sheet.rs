//! Reusable bottom sheet overlay component.

use dioxus::prelude::*;

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
    // On the very first render the closed sheet must carry `transition: none`,
    // then re-enable it after mount. Without this, iOS WebKit replays the
    // transform transition on insert — animating the sheet from its default
    // (up/visible) position down to translateY(100%) — so a screen that mounts a
    // sheet on startup (e.g. Home with the support button) flashes it sliding
    // away. Gating the transition on a post-mount flag stops the insert-replay
    // while keeping the normal open/close animation.
    let mut mounted = use_signal(|| false);
    use_effect(move || mounted.set(true));

    rsx! {
        div {
            class: if open() { "modal-backdrop show" } else { "modal-backdrop" },
            onclick: move |_| {
                if let Some(h) = on_dismiss { h.call(()); }
                open.set(false);
            },
        }
        div {
            class: if open() { "bottom-sheet show" } else { "bottom-sheet" },
            // When open, let the CSS `.show` rules drive the slide. When closed,
            // pin the hidden state inline; before mount also kill the transition
            // so WebKit can't replay the slide on insert (see note above).
            style: if open() {
                ""
            } else if mounted() {
                "visibility: hidden; transform: translateY(100%);"
            } else {
                "visibility: hidden; transform: translateY(100%); transition: none;"
            },
            div { class: "modal-header",
                span { style: "font-size: 1rem; color: var(--accent-tertiary);", "{title}" }
            }
            div { class: "modal-content",
                div { class: "flex-col", style: "gap: 0.5rem;",
                    {children}
                }
            }
            div { class: "util-bar",
                if let Some(f) = footer {
                    {f}
                } else {
                    button {
                        class: "util-btn",
                        onclick: move |_| open.set(false),
                        "Close"
                    }
                }
            }
        }
    }
}
