//! One open filter section's rows, drawn from zwipe-ui's row model with the
//! app's classes: labels with their mode chip and `×`, text and number
//! boxes, chip rows, steppers and the searchable pickers.

use components::Chip;
use dioxus::prelude::*;
use dioxus_primitives::toast::use_toast;
use zwipe_ui::filter::draft::FilterDraft;
use zwipe_ui::filter::fields::{End, Stat};
use zwipe_ui::filter::picker::{PickerField, Sources};
use zwipe_ui::filter::sections::{
    self, ChipSet, Clear, ModeChip, NumberField, Row, Section, StepRow,
};

use crate::domain::filter_chips;
use crate::inbound::catalog::CatalogCache;
use crate::inbound::chrome::toast::raise;

/// Moves `epoch` on, remounting the boxes that hold typed text.
pub(super) fn bump(mut epoch: Signal<u32>) {
    let next = epoch.peek().wrapping_add(1);
    epoch.set(next);
}

/// The rows of `section` as the draft stands. `epoch` changes whenever the
/// draft is replaced from outside the boxes (Apply, Back, Reset, a clear),
/// which remounts the boxes so their typed text follows.
#[component]
pub(super) fn SectionRows(
    section: Section,
    draft: Signal<FilterDraft>,
    epoch: Signal<u32>,
    sources: Sources,
    on_dictionary: EventHandler<PickerField>,
) -> Element {
    let rows = {
        let d = draft.read();
        sections::rows(section, &d.draft, &d.modes)
    };
    let generation = epoch();
    let mut out: Vec<Element> = Vec::new();
    let mut i = 0usize;
    while let Some(row) = rows.get(i).copied() {
        let next = rows.get(i.saturating_add(1)).copied();
        let paired = match (row, next) {
            (Row::Number(NumberField::ManaMin), Some(Row::Number(NumberField::ManaMax))) => {
                Some((NumberField::ManaMin, NumberField::ManaMax))
            }
            (Row::Number(NumberField::PriceMin), Some(Row::Number(NumberField::PriceMax))) => {
                Some((NumberField::PriceMin, NumberField::PriceMax))
            }
            _ => None,
        };
        if let Some((min, max)) = paired {
            out.push(rsx! {
                NumberPair { key: "{generation}-{min:?}", draft, min, max }
            });
            i = i.saturating_add(2);
            continue;
        }
        out.push(rsx! {
            RowView {
                key: "{generation}-{i}",
                row,
                draft,
                epoch,
                sources: sources.clone(),
                on_dictionary,
            }
        });
        i = i.saturating_add(1);
    }
    rsx! {
        div { class: "flex-col gap-half",
            for row in out {
                {row}
            }
        }
    }
}

/// One row.
#[component]
fn RowView(
    row: Row,
    draft: Signal<FilterDraft>,
    epoch: Signal<u32>,
    sources: Sources,
    on_dictionary: EventHandler<PickerField>,
) -> Element {
    match row {
        Row::Label {
            text, mode, clear, ..
        } => rsx! {
            LabelRow { text, mode, clear, row, draft, epoch, on_dictionary }
        },
        Row::Text(field) => {
            let value = draft.read().draft.clone();
            let value = field.get(&value).to_owned();
            rsx! {
                input {
                    class: "input input-compact",
                    placeholder: field.placeholder(),
                    value,
                    r#type: "text",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    oninput: move |e| field.set(&mut draft.write().draft, &e.value()),
                }
            }
        }
        Row::Number(field) => rsx! {
            NumberBox { draft, field }
        },
        Row::Chips(set) => rsx! {
            ChipRow { draft, set }
        },
        Row::Stepper(step) => rsx! {
            StepperRow { draft, step }
        },
        Row::Picker(field) => rsx! {
            Picker { draft, field, sources }
        },
    }
}

/// A label with its mode chip, the Oracle tags `Dictionary` chip and its
/// `×`.
#[component]
fn LabelRow(
    text: &'static str,
    mode: Option<ModeChip>,
    clear: Option<Clear>,
    row: Row,
    mut draft: Signal<FilterDraft>,
    mut epoch: Signal<u32>,
    on_dictionary: EventHandler<PickerField>,
) -> Element {
    let dictionary = sections::dictionary_for(row);
    let mode_label = mode.map(|chip| {
        let d = draft.read();
        d.modes.label(&d.draft, chip)
    });
    rsx! {
        div { class: "label-row mt-2",
            label { class: "label-xs", "{text}" }
            if let Some(field) = dictionary {
                button {
                    class: "chip-xs",
                    onclick: move |_| on_dictionary.call(field),
                    "Dictionary"
                }
            }
            if let (Some(chip), Some(label)) = (mode, mode_label) {
                button {
                    class: "chip-xs",
                    onclick: move |_| {
                        let d = &mut *draft.write();
                        d.modes.flip(&mut d.draft, chip);
                        if matches!(chip, ModeChip::Range(_)) {
                            bump(epoch);
                        }
                    },
                    "{label}"
                }
            }
            if let Some(clear) = clear {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        clear.apply(&mut draft.write().draft);
                        bump(epoch);
                    },
                    "\u{00d7}"
                }
            }
        }
    }
}

/// A row of chips.
#[component]
fn ChipRow(mut draft: Signal<FilterDraft>, set: ChipSet) -> Element {
    let cache: CatalogCache = use_context();
    let roles = cache
        .0
        .read()
        .card_roles
        .ready()
        .cloned()
        .unwrap_or_default();
    let (options, selected): (Vec<filter_chips::ChipOption>, Vec<bool>) = {
        let d = draft.read();
        let options = filter_chips::options(set, &roles, &d.draft);
        let selected = options
            .iter()
            .map(|o| filter_chips::is_selected(set, &d.draft, &o.value))
            .collect();
        (options, selected)
    };
    let class = if matches!(set, ChipSet::Currency) {
        "chip-row web-filter-chips-center"
    } else {
        "flex flex-wrap gap-1 mb-1 flex-center"
    };
    rsx! {
        div { class,
            for (option, on) in options.into_iter().zip(selected) {
                {
                    let value = option.value.clone();
                    rsx! {
                        span { key: "{option.value}", title: option.title.unwrap_or_default(), class: "web-filter-chip",
                            Chip {
                                selected: on,
                                onclick: move |_| {
                                    let d = &mut *draft.write();
                                    filter_chips::tap(set, &mut d.draft, &d.modes, &value);
                                },
                                if let Some(symbol) = option.symbol.as_ref() {
                                    i { class: "ms ms-{symbol} ms-cost" }
                                } else {
                                    "{option.label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A power or toughness stepper, or a range's two on one line.
#[component]
fn StepperRow(mut draft: Signal<FilterDraft>, step: StepRow) -> Element {
    let b = draft.read().draft.clone();
    match step {
        StepRow::Exact(stat) => {
            let shown = stat
                .exact(&b)
                .map_or_else(|| "-".to_owned(), |v| v.to_string());
            rsx! {
                div { class: "stepper",
                    button { class: "stepper-btn", onclick: move |_| stat.step_exact(&mut draft.write().draft, -1), "-" }
                    span { class: "stepper-value", "{shown}" }
                    button { class: "stepper-btn", onclick: move |_| stat.step_exact(&mut draft.write().draft, 1), "+" }
                }
            }
        }
        StepRow::Range(stat) => {
            let range = stat.range(&b);
            let end_text = |end: End| {
                range.map_or_else(
                    || "-".to_owned(),
                    |(min, max)| match end {
                        End::Min => min.to_string(),
                        End::Max => max.to_string(),
                    },
                )
            };
            rsx! {
                div { class: "flex-row gap-2 flex-center",
                    RangeStepper { draft, stat, end: End::Min, shown: end_text(End::Min) }
                    span { class: "text-muted", "to" }
                    RangeStepper { draft, stat, end: End::Max, shown: end_text(End::Max) }
                }
            }
        }
    }
}

/// One end of a range stepper.
#[component]
fn RangeStepper(mut draft: Signal<FilterDraft>, stat: Stat, end: End, shown: String) -> Element {
    rsx! {
        div { class: "stepper",
            button { class: "stepper-btn", onclick: move |_| stat.step_range(&mut draft.write().draft, end, -1), "-" }
            span { class: "stepper-value", "{shown}" }
            button { class: "stepper-btn", onclick: move |_| stat.step_range(&mut draft.write().draft, end, 1), "+" }
        }
    }
}

/// A number box parsed as it loses focus: the exact mana value.
#[component]
fn NumberBox(mut draft: Signal<FilterDraft>, field: NumberField) -> Element {
    let toast = use_toast();
    let mut text = use_signal(|| field.text(&draft.peek().draft));
    rsx! {
        input {
            class: "input input-compact input-narrow",
            placeholder: field.placeholder(),
            value: text(),
            r#type: "text",
            inputmode: "decimal",
            autocapitalize: "none",
            autocorrect: "off",
            spellcheck: "false",
            oninput: move |e| text.set(e.value()),
            onblur: move |_| {
                let typed = text.peek().clone();
                let result = draft.write().set_number(field, &typed, "");
                match result {
                    Ok(()) => text.set(field.text(&draft.peek().draft)),
                    Err(t) => raise(toast, &t),
                }
            },
        }
    }
}

/// A min box, `to` and a max box, each parsed as it loses focus.
#[component]
fn NumberPair(mut draft: Signal<FilterDraft>, min: NumberField, max: NumberField) -> Element {
    let toast = use_toast();
    let mut min_text = use_signal(|| min.text(&draft.peek().draft));
    let mut max_text = use_signal(|| max.text(&draft.peek().draft));
    let mut commit = move |field: NumberField| {
        let (typed, other) = if field == min {
            (min_text.peek().clone(), max_text.peek().clone())
        } else {
            (max_text.peek().clone(), min_text.peek().clone())
        };
        let result = draft.write().set_number(field, &typed, &other);
        if let Err(t) = result {
            raise(toast, &t);
        }
    };
    rsx! {
        div { class: "flex-row gap-1 flex-center mb-1",
            input {
                class: "input input-compact input-narrow",
                placeholder: min.placeholder(),
                value: min_text(),
                r#type: "text",
                inputmode: "decimal",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |e| min_text.set(e.value()),
                onblur: move |_| commit(min),
            }
            span { class: "text-muted", "to" }
            input {
                class: "input input-compact input-narrow",
                placeholder: max.placeholder(),
                value: max_text(),
                r#type: "text",
                inputmode: "decimal",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |e| max_text.set(e.value()),
                onblur: move |_| commit(max),
            }
        }
    }
}

/// A searchable picker: the picks as removable pills, then the search box
/// with its results floating above it. ↑ and ↓ move through the results and
/// Enter picks the highlighted one.
#[component]
fn Picker(mut draft: Signal<FilterDraft>, field: PickerField, sources: Sources) -> Element {
    let cache: CatalogCache = use_context();
    let mut query = use_signal(String::new);
    let mut highlight = use_signal(|| 0usize);
    let (picked, results) = {
        let d = draft.read();
        let picked = field.picked(&d.draft);
        let results = field.results(&d.draft, &query(), &sources, &cache.0.read());
        (picked, results)
    };
    let lit = highlight().min(results.len().saturating_sub(1));
    let results_for_keys = results.clone();
    let mut pick = move |value: String| {
        let d = &mut *draft.write();
        field.pick(&mut d.draft, &d.modes, &value);
    };
    rsx! {
        if !picked.is_empty() {
            div { class: "flex flex-wrap gap-1 mb-1",
                for value in picked {
                    div { key: "{value}", class: "chip selected flex items-center gap-05",
                        "{value}"
                        button {
                            class: "chip-remove",
                            onclick: {
                                move |_| {
                                    let d = &mut *draft.write();
                                    field.unpick(&mut d.draft, &d.modes, &value);
                                }
                            },
                            "\u{00d7}"
                        }
                    }
                }
            }
        }
        if !results.is_empty() {
            div { class: "search-float-anchor",
                div { class: "search-float-results",
                    for (i, value) in results.into_iter().enumerate() {
                        div {
                            key: "{value}",
                            class: if i == lit { "chip-unselected search-float-chip web-float-highlight" } else { "chip-unselected search-float-chip" },
                            style: "animation-delay: {i * 40}ms, {250 + i * 40}ms;",
                            onmousedown: move |e| e.prevent_default(),
                            onclick: move |_| pick(value.clone()),
                            "{value}"
                        }
                    }
                }
            }
        }
        div { class: "flex items-center gap-05",
            input {
                class: "input input-compact",
                style: "flex: 1; margin-bottom: 0;",
                placeholder: field.placeholder(),
                value: query(),
                r#type: "text",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |e| {
                    query.set(e.value());
                    highlight.set(0);
                },
                onkeydown: move |e: Event<KeyboardData>| match e.key() {
                    Key::ArrowDown => {
                        e.prevent_default();
                        let last = results_for_keys.len().saturating_sub(1);
                        highlight.set(lit.saturating_add(1).min(last));
                    }
                    Key::ArrowUp => {
                        e.prevent_default();
                        highlight.set(lit.saturating_sub(1));
                    }
                    Key::Enter => {
                        if let Some(value) = results_for_keys.get(lit) {
                            e.prevent_default();
                            pick(value.clone());
                        }
                    }
                    _ => {}
                },
            }
            if !query().is_empty() {
                button {
                    class: "clear-btn",
                    onclick: move |_| query.set(String::new()),
                    "\u{00d7}"
                }
            }
        }
    }
}
