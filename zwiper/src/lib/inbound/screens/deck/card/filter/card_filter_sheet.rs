use components::icons::{self};
use components::{BottomSheet, DockAction, Toasts, use_toast};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::Format;
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
    /// The deck's format on Add's search, where "Is legal in" holding only it
    /// is the default rather than a filter.
    #[props(default)]
    deck_format: Option<Format>,
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
                    deck_format,
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
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::Name.title()}
                        }
                        SectionClear { section: Section::Name, shown: active.contains(&Section::Name), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { Name {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::OracleText.title()}
                        }
                        SectionClear { section: Section::OracleText, shown: active.contains(&Section::OracleText), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { OracleText {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::Types.title()}
                        }
                        SectionClear { section: Section::Types, shown: active.contains(&Section::Types), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { Types {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::Mana.title()}
                        }
                        SectionClear { section: Section::Mana, shown: active.contains(&Section::Mana), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { Mana {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::Combat.title()}
                        }
                        SectionClear { section: Section::Combat, shown: active.contains(&Section::Combat), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { Combat {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::FlavorText.title()}
                        }
                        SectionClear { section: Section::FlavorText, shown: active.contains(&Section::FlavorText), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { FlavorText {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::Artist.title()}
                        }
                        SectionClear { section: Section::Artist, shown: active.contains(&Section::Artist), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { Artist {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::Rarity.title()}
                        }
                        SectionClear { section: Section::Rarity, shown: active.contains(&Section::Rarity), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { Rarity {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::CardRoles.title()}
                        }
                        SectionClear { section: Section::CardRoles, shown: active.contains(&Section::CardRoles), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { CardRole {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::OracleTags.title()}
                            InfoButton { topic: HintTopic::FilterOracleTags }
                        }
                        SectionClear { section: Section::OracleTags, shown: active.contains(&Section::OracleTags), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent {
                            OracleTags {
                                on_dictionary: move |field: PickerField| {
                                    driver.dispatch(Action::OpenDictionary(field));
                                },
                            }
                        }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::Set.title()}
                        }
                        SectionClear { section: Section::Set, shown: active.contains(&Section::Set), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { Set {} }
                    }

                    if show_format_filter {
                        AccordionItem { index: next_idx(),
                            on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                            },
                            AccordionTrigger {
                                {Section::Format.title()}
                            }
                            SectionClear { section: Section::Format, shown: active.contains(&Section::Format), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                            AccordionContent { FormatFilter {} }
                        }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::Price.title()}
                        }
                        SectionClear { section: Section::Price, shown: active.contains(&Section::Price), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { PriceFilter {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::Sort.title()}
                        }
                        SectionClear { section: Section::Sort, shown: active.contains(&Section::Sort), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
                        AccordionContent { Sort {} }
                    }

                    AccordionItem { index: next_idx(),
                        on_change: move |_| {
                            let _ = document::eval(components::OPEN_SECTION_SCROLL_JS);
                        },
                        AccordionTrigger {
                            {Section::Config.title()}
                        }
                        SectionClear { section: Section::Config, shown: active.contains(&Section::Config), on_clear: move |section| driver.dispatch(Action::ClearSection(section)) }
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

/// A filter section's clear button, beside its title while the section
/// holds a value. A tap clears that section without toggling it open.
#[component]
fn SectionClear(section: Section, shown: bool, on_clear: EventHandler<Section>) -> Element {
    rsx! {
        if shown {
            button {
                class: "clear-btn section-clear",
                "aria-label": "Clear",
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_clear.call(section);
                },
                "\u{00d7}"
            }
        }
    }
}
