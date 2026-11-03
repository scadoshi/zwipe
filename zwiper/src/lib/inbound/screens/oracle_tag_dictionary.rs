//! Oracle-tag dictionary screen.
//!
//! In-app, read-only reference of the full ~4,500-tag oracle-tag catalog with our
//! authored descriptions. **Letter-first browse**: a horizontally scrolling A–Z
//! rail where only the selected letter's tags mount, with optional search over
//! slug + label + description. Reached from the oracle-tag picker and its hint;
//! authed like every app screen (behind the router `AuthGate`).
//!
//! The catalog is read from the shared app-wide [`CatalogCache`](crate::inbound::components::catalog_cache::CatalogCache) (prefetched at
//! startup, 1-day TTL, stale-while-revalidate), the same copy the oracle-tag
//! picker and card filter read: one fetch of the ~4,500-row list per session.

use components::{
    HintBullet, HintBullets, HintDialog, HintKey, Toasts, use_overlay_back, use_toast,
};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::hints::HINT_ORACLE_TAG_DICTIONARY;
use zwipe_ui::screens::oracle_tag_dictionary::{
    self as model, Action, Body, Catalog, Destination, Effect, List, State,
};

use crate::inbound::components::catalog_cache::{CatalogCache, CatalogCell};
use crate::inbound::components::hint_dialog::{self, HintColored};
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::{guide_link, ui_toast};
use crate::inbound::screens::oracle_tag_examples::OracleTagExamples;

/// The dictionary's model with the handles that perform its effects. Copy,
/// so every handler holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    open: Signal<bool>,
    on_use: EventHandler<String>,
    cache: CatalogCache,
    toast: Toasts,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    hint_open: Signal<bool>,
    examples_open: Signal<bool>,
    examples_slug: Signal<String>,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one effect.
    fn perform(self, effect: Effect) {
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(
                    hint.key(),
                    self.session,
                    self.client,
                    self.hint_open,
                );
            }
            Effect::EnsureCatalog => self.cache.ensure_oracle_tags(self.client),
            Effect::AdoptTag(slug) => self.on_use.call(slug),
            Effect::Navigate(Destination::Previous) => {
                let mut open = self.open;
                open.set(false);
            }
            Effect::Navigate(Destination::Examples(slug)) => {
                let (mut examples_slug, mut examples_open) =
                    (self.examples_slug, self.examples_open);
                examples_slug.set(slug);
                examples_open.set(true);
            }
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
        }
    }
}

/// Read-only, searchable dictionary of every oracle tag and its description. An
/// in-place overlay (the host renders it while `open`); each row offers **Examples**
/// (opens the example-cards browse, stacked on top) and **Use** (`on_use`, which the
/// host wires to adopt the tag into the deck or filter without leaving the picker).
///
/// What the dictionary knows and does is `zwipe_ui::screens::oracle_tag_dictionary`;
/// this draws its state and performs its effects.
#[component]
pub fn OracleTagDictionary(open: Signal<bool>, on_use: EventHandler<String>) -> Element {
    use_overlay_back(open);
    let driver = Driver {
        model: use_signal(State::new),
        open,
        on_use,
        cache: use_context(),
        toast: use_toast(),
        session: use_context(),
        client: use_context(),
        hint_open: use_signal(|| false),
        // Nested example-cards browse, stacked above the dictionary. Owned here so
        // the dictionary stays mounted underneath while examples are open.
        examples_open: use_signal(|| false),
        examples_slug: use_signal(String::new),
    };
    let (hint, examples_open, examples_slug) =
        (driver.hint_open, driver.examples_open, driver.examples_slug);

    // Offers the hint and warms or revalidates the shared oracle-tag catalog as
    // the dictionary first draws. The catalog is single-flight and usually
    // already Loaded from the startup prefetch, so this is a cheap no-op.
    use_hook(move || driver.dispatch(Action::Load));
    let cell = driver.cache.oracle_tags.cell();

    // Surface a hard failure (no cached data and the fetch failed).
    use_effect(move || {
        if matches!(&*cell.read(), CatalogCell::Failed) {
            driver.dispatch(Action::CatalogFailed);
        }
    });

    let (body, query) = {
        let model = driver.model.read();
        let cell_read = cell.read();
        let catalog = match &*cell_read {
            CatalogCell::Loading => Catalog::Loading,
            CatalogCell::Failed => Catalog::Failed,
            CatalogCell::Loaded { data, .. } => Catalog::Loaded(data),
        };
        (model.body(catalog), model.query().to_owned())
    };

    rsx! {
        div { class: "screen dict-overlay",
            // An overlay screen paints its own backdrop, since .screen is clear
            // and the screen beneath would show through.
            ::components::GridCells { still: true }
            ScreenHeader { title: model::TITLE, hint, back: move |()| driver.dispatch(Action::Back) }

            div { class: "screen-content content-enter",
                match body {
                    Body::Loading => rsx! { DictionarySkeleton {} },
                    Body::Failed(line) => rsx! {
                        div { class: "dict-empty", "{line}" }
                    },
                    // Switching letters / searching changes every row's key, so
                    // Dioxus remounts the rows and their entrance animation replays
                    // (mirrors the changelog filter pattern, per-row not per-list).
                    Body::Browse { rail, list } => rsx! {
                        div { class: "dict-controls",
                            input {
                                class: "input",
                                id: "oracle-tag-dictionary-search",
                                r#type: "text",
                                placeholder: model::SEARCH_PLACEHOLDER,
                                value: "{query}",
                                autocapitalize: "none",
                                autocorrect: "off",
                                spellcheck: "false",
                                oninput: move |event| driver.dispatch(Action::Search(event.value())),
                            }

                            div { class: "dict-letter-rail scroll-fade-x",
                                for chip in rail {
                                    button {
                                        key: "{chip.letter}",
                                        class: if chip.picked { "chip selected" } else { "chip" },
                                        onclick: move |_| driver.dispatch(Action::PickLetter(chip.letter)),
                                        "{chip.label}"
                                    }
                                }
                            }
                        }

                        div { class: "dict-list",
                            match list {
                                List::Empty(line) => rsx! {
                                    div { class: "dict-empty", "{line}" }
                                },
                                List::Rows(rows) => rsx! {
                                    for t in rows {
                                        {
                                            let slug_ex = t.slug.clone();
                                            let slug_use = t.slug.clone();
                                            rsx! {
                                                div {
                                                    key: "{t.slug}",
                                                    class: "dict-row",
                                                    div { class: "dict-slug", "{t.slug}" }
                                                    div { class: "dict-desc",
                                                        if let Some(desc) = t.description.clone() {
                                                            "{desc}"
                                                        } else {
                                                            span { class: "dict-desc-missing", {model::NO_DESCRIPTION} }
                                                        }
                                                    }
                                                    if !t.parent_slugs.is_empty() {
                                                        div { class: "dict-parents",
                                                            for parent in t.parent_slugs.iter() {
                                                                span { key: "{parent}", class: "dict-parent", "{parent}" }
                                                            }
                                                        }
                                                    }
                                                    hr { class: "dict-row-divider" }
                                                    div { class: "dict-row-actions",
                                                        button {
                                                            class: "chip",
                                                            onclick: move |_| driver.dispatch(Action::OpenExamples(slug_ex.clone())),
                                                            {model::EXAMPLES}
                                                        }
                                                        button {
                                                            class: "chip",
                                                            onclick: move |_| driver.dispatch(Action::Use(slug_use.clone())),
                                                            {model::USE}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                            }
                        }
                    },
                }
            }


            HintDialog { open: hint, actions: guide_link::one_time(HINT_ORACLE_TAG_DICTIONARY), title: model::TITLE,
            HintBullets {
                HintBullet { "Tap a " HintKey { color: "--palette-1", "letter" } " to browse, or " HintColored { color: "--accent-secondary", "search" } " by name or description" }
                HintBullet { HintKey { "Examples" } " shows real cards with that tag" }
                HintBullet { HintKey { "Use" } " adds the tag where you came from" }
                HintBullet { "Some tags have no description yet" }
            }
        }

            if examples_open() {
                OracleTagExamples { open: examples_open, slug: examples_slug() }
            }
        }
    }
}

/// Placeholder shown while the catalog fetch is in flight: a fake letter rail plus
/// a few entry bars, in the app's skeleton language.
#[component]
fn DictionarySkeleton() -> Element {
    rsx! {
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
