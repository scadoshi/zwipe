//! Oracle tags filter component.
//!
//! Oracle tags are Scryfall's community-maintained functional card tags. The
//! full catalog (~4,500 tags) is fetched from `GET /api/card/oracle-tags`; with
//! that many tags there's no useful default set, so this picker is search-only:
//! type to find tags, click to add. Selections write slugs into the
//! `oracle_tags_*` filter criteria, mirroring the include/any-all/exclude
//! structure of the other multi-select filters.

use components::ClearWrap;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::oracle_tag::{self, OracleTag};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_ui::filter::fields::MatchGroup;
use zwipe_ui::filter::picker::PickerField;

use super::match_mode::MatchMode;
use crate::inbound::components::catalog_cache::CatalogCache;

fn read_selected(fb: &CardQueryBuilder, mode: MatchMode) -> Vec<String> {
    match mode {
        MatchMode::Any => fb
            .oracle_tags_contains_any()
            .map(<[_]>::to_vec)
            .unwrap_or_default(),
        MatchMode::All => fb
            .oracle_tags_contains_all()
            .map(<[_]>::to_vec)
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
        .map(<[_]>::to_vec)
        .unwrap_or_default()
}

fn write_excluded(fb: &mut CardQueryBuilder, values: Vec<String>) {
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
/// include/exclude "Dictionary" buttons here just open it through
/// `on_dictionary`, naming the list its Use button should feed.
#[component]
pub(crate) fn OracleTags(on_dictionary: EventHandler<PickerField>) -> Element {
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
    let tags: &[OracleTag] = cell_read.loaded().map_or(&[], Vec::as_slice);

    rsx! {
        div { class: "flex-col gap-half",
            // == includes ==
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "oracle-tags-search", "Oracle tags include" }
                button {
                    class: "chip-xs",
                    onclick: move |_| on_dictionary.call(PickerField::Include(MatchGroup::Tags)),
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
            ClearWrap {
                filled: !includes_search().is_empty(),
                on_clear: move |()| {
                    includes_search.set(String::new());
                },
                input { class: "input input-compact",
                        style: "flex: 1; margin-bottom: 0;",
                        id: "oracle-tags-search",
                        placeholder: "Search all Oracle tags",
                        value: "{includes_search()}",
                        r#type: "text",
                        autocapitalize: "none",
                        autocorrect: "off",
                        spellcheck: "false",
                        oninput: move |event| {
                            includes_search.set(event.value());
                        }
                    }
            }
            if !includes_search().is_empty() {
                {
                    let results: Vec<OracleTag> = oracle_tag::search_oracle_tags(tags, &includes_search())
                        .into_iter()
                        .filter(|t| !selected.contains(&t.slug))
                        .take(8)
                        .collect();

                    rsx! {
                        if !results.is_empty() {
                            div {
                                div { class: "card-filter-results",
                                    for (i, tag) in results.into_iter().enumerate() {
                                        button {
                                            r#type: "button",
                                            class: "chip-unselected card-filter-result",
                                            style: "animation-delay: {i * 40}ms;",
                                            onclick: move |_| {
                                                includes_search.set(String::new());
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
            }

            // == excludes ==
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "oracle-tags-excludes-search", "Oracle tags exclude" }
                button {
                    class: "chip-xs",
                    onclick: move |_| on_dictionary.call(PickerField::Exclude(MatchGroup::Tags)),
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

            ClearWrap {
                filled: !excludes_search().is_empty(),
                on_clear: move |()| {
                    excludes_search.set(String::new());
                },
                input { class: "input input-compact",
                        style: "flex: 1; margin-bottom: 0;",
                        id: "oracle-tags-excludes-search",
                        placeholder: "Search all Oracle tags",
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
            if !excludes_search().is_empty() {
                {
                    let results: Vec<OracleTag> = oracle_tag::search_oracle_tags(tags, &excludes_search())
                        .into_iter()
                        .filter(|t| !selected.contains(&t.slug) && !excluded.contains(&t.slug))
                        .take(8)
                        .collect();

                    rsx! {
                        if !results.is_empty() {
                            div {
                                div { class: "card-filter-results",
                                    for (i, tag) in results.into_iter().enumerate() {
                                        button {
                                            r#type: "button",
                                            class: "chip-unselected card-filter-result",
                                            style: "animation-delay: {i * 40}ms;",
                                            onclick: move |_| {
                                                excludes_search.set(String::new());
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
            }
        }
    }
}
