//! Artist filter component.

use super::deck_cards::{DeckCards, extract_artists};
use crate::outbound::client::{card::get_artists::ClientGetArtists, ZwipeClient};
use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;

/// Filter component for card artist search.
#[component]
pub fn Artist() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let filter_reset: Signal<u32> = use_context();
    let deck_ctx: Option<DeckCards> = try_use_context();

    // Fetch artists from deck context (if present) or server
    let all_artists: Resource<Result<Vec<String>, ApiError>> = use_resource(move || async move {
        if let Some(dc) = deck_ctx {
            return Ok(extract_artists(&dc.0()));
        }
        let Some(session) = session() else {
            return Err(ApiError::Unauthorized("session expired".to_string()));
        };
        client().get_artists(&session).await
    });

    let mut artist_search_query = use_signal(String::new);
    let mut artist_is_typing = use_signal(|| false);

    // Clear search query when filter applied
    use_effect(move || {
        let _ = filter_reset();
        artist_search_query.set(String::new());
        artist_is_typing.set(false);
    });

    let selected_artists = filter_builder()
        .artist_equals_any()
        .map(|v| v.to_vec())
        .unwrap_or_default();

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row mt-2",
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
