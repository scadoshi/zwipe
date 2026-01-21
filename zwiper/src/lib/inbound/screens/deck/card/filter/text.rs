use crate::outbound::client::{card::get_artists::ClientGetArtists, ZwipeClient};
use dioxus::prelude::*;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::search_card::card_filter::builder::CardFilterBuilder,
    },
    inbound::http::ApiError,
};

#[component]
pub fn Text() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let filter_reset: Signal<u32> = use_context();

    // Fetch all artists from backend
    let all_artists: Resource<Result<Vec<String>, ApiError>> = use_resource(move || async move {
        let Some(session) = session() else {
            return Err(ApiError::Unauthorized("session expired".to_string()));
        };
        client().get_artists(&session).await
    });

    // Artist search state
    let mut artist_search_query = use_signal(String::new);
    let mut artist_is_typing = use_signal(|| false);

    // Clear artist search when filter applied
    use_effect(move || {
        let _ = filter_reset();
        artist_search_query.set(String::new());
        artist_is_typing.set(false);
    });

    // Get selected artists
    let selected_artists = filter_builder()
        .artist_equals_any()
        .map(|v| v.to_vec())
        .unwrap_or_default();

    // Extract text filter values to avoid borrowed reference lifetime issues
    let name_value = filter_builder().name_contains().unwrap_or("").to_string();

    let oracle_text_value = filter_builder()
        .oracle_text_contains()
        .unwrap_or("")
        .to_string();

    let flavor_text_value = filter_builder()
        .flavor_text_contains()
        .unwrap_or("")
        .to_string();

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
                value : name_value,
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
                value: oracle_text_value,
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    filter_builder.write().set_oracle_text_contains(event.value());
                }
            }

            div { class: "label-row",
                label { class: "label-xs", r#for: "flavor-text-contains", "flavor text contains" }
                if filter_builder().flavor_text_contains().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_flavor_text_contains();
                        },
                        "×"
                    }
                }
            }
            input { class: "input input-compact",
                id: "flavor-text-contains",
                placeholder: "flavor text contains",
                value: flavor_text_value,
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    filter_builder.write().set_flavor_text_contains(event.value());
                }
            }

            // Artist filter (chip-based multi-select)
            div { class: "label-row",
                label { class: "label-xs", r#for: "artist-search", "artist equals any" }
                if !selected_artists.is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_artist_equals_any();
                            artist_search_query.set(String::new());
                        },
                        "×"
                    }
                }
            }

            // Selected artists (chips with remove button)
            if !selected_artists.is_empty() {
                div { class: "flex flex-wrap gap-1 mb-1",
                    for artist in selected_artists.iter().cloned() {
                        div { class: "chip flex items-center gap-05",
                            "{artist}"
                            button { class: "chip-remove",
                                onclick: move |_| {
                                    let current = filter_builder()
                                        .artist_equals_any()
                                        .map(|v| v.to_vec())
                                        .unwrap_or_default();
                                    let new_artists: Vec<String> = current
                                        .into_iter()
                                        .filter(|a| *a != artist)
                                        .collect();
                                    if new_artists.is_empty() {
                                        filter_builder.write().unset_artist_equals_any();
                                    } else {
                                        filter_builder.write().set_artist_equals_any(new_artists);
                                    }
                                },
                                "×"
                            }
                        }
                    }
                }
            }

            // Search results (filtered suggestions)
            if !artist_search_query().is_empty() {
                if let Some(Ok(artists)) = all_artists.read().as_ref() {
                    {
                        let query = artist_search_query().to_lowercase();
                        let already_selected = selected_artists.as_slice();

                        let results: Vec<String> = artists
                            .iter()
                            .filter(|a| {
                                a.to_lowercase().contains(&query)
                                && !already_selected.contains(a)
                            })
                            .take(5)
                            .cloned()
                            .collect();

                        if !results.is_empty() {
                            rsx! {
                                div { class: "flex flex-wrap gap-1 mb-1",
                                    for artist in results {
                                        div { class: "chip-unselected",
                                            onclick: move |_| {
                                                let mut current = filter_builder()
                                                    .artist_equals_any()
                                                    .map(|v| v.to_vec())
                                                    .unwrap_or_default();
                                                current.push(artist.clone());
                                                filter_builder.write().set_artist_equals_any(current);
                                                artist_is_typing.set(false);
                                            },
                                            "{artist}"
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }
            }

            // Search input
            input { class: "input input-compact",
                id: "artist-search",
                placeholder: "type to search",
                value: "{artist_search_query()}",
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    artist_is_typing.set(true);
                    artist_search_query.set(event.value());
                },
                onblur: move |_| {
                    artist_is_typing.set(false);
                }
            }
        }
    }
}
