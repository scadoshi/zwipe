//! A card role's oracle tags as chips, each tappable for its description and
//! Examples button eased open beneath them. Shared by the role charts that
//! reveal tags under a tapped role: the Playstyle screen's Card roles, the
//! deck view's Roles chart and the shared deck page's.

use dioxus::prelude::*;

/// An oracle tag's description with its Examples button.
///
/// `describe_tag` resolves the description; a missing callback, or one that
/// returns `None`, reads "No description yet". The Examples button shows only
/// with `on_examples` set, which the app passes and the site does not.
#[component]
pub fn OtagDef(
    slug: String,
    #[props(default)] describe_tag: Option<Callback<String, Option<String>>>,
    #[props(default)] on_examples: Option<Callback<String>>,
) -> Element {
    let def = describe_tag
        .and_then(|cb| cb.call(slug.clone()))
        .unwrap_or_else(|| "No description yet".to_string());
    rsx! {
        div { class: "otag-def",
            p { class: "otag-def-text", "{def}" }
            if let Some(handler) = on_examples {
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

/// A tapped role's oracle tags as chips, each tappable for its [`OtagDef`]
/// eased open beneath them, one at a time, the "Why this commander?" chips'
/// reveal. Renders nothing for an empty list, so no bare frame shows.
#[component]
pub fn RoleTags(
    tags: Vec<String>,
    #[props(default)] describe_tag: Option<Callback<String, Option<String>>>,
    #[props(default)] on_examples: Option<Callback<String>>,
) -> Element {
    if tags.is_empty() {
        return rsx! {};
    }
    let mut open_tag: Signal<Option<String>> = use_signal(|| None);
    // The last opened tag, kept through a close so its description eases out.
    let mut shown_tag: Signal<Option<String>> = use_signal(|| None);
    let open = open_tag().filter(|slug| tags.contains(slug));
    let shown = shown_tag().filter(|slug| tags.contains(slug));
    let reveal_class = if open.is_some() {
        "keyword-reveal open"
    } else {
        "keyword-reveal"
    };
    rsx! {
        div { class: "otag-reveal-block",
            div { class: "card-detail-meta card-detail-otags",
                for tag in tags.iter().cloned() {
                    {
                        let slug = tag.clone();
                        rsx! {
                            button {
                                key: "{tag}",
                                class: if open.as_deref() == Some(tag.as_str()) { "keyword-chip active" } else { "keyword-chip" },
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
                }
            }
            div { class: "{reveal_class}",
                div { class: "keyword-reveal-inner",
                    if let Some(slug) = shown {
                        OtagDef { slug, describe_tag, on_examples }
                    }
                }
            }
        }
    }
}
