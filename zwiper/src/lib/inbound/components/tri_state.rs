use dioxus::prelude::*;

/// A 3-state filter chip selector for Option<bool> values.
///
/// Displays three mutually-exclusive chips: "yes" | "no" | "any"
/// - `Some(true)` → "yes" selected
/// - `Some(false)` → "no" selected
/// - `None` → "any" selected
#[component]
pub fn TriStateFilter(
    label: &'static str,
    value: Option<bool>,
    on_change: EventHandler<Option<bool>>,
) -> Element {
    rsx! {
        div { class: "flex-col gap-half",
            label { class: "label-xs", "{label}" }
            div { class: "flex gap-1 flex-center",
                // "yes" chip - Some(true)
                div {
                    class: if value == Some(true) { "chip selected" } else { "chip" },
                    onclick: move |_| {
                        if value == Some(true) {
                            on_change.call(None);
                        } else {
                            on_change.call(Some(true));
                        }
                    },
                    "yes"
                }
                // "no" chip - Some(false)
                div {
                    class: if value == Some(false) { "chip selected" } else { "chip" },
                    onclick: move |_| {
                        if value == Some(false) {
                            on_change.call(None);
                        } else {
                            on_change.call(Some(false));
                        }
                    },
                    "no"
                }
                // "any" chip - None
                div {
                    class: if value.is_none() { "chip selected" } else { "chip" },
                    onclick: move |_| {
                        on_change.call(None);
                    },
                    "any"
                }
            }
        }
    }
}
