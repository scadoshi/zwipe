use dioxus::prelude::*;
use zwipe::domain::card::models::{
    scryfall_data::language::Language, search_card::card_filter::builder::CardFilterBuilder,
};

#[component]
pub fn Config() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    rsx! {
        div { class: "flex-col gap-1",
            // Language chips section
            div { class: "flex-col gap-half",
                label { class: "label-xs", "language" }
                div { class: "flex flex-wrap gap-1",
                    for lang in Language::all() {
                        div {
                            class: if filter_builder().language().is_some_and(|l| l == lang) {
                                "chip selected"
                            } else {
                                "chip"
                            },
                            onclick: move |_| {
                                if filter_builder().language().is_some_and(|l| l == lang) {
                                    filter_builder.write().unset_language();
                                } else {
                                    filter_builder.write().set_language(lang);
                                }
                            },
                            { lang.to_name().to_lowercase() }
                        }
                    }
                }
            }

            // Toggle switches section
            div { class: "flex-col gap-half",
                label { class: "label-xs", "filters" }

                // is_playable toggle
                ToggleSwitch {
                    label: "playable",
                    checked: filter_builder().is_playable().unwrap_or(false),
                    on_toggle: move |checked| {
                        if checked {
                            filter_builder.write().set_is_playable(true);
                        } else {
                            filter_builder.write().unset_is_playable();
                        }
                    }
                }

                // digital toggle
                ToggleSwitch {
                    label: "digital-only",
                    checked: filter_builder().digital().is_none(),
                    on_toggle: move |checked| {
                        if checked {
                            filter_builder.write().unset_digital();
                        } else {
                            filter_builder.write().set_digital(false);
                        }
                    }
                }

                // oversized toggle
                ToggleSwitch {
                    label: "w oversized cards",
                    checked: filter_builder().oversized().is_none(),
                    on_toggle: move |checked| {
                        if checked {
                            filter_builder.write().unset_oversized();
                        } else {
                            filter_builder.write().set_oversized(false);
                        }
                    }
                }

                // promo toggle
                ToggleSwitch {
                    label: "show promo cards",
                    checked: filter_builder().promo().is_none(),
                    on_toggle: move |checked| {
                        if checked {
                            filter_builder.write().unset_promo();
                        } else {
                            filter_builder.write().set_promo(false);
                        }
                    }
                }

                // content_warning toggle
                ToggleSwitch {
                    label: "show content warning cards",
                    checked: filter_builder().content_warning().is_none(),
                    on_toggle: move |checked| {
                        if checked {
                            filter_builder.write().unset_content_warning();
                        } else {
                            filter_builder.write().set_content_warning(false);
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ToggleSwitch(label: String, checked: bool, on_toggle: EventHandler<bool>) -> Element {
    rsx! {
        div { class: "toggle-row",
            span { class: "toggle-label", "{label}" }
            label { class: "toggle-switch",
                input {
                    r#type: "checkbox",
                    checked: checked,
                    onchange: move |evt| {
                        on_toggle.call(evt.checked());
                    }
                }
                span { class: "toggle-slider" }
            }
        }
    }
}
