//! A one-action Danger zone panel for the foot of a list screen.

use dioxus::prelude::*;
use orangui::{Button, ButtonVariant};
use zwipe_ui::screens::deck_view::DANGER_ZONE;

/// The Danger zone panel with a single row: `label`, then a red `button`
/// that calls `onclick`, which opens the action's confirmation.
#[component]
pub fn DangerZoneAction(
    label: &'static str,
    button: &'static str,
    onclick: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "settings-list danger-zone",
            div { class: "settings-list-header",
                span { class: "settings-list-title danger-zone-title", {DANGER_ZONE} }
            }
            div { class: "settings-row",
                span { class: "settings-row-label", {label} }
                div { class: "settings-row-value",
                    Button {
                        variant: ButtonVariant::Action,
                        danger: true,
                        onclick: move |_| onclick.call(()),
                        {button}
                    }
                }
            }
        }
    }
}
