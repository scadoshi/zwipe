use crate::inbound::components::auth::bouncer::Bouncer;
use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

#[component]
pub fn Text() -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    rsx! {
        Bouncer {
            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center;",
                div { class : "container-sm",

                    form { class : "flex-col text-center",
                        label { class: "label", r#for : "name-contains", "name contains" }
                        input { class : "input",
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

                        label { class: "label", r#for: "oracle-text-contains", "oracle text contains" }
                        input { class: "input",
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

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "back"
                }
            }
        }
    }
}

#[component]
pub fn TextFilterContent() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    rsx! {
        div { class: "flex-col gap-half",
            label { class: "label-xs", r#for : "name-contains", "name contains" }
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

            label { class: "label-xs", r#for: "oracle-text-contains", "oracle text contains" }
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
