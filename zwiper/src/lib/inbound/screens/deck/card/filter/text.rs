use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

#[component]
pub fn Text() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row",
                label { class: "label-xs", r#for: "name-contains", "name contains" }
                if filter_builder().name_contains().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_name_contains();
                        },
                        "×"
                    }
                }
            }
            input { class : "input input-compact",
                id : "name-contains",
                placeholder : "name contains",
                value : if let Some(name) = filter_builder().name_contains() {
                    name
                } else { "" },
                r#type : "text",
                autocapitalize : "none",
                spellcheck : "false",
                oninput : move |event| {
                    filter_builder.write().set_name_contains(event.value());
                }
            }

            div { class: "label-row",
                label { class: "label-xs", r#for: "oracle-text-contains", "oracle text contains" }
                if filter_builder().oracle_text_contains().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_oracle_text_contains();
                        },
                        "×"
                    }
                }
            }
            input { class: "input input-compact",
                id: "oracle-text-contains",
                placeholder: "oracle text contains",
                value: if let Some(text) = filter_builder().oracle_text_contains() {
                    text
                } else { "" },
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    filter_builder.write().set_oracle_text_contains(event.value());
                }
            }
        }
    }
}
