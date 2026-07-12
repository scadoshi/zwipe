//! Card roles with drill-down to their oracle tags.
//!
//! Renders the coarse roles a card fulfills (Removal, Ramp, ...) as chips. A role
//! that has functional oracle tags beneath it (server-grouped) eases those tags
//! open when tapped, reusing the keyword hint reveal; a role with none (the
//! heuristic/token roles) is a plain, non-expandable chip. A trailing "Other
//! tags" chip holds the card's functional tags that fall under no role. This is
//! the card's whole tag story: role = the high-level thing, oracle tags = the
//! specific things underneath, distinct from deck-level Deck Tags.

use dioxus::prelude::*;
use std::collections::BTreeMap;
use zwipe_core::domain::card::{
    mechanical_category::MechanicalCategory, oracle_tag::prettify_oracle_tag_slug,
};

/// Card roles as chips; expandable to their grouped oracle tags, plus an "Other
/// tags" bucket. `tags_by_role` is keyed by role slug (`MechanicalCategory`'s
/// snake_case form); `other_tags` are the uncategorized functional tags.
#[component]
pub fn CardRoleChips(
    roles: Vec<MechanicalCategory>,
    tags_by_role: BTreeMap<String, Vec<String>>,
    other_tags: Vec<String>,
) -> Element {
    if roles.is_empty() && other_tags.is_empty() {
        return rsx! {};
    }

    // Chip list: each role (label + its tags), then an "Other tags" entry. A chip
    // is expandable iff it has tags; empty roles render as plain chips.
    let mut items: Vec<(String, Vec<String>)> = roles
        .iter()
        .map(|r| {
            let tags = tags_by_role
                .get(&r.to_string())
                .cloned()
                .unwrap_or_default();
            (r.display_name().to_string(), tags)
        })
        .collect();
    if !other_tags.is_empty() {
        items.push(("Other tags".to_string(), other_tags));
    }

    let mut open = use_signal(|| None::<usize>);
    let open_idx = open();
    let reveal_tags: Option<Vec<String>> = open_idx.and_then(|i| items.get(i)).map(|(_, tags)| {
        tags.iter()
            .map(|t| prettify_oracle_tag_slug(t))
            .collect::<Vec<_>>()
    });
    let reveal_class = if reveal_tags.is_some() {
        "keyword-reveal open"
    } else {
        "keyword-reveal"
    };

    rsx! {
        div { class: "card-roles",
            span { class: "chips-label", "Card roles" }
            div { class: "keyword-chips",
                for (i , (label , tags)) in items.iter().enumerate() {
                    if tags.is_empty() {
                        span { key: "{i}", class: "detail-chip", "{label}" }
                    } else {
                        button {
                            key: "{i}",
                            class: if open_idx == Some(i) { "keyword-chip active" } else { "keyword-chip" },
                            onclick: move |evt| {
                                evt.stop_propagation();
                                if open() == Some(i) {
                                    open.set(None);
                                } else {
                                    open.set(Some(i));
                                }
                            },
                            "{label}"
                        }
                    }
                }
            }
            div { class: "{reveal_class}",
                div { class: "keyword-reveal-inner",
                    if let Some(tags) = reveal_tags {
                        div { class: "card-detail-meta card-detail-otags",
                            for tag in tags {
                                span { key: "{tag}", class: "detail-chip", "{tag}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
