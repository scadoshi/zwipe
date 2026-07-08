//! Card rarity filter component.

use dioxus::prelude::*;
use zwipe_core::domain::card::{
    scryfall_data::rarity::Rarity as CardRarity,
    search_card::card_filter::builder::CardQueryBuilder,
};

/// Whether the rarity filter is in include or exclude mode.
#[derive(Debug, Clone, Copy, PartialEq)]
enum IncludeExclude {
    Include,
    Exclude,
}

impl IncludeExclude {
    fn toggle(self) -> Self {
        match self {
            Self::Include => Self::Exclude,
            Self::Exclude => Self::Include,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Include => "Include",
            Self::Exclude => "Exclude",
        }
    }
}

/// Filter component for card rarity (common, uncommon, rare, mythic).
#[component]
pub fn Rarity() -> Element {
    let mut filter_builder: Signal<CardQueryBuilder> = use_context();

    let mut mode = use_signal(|| {
        if filter_builder().rarity_excludes_any().is_some() {
            IncludeExclude::Exclude
        } else {
            IncludeExclude::Include
        }
    });

    let selected = match mode() {
        IncludeExclude::Include => filter_builder()
            .rarity_equals_any()
            .map(|r| r.to_vec())
            .unwrap_or_default(),
        IncludeExclude::Exclude => filter_builder()
            .rarity_excludes_any()
            .map(|r| r.to_vec())
            .unwrap_or_default(),
    };

    let has_selection = !selected.is_empty();

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row mt-2",
                label { class: "label-xs", "Rarity" }
                if has_selection {
                    button {
                        class: "chip-xs",
                        onclick: move |_| {
                            let new_mode = mode().toggle();
                            // Move values to the other field
                            let current = match mode() {
                                IncludeExclude::Include => filter_builder()
                                    .rarity_equals_any()
                                    .map(|r| r.to_vec())
                                    .unwrap_or_default(),
                                IncludeExclude::Exclude => filter_builder()
                                    .rarity_excludes_any()
                                    .map(|r| r.to_vec())
                                    .unwrap_or_default(),
                            };
                            let fb = &mut *filter_builder.write();
                            fb.unset_rarity_equals_any();
                            fb.unset_rarity_excludes_any();
                            if !current.is_empty() {
                                match new_mode {
                                    IncludeExclude::Include => {
                                        fb.set_rarity_equals_any(current.into());
                                    }
                                    IncludeExclude::Exclude => {
                                        fb.set_rarity_excludes_any(current.into());
                                    }
                                }
                            }
                            mode.set(new_mode);
                        },
                        "{mode().label()}"
                    }
                }
                if has_selection {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            let fb = &mut *filter_builder.write();
                            fb.unset_rarity_equals_any();
                            fb.unset_rarity_excludes_any();
                        },
                        "×"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 flex-center",
                for rarity in CardRarity::all() {
                    div {
                        class: if selected.contains(&rarity) {
                            match mode() {
                                IncludeExclude::Include => "chip selected",
                                IncludeExclude::Exclude => "chip selected",
                            }
                        } else {
                            "chip"
                        },
                        onclick: move |_| {
                            let current = match mode() {
                                IncludeExclude::Include => filter_builder()
                                    .rarity_equals_any()
                                    .map(|r| r.to_vec())
                                    .unwrap_or_default(),
                                IncludeExclude::Exclude => filter_builder()
                                    .rarity_excludes_any()
                                    .map(|r| r.to_vec())
                                    .unwrap_or_default(),
                            };

                            let mut new = current;
                            if new.contains(&rarity) {
                                new.retain(|r| r != &rarity);
                            } else {
                                new.push(rarity);
                            }

                            let fb = &mut *filter_builder.write();
                            match mode() {
                                IncludeExclude::Include => {
                                    if new.is_empty() {
                                        fb.unset_rarity_equals_any();
                                    } else {
                                        fb.set_rarity_equals_any(new.into());
                                    }
                                }
                                IncludeExclude::Exclude => {
                                    if new.is_empty() {
                                        fb.unset_rarity_excludes_any();
                                    } else {
                                        fb.set_rarity_excludes_any(new.into());
                                    }
                                }
                            }
                        },
                        { rarity.to_long_name() }
                    }
                }
            }
        }
    }
}
