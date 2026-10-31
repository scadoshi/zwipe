//! The app's buttons: a row of rounded boxes the focus moves across, or one-line pills where a box has no room.
//!
//! A plain button has the border color around text in the text color, as the app's `.btn-action`. A primary one, the screen's main action, takes the accent for both; a danger one, like `Log out` and `Delete`, the error color. The focused button fills with its color (the accent for a plain one) under a bold label. A disabled one is muted.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::measure::{self, BORDERS};
use super::panel;
use crate::domain::theme::Palette;

/// Rows a boxed button takes: its border and its label.
pub(crate) const HEIGHT: u16 = 3;

/// Cells between two buttons in a row.
const GAP: u16 = 1;

/// What a button does, which sets its color.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    #[default]
    Plain,
    /// The screen's main action, in the accent.
    Primary,
    /// A destructive action, in the error color.
    Danger,
}

/// One button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Button<'label> {
    pub(crate) label: &'label str,
    pub(crate) kind: Kind,
    /// Muted, and not to be pressed.
    pub(crate) disabled: bool,
}

impl<'label> Button<'label> {
    pub(crate) fn new(label: &'label str) -> Self {
        Self {
            label,
            kind: Kind::Plain,
            disabled: false,
        }
    }

    pub(crate) fn primary(self) -> Self {
        Self {
            kind: Kind::Primary,
            ..self
        }
    }

    pub(crate) fn danger(self) -> Self {
        Self {
            kind: Kind::Danger,
            ..self
        }
    }

    pub(crate) fn disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }

    /// The button's own color: muted while disabled, else by its kind, with the text color for a plain one.
    fn color(&self, palette: &Palette) -> Style {
        if self.disabled {
            return palette.muted;
        }
        match self.kind {
            Kind::Plain => Style::new().fg(palette.base.fg.unwrap_or_default()),
            Kind::Primary => palette.accent,
            Kind::Danger => palette.error,
        }
    }

    /// The color the button fills with while focused: its own, or the accent for a plain one.
    fn fill(&self, palette: &Palette) -> Style {
        match self.kind {
            Kind::Plain if !self.disabled => palette.accent,
            _ => self.color(palette),
        }
    }

    /// The label padded a cell on each side, in the button's color, or filled with it while focused.
    fn label_span(&self, palette: &Palette, focused: bool) -> Span<'static> {
        let style = if focused {
            self.fill(palette).bold().reversed()
        } else if self.kind == Kind::Primary && !self.disabled {
            self.color(palette).bold()
        } else {
            self.color(palette)
        };
        Span::styled(format!(" {} ", self.label), style)
    }

    /// Cells the button takes boxed: its label, a cell of padding and a border each side.
    fn box_width(&self) -> u16 {
        measure::width_of(self.label)
            .saturating_add(2)
            .saturating_add(BORDERS)
    }
}

/// Cells a row of boxed buttons takes, gaps included.
pub(crate) fn row_width(row: &[Button<'_>]) -> u16 {
    let gaps = GAP.saturating_mul(u16::try_from(row.len().saturating_sub(1)).unwrap_or(u16::MAX));
    row.iter()
        .fold(gaps, |sum, button| sum.saturating_add(button.box_width()))
}

/// Rows to give a row of buttons `width` cells wide with `room` rows to spare: [`HEIGHT`] when the boxes fit both, else one for pills.
pub(crate) fn height(row: &[Button<'_>], width: u16, room: u16) -> u16 {
    if row_width(row) <= width && room >= HEIGHT {
        HEIGHT
    } else {
        1
    }
}

/// One button as a pill on a single line: the label padded a cell each side on the skeleton shade, or filled with its color while focused. For buttons inside a line of text, such as a list row's `Change`.
pub(crate) fn pill(palette: &Palette, button: &Button<'_>, focused: bool) -> Span<'static> {
    let span = button.label_span(palette, focused);
    match (focused, palette.skeleton.fg) {
        (false, Some(shade)) => {
            let style = span.style.bg(shade);
            span.style(style)
        }
        _ => span,
    }
}

/// A row of pills, a space between each.
pub(crate) fn pills(
    palette: &Palette,
    row: &[Button<'_>],
    focused: Option<usize>,
) -> Line<'static> {
    let mut spans = Vec::with_capacity(row.len().saturating_mul(2));
    for (i, button) in row.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(pill(palette, button, focused == Some(i)));
    }
    Line::from(spans)
}

/// Draws `row` centered in `area`, the button at `focused` filled: as boxes when `area` is [`HEIGHT`] rows tall and wide enough for them, else as pills on its middle row.
pub(crate) fn draw_row(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    row: &[Button<'_>],
    focused: Option<usize>,
) {
    if area.height >= HEIGHT && row_width(row) <= area.width {
        let boxes = Layout::horizontal(row.iter().map(|b| Constraint::Length(b.box_width())))
            .flex(Flex::Center)
            .spacing(GAP)
            .split(Rect {
                height: HEIGHT,
                y: area
                    .y
                    .saturating_add(area.height.saturating_sub(HEIGHT) / 2),
                ..area
            });
        for (i, (button, spot)) in row.iter().zip(boxes.iter()).enumerate() {
            draw_box(f, palette, *spot, button, focused == Some(i));
        }
    } else {
        let middle = Rect {
            y: area.y.saturating_add(area.height.saturating_sub(1) / 2),
            height: area.height.min(1),
            ..area
        };
        f.render_widget(pills(palette, row, focused).centered(), middle);
    }
}

/// One boxed button: its border in its color (the border color for a plain one at rest), and its label inside, filled while focused.
fn draw_box(f: &mut Frame<'_>, palette: &Palette, area: Rect, button: &Button<'_>, focused: bool) {
    let border = match (focused, button.kind) {
        (true, _) => button.fill(palette),
        (false, Kind::Plain) => palette.border,
        (false, _) => button.color(palette),
    };
    let block = panel::bordered().border_style(border);
    f.render_widget(
        Paragraph::new(Line::from(button.label_span(palette, focused)).centered()).block(block),
        area,
    );
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::Modifier;

    use super::*;

    fn row() -> [Button<'static>; 3] {
        [
            Button::new("Log in").primary(),
            Button::new("Create profile"),
            Button::new("Log out").danger(),
        ]
    }

    fn drawn(
        palette: &Palette,
        width: u16,
        height: u16,
        focused: Option<usize>,
    ) -> Terminal<TestBackend> {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|f| draw_row(f, palette, f.area(), &row(), focused))
            .unwrap();
        terminal
    }

    fn text(terminal: &Terminal<TestBackend>) -> Vec<String> {
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer.cell((x, y)).unwrap().symbol())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn a_row_that_fits_draws_rounded_boxes_centered() {
        let palette = Palette::for_account("gruvbox", true);
        let rows = text(&drawn(&palette, 50, 3, None));
        let trimmed: Vec<&str> = rows.iter().map(|r| r.trim()).collect();
        assert_eq!(
            trimmed,
            [
                "╭────────╮ ╭────────────────╮ ╭─────────╮",
                "│ Log in │ │ Create profile │ │ Log out │",
                "╰────────╯ ╰────────────────╯ ╰─────────╯",
            ]
        );
        let left = rows[0].chars().take_while(|c| *c == ' ').count();
        let right = rows[0].chars().rev().take_while(|c| *c == ' ').count();
        assert!(left.abs_diff(right) <= 1, "centered: {left} and {right}");
    }

    #[test]
    fn the_focused_box_fills_with_its_color_under_a_bold_label() {
        let palette = Palette::for_account("gruvbox", true);
        let terminal = drawn(&palette, 50, 3, Some(1));
        let buffer = terminal.backend().buffer();
        let rows = text(&terminal);
        let at = rows[1].find("Create").unwrap();
        let x = u16::try_from(rows[1][..at].chars().count()).unwrap();
        let label = buffer.cell((x, 1)).unwrap();
        assert!(label.modifier.contains(Modifier::REVERSED | Modifier::BOLD));
        assert_eq!(label.fg, palette.accent.fg.unwrap());
        let pad = buffer.cell((x - 1, 1)).unwrap();
        assert!(
            pad.modifier.contains(Modifier::REVERSED),
            "the padding fills too"
        );
        let border = buffer.cell((x - 2, 1)).unwrap();
        assert_eq!(
            border.fg,
            palette.accent.fg.unwrap(),
            "the border takes the accent"
        );
    }

    #[test]
    fn primary_and_danger_take_their_colors_at_rest() {
        let palette = Palette::for_account("gruvbox", true);
        let terminal = drawn(&palette, 50, 3, None);
        let buffer = terminal.backend().buffer();
        let rows = text(&terminal);
        let cell = |word: &str| {
            let at = rows[1].find(word).unwrap();
            let x = u16::try_from(rows[1][..at].chars().count()).unwrap();
            (
                buffer.cell((x, 1)).unwrap().clone(),
                buffer.cell((x - 2, 1)).unwrap().clone(),
            )
        };
        let (log_in, log_in_border) = cell("Log in");
        assert_eq!(log_in.fg, palette.accent.fg.unwrap());
        assert!(log_in.modifier.contains(Modifier::BOLD));
        assert_eq!(log_in_border.fg, palette.accent.fg.unwrap());
        let (create, create_border) = cell("Create");
        assert_eq!(create.fg, palette.base.fg.unwrap());
        assert!(!create.modifier.contains(Modifier::REVERSED));
        assert_eq!(create_border.fg, palette.border.fg.unwrap());
        let (log_out, log_out_border) = cell("Log out");
        assert_eq!(log_out.fg, palette.error.fg.unwrap());
        assert_eq!(log_out_border.fg, palette.error.fg.unwrap());
    }

    #[test]
    fn a_row_with_no_room_for_boxes_falls_back_to_pills() {
        let palette = Palette::for_account("gruvbox", true);
        assert_eq!(height(&row(), 80, 3), HEIGHT);
        assert_eq!(height(&row(), 80, 2), 1, "too short");
        assert_eq!(height(&row(), 30, 3), 1, "too narrow");
        let rows = text(&drawn(&palette, 40, 1, Some(0)));
        assert_eq!(rows[0].trim(), "Log in   Create profile   Log out");
    }

    #[test]
    fn a_pill_rests_on_the_skeleton_shade_and_a_disabled_one_mutes() {
        let palette = Palette::for_account("gruvbox", true);
        let rest = pill(&palette, &Button::new("Change"), false);
        assert_eq!(rest.content, " Change ");
        assert_eq!(rest.style.bg, palette.skeleton.fg);
        let focused = pill(&palette, &Button::new("Change"), true);
        assert!(focused.style.add_modifier.contains(Modifier::REVERSED));
        assert_eq!(focused.style.fg, palette.accent.fg);
        let disabled = pill(&palette, &Button::new("Resend").disabled(true), false);
        assert_eq!(disabled.style.fg, palette.muted.fg);
    }
}
