//! The deck form in its three sections, Profile, Budget and Tags, and the pickers over it.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget, Wrap};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{
    DeckOtherTag, MAX_DECK_ORACLE_TAGS, MAX_DECK_TAGS, PowerLevel, deck_tag,
};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck::pickers::{ORACLE_TAG_NO_DESCRIPTION, PICKER_BAR_HINT, Picker, TagPicker};
use zwipe_ui::screens::deck_form::{self as model, Results, Status};

use super::pickers::{self, FormatPicker, TagGrid};
use super::swipe_pick::SwipePicker;
use super::{DeckForm, Field, Overlay};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::card_sheets;
use crate::inbound::widgets::card_motion::{self, CardMotion, Phase};
use crate::inbound::widgets::pill::{self, PillState};
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::{action_bar, button, measure, skeleton};

/// Values sit this far in from the left, under their labels.
const INDENT: &str = "    ";

/// The skeleton chips the oracle tag picker shows while its catalog loads.
const SKELETON_CHIPS: usize = 12;

/// The currencies in the app's chip order.
const CURRENCIES: [PriceCurrency; 3] = [PriceCurrency::Usd, PriceCurrency::Eur, PriceCurrency::Tix];

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, form: &DeckForm) {
    let palette = &shared.palette;
    // The form never reads `!` as Help, so its bar names F1 over every field and picker.
    // `?` reopens the hint only on a field that takes no typing, with no picker open.
    let hint = if form.overlay.is_none() && !form.focus.takes_typing() {
        action_bar::Hint::Question
    } else {
        action_bar::Hint::Absent
    };
    let bar = action_bar::with_help(palette, keys_line(palette, form), hint);
    let bar_height = action_bar::height(&bar, f.area().width.saturating_sub(2));
    let [body, status, keys] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(bar_height),
    ])
    .horizontal_margin(1)
    .areas(f.area());
    f.render_widget(action_bar::wrap(bar, keys.width), keys);

    match &form.overlay {
        Some(Overlay::Format(picker)) => {
            return draw_format(f, palette, form, picker, [body, status, keys]);
        }
        Some(Overlay::Tags(TagPicker::DeckTags, grid)) => {
            return draw_tags(f, palette, form, grid, [body, status, keys]);
        }
        Some(Overlay::Tags(TagPicker::OracleTags, grid)) => {
            return draw_oracle(f, palette, form, grid, [body, status, keys]);
        }
        Some(Overlay::Swipe(picker)) => {
            return draw_swipe(f, shared, picker, [body, status, keys]);
        }
        None => {}
    }

    let block = crate::inbound::widgets::panel::bordered(palette).border_style(palette.border);
    let inner = block.inner(body);
    f.render_widget(block, body);
    match form.model.status() {
        Status::Failed => {
            f.render_widget(Line::from(model::LOAD_FAILED).style(palette.muted), inner);
        }
        Status::Loading => f.render_widget(
            Paragraph::new(skeleton::rows(palette, 12, usize::from(inner.width))),
            inner,
        ),
        Status::Ready => {
            let (lines, focus_line, cursor) = form_lines(palette, form, usize::from(inner.width));
            let rows = usize::from(inner.height);
            let scroll = focus_line.saturating_sub(rows.saturating_sub(4));
            f.render_widget(
                Paragraph::new(lines).scroll((u16::try_from(scroll).unwrap_or(u16::MAX), 0)),
                inner,
            );
            if let Some((line, column)) = cursor
                && line >= scroll
                && line < scroll.saturating_add(rows)
            {
                let y = inner
                    .y
                    .saturating_add(u16::try_from(line.saturating_sub(scroll)).unwrap_or(u16::MAX));
                let x = inner
                    .x
                    .saturating_add(u16::try_from(column).unwrap_or(u16::MAX))
                    .min(inner.right().saturating_sub(1));
                f.set_cursor_position(Position { x, y });
            }
        }
    }

    f.render_widget(
        Line::from(shared.status.clone()).style(palette.muted),
        status,
    );
}

/// The action bar for what is up: the open picker's keys, else the form's.
fn keys_line(palette: &Palette, form: &DeckForm) -> Line<'static> {
    match &form.overlay {
        Some(Overlay::Format(_) | Overlay::Tags(..)) => {
            picker_keys(palette, !form.model.picker_unchanged())
        }
        Some(Overlay::Swipe(picker)) => swipe_keys(palette, picker),
        None => form_keys(palette, form),
    }
}

/// The action bar: what the focused field takes, then Back and Save.
fn form_keys(palette: &Palette, form: &DeckForm) -> Line<'static> {
    let mut keys: Vec<(&str, &str)> = vec![("Tab", "next")];
    match form.focus {
        Field::Format | Field::DeckTags | Field::OracleTags => keys.push(("Enter", "pick")),
        Field::Card(_) => keys.extend([("F3", "Filter"), ("F4", "Swipe")]),
        Field::Power | Field::OtherTags => keys.push(("h/l Space", "pick")),
        Field::Land => keys.push(("-/+", "step")),
        Field::Currency => keys.push(("h/l", "currency")),
        Field::Name | Field::Price => {}
    }
    keys.push(("F8", "×"));
    if !form.focus.takes_typing() {
        keys.push(("i", "about"));
    }
    keys.push(("Esc", "Back"));
    if let Some(save) = form.model.save_label() {
        keys.push(("F2", save));
    }
    action_bar::line(palette, &keys)
}

/// A field's label row: a marker while focused, the label, its `[?]` About marker, and any extras after it. The marker lights while `i` opens it: the field has the focus and takes no typing.
fn label_line(
    palette: &Palette,
    label: &str,
    field: Field,
    focus: Field,
    extras: Vec<Span<'static>>,
) -> Line<'static> {
    let focused = field == focus;
    let style = if focused {
        palette.accent.bold()
    } else {
        palette.muted
    };
    let mut spans = vec![
        Span::raw(if focused { "> " } else { "  " }),
        Span::styled(label.to_owned(), style),
        Span::raw(" "),
    ];
    let lit = field.topic() == focus.topic() && !focus.takes_typing();
    spans.extend(button::about(palette, lit));
    for extra in extras {
        spans.push(Span::raw("  "));
        spans.push(extra);
    }
    Line::from(spans)
}

/// A text field's value line, or `Not set` as every unset field of the form reads.
fn text_line(palette: &Palette, input: &TextInput) -> Line<'static> {
    if input.is_empty() {
        Line::from(vec![
            Span::raw(INDENT),
            Span::styled(model::NOT_SET, palette.muted),
        ])
    } else {
        Line::from(vec![Span::raw(INDENT), Span::raw(input.value().to_owned())])
    }
}

/// Every line of the form, the focused field's first line, and where the cursor goes.
fn form_lines(
    palette: &Palette,
    form: &DeckForm,
    width: usize,
) -> (Vec<Line<'static>>, usize, Option<(usize, usize)>) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut focus_line = 0;
    let mut cursor = None;

    lines.push(heading(model::PROFILE, palette.accent));
    for field in form.fields() {
        if form.focus == field {
            focus_line = lines.len();
        }
        let at = match field {
            Field::Name => name_lines(&mut lines, palette, form),
            Field::Format => {
                format_lines(&mut lines, palette, form);
                None
            }
            Field::Card(slot) => card_slot_lines(&mut lines, palette, form, slot),
            Field::Power => {
                power_lines(&mut lines, palette, form, width);
                None
            }
            Field::Land => {
                land_lines(&mut lines, palette, form);
                None
            }
            Field::Currency => {
                currency_lines(&mut lines, palette, form, width);
                None
            }
            Field::Price => price_lines(&mut lines, palette, form),
            Field::DeckTags => {
                deck_tag_lines(&mut lines, palette, form, width);
                None
            }
            Field::OracleTags => {
                oracle_tag_lines(&mut lines, palette, form, width);
                None
            }
            Field::OtherTags => {
                other_tag_lines(&mut lines, palette, form, width);
                None
            }
        };
        if let Some(at) = at {
            cursor = Some(at);
        }
    }
    (lines, focus_line, cursor)
}

/// A bold section heading in the given style.
fn heading(text: &str, style: Style) -> Line<'static> {
    Line::from(Span::styled(text.to_owned(), style.bold()))
}

/// The clear mark when the field has a value to clear, or nothing.
fn clear(palette: &Palette, shown: bool) -> Vec<Span<'static>> {
    if shown {
        vec![Span::styled("×", palette.muted)]
    } else {
        Vec::new()
    }
}

/// The cursor's column inside an indented text input.
fn cursor_at(input: &TextInput) -> usize {
    INDENT.len().saturating_add(input.cursor())
}

/// The Deck name field and its error; returns the cursor when focused.
fn name_lines(
    lines: &mut Vec<Line<'static>>,
    palette: &Palette,
    form: &DeckForm,
) -> Option<(usize, usize)> {
    lines.push(label_line(
        palette,
        model::DECK_NAME,
        Field::Name,
        form.focus,
        Vec::new(),
    ));
    let cursor = (form.focus == Field::Name).then(|| (lines.len(), cursor_at(&form.name)));
    lines.push(text_line(palette, &form.name));
    if let Some(error) = form.model.name_error() {
        lines.push(Line::from(vec![
            Span::raw(INDENT),
            Span::styled(error.to_owned(), palette.error),
        ]));
    }
    cursor
}

/// The Format field.
fn format_lines(lines: &mut Vec<Line<'static>>, palette: &Palette, form: &DeckForm) {
    let format = form.model.values().format;
    lines.push(label_line(
        palette,
        model::FORMAT,
        Field::Format,
        form.focus,
        clear(palette, format.is_some()),
    ));
    lines.push(Line::from(vec![
        Span::raw(INDENT),
        match format {
            Some(format) => Span::raw(format.display_name().to_owned()),
            None => Span::styled(model::NOT_SET, palette.muted),
        },
    ]));
}

/// A card slot's field and its search results; returns the cursor when focused.
fn card_slot_lines(
    lines: &mut Vec<Line<'static>>,
    palette: &Palette,
    form: &DeckForm,
    slot: Slot,
) -> Option<(usize, usize)> {
    let field = Field::Card(slot);
    let focused = form.focus == field;
    let input = form.inputs.get(slot);
    let mut extras = vec![pill::pill(
        model::FILTER,
        palette.accent,
        PillState {
            picked: form.model.filter_on(slot),
            focused: false,
        },
    )];
    extras.extend(clear(palette, form.model.values().slot(slot).is_some()));
    extras.push(pill::pill(model::SWIPE, palette.add, PillState::default()));
    lines.push(label_line(
        palette,
        form.model.label(slot),
        field,
        form.focus,
        extras,
    ));
    let cursor = focused.then(|| (lines.len(), cursor_at(&input.input)));
    lines.push(text_line(palette, &input.input));
    match form.model.results(slot) {
        None => {}
        Some(Results::Searching) => lines.push(Line::from(vec![
            Span::raw(INDENT),
            pill::pill(model::SEARCHING, palette.muted, PillState::default()),
        ])),
        Some(Results::Empty) => lines.push(Line::from(vec![
            Span::raw(INDENT),
            pill::pill(model::NO_RESULTS, palette.muted, PillState::default()),
        ])),
        Some(Results::Found(cards)) => {
            lines.extend(cards.iter().enumerate().map(|(i, c)| {
                Line::from(vec![
                    Span::raw(INDENT),
                    pill::pill(
                        &c.scryfall_data.name,
                        palette.accent,
                        PillState {
                            picked: focused && i == input.highlighted,
                            focused: false,
                        },
                    ),
                ])
            }));
        }
    }
    cursor
}

/// The Power level chips, then the Budget heading.
fn power_lines(lines: &mut Vec<Line<'static>>, palette: &Palette, form: &DeckForm, width: usize) {
    let focused = form.focus == Field::Power;
    let power = form.model.values().power_level;
    lines.push(Line::default());
    lines.push(label_line(
        palette,
        model::POWER_LEVEL,
        Field::Power,
        form.focus,
        clear(palette, power.is_some()),
    ));
    let chips: Vec<Span<'static>> = PowerLevel::all()
        .iter()
        .enumerate()
        .map(|(i, p)| {
            pill::pill(
                p.display_name(),
                palette.muted,
                PillState {
                    picked: power == Some(*p),
                    focused: focused && i == form.power_focus,
                },
            )
        })
        .collect();
    lines.extend(pill::wrap(INDENT, chips, width));
    lines.push(Line::default());
    lines.push(heading(model::BUDGET, palette.warning));
}

/// The Land target stepper.
fn land_lines(lines: &mut Vec<Line<'static>>, palette: &Palette, form: &DeckForm) {
    let land_target = form.model.values().land_target;
    lines.push(label_line(
        palette,
        model::LAND_TARGET,
        Field::Land,
        form.focus,
        clear(palette, land_target.is_some()),
    ));
    let value = land_target.map_or_else(|| model::NOT_SET.to_owned(), |v| v.to_string());
    lines.push(Line::from(vec![
        Span::raw(INDENT),
        Span::styled("-", palette.accent.bold()),
        Span::raw(format!("  {value}  ")),
        Span::styled("+", palette.accent.bold()),
    ]));
}

/// The Price target label and its currency chips.
fn currency_lines(
    lines: &mut Vec<Line<'static>>,
    palette: &Palette,
    form: &DeckForm,
    width: usize,
) {
    let focused = form.focus == Field::Currency;
    let currency = form.model.values().currency;
    lines.push(label_line(
        palette,
        model::PRICE_TARGET,
        Field::Currency,
        form.focus,
        clear(palette, !form.price.is_empty()),
    ));
    let chips: Vec<Span<'static>> = CURRENCIES
        .iter()
        .map(|c| {
            pill::pill(
                c.label(),
                palette.muted,
                PillState {
                    picked: currency == *c,
                    focused: focused && currency == *c,
                },
            )
        })
        .collect();
    lines.extend(pill::wrap(INDENT, chips, width));
}

/// The price input, then the Tags heading; returns the cursor when focused.
fn price_lines(
    lines: &mut Vec<Line<'static>>,
    palette: &Palette,
    form: &DeckForm,
) -> Option<(usize, usize)> {
    let focused = form.focus == Field::Price;
    let cursor = focused.then(|| (lines.len(), cursor_at(&form.price)));
    let mut line = text_line(palette, &form.price);
    if focused && let Some(first) = line.spans.first_mut() {
        *first = Span::raw("  > ");
    }
    lines.push(line);
    lines.push(Line::default());
    lines.push(heading(model::TAGS, palette.add));
    cursor
}

/// The Deck tags label with its count, then the chosen tags.
fn deck_tag_lines(
    lines: &mut Vec<Line<'static>>,
    palette: &Palette,
    form: &DeckForm,
    width: usize,
) {
    let tags = form.model.tags(TagPicker::DeckTags);
    let count = Span::styled(format!("{}/{MAX_DECK_TAGS}", tags.len()), palette.muted);
    let mut extras = vec![count];
    extras.extend(clear(palette, !tags.is_empty()));
    lines.push(label_line(
        palette,
        model::DECK_TAGS,
        Field::DeckTags,
        form.focus,
        extras,
    ));
    let labels: Vec<String> = tags.iter().map(|t| deck_tag::deck_tag_label(t)).collect();
    lines.extend(chip_box(
        palette,
        &labels,
        (form.focus == Field::DeckTags).then_some(form.tag_focus),
        width,
    ));
}

/// The Oracle tags label with its count, then the chosen tags.
fn oracle_tag_lines(
    lines: &mut Vec<Line<'static>>,
    palette: &Palette,
    form: &DeckForm,
    width: usize,
) {
    let tags = form.model.tags(TagPicker::OracleTags);
    let count = Span::styled(
        format!("{}/{MAX_DECK_ORACLE_TAGS}", tags.len()),
        palette.muted,
    );
    let mut extras = vec![count];
    extras.extend(clear(palette, !tags.is_empty()));
    lines.push(label_line(
        palette,
        model::ORACLE_TAGS,
        Field::OracleTags,
        form.focus,
        extras,
    ));
    lines.extend(chip_box(
        palette,
        tags,
        (form.focus == Field::OracleTags).then_some(form.oracle_focus),
        width,
    ));
}

/// The Other tags chips.
fn other_tag_lines(
    lines: &mut Vec<Line<'static>>,
    palette: &Palette,
    form: &DeckForm,
    width: usize,
) {
    let focused = form.focus == Field::OtherTags;
    let other_tags = &form.model.values().other_tags;
    lines.push(label_line(
        palette,
        model::OTHER_TAGS,
        Field::OtherTags,
        form.focus,
        clear(palette, !other_tags.is_empty()),
    ));
    let chips: Vec<Span<'static>> = DeckOtherTag::all()
        .iter()
        .enumerate()
        .map(|(i, t)| {
            pill::pill(
                t.display_name(),
                palette.muted,
                PillState {
                    picked: other_tags.contains(t),
                    focused: focused && i == form.other_focus,
                },
            )
        })
        .collect();
    lines.extend(pill::wrap(INDENT, chips, width));
}

/// The selected tags, each with its "×", or "Not set".
fn chip_box(
    palette: &Palette,
    labels: &[String],
    focus: Option<usize>,
    width: usize,
) -> Vec<Line<'static>> {
    if labels.is_empty() {
        return vec![Line::from(vec![
            Span::raw(INDENT),
            Span::styled(model::NOT_SET, palette.muted),
        ])];
    }
    pill::wrap(
        INDENT,
        labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                pill::pill(
                    &format!("{label} ×"),
                    palette.accent,
                    PillState {
                        picked: false,
                        focused: focus == Some(i),
                    },
                )
            })
            .collect(),
        width,
    )
}

/// What a picker's frame shows around its grid.
struct PickerParts {
    /// What follows "Search" in the controls row.
    extras: Vec<Span<'static>>,
    /// The definition bar's name and lines.
    bar: (String, Vec<String>),
}

/// A picker's frame: the search box and its extras, the definition bar, then the grid. Returns the grid's area.
fn picker_frame(
    f: &mut Frame<'_>,
    palette: &Palette,
    areas: [Rect; 3],
    search: &TextInput,
    frame: PickerParts,
) -> Rect {
    let PickerParts { extras, bar } = frame;
    let [body, _, _] = areas;
    let mut bar_lines = vec![Line::from(Span::styled(bar.0, palette.accent.bold()))];
    bar_lines.extend(bar.1.into_iter().map(Line::from));
    let bar = Paragraph::new(bar_lines)
        .wrap(Wrap { trim: true })
        .block(crate::inbound::widgets::panel::bordered(palette).border_style(palette.border));
    // The definition wraps to the width, but never takes more than half the room under the search box.
    let bar_height = measure::height_of(bar.line_count(body.width))
        .min(body.height.saturating_sub(4) / 2)
        .max(3);
    let [controls, search_area, bar_area, grid] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(bar_height),
        Constraint::Fill(1),
    ])
    .areas(body);
    let mut head = vec![Span::styled("Search", palette.muted)];
    for extra in extras {
        head.push(Span::raw("  "));
        head.push(extra);
    }
    f.render_widget(Line::from(head), controls);
    search.draw(f, palette, search_area, "Search", /* focused */ true);
    f.set_cursor_position(search.cursor_position(search_area));
    f.render_widget(bar, bar_area);
    grid
}

/// A picker's action bar, its Save bright only once the picks differ from where the picker opened.
fn picker_keys(palette: &Palette, save_enabled: bool) -> Line<'static> {
    let save_style = if save_enabled {
        palette.text().bold()
    } else {
        palette.muted
    };
    let mut bar = action_bar::line(
        palette,
        &[
            ("arrows", "move"),
            ("Enter", "pick"),
            ("F6", "hint"),
            ("Esc", "Back"),
        ],
    );
    bar.spans.push(Span::styled("F2 ", save_style));
    bar.spans.push(Span::styled("Save", palette.muted));
    bar
}

/// Chips wrapped across the grid, scrolled so the focused one shows.
fn draw_grid(f: &mut Frame<'_>, chips: Vec<Span<'static>>, focus: usize, area: Rect) {
    let width = usize::from(area.width);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut used: usize = 0;
    let mut focus_line = 0;
    for (i, chip) in chips.into_iter().enumerate() {
        let len = chip.width().saturating_add(1);
        if used > 0 && used.saturating_add(len) > width {
            lines.push(Line::from(std::mem::take(&mut current)));
            used = 0;
        }
        if i == focus {
            focus_line = lines.len();
        }
        current.push(chip);
        current.push(Span::raw(" "));
        used = used.saturating_add(len);
    }
    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    let scroll = focus_line.saturating_sub(usize::from(area.height).saturating_sub(1));
    f.render_widget(
        Paragraph::new(lines).scroll((u16::try_from(scroll).unwrap_or(u16::MAX), 0)),
        area,
    );
}

fn no_results(f: &mut Frame<'_>, palette: &Palette, area: Rect) {
    f.render_widget(Line::from("No results").style(palette.muted), area);
}

fn draw_format(
    f: &mut Frame<'_>,
    palette: &Palette,
    form: &DeckForm,
    picker: &FormatPicker,
    areas: [Rect; 3],
) {
    let grid = picker_frame(
        f,
        palette,
        areas,
        &picker.search,
        PickerParts {
            extras: Vec::new(),
            bar: pickers::format_bar(picker.detail),
        },
    );
    let formats = picker.formats();
    if formats.is_empty() {
        return no_results(f, palette, grid);
    }
    let chosen = form.model.values().format;
    let chips = formats
        .iter()
        .enumerate()
        .map(|(i, format)| {
            pill::pill(
                format.display_name(),
                palette.muted,
                PillState {
                    picked: chosen == Some(*format),
                    focused: i == picker.focus,
                },
            )
        })
        .collect();
    draw_grid(f, chips, picker.focus, grid);
}

/// The bar before a chip is tapped: the hint for `picker`.
fn empty_bar(picker: Picker) -> (String, Vec<String>) {
    (
        PICKER_BAR_HINT.to_owned(),
        vec![picker.bar_empty().to_owned()],
    )
}

fn draw_tags(
    f: &mut Frame<'_>,
    palette: &Palette,
    form: &DeckForm,
    grid: &TagGrid,
    areas: [Rect; 3],
) {
    let picked = form.model.tags(TagPicker::DeckTags);
    let bar = match grid
        .detail
        .as_ref()
        .and_then(|slug| form.deck_tag_catalog.iter().find(|t| t.slug == *slug))
    {
        Some(tag) => (tag.display_name.clone(), vec![tag.description.clone()]),
        None => empty_bar(Picker::DeckTags),
    };
    let area = picker_frame(
        f,
        palette,
        areas,
        &grid.search,
        PickerParts {
            extras: vec![
                Span::styled(format!("{}/{MAX_DECK_TAGS}", picked.len()), palette.muted),
                Span::styled("F8 ×", palette.muted),
            ],
            bar,
        },
    );
    let tags = grid.deck_tags(&form.deck_tag_catalog);
    if tags.is_empty() {
        return no_results(f, palette, area);
    }
    let chips = tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            pill::pill(
                &tag.display_name,
                palette.muted,
                PillState {
                    picked: picked.contains(&tag.slug),
                    focused: i == grid.focus,
                },
            )
        })
        .collect();
    draw_grid(f, chips, grid.focus, area);
}

fn draw_oracle(
    f: &mut Frame<'_>,
    palette: &Palette,
    form: &DeckForm,
    grid: &TagGrid,
    areas: [Rect; 3],
) {
    let catalog = form.oracle_catalog.as_deref();
    let picked = form.model.tags(TagPicker::OracleTags);
    let bar = match &grid.detail {
        Some(slug) => (
            slug.clone(),
            vec![
                catalog
                    .and_then(|c| c.iter().find(|t| t.slug == *slug))
                    .and_then(|t| t.description.clone())
                    .unwrap_or_else(|| ORACLE_TAG_NO_DESCRIPTION.to_owned()),
            ],
        ),
        None => empty_bar(Picker::OracleTags),
    };
    let area = picker_frame(
        f,
        palette,
        areas,
        &grid.search,
        PickerParts {
            extras: vec![
                pill::pill("F3 Dictionary", palette.accent, PillState::default()),
                Span::styled(
                    format!("{}/{MAX_DECK_ORACLE_TAGS}", picked.len()),
                    palette.muted,
                ),
                Span::styled("F8 ×", palette.muted),
            ],
            bar,
        },
    );
    let Some(catalog) = catalog else {
        let ghosts = (0..SKELETON_CHIPS)
            .map(|i| {
                Span::styled(
                    "░".repeat(6_usize.saturating_add((i % 4).saturating_mul(2))),
                    palette.muted,
                )
            })
            .collect();
        return draw_grid(f, ghosts, usize::MAX, area);
    };
    let tags = grid.oracle_tags(catalog, picked);
    if tags.is_empty() {
        return no_results(f, palette, area);
    }
    let chips = tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            pill::pill(
                &tag.slug,
                palette.muted,
                PillState {
                    picked: picked.contains(&tag.slug),
                    focused: i == grid.focus,
                },
            )
        })
        .collect();
    draw_grid(f, chips, grid.focus, area);
}

/// The swipe picker: the card up now and what it says, or a ghost while loading, with the filter sheet over it while open.
fn draw_swipe(f: &mut Frame<'_>, shared: &Shared, picker: &SwipePicker, areas: [Rect; 3]) {
    let palette = &shared.palette;
    let [body, _, _] = areas;
    let block = crate::inbound::widgets::panel::bordered(palette).border_style(palette.border);
    let inner = block.inner(body);
    f.render_widget(block, body);
    let now = Instant::now();
    let motion = picker.motion.as_ref().filter(|m| m.running(now));
    // While a card comes back after an undo, the card it covers rests under it.
    let resting = match motion.map(CardMotion::phase) {
        Some(Phase::Enter) => picker.under(),
        _ => picker.current(),
    };
    match resting {
        Some(card) => f.render_widget(
            Paragraph::new(card_lines(palette, card)).wrap(Wrap { trim: true }),
            inner,
        ),
        None if picker.current().is_none() => f.render_widget(
            Paragraph::new(skeleton::rows(palette, 6, usize::from(inner.width))),
            inner,
        ),
        None => {}
    }
    if let Some(motion) = motion {
        let moving = match motion.phase() {
            Phase::Exit(card) => Some(card),
            Phase::Enter => picker.current(),
        };
        if let Some(card) = moving
            && let Some(placement) = motion.placement(now, inner, inner)
        {
            let text = Paragraph::new(card_lines(palette, card)).wrap(Wrap { trim: true });
            card_motion::draw(f.buffer_mut(), &placement, palette.base, |buf| {
                text.render(inner, buf);
            });
        }
    }
    if let Some(sheet) = &picker.sheet {
        card_sheets::filter::draw::draw(f, shared, sheet, f.area());
    }
}

/// The swipe picker's action bar: the swipes in their colors, maybeboard only for the commander and partner, then the rest.
fn swipe_keys(palette: &Palette, picker: &SwipePicker) -> Line<'static> {
    let up = matches!(picker.slot, Slot::Commander | Slot::Partner).then_some(("↑", "maybeboard"));
    let mut shown = action_bar::swipes(palette, ("→", "add"), ("←", "skip"), up, ("↓", "undo"));
    let filter_label = if picker.filter_dot() {
        "Filter •"
    } else {
        "Filter"
    };
    shown.extend(
        [
            ("F3", filter_label),
            ("F5", "Refresh"),
            ("F6", "hint"),
            ("Esc", "Back"),
        ]
        .into_iter()
        .map(|(k, a)| (k, a, palette.muted)),
    );
    action_bar::styled_line(&shown)
}

/// A card's name, cost, type, rules and stats.
fn card_lines(palette: &Palette, card: &Card) -> Vec<Line<'static>> {
    let data = &card.scryfall_data;
    let mut lines = vec![Line::from(vec![
        Span::styled(data.name.clone(), palette.accent.bold()),
        Span::styled(
            format!("  {}", data.mana_cost.clone().unwrap_or_default()),
            palette.muted,
        ),
    ])];
    if let Some(type_line) = &data.type_line {
        lines.push(Line::from(Span::styled(type_line.clone(), palette.muted)));
    }
    lines.push(Line::default());
    if let Some(text) = &data.oracle_text {
        lines.extend(text.lines().map(|l| Line::from(l.to_owned())));
    }
    if let (Some(power), Some(toughness)) = (&data.power, &data.toughness) {
        lines.push(Line::default());
        lines.push(Line::from(format!("{power}/{toughness}")).bold());
    }
    lines
}
