//! Oracle tags filter component.
//!
//! Oracle tags are Scryfall's community-maintained functional card tags. The
//! full catalog (~4,500 tags) is fetched from `GET /api/card/oracle-tags`; this
//! picker shows a curated default set up front (the functional gameplay tags,
//! only those the backend still serves) and exposes the rest through search.
//! Selections write slugs into the `oracle_tags_*` filter criteria, mirroring
//! the mechanical-category filter's include/any-all/exclude structure.

use super::match_mode::MatchMode;
use crate::{inbound::components::catalog_cache::CatalogCache, outbound::client::ZwipeClient};
use dioxus::prelude::*;
use zwipe_core::domain::card::{
    oracle_tag::{CURATED_ORACLE_TAGS, OracleTag},
    search_card::card_filter::builder::CardQueryBuilder,
};

fn read_selected(fb: &CardQueryBuilder, mode: MatchMode) -> Vec<String> {
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

fn write_selected(fb: &mut CardQueryBuilder, mode: MatchMode, values: Vec<String>) {
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

fn read_excluded(fb: &CardQueryBuilder) -> Vec<String> {
    fb.oracle_tags_excludes()
        .map(|v| v.to_vec())
        .unwrap_or_default()
}

fn write_excluded(fb: &mut CardQueryBuilder, values: Vec<String>) {
    if values.is_empty() {
        fb.unset_oracle_tags_excludes();
    } else {
        fb.set_oracle_tags_excludes(values);
    }
}

/// Human label for a slug from the fetched catalog, falling back to the slug
/// itself (e.g. for a selected tag the backend no longer serves).
fn label_for<'a>(catalog: &'a [OracleTag], slug: &'a str) -> &'a str {
    catalog
        .iter()
        .find(|t| t.slug == slug)
        .map(|t| t.label.as_str())
        .unwrap_or(slug)
}

/// Oracle tag multi-select: curated default grid + full-catalog search, with an
/// any/all match toggle and a separate exclude section.
#[component]
pub(crate) fn OracleTags() -> Element {
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

    // The default grid: curated slugs the backend still serves, plus any selected
    // slug not in the curated set (so an active selection is always visible).
    let mut grid_slugs: Vec<String> = CURATED_ORACLE_TAGS
        .iter()
        .filter(|s| tags.iter().any(|t| t.slug == **s))
        .map(|s| (*s).to_string())
        .collect();
    for s in &selected {
        if !grid_slugs.contains(s) {
            grid_slugs.push(s.clone());
        }
    }

    rsx! {
        div { class: "flex-col gap-half",
            // ── includes ──────────────────────────────────────────
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "oracle-tags-search", "Oracle tags include" }
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

            div { class: "flex flex-wrap gap-1 flex-center",
                for slug in grid_slugs.iter().cloned() {
                    {
                        let is_selected = selected.contains(&slug);
                        let display = label_for(tags, &slug).to_string();
                        rsx! {
                            div {
                                class: if is_selected { "chip selected" } else { "chip" },
                                onclick: move |_| {
                                    let m = mode();
                                    let mut current = read_selected(&filter_builder(), m);
                                    if current.contains(&slug) {
                                        current.retain(|s| s != &slug);
                                    } else {
                                        current.push(slug.clone());
                                    }
                                    write_selected(&mut filter_builder.write(), m, current);
                                },
                                { display }
                            }
                        }
                    }
                }
            }

            // search-to-add over the full catalog
            if !includes_search().is_empty() {
                {
                    let query = includes_search().to_lowercase();
                    let results: Vec<OracleTag> = tags
                        .iter()
                        .filter(|t| {
                            (t.label.to_lowercase().contains(&query)
                                || t.slug.contains(&query))
                                && !selected.contains(&t.slug)
                                && !grid_slugs.contains(&t.slug)
                        })
                        .take(8)
                        .cloned()
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
                                        "{tag.label}"
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
                        {
                            let display = label_for(tags, &slug).to_string();
                            rsx! {
                                div { class: "chip selected flex items-center gap-05",
                                    { display }
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
                }
            }

            if !excludes_search().is_empty() {
                {
                    let query = excludes_search().to_lowercase();
                    let results: Vec<OracleTag> = tags
                        .iter()
                        .filter(|t| {
                            (t.label.to_lowercase().contains(&query)
                                || t.slug.contains(&query))
                                && !selected.contains(&t.slug)
                                && !excluded.contains(&t.slug)
                        })
                        .take(8)
                        .cloned()
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
                                        "{tag.label}"
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
