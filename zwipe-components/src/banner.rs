//! Dismissible announcement toast. Slides in, runs a countdown, and fades out
//! on timeout or close. Consumers wrap one or more in a `div.banner-stack` and
//! pass the message and call-to-action link as `children`, since routing is
//! app-specific.

use dioxus::prelude::*;

/// The colored status pill in the banner header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerStatus {
    /// Green pill, default label "Done".
    Done,
    /// Amber pill, default label "Doing".
    Doing,
}

impl BannerStatus {
    /// CSS classes for the pill. Shared with [`Panel`](crate::Panel).
    pub(crate) fn class(self) -> &'static str {
        match self {
            BannerStatus::Done => "status-tag status-done",
            BannerStatus::Doing => "status-tag status-doing",
        }
    }

    /// Label used when the caller doesn't override it.
    pub(crate) fn default_label(self) -> &'static str {
        match self {
            BannerStatus::Done => "Done",
            BannerStatus::Doing => "Doing",
        }
    }
}

/// Banner lifecycle: visible, animating out, gone.
#[derive(Clone, Copy, PartialEq)]
enum BannerState {
    Shown,
    Leaving,
    Dismissed,
}

impl BannerState {
    fn class(self) -> &'static str {
        match self {
            BannerState::Leaving => "announcement-banner banner-leaving",
            _ => "announcement-banner",
        }
    }
}

/// A dismissible announcement toast. `category` is the eyebrow, `status` the
/// pill (`status_label` overrides its text). Auto-dismisses after
/// `auto_dismiss_secs`, pausing on hover.
#[component]
pub fn Banner(
    category: String,
    status: BannerStatus,
    #[props(default)] status_label: Option<String>,
    #[props(default = 10)] auto_dismiss_secs: u32,
    children: Element,
) -> Element {
    let mut state = use_signal(|| BannerState::Shown);
    if state() == BannerState::Dismissed {
        return rsx! {};
    }
    let label = status_label.unwrap_or_else(|| status.default_label().to_string());

    rsx! {
        div {
            class: state().class(),
            onanimationend: move |evt| {
                if evt.animation_name() == "banner-leave" {
                    state.set(BannerState::Dismissed);
                }
            },
            div { class: "banner-header",
                span { class: "banner-category", "{category}" }
                span { class: status.class(), "{label}" }
            }
            span { class: "banner-text", {children} }
            button {
                class: "banner-dismiss",
                onclick: move |_| state.set(BannerState::Leaving),
                "\u{2715}"
            }
            div {
                class: "banner-progress",
                style: "animation-duration: {auto_dismiss_secs}s;",
                onanimationend: move |_| state.set(BannerState::Leaving),
            }
        }
    }
}
