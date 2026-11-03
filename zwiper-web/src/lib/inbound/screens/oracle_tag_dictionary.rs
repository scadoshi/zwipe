//! The oracle tag dictionary: every oracle tag and its description, browsed
//! by letter or searched, with `Examples` and `Use` on each row. Not a
//! route; it opens over the picker or filter that asked.
//!
//! A host mounts it while open and adopts the tag `Use` hands back:
//!
//! ```ignore
//! if dictionary_open() {
//!     OracleTagDictionary { open: dictionary_open, on_use: move |slug: String| adopt(slug) }
//! }
//! ```
//!
//! On a wide window it is a large modal with the letter rail as a column on
//! the left; below the breakpoint it is the app's full-screen overlay. `/`
//! focuses the search, a letter key jumps to that letter while the search
//! is empty and unfocused, and Esc is `Back`.

use components::{ActionBar, Button, ButtonVariant, PageHeader, TOAST_NORMAL, use_overlay_back};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_ui::dictionary;

use crate::inbound::catalog::CatalogCache;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::chrome::page_keys::{focus_by_id, use_page_keys};
use crate::inbound::screens::oracle_tag_examples::OracleTagExamples;

/// The search input's id, for `/`.
const SEARCH_ID: &str = "oracle-tag-dictionary-search";

/// The dictionary over its host while `open`. `on_use` takes the slug of
/// the row whose `Use` was pressed.
#[component]
pub fn OracleTagDictionary(mut open: Signal<bool>, on_use: EventHandler<String>) -> Element {
    use_overlay_back(open);
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();
    let toast = use_toast();
    let hint_open = use_one_time_hint(OneTimeHint::OracleTagDictionary);
    let mut selected_letter = use_signal(|| 'a');
    let mut query = use_signal(String::new);
    let mut examples_open = use_signal(|| false);
    let mut examples_slug = use_signal(String::new);

    use_hook(move || cache.ensure_oracle_tags(client));
    let mut warned = use_signal(|| false);
    use_effect(move || {
        if cache.0.read().oracle_tags.is_failed() && !*warned.peek() {
            warned.set(true);
            toast.error(
                "Couldn't load the oracle-tag dictionary".to_string(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
        }
    });

    use_page_keys(Callback::new(move |key: String| {
        if key == "/" {
            focus_by_id(SEARCH_ID);
            return;
        }
        let mut chars = key.chars();
        if let (Some(c), None) = (chars.next(), chars.next())
            && c.is_ascii_alphabetic()
            && query.peek().trim().is_empty()
            && !*examples_open.peek()
        {
            selected_letter.set(c.to_ascii_lowercase());
        }
    }));

    let catalog = cache.0.read();
    let slot = &catalog.oracle_tags;
    let body = if let Some(tags) = slot.ready() {
        let q = query().trim().to_lowercase();
        let searching = !q.is_empty();
        let rail = dictionary::letters(tags);
        let letter = selected_letter();
        let rows = dictionary::rows_for(tags, &q, letter);
        rsx! {
            div { class: "web-dict-browse",
                div { class: "dict-controls",
                    input {
                        class: "input",
                        id: SEARCH_ID,
                        r#type: "text",
                        placeholder: "Search tags or descriptions",
                        value: "{query()}",
                        autocapitalize: "none",
                        autocorrect: "off",
                        spellcheck: "false",
                        oninput: move |event| query.set(event.value()),
                    }
                    div { class: "dict-letter-rail scroll-fade-x",
                        for c in rail {
                            button {
                                key: "{c}",
                                class: if !searching && c == letter { "chip selected" } else { "chip" },
                                onclick: move |_| {
                                    query.set(String::new());
                                    selected_letter.set(c);
                                },
                                "{c.to_ascii_uppercase()}"
                            }
                        }
                    }
                }
                div { class: "dict-list",
                    if rows.is_empty() {
                        if searching {
                            div { class: "dict-empty", "No tags match \u{201c}{q}\u{201d}." }
                        } else {
                            div { class: "dict-empty", "No tags start with {letter.to_ascii_uppercase()}." }
                        }
                    } else {
                        for t in rows {
                            DictionaryRow {
                                key: "{t.slug}",
                                tag: t,
                                on_examples: move |slug: String| {
                                    examples_slug.set(slug);
                                    examples_open.set(true);
                                },
                                on_use,
                            }
                        }
                    }
                }
            }
        }
    } else if slot.is_failed() {
        rsx! {
            div { class: "dict-empty", "Couldn't load the dictionary. Pull back and try again." }
        }
    } else {
        rsx! { DictionarySkeleton {} }
    };
    drop(catalog);

    rsx! {
        div { class: "screen dict-overlay web-large-modal web-dictionary",
            components::GridCells { still: true }
            PageHeader {
                title: "Oracle tags",
                leading: rsx! { span { class: "page-header-corner" } },
                on_hint: move |()| {
                    let mut hint_open = hint_open;
                    hint_open.set(true);
                },
            }
            div { class: "screen-content content-enter", {body} }
            ActionBar {
                Button { variant: ButtonVariant::Action, onclick: move |_| open.set(false), "Back" }
            }
            OneTimeHintDialog { hint: OneTimeHint::OracleTagDictionary, open: hint_open }
            if examples_open() {
                for slug in [examples_slug()] {
                    OracleTagExamples { key: "{slug}", open: examples_open, slug }
                }
            }
        }
    }
}

/// One tag: its slug, description (or `No description yet`), parent tags,
/// then `Examples` and `Use`.
#[component]
fn DictionaryRow(
    tag: OracleTag,
    on_examples: EventHandler<String>,
    on_use: EventHandler<String>,
) -> Element {
    let slug_examples = tag.slug.clone();
    let slug_use = tag.slug.clone();
    rsx! {
        div { class: "dict-row",
            div { class: "dict-slug", "{tag.slug}" }
            div { class: "dict-desc",
                if let Some(desc) = tag.description.clone() {
                    "{desc}"
                } else {
                    span { class: "dict-desc-missing", "No description yet" }
                }
            }
            if !tag.parent_slugs.is_empty() {
                div { class: "dict-parents",
                    for parent in tag.parent_slugs.iter() {
                        span { key: "{parent}", class: "dict-parent", "{parent}" }
                    }
                }
            }
            hr { class: "dict-row-divider" }
            div { class: "dict-row-actions",
                button {
                    class: "chip",
                    onclick: move |_| on_examples.call(slug_examples.clone()),
                    "Examples"
                }
                button { class: "chip", onclick: move |_| on_use.call(slug_use.clone()), "Use" }
            }
        }
    }
}

/// The dictionary while the catalog loads: the search box, a rail of letter
/// ghosts and seven ghost rows.
#[component]
fn DictionarySkeleton() -> Element {
    rsx! {
        div { class: "web-dict-browse",
            div { class: "dict-controls",
                div { class: "skeleton-bar dict-skeleton-search" }
                div { class: "dict-letter-rail scroll-fade-x",
                    for i in 0..12 {
                        div { key: "{i}", class: "skeleton-bar dict-skeleton-letter" }
                    }
                }
            }
            div { class: "dict-list",
                for i in 0..7 {
                    div { key: "{i}", class: "dict-row",
                        div { class: "skeleton-bar dict-skeleton-slug" }
                        div { class: "skeleton-bar dict-skeleton-desc" }
                    }
                }
            }
        }
    }
}
