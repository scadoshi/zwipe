//! The deck fields the create and edit screens share: name, format, the command zone, power level, budget and tags, drawn from the screen model.

use components::{ClearWrap, TextInput};
use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{
    self, DeckOtherTag, MAX_DECK_ORACLE_TAGS, MAX_DECK_TAGS, PowerLevel,
};
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::deck::form::{Slot, Step};
use zwipe_ui::deck::pickers::{Picker, TagPicker};
use zwipe_ui::screens::deck_form::{self as model, Action, Results, State};
use zwipe_ui::text::sentence_case;

use crate::inbound::components::hint_dialog::OneTimeHintDialog;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::{self, InfoButton};

/// The form's fields in three panels, Profile, Budget and Tags, drawn from `model`. Every tap goes out through `on_action`; a field's Swipe chip asks the screen to open that slot's swipe picker through `on_swipe`. `name` is the name input's text, which the screen keeps in step with the model.
#[component]
pub(crate) fn DeckFields(
    model: ReadSignal<State>,
    name: Signal<String>,
    on_action: EventHandler<Action>,
    on_swipe: EventHandler<Slot>,
) -> Element {
    let state = model.read();
    let values = state.values();
    let name_error = state.name_error().map(str::to_owned);
    let format = values.format;
    let power_level = values.power_level;
    let land_target = values.land_target;
    let price_text = values.price_text.clone();
    let currency_picked = values.currency;
    let other_tags = values.other_tags.clone();
    let deck_tags = values.tags.clone();
    let oracle_tags = values.oracle_tags.clone();
    drop(state);

    rsx! {
        // Profile: name, format, command zone, power level.
        div { class: "form-panel",
            div { class: "settings-list-header",
                span { class: "settings-list-title", {model::PROFILE} }
            }
            div { class: "form-panel-fields",
                // Deck name
                TextInput {
                    label: model::DECK_NAME,
                    on_info: info_button::open_topic(HintTopic::DeckName),
                    value: name,
                    id: "deck_name",
                    placeholder: model::NOT_SET,
                    error: name_error.as_deref().map(sentence_case),
                    on_clear: |()| {},
                }

                // Format (open the full-screen picker to choose)
                div {
                    div { class: "label-row",
                        label { class: "label", {model::FORMAT} }
                        InfoButton { topic: HintTopic::Format }
                    }

                    ClearWrap {
                        filled: format.is_some(),
                        on_clear: move |()| on_action.call(Action::ClearFormat),
                        if let Some(fmt) = format {
                            div { class: "input input-tappable", onclick: move |_| on_action.call(Action::OpenPicker(Picker::Format)), "{fmt.display_name()}" }
                        } else {
                            div { class: "input input-placeholder input-tappable", onclick: move |_| on_action.call(Action::OpenPicker(Picker::Format)), {model::NOT_SET} }
                        }
                    }
                }

                for slot in Slot::ALL {
                    SlotField { key: "{slot:?}", slot, model, on_action, on_swipe }
                }

                // Power level (single-select WotC bracket; Not set = none)
                div { style: "margin-top: 1rem;",
                    div { class: "label-row",
                        label { class: "label", {model::POWER_LEVEL} }
                        InfoButton { topic: HintTopic::PowerLevel }
                        if power_level.is_some() {
                            button {
                                class: "clear-btn",
                                onclick: move |_| on_action.call(Action::ClearPowerLevel),
                                "\u{00d7}"
                            }
                        }
                    }
                    div { class: "chip-row", style: "flex-wrap: wrap; justify-content: center;",
                        for pl in PowerLevel::all().iter().copied() {
                            div {
                                key: "{pl}",
                                class: if power_level == Some(pl) { "chip selected" } else { "chip" },
                                onclick: move |_| on_action.call(Action::TogglePowerLevel(pl)),
                                "{pl.display_name()}"
                            }
                        }
                    }
                }
            }
        }

        // Budget: land target + price target.
        div { class: "form-panel",
            div { class: "settings-list-header",
                span { class: "settings-list-title", {model::BUDGET} }
            }
            div { class: "form-panel-fields",
                // Land target (Not set = use the format heuristic)
                div {
                    div { class: "label-row",
                        label { class: "label", {model::LAND_TARGET} }
                        InfoButton { topic: HintTopic::LandTarget }
                        if land_target.is_some() {
                            button {
                                class: "clear-btn",
                                onclick: move |_| on_action.call(Action::ClearLand),
                                "\u{00d7}"
                            }
                        }
                    }

                    div { class: "stepper",
                        button {
                            class: "stepper-btn",
                            r#type: "button",
                            onclick: move |_| on_action.call(Action::StepLand(Step::Down)),
                            "-"
                        }
                        span { class: "stepper-value", style: "width: auto; min-width: 4rem;",
                            if let Some(v) = land_target { "{v}" } else { {model::NOT_SET} }
                        }
                        button {
                            class: "stepper-btn",
                            r#type: "button",
                            onclick: move |_| on_action.call(Action::StepLand(Step::Up)),
                            "+"
                        }
                    }
                }

                // Price target (budget; empty = no budget)
                div { style: "margin-top: 1rem;",
                    div { class: "label-row",
                        label { class: "label", {model::PRICE_TARGET} }
                        InfoButton { topic: HintTopic::PriceTarget }
                        // The currencies on the label's line.
                        div { class: "label-row-chips",
                            for currency in PriceCurrency::all().iter().copied() {
                                div {
                                    class: if currency_picked == currency { "chip selected" } else { "chip" },
                                    onclick: move |_| on_action.call(Action::PickCurrency(currency)),
                                    "{currency.label()}"
                                }
                            }
                        }
                    }

                    ClearWrap {
                        filled: !price_text.is_empty(),
                        on_clear: move |()| on_action.call(Action::ClearPrice),
                        input { class: "input",
                            id: "price_target",
                            r#type: "text",
                            inputmode: "decimal",
                            placeholder: model::NOT_SET,
                            value: "{price_text}",
                            autocapitalize: "none",
                            autocorrect: "off",
                            spellcheck: "false",
                            oninput: move |event| on_action.call(Action::TypePrice(event.value())),
                        }
                    }
                }
            }
        }

        // Tags: deck tags, oracle tags, other tags, last to mirror the deck
        // view's Tags section.
        div { class: "form-panel",
            div { class: "settings-list-header",
                span { class: "settings-list-title", {model::TAGS} }
            }
            div { class: "form-panel-fields",
                // Deck tags (open the full-screen picker to choose)
                div {
                    div { class: "label-row",
                        label { class: "label", {model::DECK_TAGS} }
                        InfoButton { topic: HintTopic::DeckTags }
                        span { class: "field-count", "{deck_tags.len()}/{MAX_DECK_TAGS}" }
                        if !deck_tags.is_empty() {
                            button {
                                class: "clear-btn",
                                onclick: move |_| on_action.call(Action::ClearTags(TagPicker::DeckTags)),
                                "\u{00d7}"
                            }
                        }
                    }

                    if deck_tags.is_empty() {
                        div { class: "input input-placeholder input-tappable", onclick: move |_| on_action.call(Action::OpenPicker(Picker::DeckTags)), {model::NOT_SET} }
                    } else {
                        div { class: "chip-box input-tappable", onclick: move |_| on_action.call(Action::OpenPicker(Picker::DeckTags)),
                            for tag in deck_tags {
                                div {
                                    key: "{tag}",
                                    class: "chip selected flex items-center gap-05",
                                    "{deck::deck_tag_label(&tag)}"
                                    // Remove just this tag; stop propagation so the
                                    // chip-box's own onclick doesn't also open the picker.
                                    button {
                                        class: "chip-remove",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            on_action.call(Action::RemoveTag {
                                                picker: TagPicker::DeckTags,
                                                slug: tag.clone(),
                                            });
                                        },
                                        "\u{00d7}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Oracle tags (granular strategy; deck tags seed these)
                div { style: "margin-top: 1rem;",
                    div { class: "label-row",
                        label { class: "label", {model::ORACLE_TAGS} }
                        InfoButton { topic: HintTopic::OracleTags }
                        span { class: "field-count", "{oracle_tags.len()}/{MAX_DECK_ORACLE_TAGS}" }
                        if !oracle_tags.is_empty() {
                            button {
                                class: "clear-btn",
                                onclick: move |_| on_action.call(Action::ClearTags(TagPicker::OracleTags)),
                                "\u{00d7}"
                            }
                        }
                    }

                    if oracle_tags.is_empty() {
                        div {
                            class: "input input-placeholder input-tappable",
                            onclick: move |_| on_action.call(Action::OpenPicker(Picker::OracleTags)),
                            {model::NOT_SET}
                        }
                    } else {
                        div { class: "chip-box input-tappable", onclick: move |_| on_action.call(Action::OpenPicker(Picker::OracleTags)),
                            for slug in oracle_tags {
                                div {
                                    key: "{slug}",
                                    class: "chip selected flex items-center gap-05",
                                    "{slug}"
                                    button {
                                        class: "chip-remove",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            on_action.call(Action::RemoveTag {
                                                picker: TagPicker::OracleTags,
                                                slug: slug.clone(),
                                            });
                                        },
                                        "\u{00d7}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Other tags (non-gameplay labels; multi-select)
                div { style: "margin-top: 1rem;",
                    div { class: "label-row",
                        label { class: "label", {model::OTHER_TAGS} }
                        InfoButton { topic: HintTopic::OtherTags }
                        if !other_tags.is_empty() {
                            button {
                                class: "clear-btn",
                                onclick: move |_| on_action.call(Action::ClearOtherTags),
                                "\u{00d7}"
                            }
                        }
                    }
                    div { class: "chip-row", style: "flex-wrap: wrap; justify-content: center;",
                        for tag in DeckOtherTag::all().iter().copied() {
                            div {
                                key: "{tag}",
                                class: if other_tags.contains(&tag) { "chip selected" } else { "chip" },
                                onclick: move |_| on_action.call(Action::ToggleOtherTag(tag)),
                                "{tag.display_name()}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// One command-zone field: its label row with Filter, clear and Swipe, the results floating over it, and its input. Always rendered, eased open and shut by [`Collapsible`].
#[component]
fn SlotField(
    slot: Slot,
    model: ReadSignal<State>,
    on_action: EventHandler<Action>,
    on_swipe: EventHandler<Slot>,
) -> Element {
    let state = model.read();
    let show = state.shows(slot);
    let label = state.label(slot);
    let filter_on = state.filter_on(slot);
    let has_card = state.values().slot(slot).is_some();
    let results = state.results(slot);
    let display = state.display(slot).to_owned();
    let has_query = state.has_query(slot);
    let topic = state.hint(slot);
    drop(state);
    let id = match slot {
        Slot::Commander => "commander",
        Slot::Partner => "partner_commander",
        Slot::Background => "background",
        Slot::SignatureSpell => "signature_spell",
    };

    rsx! {
        Collapsible { show,
            div {
                div { class: "label-row",
                    label { class: "label", {label} }
                    InfoButton { topic }
                    div {
                        class: if filter_on { "chip-xs selected" } else { "chip-xs" },
                        onclick: move |_| on_action.call(Action::ToggleFilter(slot)),
                        {model::FILTER}
                    }
                    div {
                        class: "chip-xs chip-primary",
                        onclick: move |_| on_swipe.call(slot),
                        {model::SWIPE}
                    }
                }

                if let Some(results) = results {
                    div { class: "search-float-anchor",
                        match results {
                            Results::Searching => rsx! {
                                div { class: "search-float-results",
                                    div { class: "chip-unselected search-float-chip", {model::SEARCHING} }
                                }
                            },
                            Results::Empty => rsx! {
                                div { class: "search-float-results",
                                    div { class: "chip-unselected search-float-chip", {model::NO_RESULTS} }
                                }
                            },
                            Results::Found(cards) => rsx! {
                                div { class: "search-float-results",
                                    for (i, card) in cards.into_iter().enumerate() {
                                        {
                                            let name = card.scryfall_data.name.clone();
                                            rsx! {
                                                div {
                                                    class: "chip-unselected search-float-chip",
                                                    style: "animation-delay: {i * 40}ms, {250 + i * 40}ms;",
                                                    onclick: move |_| on_action.call(Action::PickResult {
                                                        slot,
                                                        card: Box::new(card.clone()),
                                                    }),
                                                    { name }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                        }
                    }
                }

                // The × in the box clears what it shows: the search typed
                // into it, or the card picked.
                ClearWrap {
                    filled: has_query || has_card,
                    on_clear: move |()| {
                        on_action.call(if has_query {
                            Action::ClearSearch(slot)
                        } else {
                            Action::ClearSlot(slot)
                        });
                    },
                    input { class: "input",
                        id,
                        r#type: "text",
                        placeholder: model::NOT_SET,
                        value: "{display}",
                        autocapitalize: "none",
                        autocorrect: "off",
                        spellcheck: "false",
                        onclick: move |_| on_action.call(Action::TapInput(slot)),
                        oninput: move |event| on_action.call(Action::Type { slot, text: event.value() }),
                    }
                }
            }
        }
    }
}

/// Explainer dialog for the deck create/edit form, Create's or Edit's
/// one-time `hint`, whose copy the two share. `open` is owned by the screen
/// (a one-time hint plus a "?" button).
#[component]
pub(crate) fn DeckFieldsHint(open: Signal<bool>, hint: OneTimeHint) -> Element {
    rsx! {
        OneTimeHintDialog { open, hint }
    }
}

/// Eases a command-zone field's height + opacity open/closed instead of popping
/// it in and out. Always rendered (the field's search/clear effects live at the
/// `DeckFields` top level), so this only animates appearance, never logic.
#[component]
fn Collapsible(show: bool, children: Element) -> Element {
    let class = if show {
        "collapsible open"
    } else {
        "collapsible"
    };
    rsx! {
        div { class: "{class}",
            div { class: "collapsible-inner", {children} }
        }
    }
}
