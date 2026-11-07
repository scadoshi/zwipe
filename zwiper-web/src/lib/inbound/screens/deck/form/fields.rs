//! The form's fields in the app's order: `Profile` (name, format, the
//! command zone, power level), `Budget` (land and price targets) and `Tags`
//! (deck, oracle and other tags). A wide window puts `Profile` and `Budget`
//! in the left column and `Tags` in the right.

use components::{ClearWrap, TextInput};
use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::commander_eligibility;
use zwipe_core::domain::deck::{
    DeckOtherTag, MAX_DECK_ORACLE_TAGS, MAX_DECK_TAGS, PowerLevel, deck_tag_label,
};
use zwipe_ui::deck::form::{self, Slot, Step};
use zwipe_ui::text::sentence_case;

use super::DeckForm;
use super::command_zone::CommandZoneField;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton, open_topic};

/// The deck name input's id.
pub const DECK_NAME_ID: &str = "deck_name";

/// Every field. `on_enter` runs when Enter is pressed in the deck name.
#[component]
pub fn DeckFields(form: DeckForm, on_enter: EventHandler<()>) -> Element {
    rsx! {
        div { class: "container-sm web-deck-form",
            div { class: "flex-col text-center web-deck-form-column",
                ProfileFields { form, on_enter }
                BudgetFields { form }
            }
            div { class: "flex-col text-center web-deck-form-column",
                TagFields { form }
            }
        }
    }
}

/// Eases a field open and shut instead of popping it; the field stays
/// mounted so its search keeps running.
#[component]
fn Collapsible(show: bool, children: Element) -> Element {
    rsx! {
        div { class: if show { "collapsible open" } else { "collapsible" },
            div { class: "collapsible-inner", {children} }
        }
    }
}

/// Name, format, the command zone and power level.
#[component]
fn ProfileFields(form: DeckForm, on_enter: EventHandler<()>) -> Element {
    let format = (form.format)();
    let commander = form.commander.card.read().clone();
    let show_commander = format.is_some_and(|f| f.has_commander());
    let show_partner = show_commander
        && commander
            .as_ref()
            .is_some_and(|c| commander_eligibility::partner_kind(c).is_some());
    let show_background = show_commander
        && commander
            .as_ref()
            .is_some_and(commander_eligibility::has_choose_a_background);
    let show_spell = format.is_some_and(|f| f.has_signature_spell());
    let mut format_open = form.format_open;
    let mut power_level = form.power_level;
    rsx! {
        div { class: "web-form-panel",
            div { class: "settings-list-header web-form-head",
                span { class: "settings-list-title", "Profile" }
            }
            div { class: "web-form-fields",
                div {
                    onkeydown: move |event: Event<KeyboardData>| {
                        if event.key() == Key::Enter {
                            event.prevent_default();
                            on_enter.call(());
                        }
                    },
                    TextInput {
                        label: "Deck name",
                        on_info: open_topic(HintTopic::DeckName),
                        value: form.name,
                        id: DECK_NAME_ID,
                        placeholder: "Not set",
                        error: (form.name_error)().as_deref().map(sentence_case),
                        on_clear: |()| {},
                    }
                }
                div {
                    div { class: "label-row",
                        label { class: "label", "Format" }
                        InfoButton { topic: HintTopic::Format }
                    }
                    ClearWrap {
                        filled: format.is_some(),
                        on_clear: move |()| form.clear_format(),
                        if let Some(format) = format {
                            div { class: "input input-tappable", onclick: move |_| format_open.set(true), "{format.display_name()}" }
                        } else {
                            div { class: "input input-placeholder input-tappable", onclick: move |_| format_open.set(true), "Not set" }
                        }
                    }
                }
                Collapsible { show: show_commander,
                    CommandZoneField { form, slot: Slot::Commander, label: Slot::Commander.label(format) }
                }
                Collapsible { show: show_partner,
                    CommandZoneField { form, slot: Slot::Partner, label: Slot::Partner.label(format) }
                }
                Collapsible { show: show_background,
                    CommandZoneField { form, slot: Slot::Background, label: Slot::Background.label(format) }
                }
                Collapsible { show: show_spell,
                    CommandZoneField { form, slot: Slot::SignatureSpell, label: Slot::SignatureSpell.label(format) }
                }
                div { style: "margin-top: 1rem;",
                    div { class: "label-row",
                        label { class: "label", "Power level" }
                        InfoButton { topic: HintTopic::PowerLevel }
                        if power_level().is_some() {
                            button {
                                class: "clear-btn",
                                r#type: "button",
                                onclick: move |_| power_level.set(None),
                                "\u{00d7}"
                            }
                        }
                    }
                    div { class: "chip-row", style: "flex-wrap: wrap; justify-content: center;",
                        for level in PowerLevel::all().iter().copied() {
                            div {
                                key: "{level}",
                                class: if power_level() == Some(level) { "chip selected" } else { "chip" },
                                onclick: move |_| {
                                    if power_level() == Some(level) {
                                        power_level.set(None);
                                    } else {
                                        power_level.set(Some(level));
                                    }
                                },
                                "{level.display_name()}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The land target stepper and the price target.
#[component]
fn BudgetFields(form: DeckForm) -> Element {
    let mut land_target = form.land_target;
    let mut price = form.price_text;
    let mut currency = form.currency;
    let mut step = move |direction: Step| {
        let next = form::step_land_target(land_target(), *form.format.peek(), direction);
        land_target.set(Some(next));
    };
    rsx! {
        div { class: "web-form-panel",
            div { class: "settings-list-header web-form-head",
                span { class: "settings-list-title", "Budget" }
            }
            div { class: "web-form-fields",
                div { style: "margin-top: 1rem;",
                    div { class: "label-row",
                        label { class: "label", "Land target" }
                        InfoButton { topic: HintTopic::LandTarget }
                        if land_target().is_some() {
                            button {
                                class: "clear-btn",
                                r#type: "button",
                                onclick: move |_| land_target.set(None),
                                "\u{00d7}"
                            }
                        }
                    }
                    div { class: "stepper",
                        button {
                            class: "stepper-btn",
                            r#type: "button",
                            onclick: move |_| step(Step::Down),
                            "-"
                        }
                        span { class: "stepper-value", style: "width: auto; min-width: 4rem;",
                            if let Some(v) = land_target() {
                                "{v}"
                            } else {
                                "Not set"
                            }
                        }
                        button {
                            class: "stepper-btn",
                            r#type: "button",
                            onclick: move |_| step(Step::Up),
                            "+"
                        }
                    }
                }
                div { style: "margin-top: 1rem;",
                    div { class: "label-row",
                        label { class: "label", r#for: "price_target", "Price target" }
                        InfoButton { topic: HintTopic::PriceTarget }
                        // The currencies on the label's line.
                        div { class: "label-row-chips",
                            for each in PriceCurrency::all().iter().copied() {
                                div {
                                    key: "{each.label()}",
                                    class: if currency() == each { "chip selected" } else { "chip" },
                                    onclick: move |_| currency.set(each),
                                    "{each.label()}"
                                }
                            }
                        }
                    }
                    ClearWrap {
                        filled: !price().is_empty(),
                        on_clear: move |()| price.set(String::new()),
                        input {
                            class: "input",
                            id: "price_target",
                            r#type: "text",
                            inputmode: "decimal",
                            placeholder: "Not set",
                            value: "{price}",
                            autocapitalize: "none",
                            autocorrect: "off",
                            spellcheck: "false",
                            oninput: move |event| {
                                let kept: String = event.value().chars().filter(|c| form::price_char(*c)).collect();
                                price.set(kept);
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Deck tags, oracle tags and other tags.
#[component]
fn TagFields(form: DeckForm) -> Element {
    let mut tags = form.tags;
    let mut oracle_tags = form.oracle_tags;
    let mut other_tags = form.other_tags;
    let mut tags_open = form.tags_open;
    let mut oracle_open = form.oracle_tags_open;
    rsx! {
        div { class: "web-form-panel",
            div { class: "settings-list-header web-form-head",
                span { class: "settings-list-title", "Tags" }
            }
            div { class: "web-form-fields",
                div { style: "margin-top: 1rem;",
                    div { class: "label-row",
                        label { class: "label", "Deck tags" }
                        InfoButton { topic: HintTopic::DeckTags }
                        span { class: "field-count", "{tags.read().len()}/{MAX_DECK_TAGS}" }
                        if !tags.read().is_empty() {
                            button {
                                class: "clear-btn",
                                r#type: "button",
                                onclick: move |_| tags.set(Vec::new()),
                                "\u{00d7}"
                            }
                        }
                    }
                    if tags.read().is_empty() {
                        div { class: "input input-placeholder input-tappable", onclick: move |_| tags_open.set(true), "Not set" }
                    } else {
                        div { class: "chip-box input-tappable", onclick: move |_| tags_open.set(true),
                            for tag in tags() {
                                div { key: "{tag}", class: "chip selected flex items-center gap-05",
                                    "{deck_tag_label(&tag)}"
                                    button {
                                        class: "chip-remove",
                                        r#type: "button",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            tags.write().retain(|t| *t != tag);
                                        },
                                        "\u{00d7}"
                                    }
                                }
                            }
                        }
                    }
                }
                div { style: "margin-top: 1rem;",
                    div { class: "label-row",
                        label { class: "label", "Oracle tags" }
                        InfoButton { topic: HintTopic::OracleTags }
                        span { class: "field-count", "{oracle_tags.read().len()}/{MAX_DECK_ORACLE_TAGS}" }
                        if !oracle_tags.read().is_empty() {
                            button {
                                class: "clear-btn",
                                r#type: "button",
                                onclick: move |_| oracle_tags.set(Vec::new()),
                                "\u{00d7}"
                            }
                        }
                    }
                    if oracle_tags.read().is_empty() {
                        div { class: "input input-placeholder input-tappable", onclick: move |_| oracle_open.set(true), "Not set" }
                    } else {
                        div { class: "chip-box input-tappable", onclick: move |_| oracle_open.set(true),
                            for slug in oracle_tags() {
                                div { key: "{slug}", class: "chip selected flex items-center gap-05",
                                    "{slug}"
                                    button {
                                        class: "chip-remove",
                                        r#type: "button",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            oracle_tags.write().retain(|s| *s != slug);
                                        },
                                        "\u{00d7}"
                                    }
                                }
                            }
                        }
                    }
                }
                div { style: "margin-top: 1rem;",
                    div { class: "label-row",
                        label { class: "label", "Other tags" }
                        InfoButton { topic: HintTopic::OtherTags }
                        if !other_tags.read().is_empty() {
                            button {
                                class: "clear-btn",
                                r#type: "button",
                                onclick: move |_| other_tags.set(Vec::new()),
                                "\u{00d7}"
                            }
                        }
                    }
                    div { class: "chip-row", style: "flex-wrap: wrap; justify-content: center;",
                        for tag in DeckOtherTag::all().iter().copied() {
                            div {
                                key: "{tag}",
                                class: if other_tags.read().contains(&tag) { "chip selected" } else { "chip" },
                                onclick: move |_| {
                                    let mut picked = other_tags.write();
                                    if let Some(at) = picked.iter().position(|t| *t == tag) {
                                        picked.remove(at);
                                    } else {
                                        picked.push(tag);
                                    }
                                },
                                "{tag.display_name()}"
                            }
                        }
                    }
                }
            }
        }
    }
}
