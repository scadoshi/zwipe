//! Artist filter component.

use super::deck_cards::{DeckCards, extract_artists};
use crate::outbound::client::{ZwipeClient, card::get_artists::ClientGetArtists};
use dioxus::prelude::*;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;

/// Whether the artist filter is in include or exclude mode.
#[derive(Debug, Clone, Copy, PartialEq)]
enum IncludeExclude {
    Include,
    Exclude,
}

impl IncludeExclude {
    fn toggle(self) -> Self {
        match self {
            Self::Include => Self::Exclude,
            Self::Exclude => Self::Include,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Include => "Include",
            Self::Exclude => "Exclude",
        }
    }
}

fn read_artists(fb: &CardFilterBuilder, mode: IncludeExclude) -> Vec<String> {
    match mode {
        IncludeExclude::Include => fb
            .artist_equals_any()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
        IncludeExclude::Exclude => fb
            .artist_excludes_any()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
    }
}

fn write_artists(fb: &mut CardFilterBuilder, mode: IncludeExclude, values: Vec<String>) {
    fb.unset_artist_equals_any();
    fb.unset_artist_excludes_any();
    if !values.is_empty() {
        match mode {
            IncludeExclude::Include => {
                fb.set_artist_equals_any(values);
            }
            IncludeExclude::Exclude => {
                fb.set_artist_excludes_any(values);
            }
        }
    }
}

/// Filter component for card artist search.
#[component]
pub fn Artist() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let client: Signal<ZwipeClient> = use_context();
    let filter_reset: Signal<u32> = use_context();
    let deck_ctx: Option<DeckCards> = try_use_context();

    let all_artists: Resource<Result<Vec<String>, ApiError>> = use_resource(move || async move {
        if let Some(dc) = deck_ctx {
            return Ok(extract_artists(&dc.0()));
        }
        client().get_artists().await
    });

    let mut artist_search_query = use_signal(String::new);
    let mut artist_is_typing = use_signal(|| false);

    let mut mode = use_signal(|| {
        if filter_builder().artist_excludes_any().is_some() {
            IncludeExclude::Exclude
        } else {
            IncludeExclude::Include
        }
    });

    use_effect(move || {
        let _ = filter_reset();
        artist_search_query.set(String::new());
        artist_is_typing.set(false);
    });

    let selected_artists = read_artists(&filter_builder(), mode());
    let has_selection = !selected_artists.is_empty();

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "artist-search", "Artist" }
                if has_selection {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            let new_mode = mode().toggle();
                            let current = read_artists(&filter_builder(), mode());
                            write_artists(&mut filter_builder.write(), new_mode, current);
                            mode.set(new_mode);
                        },
                        "{mode().label()}"
                    }
                }
                if has_selection {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            write_artists(&mut filter_builder.write(), mode(), vec![]);
                            artist_search_query.set(String::new());
                        },
                        "×"
                    }
                }
            }

            if !selected_artists.is_empty() {
                div { class: "flex flex-wrap gap-1 mb-1",
                    for artist in selected_artists.iter().cloned() {
                        div {
                            class: match mode() {
                                IncludeExclude::Include => "chip selected flex items-center gap-05",
                                IncludeExclude::Exclude => "chip selected flex items-center gap-05",
                            },
                            {artist.clone()}
                            button { class: "chip-remove",
                                onclick: move |_| {
                                    let new_artists: Vec<String> = read_artists(&filter_builder(), mode())
                                        .into_iter()
                                        .filter(|a| *a != artist)
                                        .collect();
                                    write_artists(&mut filter_builder.write(), mode(), new_artists);
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
                                                let mut current = read_artists(&filter_builder(), mode());
                                                current.push(artist.clone());
                                                write_artists(&mut filter_builder.write(), mode(), current);
                                                artist_is_typing.set(false);
                                            },
                                            {artist.clone()}
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
                placeholder: "Type to search",
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
