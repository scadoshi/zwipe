//! The form's fields in the app's order: `Profile` (name, format, the
//! command zone, power level), `Budget` (land and price targets) and `Tags`
//! (deck, oracle and other tags). A wide window puts `Profile` and `Budget`
//! in the left column and `Tags` in the right.

use components::{Chip, ClearWrap, TextInput};
use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::commander_eligibility;
use zwipe_core::domain::deck::{
    DeckOtherTag, MAX_DECK_ORACLE_TAGS, MAX_DECK_TAGS, PowerLevel, deck_name, deck_tag_label,
};
use zwipe_ui::deck::form::{self, Slot, Step};
use zwipe_ui::text::sentence_case;

use super::DeckForm;
use super::command_zone::CommandZoneField;
use crate::inbound::chrome::enter_submit::EnterSubmit;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton, open_topic};

/// The deck name input's id.
pub const DECK_NAME_ID: &str = "deck_name";

/// Every field, in one form: `on_submit` runs when Enter is pressed in any text field.
#[component]
pub fn DeckFields(form: DeckForm, on_submit: EventHandler<()>) -> Element {
    rsx! {
        form {
            class: "container-sm web-deck-form",
            novalidate: true,
            onsubmit: move |event| {
                event.prevent_default();
                on_submit.call(());
            },
            EnterSubmit {}
            div { class: "flex-col text-center web-deck-form-column",
                ProfileFields { form, on_submit }
                BudgetFields { form }
            }
            div { class: "flex-col text-center web-deck-form-column",
                TagFields { form, on_submit }
            }
        }
    }
}

/// Eases a field open and shut instead of popping it; the field stays
/// mounted so its search keeps running. A shut field is inert, so Tab and
/// screen readers pass over it.
#[component]
fn Collapsible(show: bool, children: Element) -> Element {
    rsx! {
        div { class: if show { "collapsible open" } else { "collapsible" },
            inert: (!show).then_some(true),
            div { class: "collapsible-inner", {children} }
        }
    }
}

/// What a key does on a field that opens a picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickerKey {
    /// Submits the form, as Enter does in every field of the deck form.
    Submit,
    /// Opens the picker.
    Open,
}

/// Enter submits the form and Space opens the picker; every other key does nothing here.
fn picker_key(key: &Key) -> Option<PickerKey> {
    if *key == Key::Enter {
        Some(PickerKey::Submit)
    } else if matches!(key, Key::Character(c) if c == " ") {
        Some(PickerKey::Open)
    } else {
        None
    }
}

/// A field that opens a picker: a click or Space opens it, and Enter submits the form. In the
/// tab order as a `role="button"`, ringed while focused.
#[component]
fn PickerField(
    #[props(into)] class: String,
    on_open: EventHandler<()>,
    on_submit: EventHandler<()>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class,
            role: "button",
            tabindex: "0",
            onclick: move |_| on_open.call(()),
            onkeydown: move |event: KeyboardEvent| {
                let Some(action) = picker_key(&event.key()) else {
                    return;
                };
                event.prevent_default();
                match action {
                    PickerKey::Submit => on_submit.call(()),
                    PickerKey::Open => on_open.call(()),
                }
            },
            {children}
        }
    }
}

/// Name, format, the command zone and power level.
#[component]
fn ProfileFields(form: DeckForm, on_submit: EventHandler<()>) -> Element {
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
                    TextInput {
                        label: "Deck name",
                        on_info: open_topic(HintTopic::DeckName),
                        value: form.name,
                        id: DECK_NAME_ID,
                        maxlength: deck_name::MAX_LENGTH,
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
                            PickerField { class: "input input-tappable", on_open: move |()| format_open.set(true), on_submit, "{format.display_name()}" }
                        } else {
                            PickerField { class: "input input-placeholder input-tappable", on_open: move |()| format_open.set(true), on_submit, "Not set" }
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
                div {
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
                            Chip {
                                key: "{level}",
                                selected: power_level() == Some(level),
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
                div { style: "margin-top: var(--space-4);",
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
                div { style: "margin-top: var(--space-4);",
                    div { class: "label-row",
                        label { class: "label", r#for: "price_target", "Price target" }
                        InfoButton { topic: HintTopic::PriceTarget }
                        // The currencies on the label's line.
                        div { class: "label-row-chips",
                            for each in PriceCurrency::all().iter().copied() {
                                Chip {
                                    key: "{each.label()}",
                                    selected: currency() == each,
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
fn TagFields(form: DeckForm, on_submit: EventHandler<()>) -> Element {
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
                div { style: "margin-top: var(--space-4);",
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
                        PickerField { class: "input input-placeholder input-tappable", on_open: move |()| tags_open.set(true), on_submit, "Not set" }
                    } else {
                        PickerField { class: "chip-box input-tappable", on_open: move |()| tags_open.set(true), on_submit,
                            for tag in tags() {
                                div { key: "{tag}", class: "chip selected flex items-center gap-05",
                                    "{deck_tag_label(&tag)}"
                                    button {
                                        class: "chip-remove",
                                        // Its keys stay with it: Enter removes the chip without opening the picker.
                                        onkeydown: move |evt| evt.stop_propagation(),
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
                div { style: "margin-top: var(--space-4);",
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
                        PickerField { class: "input input-placeholder input-tappable", on_open: move |()| oracle_open.set(true), on_submit, "Not set" }
                    } else {
                        PickerField { class: "chip-box input-tappable", on_open: move |()| oracle_open.set(true), on_submit,
                            for slug in oracle_tags() {
                                div { key: "{slug}", class: "chip selected flex items-center gap-05",
                                    "{slug}"
                                    button {
                                        class: "chip-remove",
                                        // Its keys stay with it: Enter removes the chip without opening the picker.
                                        onkeydown: move |evt| evt.stop_propagation(),
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
                div { style: "margin-top: var(--space-4);",
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
                            Chip {
                                key: "{tag}",
                                selected: other_tags.read().contains(&tag),
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

#[cfg(test)]
mod tests {
    use reqwest::Url;
    use zwipe_client::ZwipeClient;
    use zwipe_core::domain::auth::models::platform::ClientPlatform;
    use zwipe_core::domain::auth::models::session::Session;
    use zwipe_ui::telemetry::SharedUsageBuffer;

    use super::*;
    use crate::config::APP_VERSION;
    use crate::inbound::screens::deck::form::use_deck_form;

    /// The contexts the fields read: no session, a client, telemetry and the hint sheet.
    #[component]
    fn Host() -> Element {
        use_context_provider(|| Signal::new(None::<Session>));
        use_context_provider(|| {
            Signal::new(ZwipeClient::new(Url::parse("http://127.0.0.1:9").unwrap()))
        });
        use_context_provider(|| SharedUsageBuffer::new(ClientPlatform::Web, APP_VERSION));
        use_context_provider(|| Signal::new(None::<HintTopic>));
        rsx! {
            components::ToastStack { default_duration: components::TOAST_NORMAL, Fields {} }
        }
    }

    #[component]
    fn Fields() -> Element {
        let form = use_deck_form();
        rsx! {
            DeckFields { form, on_submit: |()| {} }
        }
    }

    #[test]
    fn enter_on_a_picker_field_submits_and_space_opens_it() {
        assert_eq!(picker_key(&Key::Enter), Some(PickerKey::Submit));
        assert_eq!(
            picker_key(&Key::Character(" ".into())),
            Some(PickerKey::Open)
        );
        assert_eq!(picker_key(&Key::Character("a".into())), None);
        assert_eq!(picker_key(&Key::Tab), None);
    }

    /// Enter in any of the form's fields submits it: the fields sit in a real form whose first
    /// button, its default, is a submit button, and every other button in it is a plain button,
    /// so Enter on one runs that button and never sends the form.
    #[test]
    fn enter_in_any_field_submits_the_deck_form() {
        let mut dom = VirtualDom::new(Host);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let form = html
            .split_once("<form")
            .and_then(|(_, rest)| rest.split_once("</form>"))
            .map_or_else(
                || panic!("the fields sit in a form: {html}"),
                |(body, _)| body,
            );
        for field in [
            "id=\"deck_name\"",
            "id=\"price_target\"",
            "id=\"commander\"",
        ] {
            assert!(form.contains(field), "{field} in the form: {form}");
        }
        let mut buttons = form.split("<button").skip(1);
        let first = buttons.next().unwrap_or_default();
        assert!(
            first.contains("type=\"submit\"") && first.contains("hidden"),
            "the default button is the hidden submit: {first}"
        );
        for button in buttons {
            assert!(
                button.contains("type=\"button\""),
                "a plain button: {button}"
            );
        }
    }
}
