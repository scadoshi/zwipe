//! Card-role filter component. The role list is server-driven: fetched from the
//! card-role catalog (`GET /api/card/roles`), not a compiled enum — so new roles
//! appear without a client release. No fallback (same as artists/oracle-tags).

use crate::{inbound::components::catalog_cache::CatalogCache, outbound::client::ZwipeClient};
use dioxus::prelude::*;
use zwipe_core::domain::card::{
    card_role::CardRoleView, search_card::card_filter::builder::CardQueryBuilder,
};

use super::match_mode::MatchMode;

/// Filter component for card roles with separate include and exclude grids.
#[component]
pub fn Category() -> Element {
    let mut filter_builder: Signal<CardQueryBuilder> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();

    // Server-driven role catalog; no compiled fallback. Read from the app-wide
    // catalog cache (prefetched at startup) instead of fetching on open.
    use_effect(move || cache.ensure_card_roles(client));
    let all_roles = use_memo(move || -> Option<Vec<CardRoleView>> {
        cache.card_roles.cell().read().loaded().cloned()
    });

    let mode = use_memo(move || {
        let fb = filter_builder();
        if fb.mechanical_categories_contains_all().is_some() {
            MatchMode::All
        } else {
            MatchMode::Any
        }
    });

    let read_selected = move || -> Vec<String> {
        let fb = filter_builder();
        match mode() {
            MatchMode::Any => fb
                .mechanical_categories_contains_any()
                .map(|v| v.to_vec())
                .unwrap_or_default(),
            MatchMode::All => fb
                .mechanical_categories_contains_all()
                .map(|v| v.to_vec())
                .unwrap_or_default(),
        }
    };

    let read_excluded = move || -> Vec<String> {
        filter_builder()
            .mechanical_categories_excludes()
            .map(|v| v.to_vec())
            .unwrap_or_default()
    };

    let mut write_categories = move |cats: Vec<String>, m: MatchMode| {
        let fb = &mut *filter_builder.write();
        fb.unset_mechanical_categories_contains_any();
        fb.unset_mechanical_categories_contains_all();
        if !cats.is_empty() {
            match m {
                MatchMode::Any => {
                    fb.set_mechanical_categories_contains_any(cats);
                }
                MatchMode::All => {
                    fb.set_mechanical_categories_contains_all(cats);
                }
            }
        }
    };

    let mut write_excluded_cats = move |cats: Vec<String>| {
        let fb = &mut *filter_builder.write();
        if cats.is_empty() {
            fb.unset_mechanical_categories_excludes();
        } else {
            fb.set_mechanical_categories_excludes(cats);
        }
    };

    let selected = read_selected();
    let excluded = read_excluded();

    rsx! {
        div { class: "flex-col gap-half",
            // ── role includes ─────────────────────────────────────
            div { class: "label-row mt-2",
                label { class: "label-xs", "Card roles include" }
                if !selected.is_empty() {
                    button {
                        class: "chip-xs",
                        onclick: move |_| {
                            let current = read_selected();
                            let new_mode = mode().toggle();
                            write_categories(current, new_mode);
                        },
                        { mode().label() }
                    }
                }
                if !selected.is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            write_categories(vec![], mode());
                        },
                        "\u{00d7}"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 flex-center",
                if let Some(roles) = all_roles.read().as_ref() {
                    for role in roles {
                        {
                            let slug = role.slug.clone();
                            let display = role.display_name.clone();
                            let is_selected = selected.contains(&slug);
                            rsx! {
                                div {
                                    class: if is_selected { "chip selected" } else { "chip" },
                                    onclick: move |_| {
                                        let mut current = read_selected();
                                        if current.contains(&slug) {
                                            current.retain(|s| s != &slug);
                                        } else {
                                            current.push(slug.clone());
                                        }
                                        write_categories(current, mode());
                                    },
                                    { display }
                                }
                            }
                        }
                    }
                }
            }

            // ── role excludes ─────────────────────────────────────
            div { class: "label-row mt-2",
                label { class: "label-xs", "Card roles exclude" }
                if !excluded.is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            write_excluded_cats(vec![]);
                        },
                        "\u{00d7}"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 flex-center",
                if let Some(roles) = all_roles.read().as_ref() {
                    for role in roles {
                        {
                            let slug = role.slug.clone();
                            let display = role.display_name.clone();
                            let is_excluded = excluded.contains(&slug);
                            rsx! {
                                div {
                                    class: if is_excluded { "chip selected" } else { "chip" },
                                    onclick: move |_| {
                                        let mut current = read_excluded();
                                        if current.contains(&slug) {
                                            current.retain(|s| s != &slug);
                                        } else {
                                            current.push(slug.clone());
                                        }
                                        write_excluded_cats(current);
                                    },
                                    { display }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
