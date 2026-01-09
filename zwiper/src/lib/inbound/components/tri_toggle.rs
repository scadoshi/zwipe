use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

/// A 3-state filter chip selector for Option<bool> values.
///
/// Displays three mutually-exclusive chips with programmable labels:
/// - `Some(true)` → true_label (default: "true")
/// - `Some(false)` → false_label (default: "false")
/// - `None` → none_label (default: "none")
#[allow(unpredictable_function_pointer_comparisons)]
#[component]
pub fn TriToggle(
    label: &'static str,
    filter_builder: Signal<CardFilterBuilder>,
    getter: fn(&CardFilterBuilder) -> Option<bool>,
    setter_true: fn(&mut CardFilterBuilder),
    setter_false: fn(&mut CardFilterBuilder),
    unsetter: fn(&mut CardFilterBuilder),
    #[props(default = "true")] true_label: &'static str,
    #[props(default = "false")] false_label: &'static str,
    #[props(default = "none")] none_label: &'static str,
) -> Element {
    let value = getter(&filter_builder());

    rsx! {
        div { class: "flex-col gap-half",
            label { class: "label-xs", "{label}" }
            div { class: "flex gap-1 flex-center",
                // true chip - Some(true)
                div {
                    class: if value == Some(true) { "chip selected" } else { "chip" },
                    onclick: move |_| {
                        if value == Some(true) {
                            unsetter(&mut filter_builder.write());
                        } else {
                            setter_true(&mut filter_builder.write());
                        }
                    },
                    "{true_label}"
                }
                // false chip - Some(false)
                div {
                    class: if value == Some(false) { "chip selected" } else { "chip" },
                    onclick: move |_| {
                        if value == Some(false) {
                            unsetter(&mut filter_builder.write());
                        } else {
                            setter_false(&mut filter_builder.write());
                        }
                    },
                    "{false_label}"
                }
                // none chip - None
                div {
                    class: if value.is_none() { "chip selected" } else { "chip" },
                    onclick: move |_| {
                        unsetter(&mut filter_builder.write());
                    },
                    "{none_label}"
                }
            }
        }
    }
}
