//! Oracle tags filter component.
//!
//! Oracle tags are Scryfall's community-maintained functional card tags. The
//! full catalog (~4,500 tags) is fetched from `GET /api/card/oracle-tags`; with
//! that many tags there's no useful default set, so this picker is search-only:
//! type to find tags, click to add. Selections write slugs into the
//! `oracle_tags_*` filter criteria, mirroring the include/any-all/exclude
//! structure of the other multi-select filters.

use super::match_mode::MatchMode;
use crate::{inbound::components::catalog_cache::CatalogCache, outbound::client::ZwipeClient};
use dioxus::prelude::*;
use zwipe_core::domain::card::{
    oracle_tag::{OracleTag, search_oracle_tags},
    search_card::card_filter::builder::CardQueryBuilder,
};

pub(super) fn read_selected(fb: &CardQueryBuilder, mode: MatchMode) -> Vec<String> {
    match mode {
        MatchMode::Any => fb
            .oracle_tags_contains_any()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
        MatchMode::All => fb
            .oracle_tags_contains_all()
            .map(|v| v.to_vec())
            .unwrap_or_default(),
    }
}

pub(super) fn write_selected(fb: &mut CardQueryBuilder, mode: MatchMode, values: Vec<String>) {
    fb.unset_oracle_tags_contains_any();
    fb.unset_oracle_tags_contains_all();
    if !values.is_empty() {
        match mode {
            MatchMode::Any => {
                fb.set_oracle_tags_contains_any(values);
            }
            MatchMode::All => {
                fb.set_oracle_tags_contains_all(values);
            }
        }
    }
}

pub(super) fn read_excluded(fb: &CardQueryBuilder) -> Vec<String> {
    fb.oracle_tags_excludes()
        .map(|v| v.to_vec())
        .unwrap_or_default()
}

pub(super) fn write_excluded(fb: &mut CardQueryBuilder, values: Vec<String>) {
    if values.is_empty() {
        fb.unset_oracle_tags_excludes();
    } else {
        fb.set_oracle_tags_excludes(values);
    }
}

/// Oracle tag multi-select: search-only chips show raw slugs (same as the deck
/// strategy picker), with an any/all match toggle and a separate exclude section.
///
/// The dictionary overlay is owned by the parent filter sheet (rendered outside the
/// sheet's `transform`, which would otherwise trap its `position: fixed`). The
/// include/exclude "Dictionary" buttons here just open it, recording via
/// `dict_exclude` which list its Use button should feed.
#[component]
pub(crate) fn OracleTags(mut dict_open: Signal<bool>, mut dict_exclude: Signal<bool>) -> Element {
    let mut filter_builder: Signal<CardQueryBuilder> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let filter_reset: Signal<u32> = use_context();
    let cache: CatalogCache = use_context();

    // Read the shared app-wide oracle-tag catalog (prefetched at startup), warming
    // / revalidating it on open. Same copy the picker + dictionary read.
    use_effect(move || {
        cache.ensure_oracle_tags(client);
    });
    let cell = cache.oracle_tags.cell();

    let mut includes_search = use_signal(String::new);
    let mut excludes_search = use_signal(String::new);

    let mut mode = use_signal(|| {
        if filter_builder().oracle_tags_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });

    use_effect(move || {
        let _ = filter_reset();
        includes_search.set(String::new());
        excludes_search.set(String::new());
    });

    let selected = read_selected(&filter_builder(), mode());
    let excluded = read_excluded(&filter_builder());

    let cell_read = cell.read();
    let tags: &[OracleTag] = cell_read.loaded().map(Vec::as_slice).unwrap_or(&[]);

    rsx! {
        div { class: "flex-col gap-half",
            // ── includes ──────────────────────────────────────────
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "oracle-tags-search", "Oracle tags include" }
                button {
                    class: "chip-xs",
                    onclick: move |_| {
                        dict_exclude.set(false);
                        dict_open.set(true);
                    },
                    "Dictionary"
                }
                if !selected.is_empty() {
                    button {
                        class: "chip-xs",
                        onclick: move |_| {
                            let new_mode = mode().toggle();
                            let current = read_selected(&filter_builder(), mode());
                            write_selected(&mut filter_builder.write(), new_mode, current);
                            mode.set(new_mode);
                        },
                        "{mode().label()}"
                    }
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            write_selected(&mut filter_builder.write(), mode(), vec![]);
                            includes_search.set(String::new());
                        },
                        "\u{00d7}"
                    }
                }
            }

            if !selected.is_empty() {
                div { class: "flex flex-wrap gap-1 mb-1",
                    for slug in selected.iter().cloned() {
                        div { class: "chip selected flex items-center gap-05",
                            "{slug}"
                            button { class: "chip-remove",
                                onclick: move |_| {
                                    let m = mode();
                                    let new_selected: Vec<String> = read_selected(&filter_builder(), m)
                                        .into_iter()
                                        .filter(|s| *s != slug)
                                        .collect();
                                    write_selected(&mut filter_builder.write(), m, new_selected);
                                },
                                "\u{00d7}"
                            }
                        }
                    }
                }
            }

            // search-to-add over the full catalog
            if !includes_search().is_empty() {
                {
                    let results: Vec<OracleTag> = search_oracle_tags(tags, &includes_search())
                        .into_iter()
                        .filter(|t| !selected.contains(&t.slug))
                        .take(8)
                        .collect();

                    rsx! {
                        if !results.is_empty() {
                            div { class: "flex flex-wrap gap-1 mb-1",
                                for tag in results {
                                    div { class: "chip-unselected",
                                        onclick: move |_| {
                                            let m = mode();
                                            let mut current = read_selected(&filter_builder(), m);
                                            current.push(tag.slug.clone());
                                            write_selected(&mut filter_builder.write(), m, current);
                                        },
                                        "{tag.slug}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            input { class: "input input-compact",
                id: "oracle-tags-search",
                placeholder: "Search all oracle tags",
                value: "{includes_search()}",
                r#type: "text",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| {
                    includes_search.set(event.value());
                }
            }

            // ── excludes ──────────────────────────────────────────
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "oracle-tags-excludes-search", "Oracle tags exclude" }
                button {
                    class: "chip-xs",
                    onclick: move |_| {
                        dict_exclude.set(true);
                        dict_open.set(true);
                    },
                    "Dictionary"
                }
                if !excluded.is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            write_excluded(&mut filter_builder.write(), vec![]);
                            excludes_search.set(String::new());
                        },
                        "\u{00d7}"
                    }
                }
            }

            if !excluded.is_empty() {
                div { class: "flex flex-wrap gap-1 mb-1",
                    for slug in excluded.iter().cloned() {
                        div { class: "chip selected flex items-center gap-05",
                            "{slug}"
                            button { class: "chip-remove",
                                onclick: move |_| {
                                    let new_excluded: Vec<String> = read_excluded(&filter_builder())
                                        .into_iter()
                                        .filter(|s| *s != slug)
                                        .collect();
                                    write_excluded(&mut filter_builder.write(), new_excluded);
                                },
                                "\u{00d7}"
                            }
                        }
                    }
                }
            }

            if !excludes_search().is_empty() {
                {
                    let results: Vec<OracleTag> = search_oracle_tags(tags, &excludes_search())
                        .into_iter()
                        .filter(|t| !selected.contains(&t.slug) && !excluded.contains(&t.slug))
                        .take(8)
                        .collect();

                    rsx! {
                        if !results.is_empty() {
                            div { class: "flex flex-wrap gap-1 mb-1",
                                for tag in results {
                                    div { class: "chip-unselected",
                                        onclick: move |_| {
                                            let mut current = read_excluded(&filter_builder());
                                            current.push(tag.slug.clone());
                                            write_excluded(&mut filter_builder.write(), current);
                                        },
                                        "{tag.slug}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            input { class: "input input-compact",
                id: "oracle-tags-excludes-search",
                placeholder: "Search all oracle tags",
                value: "{excludes_search()}",
                r#type: "text",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| {
                    excludes_search.set(event.value());
                }
            }
        }
    }
}
