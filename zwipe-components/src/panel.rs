//! Shared content panel.
//!
//! The canonical Zwipe card/dialog shape: an optional header (eyebrow + title +
//! status pill), a rule, the body, and an optional actions row under a second
//! rule — `header <hr> body <hr> actions`. Used for content cards on the site,
//! in the app, and on the portfolio so they all read as one system. `eyebrow`,
//! `title`, `status`, and `actions` are all optional; a rule is only drawn where
//! the adjacent section exists. Styling is in `assets/components.css`.

use crate::banner::BannerStatus;
use dioxus::prelude::*;

/// A content panel: `header <hr> body <hr> actions`.
///
/// The body is `children`; the footer button row is the `actions` slot (each
/// consumer supplies its own links/buttons — internal routing is app-specific).
/// Give action links/buttons the `panel-action` class for the shared pill look.
#[component]
pub fn Panel(
    /// Uppercase eyebrow label above the title (e.g. "One-Time").
    #[props(default)]
    eyebrow: Option<String>,
    /// Panel title.
    #[props(default)]
    title: Option<String>,
    /// Optional status pill shown beside the eyebrow.
    #[props(default)]
    status: Option<BannerStatus>,
    /// Overrides the status pill's default label.
    #[props(default)]
    status_label: Option<String>,
    /// Footer button row. Omit for a panel with no actions.
    #[props(default)]
    actions: Option<Element>,
    /// Body content.
    children: Element,
) -> Element {
    let has_header = eyebrow.is_some() || title.is_some() || status.is_some();

    rsx! {
        div { class: "panel-card",
            if has_header {
                div { class: "panel-head",
                    if eyebrow.is_some() || status.is_some() {
                        div { class: "panel-eyebrow-row",
                            if let Some(eyebrow) = eyebrow {
                                span { class: "panel-eyebrow", "{eyebrow}" }
                            }
                            if let Some(status) = status {
                                span {
                                    class: status.class(),
                                    {status_label.unwrap_or_else(|| status.default_label().to_string())}
                                }
                            }
                        }
                    }
                    if let Some(title) = title {
                        h3 { class: "panel-title", "{title}" }
                    }
                }
                hr { class: "panel-rule" }
            }
            div { class: "panel-body", {children} }
            if let Some(actions) = actions {
                hr { class: "panel-rule" }
                div { class: "panel-actions", {actions} }
            }
        }
    }
}
