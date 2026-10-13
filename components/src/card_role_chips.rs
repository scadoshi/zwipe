//! Card roles with drill-down to their oracle tags.
//!
//! Renders the coarse roles a card fulfills (Removal, Ramp, ...) as chips. A role
//! that has functional oracle tags beneath it (server-grouped) eases those tags
//! open when tapped, reusing the keyword hint reveal; a role with none (the
//! heuristic/token roles) is a plain, non-expandable chip. A trailing "Other
//! tags" chip holds the card's functional tags that fall under no role. This is
//! the card's whole tag story: role = the high-level thing, oracle tags = the
//! specific things underneath, distinct from deck-level Deck Tags.
//!
//! When the host supplies `describe_tag` / `on_examples`, each exposed oracle tag
//! becomes tappable too: it eases its dictionary definition open beneath the tag
//! row (with an optional "Examples" button), telescoping under the role. The tag
//! reveal is nested inside the role reveal, so collapsing the role hides it and
//! reopening the role restores it (its open state persists, like the keywords).

use dioxus::prelude::*;
use std::collections::BTreeMap;
use zwipe_core::domain::card::card_role::role_label;

/// Card roles as chips; expandable to their grouped oracle tags, plus an "Other
/// tags" bucket. `tags_by_role` is keyed by role slug (`CardRole`'s
/// snake_case form); `other_tags` are the uncategorized functional tags.
#[component]
pub fn CardRoleChips(
    roles: Vec<String>,
    tags_by_role: BTreeMap<String, Vec<String>>,
    other_tags: Vec<String>,
    /// Optional help affordance rendered beside the "Card roles" label (e.g.
    /// an `InfoButton`). Left to the consumer since this crate can't depend
    /// on zwiper's session-aware hint plumbing.
    #[props(default)]
    help: Option<Element>,
    /// Resolve an oracle tag's plain-language description (from the host's catalog
    /// cache). When set, exposed tags become tappable and reveal their definition
    /// inline. `None` from the callback renders as "No description yet".
    #[props(default)]
    describe_tag: Option<Callback<String, Option<String>>>,
    /// Open the example-cards browse for a tag slug. When set, an expanded tag's
    /// reveal shows an "Examples" button. Left to the host since the browse is a
    /// zwiper overlay.
    #[props(default)]
    on_examples: Option<Callback<String>>,
) -> Element {
    if roles.is_empty() && other_tags.is_empty() {
        return rsx! {};
    }

    // Chip list: each role (label + its tags), then an "Other tags" entry. A chip
    // is expandable iff it has tags; empty roles render as plain chips.
    let mut items: Vec<(String, Vec<String>)> = roles
        .iter()
        .map(|slug| {
            let tags = tags_by_role.get(slug).cloned().unwrap_or_default();
            (role_label(slug), tags)
        })
        .collect();
    if !other_tags.is_empty() {
        items.push(("Other tags".to_string(), other_tags));
    }

    let mut open = use_signal(|| None::<usize>);
    // `shown` holds the last-opened index and is NOT cleared on close, so the
    // revealed tags stay mounted while the container animates collapsing. Clearing
    // it (like `open`) would yank the DOM node instantly and snap the close shut.
    let mut shown = use_signal(|| None::<usize>);
    let open_idx = open();
    let reveal_tags: Vec<String> = shown()
        .and_then(|i| items.get(i))
        .map(|(_, tags)| tags.to_vec())
        .unwrap_or_default();
    let reveal_class = if open_idx.is_some() {
        "keyword-reveal open"
    } else {
        "keyword-reveal"
    };

    // Second telescope level: which exposed tag's definition is open. Persists
    // across role collapse/reopen (never cleared on role toggle), mirroring how the
    // role reveal itself keeps state. `shown_tag` keeps the last one mounted through
    // the collapse animation, same reason as `shown`.
    let mut open_tag = use_signal(|| None::<String>);
    let mut shown_tag = use_signal(|| None::<String>);
    let tags_expandable = describe_tag.is_some() || on_examples.is_some();

    rsx! {
        div { class: "card-roles",
            span { class: "chips-label", "Card roles" }
            if let Some(help) = help {
                {help}
            }
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
                                    shown.set(Some(i));
                                }
                            },
                            "{label}"
                        }
                    }
                }
            }
            div { class: "{reveal_class}",
                div { class: "keyword-reveal-inner",
                    if !reveal_tags.is_empty() {
                        // Block-quote frame (matches the keyword reminder) so it
                        // reads as the tapped role's exposed oracle tags.
                        div { class: "otag-reveal-block",
                            div { class: "card-detail-meta card-detail-otags",
                                for tag in reveal_tags.iter().cloned() {
                                    if tags_expandable {
                                        {
                                            // One clone owned by the toggle closure; `tag` itself
                                            // stays for the key/class/label.
                                            let slug = tag.clone();
                                            rsx! {
                                                button {
                                                    key: "{tag}",
                                                    class: if open_tag().as_deref() == Some(tag.as_str()) { "keyword-chip active" } else { "keyword-chip" },
                                                    onclick: move |evt| {
                                                        evt.stop_propagation();
                                                        if open_tag().as_deref() == Some(slug.as_str()) {
                                                            open_tag.set(None);
                                                        } else {
                                                            open_tag.set(Some(slug.clone()));
                                                            shown_tag.set(Some(slug.clone()));
                                                        }
                                                    },
                                                    "{tag}"
                                                }
                                            }
                                        }
                                    } else {
                                        span { key: "{tag}", class: "detail-chip", "{tag}" }
                                    }
                                }
                            }
                            if tags_expandable {
                                {
                                    // Only reveal for a tag that belongs to the
                                    // currently-shown role, so switching roles hides
                                    // a stale definition (and coming back restores it).
                                    let shown = shown_tag();
                                    let in_role = shown.as_ref().is_some_and(|s| reveal_tags.contains(s));
                                    let is_open = open_tag().as_ref().is_some_and(|s| reveal_tags.contains(s));
                                    let slug = shown.filter(|_| in_role);
                                    let def = slug.as_ref().map(|s| {
                                        describe_tag
                                            .and_then(|cb| cb.call(s.clone()))
                                            .unwrap_or_else(|| "No description yet".to_string())
                                    });
                                    let tag_reveal_class = if is_open { "keyword-reveal open" } else { "keyword-reveal" };
                                    rsx! {
                                        div { class: "{tag_reveal_class}",
                                            div { class: "keyword-reveal-inner",
                                                if let Some(def) = def {
                                                    div { class: "otag-def",
                                                        p { class: "otag-def-text", "{def}" }
                                                        if let (Some(handler), Some(slug)) = (on_examples, slug) {
                                                            button {
                                                                class: "otag-examples-btn",
                                                                onclick: move |evt| {
                                                                    evt.stop_propagation();
                                                                    handler.call(slug.clone());
                                                                },
                                                                "Examples"
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
                }
            }
        }
    }
}
