use crate::inbound::{
    components::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger},
    screens::deck::card::filter::{
        artist::Artist, category::Category, combat::Combat, config::Config, flavor_text::FlavorText,
        format::FormatFilter, mana::Mana, name::Name, oracle_text::OracleText, rarity::Rarity,
        set::Set, sort::Sort, types::Types,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;

/// Shared bottom-sheet filter accordion used by add, view, and remove card screens.
///
/// Reads `Signal<CardFilterBuilder>` and `Signal<u32>` (filter_reset_counter) from context.
#[component]
pub(crate) fn CardFilterSheet(
    mut open: Signal<bool>,
    #[props(default = false)] show_format_filter: bool,
    #[props(default = false)] show_active_indicators: bool,
    #[props(default = false)] validate_before_apply: bool,
    on_clear: Option<EventHandler>,
) -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    let mut filter_reset_counter: Signal<u32> = use_context();
    let should_collapse: Option<Signal<bool>> = try_use_context::<Signal<bool>>();
    let toast = use_toast();
    let mut accordion_key = use_signal(|| 0u32);

    // Bump the filter counter and signal that the expanded card should collapse.
    let mut bump_filter = move || {
        if let Some(mut collapse) = should_collapse {
            collapse.set(true);
        }
        filter_reset_counter.set(filter_reset_counter() + 1);
    };

    // Active indicator booleans (only computed when needed)
    let (
        name_active,
        oracle_active,
        types_active,
        mana_active,
        combat_active,
        flavor_active,
        artist_active,
        rarity_active,
        category_active,
        set_active,
        sort_active,
        config_active,
        format_active,
    ) = if show_active_indicators {
        let fb = filter_builder();
        let def = CardFilterBuilder::default();
        (
            fb.name_contains().is_some(),
            fb.oracle_text_contains().is_some()
                || fb.oracle_text_contains_any().is_some()
                || fb.oracle_text_contains_all().is_some()
                || fb.keywords_contains_any().is_some()
                || fb.keywords_contains_all().is_some(),
            fb.type_line_contains().is_some()
                || fb.type_line_contains_any().is_some()
                || fb.type_line_contains_all().is_some()
                || fb.card_type_contains_any().is_some()
                || fb.card_type_contains_all().is_some(),
            fb.cmc_equals().is_some()
                || fb.cmc_range().is_some()
                || fb.color_identity_equals().is_some()
                || fb.color_identity_within().is_some()
                || fb.produced_mana_contains_any().is_some()
                || fb.produced_mana_contains_all().is_some(),
            fb.power_equals().is_some()
                || fb.power_range().is_some()
                || fb.toughness_equals().is_some()
                || fb.toughness_range().is_some(),
            fb.flavor_text_contains().is_some() || fb.has_flavor_text().is_some(),
            fb.artist_equals_any().is_some(),
            fb.rarity_equals_any().is_some(),
            fb.mechanical_categories_contains_any().is_some()
                || fb.mechanical_categories_contains_all().is_some(),
            fb.set_equals_any().is_some(),
            fb.order_by().is_some(),
            fb.is_playable() != def.is_playable()
                || fb.digital() != def.digital()
                || fb.oversized() != def.oversized()
                || fb.promo() != def.promo()
                || fb.content_warning() != def.content_warning(),
            fb.legalities_contains_any().is_some()
                || fb.is_commander_in_format().is_some()
                || fb.is_partner().is_some()
                || fb.is_background().is_some()
                || fb.is_signature_spell().is_some(),
        )
    } else {
        (false, false, false, false, false, false, false, false, false, false, false, false, false)
    };

    // Track accordion item index — shifts when format filter is included
    let mut idx = 0usize;
    let mut next_idx = move || {
        idx += 1;
        idx
    };

    rsx! {
        // Modal backdrop
        div {
            class: if open() { "modal-backdrop show" } else { "modal-backdrop" },
            onclick: move |_| open.set(false),
        }

        // Bottom sheet
        div {
            class: if open() { "bottom-sheet show" } else { "bottom-sheet" },

            div { class: "modal-header",
                span { class: "text-muted", style: "font-size: 1rem;", "filter" }
            }

            div { class: "modal-content",
                Accordion {
                    key: "{accordion_key()}",
                    id: "filter-accordion",
                    allow_multiple_open: false,
                    collapsible: true,

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(1)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            "name"
                            if name_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        filter_builder.write().unset_name_contains();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Name {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(2)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            "oracle text"
                            if oracle_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let fb = &mut *filter_builder.write();
                                        fb.unset_oracle_text_contains();
                                        fb.unset_oracle_text_contains_any();
                                        fb.unset_oracle_text_contains_all();
                                        fb.unset_keywords_contains_any();
                                        fb.unset_keywords_contains_all();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { OracleText {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(3)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            "types"
                            if types_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let fb = &mut *filter_builder.write();
                                        fb.unset_type_line_contains();
                                        fb.unset_type_line_contains_any();
                                        fb.unset_type_line_contains_all();
                                        fb.unset_card_type_contains_any();
                                        fb.unset_card_type_contains_all();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Types {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(4)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            "mana"
                            if mana_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let fb = &mut *filter_builder.write();
                                        fb.unset_cmc_equals();
                                        fb.unset_cmc_range();
                                        fb.unset_color_identity_equals();
                                        fb.unset_color_identity_within();
                                        fb.unset_produced_mana_contains_any();
                                        fb.unset_produced_mana_contains_all();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Mana {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(5)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            "combat"
                            if combat_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let fb = &mut *filter_builder.write();
                                        fb.unset_power_equals();
                                        fb.unset_power_range();
                                        fb.unset_toughness_equals();
                                        fb.unset_toughness_range();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Combat {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(6)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            "flavor text"
                            if flavor_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let fb = &mut *filter_builder.write();
                                        fb.unset_flavor_text_contains();
                                        fb.unset_has_flavor_text();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { FlavorText {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(7)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            "artist"
                            if artist_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        filter_builder.write().unset_artist_equals_any();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Artist {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(8)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            "rarity"
                            if rarity_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        filter_builder.write().unset_rarity_equals_any();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Rarity {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(9)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            "category"
                            if category_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let fb = &mut *filter_builder.write();
                                        fb.unset_mechanical_categories_contains_any();
                                        fb.unset_mechanical_categories_contains_all();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Category {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(9)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            "set"
                            if set_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        filter_builder.write().unset_set_equals_any();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Set {} }
                    }

                    if show_format_filter {
                        AccordionItem { index: next_idx(),
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(10)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger {
                                "format"
                                if format_active {
                                    button {
                                        class: "clear-btn",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            let fb = &mut *filter_builder.write();
                                            fb.unset_legalities_contains_any();
                                            fb.unset_is_commander_in_format();
                                            fb.unset_is_partner();
                                            fb.unset_is_background();
                                            fb.unset_is_signature_spell();
                                            bump_filter();
                                        },
                                        "\u{00d7}"
                                    }
                                }
                            }
                            AccordionContent { FormatFilter {} }
                        }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval(
                                &format!("setTimeout(() => {{ const el = document.querySelector('#filter-accordion .accordion-item:nth-child({})'); if (el) el.scrollIntoView({{ behavior: 'smooth', block: 'start' }}); }}, 50)",
                                if show_format_filter { 11 } else { 10 })
                            ); }
                        },
                        AccordionTrigger {
                            "sort"
                            if sort_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        filter_builder.write().unset_order_by();
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Sort {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval(
                                &format!("setTimeout(() => {{ const el = document.querySelector('#filter-accordion .accordion-item:nth-child({})'); if (el) el.scrollIntoView({{ behavior: 'smooth', block: 'start' }}); }}, 50)",
                                if show_format_filter { 12 } else { 11 })
                            ); }
                        },
                        AccordionTrigger {
                            "config"
                            if config_active {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let fb = &mut *filter_builder.write();
                                        fb.set_is_playable(true);
                                        fb.set_digital(false);
                                        fb.set_oversized(false);
                                        fb.set_promo(false);
                                        fb.set_content_warning(false);
                                        bump_filter();
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Config {} }
                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        if validate_before_apply && filter_builder.read().is_empty_ignoring_deck_context() {
                            toast.warning("filter is empty".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                        } else {
                            bump_filter();
                        }
                        accordion_key.set(accordion_key() + 1);
                        open.set(false);
                    },
                    "apply"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        if let Some(handler) = &on_clear {
                            handler.call(());
                        } else {
                            filter_builder.write().clear();
                            bump_filter();
                            toast.info(
                                "filter cleared".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                    },
                    "clear"
                }
            }
        }
    }
}
