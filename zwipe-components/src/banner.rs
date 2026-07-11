//! Shared announcement banner.
//!
//! A dismissible toast for site announcements (a new release, a featured
//! project). It self-manages its lifecycle: slides in, runs a countdown, and on
//! either the countdown finishing or the user pressing close it fades and
//! collapses out of the stack. Consumers wrap one or more in a
//! `div.banner-stack` (the fixed-position column is a site layout concern) and
//! pass the message plus their own call-to-action link as `children` — the link
//! can't live here because internal routing is app-specific. Styling lives in
//! `assets/components.css`.

use dioxus::prelude::*;

/// The colored status pill shown in the banner header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerStatus {
    /// Shipped / live (green pill), default label "Live".
    Done,
    /// In progress (amber pill), default label "Doing".
    Doing,
}

impl BannerStatus {
    /// CSS classes for the pill. `pub(crate)` so [`Panel`](crate::Panel) renders
    /// an identical status pill.
    pub(crate) fn class(self) -> &'static str {
        match self {
            BannerStatus::Done => "status-tag status-done",
            BannerStatus::Doing => "status-tag status-doing",
        }
    }

    /// Label used when the caller doesn't override it.
    pub(crate) fn default_label(self) -> &'static str {
        match self {
            BannerStatus::Done => "Live",
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

/// A dismissible announcement toast.
///
/// `category` is the eyebrow label, `status` the colored pill (`status_label`
/// overrides its text). The message and its call-to-action link go in
/// `children`. The `.banner-progress` bar auto-dismisses after
/// `auto_dismiss_secs` (pauses on hover); the `✕` button dismisses immediately.
/// Wrap one or more in a `div.banner-stack`.
#[component]
pub fn Banner(
    category: String,
    status: BannerStatus,
    #[props(default)] status_label: Option<String>,
    #[props(default = 10)] auto_dismiss_secs: u32,
    children: Element,
) -> Element {
    let mut state = use_signal(|| BannerState::Shown);
    // Once dismissed the element leaves the DOM; the `.banner-leave` animation
    // has already collapsed the stack gap by then.
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
