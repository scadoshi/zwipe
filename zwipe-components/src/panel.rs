//! Shared content panel.
//!
//! The Zwipe card/dialog shape, `header <hr> body <hr> actions`. Header and
//! actions are optional; a rule is only drawn where the adjacent section exists.

use crate::banner::BannerStatus;
use dioxus::prelude::*;

/// A content panel. The body is `children`; `actions` is the footer button
/// row, whose links get the `panel-action` class for the shared pill look.
#[component]
pub fn Panel(
    /// Uppercase label above the title.
    #[props(default)]
    eyebrow: Option<String>,
    /// Panel title.
    #[props(default)]
    title: Option<String>,
    /// Render the title as an `h1`, for page heroes where it is the document
    /// heading; an `h3`-only page reads as a fragment to crawlers.
    #[props(default = false)]
    title_h1: bool,
    /// Status pill beside the eyebrow.
    #[props(default)]
    status: Option<BannerStatus>,
    /// Overrides the pill's default label.
    #[props(default)]
    status_label: Option<String>,
    /// Footer button row.
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
                        if title_h1 {
                            h1 { class: "panel-title", "{title}" }
                        } else {
                            h3 { class: "panel-title", "{title}" }
                        }
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
