//! The app's toast stack, behind the `toast` feature. Styled by
//! `assets/toast.css` ([`TOAST_CSS`](crate::TOAST_CSS)).
//!
//! Toasts collapse into a stack and expand on tap, the way grouped
//! notifications do, so three of them cost about one toast of screen.

use std::time::Duration;

use dioxus::prelude::*;
use dioxus_primitives::toast::{Toast, ToastPropsWithOwner, ToastProvider};

use crate::{TOAST_NORMAL, entrance::sleep_ms};

/// The toast provider with the collapsing stack. Wrap the router in it; every
/// `use_toast` below reaches it.
#[component]
pub fn ToastStack(
    /// How many toasts show at once.
    #[props(default = 3)]
    max_toasts: usize,
    /// How long a toast stays when its call passes no duration.
    #[props(default = TOAST_NORMAL)]
    default_duration: Duration,
    /// Fade each toast out in CSS on a life read off its type: 3s for info,
    /// 5s for the rest. For a host whose info toasts all use `TOAST_QUICK`
    /// and the rest `TOAST_NORMAL`; without it a toast drops in and the
    /// library removes it.
    #[props(default)]
    timed_fade: bool,
    children: Element,
) -> Element {
    let mut expanded = use_signal(|| false);
    // Set only for the length of a tap-driven toggle. The collapse offset is
    // a margin, and an arriving toast changes that same margin by taking over
    // as `:first-child`, so a permanent transition animated the stack
    // expanding and dropping back every time one landed. CSS cannot tell the
    // two apart; this can, because only the tap sets it.
    let mut animating = use_signal(|| false);
    let mut class = String::from("toast-container");
    if timed_fade {
        class.push_str(" timed-fade");
    }
    if expanded() {
        class.push_str(" expanded");
    }
    if animating() {
        class.push_str(" animating");
    }

    rsx! {
        ToastProvider {
            max_toasts,
            default_duration: Some(default_duration),
            class,
            // The library owns the container and list DOM and does not pass
            // event handlers through `attributes`, so the tap target has to
            // be the toast itself. `display: contents` keeps this wrapper out
            // of the layout.
            render_toast: move |props: ToastPropsWithOwner| rsx! {
                div {
                    class: "toast-tap",
                    onclick: move |_| {
                        expanded.toggle();
                        animating.set(true);
                        spawn(async move {
                            // Outlasts the 0.2s transition in toast.css.
                            sleep_ms(250).await;
                            animating.set(false);
                        });
                    },
                    Toast { ..props }
                }
            },
            {children}
        }
    }
}
