use crate::{
    domain::language::LanguageCodeToFullName,
    inbound::components::tri_toggle::TriToggle,
    outbound::client::{card::get_languages::ClientGetLanguages, ZwipeClient},
};
use dioxus::prelude::*;
use zwipe::{
    domain::auth::models::session::Session,
    domain::card::models::search_card::card_filter::builder::CardFilterBuilder,
    inbound::http::ApiError,
};

#[component]
pub fn Config() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // Fetch all languages from backend
    let all_languages: Resource<Result<Vec<String>, ApiError>> = use_resource(move || async move {
        let Some(sesh) = session() else {
            return Err(ApiError::Unauthorized("session expired".to_string()));
        };
        client().get_languages(&sesh).await
    });

    rsx! {
        div { class: "flex-col gap-1",
            // Language chips section
            div { class: "flex-col gap-half",
                label { class: "label-xs", "language" }

                // Show loading or languages
                match &*all_languages.read_unchecked() {
                    Some(Ok(languages)) => rsx! {
                        div { class: "flex flex-wrap gap-1",
                            for lang_code in languages.iter() {{
                                let lang_code = lang_code.clone();
                                let display_name = lang_code.language_code_to_full_name().to_lowercase();
                                rsx! {
                                    div {
                                        class: if filter_builder()
                                            .language()
                                            .is_some_and(|l| l == lang_code.as_str())
                                        {
                                            "chip selected"
                                        } else {
                                            "chip"
                                        },
                                        onclick: move |_| {
                                            let lang_to_set = lang_code.clone();
                                            filter_builder.write().set_language(lang_to_set);
                                        },
                                        { display_name }
                                    }
                                }
                            }}
                        }
                    },
                    Some(Err(_)) => rsx! {
                        div { class: "error", "Failed to load languages" }
                    },
                    None => rsx! {
                        div { class: "loading", "Loading languages..." }
                    },
                }
            }

            // Tri-state filters section
            TriToggle {
                label: "playable",
                filter_builder,
                getter: |fb| fb.is_playable(),
                setter_true: |fb| { fb.set_is_playable(true); },
                setter_false: |fb| { fb.set_is_playable(false); },
                unsetter: |fb| { fb.unset_is_playable(); },
                true_label: "show",
                false_label: "hide",
                none_label: "any"
            }

            TriToggle {
                label: "digital",
                filter_builder,
                getter: |fb| fb.digital(),
                setter_true: |fb| { fb.set_digital(true); },
                setter_false: |fb| { fb.set_digital(false); },
                unsetter: |fb| { fb.unset_digital(); },
                true_label: "show",
                false_label: "hide",
                none_label: "any"
            }

            TriToggle {
                label: "oversized",
                filter_builder,
                getter: |fb| fb.oversized(),
                setter_true: |fb| { fb.set_oversized(true); },
                setter_false: |fb| { fb.set_oversized(false); },
                unsetter: |fb| { fb.unset_oversized(); },
                true_label: "show",
                false_label: "hide",
                none_label: "any"
            }

            TriToggle {
                label: "promo",
                filter_builder,
                getter: |fb| fb.promo(),
                setter_true: |fb| { fb.set_promo(true); },
                setter_false: |fb| { fb.set_promo(false); },
                unsetter: |fb| { fb.unset_promo(); },
                true_label: "show",
                false_label: "hide",
                none_label: "any"
            }

            TriToggle {
                label: "content warning",
                filter_builder,
                getter: |fb| fb.content_warning(),
                setter_true: |fb| { fb.set_content_warning(true); },
                setter_false: |fb| { fb.set_content_warning(false); },
                unsetter: |fb| { fb.unset_content_warning(); },
                true_label: "show",
                false_label: "hide",
                none_label: "any"
            }
        }
    }
}
