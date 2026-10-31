//! Activity: the title, the user's counters, then the sections, the Most skipped rows and the action bar.
//!
//! The sections and rows are one column of lines scrolled so the focused piece is in view. The heatmap draws a day as a two-cell square (`▆` and a gap) when the year fits, else one cell per day, cropped to the newest weeks when even that does not fit; the crop follows the day cursor.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Color as TermColor, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};

use super::{ActivityView, Focus, Load, Loaded, PASSES_TITLE, Section, pass_row};
use crate::domain::activity::{self, Bar, DAYS_PER_WEEK, HeatLayout, RateKind, WEEKDAY_LABELS};
use crate::domain::count_up;
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::home::draw as home_draw;
use crate::inbound::widgets::card_rows::CardRow;
use crate::inbound::widgets::{
    action_bar, charts, color_map, entrance, figures, header, item_list, otag,
};

/// Rows the counters panel takes: its frame, the figures and the chips.
const COUNTS_HEIGHT: u16 = figures::HEIGHT.saturating_add(3);

/// Columns left of every body line for the focus marker.
pub(crate) const GUTTER: &str = "  ";

/// Marks the focused piece's first line.
const MARKER: &str = "> ";

/// Indents a section's body under its header.
pub(crate) const INDENT: &str = "  ";

/// Room left of the heatmap for the weekday labels.
const WEEKDAY_WIDTH: usize = 4;

/// A day on the heatmap, and a peak day.
const DAY: &str = "▆";
const PEAK: &str = "█";

/// Each heat level's share of the accent over the background, as orangui's `.heat-1` to `.heat-4`.
const HEAT_PERCENTS: [u32; 4] = [30, 55, 80, 100];

/// A peak's lit edge: the accent this far toward white, as `.heat-peak`.
const PEAK_PERCENT: u32 = 60;

/// The share of a cell's sweep spent brightened, as `heat-sweep`'s 55% keyframe.
const BRIGHT_SHARE: u32 = 55;

/// The Totals columns: their height in rows, each column's width and its bar's.
const TOTALS_ROWS: usize = 5;
const TOTALS_COLUMN: usize = 12;
const TOTALS_BAR: usize = 6;

/// The Totals ghost columns' heights, as the app's skeleton.
const GHOST_HEIGHTS: [usize; 4] = [70, 100, 20, 35];

/// Ghost rows in the Most skipped group while loading.
const GHOST_PASSES: usize = 10;

/// The ghost block.
pub(crate) const GHOST: &str = "░";

/// Eighths of a cell, emptiest first, for a column's partial top.
const EIGHTHS: [&str; 8] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇"];

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &ActivityView) {
    let palette = &shared.palette;
    let area = f.area().inner(Margin::new(1, 0));
    let bar = action_bar::line(palette, &key_hints(view));
    let bar_height = action_bar::height(&bar, area.width);
    let [title, counts, body, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(COUNTS_HEIGHT),
        Constraint::Fill(1),
        Constraint::Length(bar_height),
    ])
    .areas(area);
    header::draw(f, palette, title, super::TITLE, header::Corner::Hint);
    draw_counts(f, shared, view, counts);
    match &view.load {
        Load::Loading => draw_lines(f, skeleton(palette, body.width), (0, 0), body),
        Load::Failed => draw_note(f, palette, super::LOAD_FAILED, body),
        Load::Loaded(loaded) if !activity::has_anything(&loaded.activity) => {
            draw_note(f, palette, super::NOTHING_YET, body);
        }
        Load::Loaded(loaded) => {
            let (lines, focus) = loaded_lines(shared, view, loaded, body.width);
            draw_lines(f, lines, focus, body);
        }
    }
    f.render_widget(action_bar::wrap(bar, keys.width), keys);
    if let Some(info) = &view.overlay {
        info.draw(f, palette);
    }
}

/// The user's own counters, rolling until the payload lands, with "Live" and "Only me" under them.
fn draw_counts(f: &mut Frame<'_>, shared: &Shared, view: &ActivityView, area: Rect) {
    let palette = &shared.palette;
    let block = Block::bordered().border_style(palette.border);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let [strip, chips] =
        Layout::vertical([Constraint::Length(figures::HEIGHT), Constraint::Length(1)]).areas(inner);
    let shown = view.figures(shared);
    let since_arrival = view
        .figures_arrived
        .get()
        .map_or(Duration::ZERO, |t| t.elapsed());
    figures::draw(
        f,
        palette,
        strip,
        shown,
        view.opened.elapsed(),
        since_arrival,
    );
    let mut spans = Vec::new();
    if shown.is_some() {
        spans.push(home_draw::chip("Live", palette.add));
        spans.push(Span::raw(" "));
    }
    spans.push(home_draw::chip("Only me", palette.accent));
    f.render_widget(Line::from(spans).centered(), chips);
}

pub(crate) fn draw_note(f: &mut Frame<'_>, palette: &Palette, note: &str, area: Rect) {
    f.render_widget(
        Paragraph::new(Line::styled(note.to_owned(), palette.muted))
            .centered()
            .wrap(Wrap { trim: true }),
        area.centered_vertically(Constraint::Length(1)),
    );
}

/// Draws `lines` wrapped into `area`, scrolled so the lines from `focus.0` to `focus.1` are in view.
pub(crate) fn draw_lines(
    f: &mut Frame<'_>,
    lines: Vec<Line<'static>>,
    focus: (usize, usize),
    area: Rect,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let rows_before = |end: usize| {
        let head = lines.get(..end).unwrap_or(&lines);
        if head.is_empty() {
            0
        } else {
            item_list::wrapped_height(head, area.width)
        }
    };
    let start = rows_before(focus.0);
    let end = rows_before(focus.1.saturating_add(1));
    // A focused piece taller than the view shows its top and keeps its last line, a card row's keys, in the bottom rows.
    if end.saturating_sub(start) > area.height
        && let Some(last) = lines.get(focus.1).cloned()
    {
        let last = vec![last];
        let last_height = item_list::wrapped_height(&last, area.width).min(area.height);
        let [top, bottom] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(last_height)]).areas(area);
        f.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .scroll((start, 0)),
            top,
        );
        f.render_widget(Paragraph::new(last).wrap(Wrap { trim: false }), bottom);
        return;
    }
    let offset = end.saturating_sub(area.height).min(start);
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((offset, 0)),
        area,
    );
}

/// The keys for what has the focus.
fn key_hints(view: &ActivityView) -> Vec<(&'static str, &'static str)> {
    let mut keys = Vec::new();
    if view.loaded().is_some() {
        keys.push(("j/k", "Move"));
        match view.focus {
            Focus::Header(section) => {
                let label = if view.is_open(section) {
                    "Close"
                } else {
                    "Open"
                };
                keys.push(("Enter", label));
            }
            Focus::Heat => keys.push(("hjkl", "Day")),
            Focus::Chips(_) => keys.push(("h/l", "Show")),
            Focus::Bar(section, index) => {
                let revealable = view
                    .bars(section)
                    .get(index)
                    .is_some_and(|bar| bar.revealable);
                if revealable {
                    keys.push(("Enter", "Reveal"));
                }
                if section == Section::Roles && view.open_role.is_some() {
                    keys.push(("Tab", "Tags"));
                }
                let examples = match section {
                    Section::Roles => view.role_tags.open_tag().is_some(),
                    Section::Tags => view.open_tag.is_some(),
                    Section::Activity => false,
                };
                if examples {
                    keys.push(("x", "Examples"));
                }
            }
            Focus::Passes => {
                if view.loaded().is_some_and(|l| !l.activity.passes.is_empty()) {
                    keys.push(("Enter", "Expand"));
                }
            }
        }
        let topic = view.focused_section().is_none_or(|s| view.is_open(s));
        if topic {
            keys.push(("?", "About"));
        }
    }
    keys.push(("H", "Hint"));
    keys.push(("Esc", "Back"));
    keys
}

/// The body's lines and the range of the focused piece.
#[derive(Debug, Default)]
pub(crate) struct Body {
    pub(crate) lines: Vec<Line<'static>>,
    pub(crate) focus: (usize, usize),
}

impl Body {
    /// Adds a line no focus lands on.
    pub(crate) fn line(&mut self, line: Line<'static>) {
        self.push(GUTTER, line);
    }

    /// Adds a piece the focus can land on, marked when `focused`.
    pub(crate) fn item(&mut self, lines: Vec<Line<'static>>, focused: bool) {
        let start = self.lines.len();
        for (i, line) in lines.into_iter().enumerate() {
            let gutter = if focused && i == 0 { MARKER } else { GUTTER };
            self.push(gutter, line);
        }
        if focused {
            self.focus = (start, self.lines.len().saturating_sub(1));
        }
    }

    fn push(&mut self, gutter: &'static str, line: Line<'static>) {
        let mut spans = vec![Span::raw(gutter).bold()];
        spans.extend(line.spans);
        self.lines.push(Line::from(spans).style(line.style));
    }
}

/// `line` indented under a section header.
pub(crate) fn indented(line: Line<'static>) -> Line<'static> {
    let mut spans = vec![Span::raw(INDENT)];
    spans.extend(line.spans);
    Line::from(spans).style(line.style)
}

/// The loaded screen as lines: the accordion, then the Most skipped group.
fn loaded_lines(
    shared: &Shared,
    view: &ActivityView,
    loaded: &Loaded,
    width: u16,
) -> (Vec<Line<'static>>, (usize, usize)) {
    let palette = &shared.palette;
    let inner = usize::from(width)
        .saturating_sub(GUTTER.len())
        .saturating_sub(INDENT.len());
    let mut body = Body::default();
    for (i, &section) in view.sections.iter().enumerate() {
        let Some(collapsible) = view.accordion.sections.get(i) else {
            continue;
        };
        let focused = view.focus == Focus::Header(section);
        let mut header = collapsible.header(palette, focused);
        if collapsible.open {
            header.spans.push(Span::styled("  ?", palette.muted));
        }
        body.item(vec![header], focused);
        if !collapsible.open {
            continue;
        }
        match section {
            Section::Activity => activity_body(&mut body, palette, view, loaded, inner),
            Section::Roles | Section::Tags => {
                rates_body(&mut body, shared, view, section, inner);
            }
        }
    }
    let passes = &loaded.activity.passes;
    body.line(Line::default());
    let header = Line::from(vec![
        Span::raw(PASSES_TITLE).bold(),
        Span::styled("  ?", palette.muted),
    ]);
    if passes.is_empty() {
        body.item(
            vec![header, Line::styled(activity::PASSES_EMPTY, palette.muted)],
            view.focus == Focus::Passes,
        );
    } else {
        body.line(header);
        let rows: Vec<CardRow<'_>> = passes.iter().map(|p| pass_row(palette, p)).collect();
        let lines = view.rows.lines(palette, &shared.caches.oracle_tags, &rows);
        let focused_row = view.rows.focused().min(rows.len().saturating_sub(1));
        for (i, row) in lines.into_iter().enumerate() {
            body.item(row, view.focus == Focus::Passes && i == focused_row);
        }
    }
    (body.lines, body.focus)
}

/// The Activity section: the heatmap with its readout, a rule, then Totals.
fn activity_body(
    body: &mut Body,
    palette: &Palette,
    view: &ActivityView,
    loaded: &Loaded,
    width: usize,
) {
    let has_days = !loaded.activity.days.is_empty();
    let outcomes = &loaded.activity.outcomes;
    if has_days {
        let mut lines = heat_lines(palette, view, &loaded.heat, width);
        lines.push(Line::default());
        lines.push(readout_line(palette, view));
        body.item(
            lines.into_iter().map(indented).collect(),
            view.focus == Focus::Heat,
        );
    }
    if activity::has_outcomes(outcomes) {
        if has_days {
            body.line(indented(Line::styled("─".repeat(width), palette.muted)));
        }
        for line in totals_lines(palette, &activity::outcome_bars(outcomes), width) {
            body.line(indented(line));
        }
    }
}

/// The readout's three tags in one muted tone, faded in for a moment when they change.
fn readout_line(palette: &Palette, view: &ActivityView) -> Line<'static> {
    let style = if view.readout_age() < entrance::DURATION {
        palette.muted.add_modifier(Modifier::DIM)
    } else {
        palette.muted
    };
    let mut spans = Vec::new();
    for (i, tag) in view.readout().iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(home_draw::chip(tag, style));
    }
    Line::from(spans)
}

/// The style of a heat cell of `level`, as orangui's shades: an empty day in the border color, the rest the accent mixed over the background at rising strength. A palette without true colors dims the fainter shades instead.
pub(crate) fn heat_style(palette: &Palette, level: u8, peak: bool) -> Style {
    let accent = palette.accent.fg;
    let background = palette.base.bg;
    let true_color = matches!(accent, Some(TermColor::Rgb(..)))
        && matches!(background, Some(TermColor::Rgb(..)));
    if peak {
        return match accent {
            Some(color) if true_color => Style::new()
                .fg(color_map::mix(
                    TermColor::Rgb(255, 255, 255),
                    Some(color),
                    PEAK_PERCENT,
                ))
                .bold(),
            _ => palette.accent.bold(),
        };
    }
    if level == 0 {
        return if matches!(palette.border.fg, Some(TermColor::Rgb(..))) {
            palette.border
        } else {
            palette.muted
        };
    }
    let percent = HEAT_PERCENTS
        .get(usize::from(level).saturating_sub(1))
        .copied()
        .unwrap_or(100);
    match accent {
        Some(color) if true_color => Style::new().fg(color_map::mix(color, background, percent)),
        _ if level <= 2 => palette.accent.dim(),
        _ if level == 4 => palette.accent.bold(),
        _ => palette.accent,
    }
}

/// The first and past-the-last columns shown, `visible` of `columns`, ending on the newest week unless the cursor's column is further left.
fn window(columns: usize, visible: usize, cursor_column: usize) -> (usize, usize) {
    let mut start = columns.saturating_sub(visible);
    if cursor_column < start {
        start = cursor_column;
    }
    (start, start.saturating_add(visible).min(columns))
}

/// The heatmap: month labels over the columns, then a line per weekday with Mon, Wed and Fri named on the left.
fn heat_lines(
    palette: &Palette,
    view: &ActivityView,
    heat: &HeatLayout,
    width: usize,
) -> Vec<Line<'static>> {
    let room = width.saturating_sub(WEEKDAY_WIDTH);
    let cell_width = if heat.columns.saturating_mul(2) <= room {
        2
    } else {
        1
    };
    let visible = heat.columns.min(room.checked_div(cell_width).unwrap_or(0));
    let cursor_column = heat.cells.get(view.cursor).map_or(0, |c| c.column);
    let (start, end) = window(heat.columns, visible, cursor_column);

    let mut months = " ".repeat(WEEKDAY_WIDTH.saturating_add(visible.saturating_mul(cell_width)));
    for (column, name) in &heat.months {
        let Some(offset) = column.checked_sub(start) else {
            continue;
        };
        let at = WEEKDAY_WIDTH.saturating_add(offset.saturating_mul(cell_width));
        let stop = at.saturating_add(name.len());
        if *column < end && stop <= months.len() && months.is_char_boundary(at) {
            months.replace_range(at..stop, name);
        }
    }
    let mut lines = vec![Line::styled(months, palette.muted)];

    let mut grid: Vec<Option<usize>> = vec![None; heat.columns.saturating_mul(DAYS_PER_WEEK)];
    for (i, cell) in heat.cells.iter().enumerate() {
        if let Some(slot) = grid.get_mut(
            cell.column
                .saturating_mul(DAYS_PER_WEEK)
                .saturating_add(cell.row),
        ) {
            *slot = Some(i);
        }
    }
    let elapsed = view.sweep_elapsed();
    let gap = " ".repeat(cell_width.saturating_sub(1));
    for row in 0..DAYS_PER_WEEK {
        let label = WEEKDAY_LABELS
            .iter()
            .find(|(r, _)| *r == row)
            .map_or("", |(_, name)| name);
        let mut spans = vec![Span::styled(
            format!("{label:<WEEKDAY_WIDTH$}"),
            palette.muted,
        )];
        for column in start..end {
            let index = grid
                .get(column.saturating_mul(DAYS_PER_WEEK).saturating_add(row))
                .copied()
                .flatten();
            let Some((index, cell)) = index.and_then(|i| heat.cells.get(i).map(|c| (i, c))) else {
                spans.push(Span::raw(" ".repeat(cell_width)));
                continue;
            };
            let delay = super::SWEEP_STEP.saturating_mul(u32::try_from(column).unwrap_or(u32::MAX));
            let Some(into) = elapsed.checked_sub(delay) else {
                spans.push(Span::raw(" ".repeat(cell_width)));
                continue;
            };
            let mut style = heat_style(palette, cell.level, cell.peak);
            if into
                < super::SWEEP
                    .saturating_mul(BRIGHT_SHARE)
                    .checked_div(100)
                    .unwrap_or_default()
            {
                style = style.bold();
            }
            if view.focus == Focus::Heat && index == view.cursor {
                style = style.reversed();
            }
            let glyph = if cell.peak { PEAK } else { DAY };
            spans.push(Span::styled(glyph, style));
            spans.push(Span::raw(gap.clone()));
        }
        lines.push(Line::from(spans));
    }
    lines
}

/// Totals: a title chip, then four columns, each its count over a bar of its share of the largest, its label uppercase under it.
fn totals_lines(palette: &Palette, bars: &[Bar], width: usize) -> Vec<Line<'static>> {
    let column = width
        .checked_div(bars.len())
        .unwrap_or(width)
        .clamp(1, TOTALS_COLUMN);
    let bar_width = TOTALS_BAR.min(column.saturating_sub(2)).max(1);
    let mut lines = vec![
        Line::from(home_draw::chip("Totals", palette.muted)),
        Line::default(),
    ];
    let full_eighths = TOTALS_ROWS.saturating_mul(8);
    let heights: Vec<usize> = bars
        .iter()
        .map(|b| full_eighths.saturating_mul(usize::try_from(b.pct).unwrap_or(100)) / 100)
        .collect();
    // One headroom row for the tallest column's count.
    for row in 0..=TOTALS_ROWS {
        let from_bottom = TOTALS_ROWS.saturating_add(1).saturating_sub(row);
        let mut spans = Vec::new();
        for (bar, &eighths) in bars.iter().zip(&heights) {
            let filled_rows = eighths.div_ceil(8);
            let text = if from_bottom <= TOTALS_ROWS && eighths >= from_bottom.saturating_mul(8) {
                "█".repeat(bar_width)
            } else if from_bottom <= TOTALS_ROWS && filled_rows == from_bottom {
                let glyph = EIGHTHS.get(eighths % 8).copied().unwrap_or(" ");
                glyph.repeat(bar_width)
            } else if from_bottom == filled_rows.saturating_add(1) && bar.count > 0 {
                count_up::with_separators(bar.count)
            } else {
                String::new()
            };
            let style = if text.starts_with(|c: char| c.is_ascii_digit()) {
                palette.base.bold()
            } else {
                palette.accent
            };
            spans.push(Span::styled(format!("{text:^column$}"), style));
        }
        lines.push(Line::from(spans));
    }
    let labels: Vec<Span<'static>> = bars
        .iter()
        .map(|b| {
            Span::styled(
                format!("{:^column$}", b.label.to_uppercase()),
                palette.muted,
            )
        })
        .collect();
    lines.push(Line::from(labels));
    lines
}

/// A rate section: the Show chips, then its chart or a note, each row's reveal under it.
fn rates_body(
    body: &mut Body,
    shared: &Shared,
    view: &ActivityView,
    section: Section,
    width: usize,
) {
    let palette = &shared.palette;
    if !view.has_rates(section) {
        let line = match section {
            Section::Tags => activity::TAGS_EMPTY,
            _ => activity::ROLES_EMPTY,
        };
        body.line(indented(Line::styled(line, palette.muted)));
        return;
    }
    body.item(
        vec![indented(show_chips(palette, view.rate))],
        view.focus == Focus::Chips(section),
    );
    let bars = view.bars(section);
    if bars.is_empty() {
        let line = match (section, view.loaded()) {
            (Section::Tags, Some(loaded)) => {
                activity::tag_empty_line(view.rate, &loaded.activity.tags)
            }
            _ => activity::NONE_REMOVED,
        };
        body.line(indented(Line::styled(line, palette.muted)));
        return;
    }
    let open = match section {
        Section::Roles => view.open_role.as_deref(),
        Section::Tags => view.open_tag.as_deref(),
        Section::Activity => None,
    };
    let label_of = |bar: &Bar| match section {
        Section::Roles => bar.label.to_uppercase(),
        _ => bar.label.clone(),
    };
    let label_width = bars
        .iter()
        .map(|b| label_of(b).chars().count().saturating_add(2))
        .max()
        .unwrap_or(0)
        .min(width / 2);
    let track = width
        .saturating_sub(label_width)
        .saturating_sub(1)
        .saturating_sub(5)
        .max(1);
    for (i, bar) in bars.iter().enumerate() {
        let selected = open == Some(bar.key.as_str());
        let label = label_of(bar);
        let label_span = if bar.revealable {
            let style = if selected {
                palette.accent.bold().reversed()
            } else {
                palette.accent
            };
            Span::styled(format!(" {label} "), style)
        } else {
            // A role with no tags to open is a grey chip that takes no Enter.
            Span::styled(format!(" {label} "), palette.muted.reversed())
        };
        let row = charts::row(
            label_span,
            label_width,
            bar.pct,
            track,
            palette.accent,
            vec![Span::raw(format!("{}%", bar.count)).bold()],
        );
        let mut lines = vec![indented(row)];
        if selected {
            let reveal = match section {
                Section::Roles => view.role_tags.lines(palette, &shared.caches.oracle_tags),
                _ => otag::def_lines(palette, &shared.caches.oracle_tags, &bar.key),
            };
            lines.extend(reveal.into_iter().map(|l| indented(indented(l))));
        }
        body.item(lines, view.focus == Focus::Bar(section, i));
    }
}

/// The Show row: its label, then the four rates with the picked one lit.
fn show_chips(palette: &Palette, picked: RateKind) -> Line<'static> {
    let mut spans = vec![Span::raw("Show").bold(), Span::raw(" ")];
    for rate in RateKind::ALL {
        let style = if rate == picked {
            palette.accent.bold().reversed()
        } else {
            palette.muted
        };
        spans.push(Span::styled(format!(" {} ", rate.label()), style));
        spans.push(Span::raw(" "));
    }
    Line::from(spans)
}

/// The loading screen in the loaded shape, every piece a ghost: the Activity section open on the heatmap's height, its three tags, a rule and the Totals columns; Card roles and Oracle tags collapsed; the Most skipped header and ten rows.
fn skeleton(palette: &Palette, width: u16) -> Vec<Line<'static>> {
    let width = usize::from(width)
        .saturating_sub(GUTTER.len())
        .saturating_sub(INDENT.len());
    let ghost = |n: usize| Span::styled(GHOST.repeat(n.max(1)), palette.muted);
    let header = |n: usize, open: bool| {
        let mark = if open { "▾ " } else { "▸ " };
        let mut spans = vec![Span::styled(mark, palette.muted), ghost(n)];
        if open {
            spans.push(Span::raw("  "));
            spans.push(ghost(1));
        }
        Line::from(spans)
    };
    let mut body = Body::default();
    body.line(header(8, /* open */ true));
    for _ in 0..=DAYS_PER_WEEK {
        body.line(Line::default());
    }
    body.line(Line::default());
    body.line(indented(Line::from(vec![
        ghost(10),
        Span::raw(" "),
        ghost(12),
        Span::raw(" "),
        ghost(11),
    ])));
    body.line(indented(Line::styled("─".repeat(width), palette.muted)));
    body.line(indented(Line::from(ghost(8))));
    body.line(Line::default());
    let column = width
        .checked_div(GHOST_HEIGHTS.len())
        .unwrap_or(width)
        .clamp(1, TOTALS_COLUMN);
    let bar_width = TOTALS_BAR.min(column.saturating_sub(2)).max(1);
    for row in 0..=TOTALS_ROWS {
        let from_bottom = TOTALS_ROWS.saturating_add(1).saturating_sub(row);
        let spans: Vec<Span<'static>> = GHOST_HEIGHTS
            .iter()
            .map(|pct| {
                let rows = TOTALS_ROWS.saturating_mul(*pct).div_ceil(100);
                let text = if from_bottom <= rows {
                    GHOST.repeat(bar_width)
                } else {
                    String::new()
                };
                Span::styled(format!("{text:^column$}"), palette.muted)
            })
            .collect();
        body.line(indented(Line::from(spans)));
    }
    let labels: Vec<Span<'static>> = GHOST_HEIGHTS
        .iter()
        .map(|_| {
            Span::styled(
                format!("{:^column$}", GHOST.repeat(bar_width.saturating_add(2))),
                palette.muted,
            )
        })
        .collect();
    body.line(indented(Line::from(labels)));
    body.line(header(10, /* open */ false));
    body.line(header(11, /* open */ false));
    body.line(Line::default());
    body.line(Line::from(vec![ghost(12), Span::raw("  "), ghost(1)]));
    for i in 0..GHOST_PASSES {
        let share = [62, 48, 71, 55].get(i % 4).copied().unwrap_or(60);
        body.line(Line::from(vec![
            ghost(2),
            Span::raw(" "),
            ghost(width.saturating_mul(share) / 100),
        ]));
    }
    body.lines
}
