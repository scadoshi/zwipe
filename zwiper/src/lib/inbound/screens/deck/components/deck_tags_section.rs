//! Tag rows for the deck view's collapsible "Tags" section: deck tags, oracle
//! tags, and other tags. Split out of the profile card so the profile stays
//! compact (a deck can carry many oracle tags). Rendered inside a
//! [`CollapsibleSection`](super::collapsible_section::CollapsibleSection).

use dioxus::prelude::*;
use zwipe_core::domain::{
    card::oracle_tag::prettify_oracle_tag_slug, deck::deck_profile::DeckProfile,
};

/// Whether the deck has any tag worth showing the Tags section for.
pub(crate) fn has_any_tags(p: &DeckProfile) -> bool {
    !p.tags.is_empty() || !p.oracle_tags.is_empty() || !p.other_tags.is_empty()
}

/// Total tag count across all three axes (for the collapsed-section badge).
pub(crate) fn total_tag_count(p: &DeckProfile) -> usize {
    p.tags.len() + p.oracle_tags.len() + p.other_tags.len()
}

/// The tag rows (deck tags, oracle tags, other tags), each shown only when
/// non-empty. Mirrors `DeckStats`' plain flex-column layout for a collapsible body.
#[component]
pub(crate) fn DeckTagsSection(deck_profile: DeckProfile) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;",
            if !deck_profile.tags.is_empty() {
                div { class: "info-row",
                    span { class: "info-row-label", "Deck tags" }
                    span { class: "info-row-value info-row-tags",
                        for tag in deck_profile.tags.iter() {
                            span { key: "{tag}", class: "stat-chip stat-chip-tag", "{tag.display_name()}" }
                        }
                    }
                }
            }
            if !deck_profile.oracle_tags.is_empty() {
                div { class: "info-row",
                    span { class: "info-row-label", "Oracle tags" }
                    span { class: "info-row-value info-row-tags",
                        for slug in deck_profile.oracle_tags.iter() {
                            span {
                                key: "{slug}",
                                class: "stat-chip stat-chip-tag",
                                { prettify_oracle_tag_slug(slug) }
                            }
                        }
                    }
                }
            }
            if !deck_profile.other_tags.is_empty() {
                div { class: "info-row",
                    span { class: "info-row-label", "Other tags" }
                    span { class: "info-row-value info-row-tags",
                        for tag in deck_profile.other_tags.iter() {
                            span { key: "{tag}", class: "stat-chip stat-chip-tag", "{tag.display_name()}" }
                        }
                    }
                }
            }
        }
    }
}
