//! Deck warnings section with action buttons for card-specific warnings.

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use zwipe_core::domain::deck::deck_warning::DeckWarning;
use zwipe_ui::deck::warning::{self, Fix};
use zwipe_ui::screens::deck_view::{self, Action};

/// The fix buttons' warning color.
const FIX_STYLE: &str =
    "color: var(--color-warning); border-color: var(--border-warning); margin-bottom: 0;";

#[component]
pub(crate) fn DeckWarnings(
    warnings: Vec<DeckWarning>,
    ondispatch: EventHandler<Action>,
) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;",
            for warning in warnings.iter() {
                div { class: "settings-row",
                    style: "justify-content: space-between; gap: 0.5rem;",
                    span { class: "text-muted", "{warning}" }
                    // The below-minimum warning's two fixes sit together;
                    // every other warning has one.
                    if warning::fix(warning) == Some(Fix::BelowMinimum) {
                        span {
                            style: "display: flex; gap: 0.5rem;",
                            for (label, action) in deck_view::fix_buttons(warning) {
                                Button {
                                    variant: ButtonVariant::Small,
                                    style: FIX_STYLE,
                                    onclick: move |_| ondispatch.call(action.clone()),
                                    "{label}"
                                }
                            }
                        }
                    } else {
                        for (label, action) in deck_view::fix_buttons(warning) {
                            Button {
                                variant: ButtonVariant::Small,
                                style: FIX_STYLE,
                                onclick: move |_| ondispatch.call(action.clone()),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
