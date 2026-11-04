use components::icons::{self};
use components::{BottomSheet, DockAction, Toasts, use_toast};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::PickerField;
use zwipe_ui::filter::sections::Section;
use zwipe_ui::screens::card_filter::{self, Action, Destination, Effect, ResetNotice, State};

use crate::inbound::components::accordion::{
    Accordion, AccordionContent, AccordionItem, AccordionTrigger,
};
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::hint_dialog::{self, OneTimeHintDialog};
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::ui_toast;
use crate::inbound::screens::deck::card::filter::artist::Artist;
use crate::inbound::screens::deck::card::filter::card_role::CardRole;
use crate::inbound::screens::deck::card::filter::combat::Combat;
use crate::inbound::screens::deck::card::filter::config::Config;
use crate::inbound::screens::deck::card::filter::flavor_text::FlavorText;
use crate::inbound::screens::deck::card::filter::format::FormatFilter;
use crate::inbound::screens::deck::card::filter::mana::Mana;
use crate::inbound::screens::deck::card::filter::name::Name;
use crate::inbound::screens::deck::card::filter::oracle_tags::OracleTags;
use crate::inbound::screens::deck::card::filter::oracle_text::OracleText;
use crate::inbound::screens::deck::card::filter::price::PriceFilter;
use crate::inbound::screens::deck::card::filter::rarity::Rarity;
use crate::inbound::screens::deck::card::filter::set::Set;
use crate::inbound::screens::deck::card::filter::sort::Sort;
use crate::inbound::screens::deck::card::filter::types::Types;
use crate::inbound::screens::oracle_tag_dictionary::OracleTagDictionary;

/// Newtype so `try_use_context` doesn't collide with other `Signal<bool>`
/// contexts: a bare `Signal<bool>` lookup here once grabbed the root
/// min-version gate and flashed the "Update required" screen on Apply.
#[derive(Clone, Copy)]
pub(crate) struct CollapseExpanded(pub(crate) Signal<bool>);

/// The sheet's model with the handles that perform its effects. Copy, so
/// every handler moves it in.
///
/// The sections edit the host's filter from context in place, so that
/// signal is the draft while the sheet is open. Each dispatch hands the model
/// the draft as it stands first, and writes back any change the model made.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    filter: Signal<CardQueryBuilder>,
    counter: Signal<u32>,
    collapse: Option<CollapseExpanded>,
    open: Signal<bool>,
    toast: Toasts,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    hint_open: Signal<bool>,
    dictionary_open: Signal<bool>,
}

impl Driver {
    /// Runs `action` against the draft as the sections left it, writes back
    /// what it changed in the draft, and performs its effects.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let mut filter = self.filter;
        let draft = Box::new(filter.peek().clone());
        let effects = {
            let mut state = model.write();
            let mut effects = state.update(Action::Edit(draft));
            effects.extend(state.update(action));
            effects
        };
        let written = {
            let state = model.peek();
            state
                .draft()
                .filter(|draft| state.is_open() && **draft != *filter.peek())
                .cloned()
        };
        if let Some(draft) = written {
            filter.set(draft);
        }
        for effect in effects {
            self.perform(effect);
        }
    }

    fn perform(self, effect: Effect) {
        let Self {
            mut filter,
            mut counter,
            collapse,
            mut open,
            toast,
            session,
            client,
            hint_open,
            mut dictionary_open,
            ..
        } = self;
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(hint.key(), session, client, hint_open);
            }
            // Each section ensures its own list as it mounts, and reads the
            // deck's cards where the host provides them.
            Effect::EnsureLists(_) => {}
            Effect::Apply(applied) => {
                if *filter.peek() != *applied {
                    filter.set(*applied);
                }
                // The counter bump is the host's refetch; the expanded card
                // collapses with it.
                if let Some(CollapseExpanded(mut collapse)) = collapse {
                    collapse.set(true);
                }
                counter.set(counter().wrapping_add(1));
            }
            Effect::Restore(applied) => filter.set(*applied),
            Effect::Close => {
                if *open.peek() {
                    open.set(false);
                }
            }
            Effect::Navigate(Destination::OracleTagDictionary) => dictionary_open.set(true),
            Effect::Toast(t) => ui_toast::raise(toast, t),
        }
    }
}

/// Shared bottom-sheet filter accordion used by add, view, and remove card screens.
///
/// Reads `Signal<CardQueryBuilder>` and `Signal<u32>` (`filter_reset_counter`) from context.
#[component]
pub(crate) fn CardFilterSheet(
    open: Signal<bool>,
    #[props(default = false)] show_format_filter: bool,
    #[props(default = false)] show_active_indicators: bool,
    #[props(default = false)] validate_before_apply: bool,
    on_clear: Option<EventHandler>,
) -> Element {
    let filter_builder: Signal<CardQueryBuilder> = use_context();
    let filter_reset_counter: Signal<u32> = use_context();
    let should_collapse: Option<CollapseExpanded> = try_use_context::<CollapseExpanded>();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();
    let catalogs: CatalogCache = use_context();

    let hint_open = use_signal(|| false);
    // Oracle-tag dictionary overlay. Owned here (not in the `OracleTags` section)
    // so it renders as a sibling of the bottom sheet, outside the sheet's
    // `transform`; a `position: fixed` overlay nested inside that transform would
    // be trapped to the sheet's box instead of the viewport.
    let otag_dict_open = use_signal(|| false);
    let driver = Driver {
        model: use_signal(State::new),
        filter: filter_builder,
        counter: filter_reset_counter,
        collapse: should_collapse,
        open,
        toast,
        session,
        client,
        hint_open,
        dictionary_open: otag_dict_open,
    };

    // Opening takes the applied filter as the draft's start; any close but
    // Apply puts it back. The sheet stays mounted while closed, so this
    // follows `open` rather than the mount.
    use_effect(move || {
        if open() {
            let filter = Box::new(filter_builder.peek().clone());
            driver.dispatch(Action::Open {
                filter,
                config: Box::new(SheetConfig {
                    show_format: show_format_filter,
                    validate: validate_before_apply,
                    ..SheetConfig::default()
                }),
                notice: ResetNotice::Toast,
            });
        } else {
            driver.dispatch(Action::Back);
        }
    });

    // A failed catalog prefetch leaves every picker empty, which reads as "no
    // matches" rather than "didn't load". The model says so once per opening.
    // No retry button: each picker's own effect calls its `ensure_*` on mount,
    // and that refetches a failed cell.
    use_effect(move || {
        if open() && catalogs.any_public_failed() {
            driver.dispatch(Action::ListsFailed);
        }
    });

    // Active indicators and Apply read the live draft, so they match the
    // sections on the render that changed it.
    let (active, unchanged) = {
        let model = driver.model.read();
        let draft = filter_builder.read();
        let active: Vec<Section> = if show_active_indicators {
            Section::ALL
                .into_iter()
                .filter(|section| section.is_active(&draft))
                .collect()
        } else {
            Vec::new()
        };
        (
            active,
            model.applied().is_some_and(|applied| *applied == *draft),
        )
    };

    // Track accordion item index: shifts when format filter is included
    let mut idx = 0usize;
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "counts this sheet's accordion items, 15 at most"
    )]
    let mut next_idx = move || {
        idx += 1;
        idx
    };

    rsx! {
        // Closing without Apply (the ×, the backdrop, a drag down or the OS
        // back gesture) puts the applied filter back and says so.
        BottomSheet {
            open,
            title: card_filter::TITLE.to_string(),
            hint: hint_open,
            closable: true,

                // Mounted only while the sheet is open, so every section starts
                // collapsed on reopen. The modal stays in the DOM (hidden via
                // CSS) and Dioxus won't remount on a key change alone, so this
                // conditional render is what actually resets the accordion.
                if open() {
                Accordion {
                    id: "filter-accordion",
                    allow_multiple_open: false,
                    collapsible: true,

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(1)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            {Section::Name.title()}
                            if active.contains(&Section::Name) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::Name));
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
                            {Section::OracleText.title()}
                            if active.contains(&Section::OracleText) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::OracleText));
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
                            {Section::Types.title()}
                            if active.contains(&Section::Types) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::Types));
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
                            {Section::Mana.title()}
                            if active.contains(&Section::Mana) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::Mana));
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
                            {Section::Combat.title()}
                            if active.contains(&Section::Combat) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::Combat));
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
                            {Section::FlavorText.title()}
                            if active.contains(&Section::FlavorText) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::FlavorText));
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
                            {Section::Artist.title()}
                            if active.contains(&Section::Artist) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::Artist));
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
                            {Section::Rarity.title()}
                            if active.contains(&Section::Rarity) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::Rarity));
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
                            {Section::CardRoles.title()}
                            if active.contains(&Section::CardRoles) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::CardRoles));
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { CardRole {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(9)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            {Section::OracleTags.title()}
                            InfoButton { topic: HintTopic::OracleTags }
                            if active.contains(&Section::OracleTags) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::OracleTags));
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent {
                            OracleTags {
                                on_dictionary: move |field: PickerField| {
                                    driver.dispatch(Action::OpenDictionary(field));
                                },
                            }
                        }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(9)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                        },
                        AccordionTrigger {
                            {Section::Set.title()}
                            if active.contains(&Section::Set) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::Set));
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
                                {Section::Format.title()}
                                if active.contains(&Section::Format) {
                                    button {
                                        class: "clear-btn",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            driver.dispatch(Action::ClearSection(Section::Format));
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
                            {Section::Price.title()}
                            if active.contains(&Section::Price) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::Price));
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { PriceFilter {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |is_open| {
                            if is_open { let _ = document::eval(
                                &format!("setTimeout(() => {{ const el = document.querySelector('#filter-accordion .accordion-item:nth-child({})'); if (el) el.scrollIntoView({{ behavior: 'smooth', block: 'start' }}); }}, 50)",
                                if show_format_filter { 12 } else { 11 })
                            ); }
                        },
                        AccordionTrigger {
                            {Section::Sort.title()}
                            if active.contains(&Section::Sort) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::Sort));
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
                                if show_format_filter { 13 } else { 12 })
                            ); }
                        },
                        AccordionTrigger {
                            {Section::Config.title()}
                            if active.contains(&Section::Config) {
                                button {
                                    class: "clear-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        driver.dispatch(Action::ClearSection(Section::Config));
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }
                        AccordionContent { Config {} }
                    }
                }
                }

            // Reset and Apply float at the sheet's bottom right.
            div { class: "action-dock action-dock-right sheet-dock",
                DockAction {
                    label: card_filter::RESET,
                    icon: icons::ERASER,
                    onclick: move |()| {
                        // Stages the default; Apply commits it, closing undoes it.
                        if let Some(handler) = &on_clear {
                            handler.call(());
                        } else {
                            driver.dispatch(Action::Reset);
                        }
                    },
                }
                DockAction {
                    label: card_filter::APPLY,
                    icon: icons::CHECK,
                    disabled: unchanged,
                    onclick: move |()| driver.dispatch(Action::Apply),
                }
            }
        }

        OneTimeHintDialog { open: hint_open, hint: OneTimeHint::Filter }

        // Rendered outside the bottom sheet (see `otag_dict_open` note) so its
        // full-screen fixed overlay isn't trapped by the sheet's transform.
        if otag_dict_open() {
            OracleTagDictionary {
                open: otag_dict_open,
                on_use: move |slug: String| driver.dispatch(Action::UseTag(slug)),
            }
        }
    }
}
