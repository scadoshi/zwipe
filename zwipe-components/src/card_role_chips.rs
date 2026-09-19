//! Card roles (Removal, Ramp, ...) as chips. A role with oracle tags beneath it
//! eases them open when tapped; a trailing "Other tags" chip holds tags under
//! no role. With `describe_tag` or `on_examples` set, each tag is tappable too
//! and telescopes its definition open under the role.

use dioxus::prelude::*;
use std::collections::BTreeMap;
use zwipe_core::domain::card::card_role::role_label;

/// Card roles as chips, expandable to their oracle tags. `tags_by_role` is
/// keyed by role slug.
#[component]
pub fn CardRoleChips(
    roles: Vec<String>,
    tags_by_role: BTreeMap<String, Vec<String>>,
    other_tags: Vec<String>,
    /// Help affordance beside the "Card roles" label. Left to the host since
    /// this crate can't depend on zwiper's hint plumbing.
    #[props(default)]
    help: Option<Element>,
    /// Resolve an oracle tag's description. `None` renders "No description yet".
    #[props(default)]
    describe_tag: Option<Callback<String, Option<String>>>,
    /// Open the example-cards browse for a tag slug; adds an "Examples" button
    /// to an expanded tag.
    #[props(default)]
    on_examples: Option<Callback<String>>,
) -> Element {
    if roles.is_empty() && other_tags.is_empty() {
        return rsx! {};
    }

    // A chip is expandable iff it has tags.
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
    // `shown` is not cleared on close, so the revealed tags stay mounted while
    // the container animates shut.
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

    // Persists across role collapse and reopen. `shown_tag` works like `shown`.
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
                        div { class: "otag-reveal-block",
                            div { class: "card-detail-meta card-detail-otags",
                                for tag in reveal_tags.iter().cloned() {
                                    if tags_expandable {
                                        {
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
                                    // Only reveal a tag under the shown role, so switching
                                    // roles hides a stale definition.
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
