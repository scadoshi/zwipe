//! Drawing the filter sheet: the accordion of sections, the open section's rows, and the footer keys.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear as ClearWidget, Padding, Paragraph};

use super::{FilterSheet, Target};
use crate::domain::filter::sections::{ChipSet, Row, StepRow};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::{action_bar, mana};

/// The sheet takes this share of the screen's height, in percent.
const HEIGHT_PERCENT: u16 = 85;

/// Marks the focused line.
const POINTER: &str = "› ";

/// Draws the sheet over the bottom of `area`.
pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, sheet: &FilterSheet, area: Rect) {
    let palette = &shared.palette;
    let height = (area.height.saturating_mul(HEIGHT_PERCENT) / 100)
        .max(3)
        .min(area.height);
    let [_, sheet_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(height)]).areas(area);
    let apply_style = if sheet.unchanged() {
        palette.muted
    } else {
        palette.accent.bold()
    };
    let mut footer = action_bar::line(palette, &[("Esc", "back")]);
    footer.spans.push(Span::styled("a", apply_style.bold()));
    footer.spans.push(Span::styled(" apply  ", apply_style));
    footer
        .spans
        .extend(action_bar::line(palette, &[("r", "reset"), ("x", "clear"), ("?", "hint")]).spans);
    let block = Block::bordered()
        .title(Line::from(" Filter ").bold())
        .title_bottom(footer)
        .border_style(palette.accent)
        .style(palette.base)
        .padding(Padding::horizontal(1));
    let inner = block.inner(sheet_area);
    f.render_widget(ClearWidget, sheet_area);
    f.render_widget(block, sheet_area);

    let (lines, focus_line) = lines(shared, sheet, inner.width);
    let rows = usize::from(inner.height);
    let scroll = focus_line
        .saturating_sub(rows / 2)
        .min(lines.len().saturating_sub(rows));
    let shown: Vec<Line<'static>> = lines.into_iter().skip(scroll).take(rows).collect();
    f.render_widget(Paragraph::new(shown), inner);
}

/// Every line of the sheet's body, and the index of the focused one.
pub(crate) fn lines(
    shared: &Shared,
    sheet: &FilterSheet,
    width: u16,
) -> (Vec<Line<'static>>, usize) {
    let palette = &shared.palette;
    let focused = sheet.focused();
    let mut lines = Vec::new();
    let mut focus_line = 0;
    for section in sheet.sections() {
        let is_focus = focused == Some(Target::Header(section));
        if is_focus {
            focus_line = lines.len();
        }
        let open = sheet.open == Some(section);
        let mut spans = vec![
            pointer(palette, is_focus),
            Span::styled(if open { "▾ " } else { "▸ " }, palette.muted),
            Span::styled(section.title(), header_style(palette, is_focus)),
        ];
        if section.is_active(&sheet.draft) {
            spans.push(Span::styled("  ×", palette.warning));
        }
        lines.push(Line::from(spans));
        if !open {
            continue;
        }
        for row in sheet.rows() {
            let is_focus = focused == Some(Target::Row(row));
            if is_focus {
                focus_line = lines.len();
            }
            lines.extend(row_lines(shared, sheet, row, is_focus, width));
        }
    }
    (lines, focus_line)
}

fn pointer(palette: &Palette, focused: bool) -> Span<'static> {
    if focused {
        Span::styled(POINTER, palette.accent.bold())
    } else {
        Span::raw("  ")
    }
}

fn header_style(palette: &Palette, focused: bool) -> Style {
    if focused {
        palette.accent.bold()
    } else {
        palette.base
    }
}

/// A chip: on reversed in the accent, off muted, the focused one underlined.
fn chip(
    palette: &Palette,
    label: &str,
    on: bool,
    focused: bool,
    color: Option<Style>,
) -> Span<'static> {
    let style = match (on, color) {
        (true, Some(color)) => color.bold().reversed(),
        (true, None) => palette.accent.bold().reversed(),
        (false, Some(color)) => color,
        (false, None) => palette.muted,
    };
    let style = if focused { style.underlined() } else { style };
    Span::styled(format!(" {label} "), style)
}

/// The lines one row draws, indented under its section.
fn row_lines(
    shared: &Shared,
    sheet: &FilterSheet,
    row: Row,
    focused: bool,
    width: u16,
) -> Vec<Line<'static>> {
    let palette = &shared.palette;
    let indent = || vec![pointer(palette, focused), Span::raw("  ")];
    match row {
        Row::Label {
            text, mode, clear, ..
        } => {
            let mut spans = indent();
            spans.push(Span::styled(text, palette.muted.bold()));
            if let Some(chip_kind) = mode {
                spans.push(Span::raw(" "));
                spans.push(chip(
                    palette,
                    sheet.modes.label(&sheet.draft, chip_kind),
                    false,
                    focused,
                    Some(palette.accent),
                ));
            }
            if clear.is_some() {
                spans.push(Span::styled(" ×", palette.warning));
            }
            vec![Line::from(spans)]
        }
        Row::Text(field) => {
            let value = sheet.texts.get(&field).map(|i| i.value().to_owned());
            vec![input_line(
                palette,
                indent(),
                value.as_deref(),
                field.placeholder(),
                focused,
            )]
        }
        Row::Number(field) => {
            let value = sheet.numbers.get(&field).map(|i| i.value().to_owned());
            let mut spans = indent();
            if matches!(
                field,
                crate::domain::filter::sections::NumberField::ManaMax
                    | crate::domain::filter::sections::NumberField::PriceMax
            ) {
                spans.push(Span::styled("to ", palette.muted));
            }
            vec![input_line(
                palette,
                spans,
                value.as_deref(),
                field.placeholder(),
                focused,
            )]
        }
        Row::Chips(set) => {
            let chips = sheet.chips(set, &shared.cards.catalog);
            if chips.is_empty() {
                return Vec::new();
            }
            let chip_focus = sheet.chip_focus.get(&set).copied().unwrap_or_default();
            let mut spans = indent();
            for (i, (label, on)) in chips.iter().enumerate() {
                let color = match set {
                    ChipSet::Colors => Some(mana::style(palette, i)),
                    _ => None,
                };
                spans.push(chip(palette, label, *on, focused && i == chip_focus, color));
                spans.push(Span::raw(" "));
            }
            wrap(Line::from(spans), width)
        }
        Row::Stepper(stepper) => {
            let mut spans = indent();
            let value = |v: Option<i32>| v.map_or_else(|| "-".to_owned(), |v| v.to_string());
            match stepper {
                StepRow::Exact(stat) => {
                    spans.push(Span::styled("- ", palette.muted));
                    spans.push(Span::styled(
                        value(stat.exact(&sheet.draft)),
                        palette.base.bold(),
                    ));
                    spans.push(Span::styled(" +", palette.muted));
                }
                StepRow::Range(stat) => {
                    let (min, max) = stat.range(&sheet.draft).unwrap_or((0, 0));
                    let end = |v: i32, on: bool| {
                        let style = if focused && on {
                            palette.accent.bold().underlined()
                        } else {
                            palette.base.bold()
                        };
                        vec![
                            Span::styled("- ", palette.muted),
                            Span::styled(v.to_string(), style),
                            Span::styled(" +", palette.muted),
                        ]
                    };
                    spans.extend(end(min, !sheet.step_max));
                    spans.push(Span::styled("  to  ", palette.muted));
                    spans.extend(end(max, sheet.step_max));
                }
            }
            vec![Line::from(spans)]
        }
        Row::Picker(field) => {
            let mut lines = Vec::new();
            let pills = field.picked(&sheet.draft);
            if !pills.is_empty() {
                let mut spans = vec![Span::raw("    ")];
                for value in &pills {
                    spans.push(Span::styled(
                        format!(" {value} ×"),
                        palette.accent.reversed(),
                    ));
                    spans.push(Span::raw(" "));
                }
                lines.extend(wrap(Line::from(spans), width));
            }
            let picker = sheet.pickers.get(&field);
            let query = picker
                .map(|p| p.input.value().to_owned())
                .unwrap_or_default();
            lines.push(input_line(
                palette,
                indent(),
                Some(&query),
                field.placeholder(),
                focused,
            ));
            let results = field.results(
                &sheet.draft,
                &query,
                &sheet.config.sources,
                &shared.cards.catalog,
            );
            if !results.is_empty() {
                let highlight = picker.map_or(0, |p| p.highlight);
                let mut spans = vec![Span::raw("    ")];
                for (i, value) in results.iter().enumerate() {
                    spans.push(chip(palette, value, false, focused && i == highlight, None));
                    spans.push(Span::raw(" "));
                }
                lines.extend(wrap(Line::from(spans), width));
            }
            lines
        }
    }
}

/// A text box as one line: the value, or the placeholder muted, with a cursor mark when focused.
fn input_line(
    palette: &Palette,
    mut spans: Vec<Span<'static>>,
    value: Option<&str>,
    placeholder: &str,
    focused: bool,
) -> Line<'static> {
    let value = value.unwrap_or_default();
    spans.push(Span::styled("[", palette.border));
    if value.is_empty() {
        spans.push(Span::styled(placeholder.to_owned(), palette.muted.italic()));
    } else {
        spans.push(Span::styled(value.to_owned(), palette.base));
    }
    if focused {
        spans.push(Span::styled("▏", palette.accent));
    }
    spans.push(Span::styled("]", palette.border));
    if !value.is_empty() {
        spans.push(Span::styled(" ×", palette.muted));
    }
    Line::from(spans)
}

/// Splits a long line of chips into lines that fit `width`, breaking between spans.
fn wrap(line: Line<'static>, width: u16) -> Vec<Line<'static>> {
    let width = usize::from(width).max(1);
    let mut lines = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut used = 0usize;
    for span in line.spans {
        let len = span.content.chars().count();
        if used > 0 && used.saturating_add(len) > width {
            lines.push(Line::from(std::mem::take(&mut current)));
            current.push(Span::raw("    "));
            used = 4;
        }
        used = used.saturating_add(len);
        current.push(span);
    }
    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    lines
}
