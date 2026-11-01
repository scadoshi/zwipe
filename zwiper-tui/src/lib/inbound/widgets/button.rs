//! The app's buttons: a label in brackets on one line, `[ Log in ]`, at every size, alone inside a line of text or in a centered row the focus moves across.
//!
//! At rest the brackets take the border color and the label the button's color: the text color for a plain one, the error color for a danger one like `Log out` and `Delete`, muted while disabled. A focused button draws its brackets and a bold label in its focus color (the accent for a plain one, its own color for a danger one). No button fills its background, so focus is the only highlight and a filled block is always a tag.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};

use super::measure;
use crate::domain::theme::Palette;

/// Cells between two buttons in a row.
const GAP: u16 = 2;

/// Cells a button takes beyond its label: a bracket and a space each side.
const FRAME: u16 = 4;

/// What a button does, which sets its color.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    #[default]
    Plain,
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

    pub(crate) fn danger(self) -> Self {
        Self {
            kind: Kind::Danger,
            ..self
        }
    }

    pub(crate) fn disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }

    /// The label's color at rest: muted while disabled, else the text color for a plain one and the error color for a danger one.
    fn color(&self, palette: &Palette) -> Style {
        if self.disabled {
            return palette.muted;
        }
        match self.kind {
            Kind::Plain => Style::new().fg(palette.base.fg.unwrap_or_default()),
            Kind::Danger => palette.error,
        }
    }

    /// The color the brackets and label take while focused: the accent for a plain one, else its own color.
    fn focus(&self, palette: &Palette) -> Style {
        match self.kind {
            Kind::Plain if !self.disabled => palette.accent,
            _ => self.color(palette),
        }
    }

    /// Cells the button takes: its label, a space and a bracket each side.
    fn width(&self) -> u16 {
        measure::width_of(self.label).saturating_add(FRAME)
    }
}

/// One button as its spans, for a button inside a line of text such as a list row's `Change`: the opening bracket, the label padded a space each side, and the closing bracket.
pub(crate) fn spans(palette: &Palette, button: &Button<'_>, focused: bool) -> Vec<Span<'static>> {
    let (bracket, label) = if focused {
        let style = button.focus(palette).bold();
        (style, style)
    } else {
        (palette.border, button.color(palette))
    };
    vec![
        Span::styled("[", bracket),
        Span::styled(format!(" {} ", button.label), label),
        Span::styled("]", bracket),
    ]
}

/// Cells a row of buttons takes, gaps included.
pub(crate) fn row_width(row: &[Button<'_>]) -> u16 {
    let gaps = GAP.saturating_mul(u16::try_from(row.len().saturating_sub(1)).unwrap_or(u16::MAX));
    row.iter()
        .fold(gaps, |sum, button| sum.saturating_add(button.width()))
}

/// A row of buttons on one line, [`GAP`] cells between each, the one at `focused` highlighted.
pub(crate) fn line(palette: &Palette, row: &[Button<'_>], focused: Option<usize>) -> Line<'static> {
    let gap = " ".repeat(usize::from(GAP));
    let mut out = Vec::with_capacity(row.len().saturating_mul(4));
    for (i, button) in row.iter().enumerate() {
        if i > 0 {
            out.push(Span::raw(gap.clone()));
        }
        out.extend(spans(palette, button, focused == Some(i)));
    }
    Line::from(out)
}

/// Draws `row` centered on the middle line of `area`, the button at `focused` highlighted.
pub(crate) fn draw_row(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    row: &[Button<'_>],
    focused: Option<usize>,
) {
    let middle = Rect {
        y: area.y.saturating_add(area.height.saturating_sub(1) / 2),
        height: area.height.min(1),
        ..area
    };
    f.render_widget(line(palette, row, focused).centered(), middle);
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::Modifier;

    use super::*;
    use crate::test_support::BUTTON_SIZES;

    fn row() -> [Button<'static>; 3] {
        [
            Button::new("Log in"),
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

    /// The column where `word` starts on row `y` of `rows`.
    fn column(rows: &[String], y: usize, word: &str) -> u16 {
        let at = rows[y].find(word).unwrap();
        u16::try_from(rows[y][..at].chars().count()).unwrap()
    }

    #[test]
    fn a_row_draws_bracketed_labels_on_one_line_centered_at_every_size() {
        let palette = Palette::for_account("gruvbox", true);
        let want = "[ Log in ]  [ Create profile ]  [ Log out ]";
        for (width, height) in BUTTON_SIZES {
            for focused in [None, Some(1)] {
                let rows = text(&drawn(&palette, width, height, focused));
                let middle = usize::from((height - 1) / 2);
                for (y, row) in rows.iter().enumerate() {
                    if y == middle {
                        assert_eq!(row.trim(), want, "at {width}x{height}, focus {focused:?}");
                    } else {
                        assert_eq!(row.trim(), "", "one line at {width}x{height}");
                    }
                }
                let left = rows[middle].chars().take_while(|c| *c == ' ').count();
                let right = rows[middle].chars().rev().take_while(|c| *c == ' ').count();
                assert!(left.abs_diff(right) <= 1, "centered: {left} and {right}");
            }
        }
    }

    #[test]
    fn a_focused_button_is_bold_accent_brackets_and_label_with_no_fill() {
        let palette = Palette::for_account("gruvbox", true);
        for (width, height) in BUTTON_SIZES {
            let rest = drawn(&palette, width, height, None);
            let terminal = drawn(&palette, width, height, Some(1));
            let buffer = terminal.backend().buffer();
            let rows = text(&terminal);
            let y = (height - 1) / 2;
            let x = column(&rows, usize::from(y), "[ Create");
            let end = x + u16::try_from("[ Create profile ]".len()).unwrap();
            for c in x..end {
                let cell = buffer.cell((c, y)).unwrap();
                assert!(!cell.modifier.contains(Modifier::REVERSED), "cell {c}");
                assert_eq!(
                    cell.bg,
                    rest.backend().buffer().cell((c, y)).unwrap().bg,
                    "no background change at {c}"
                );
                if cell.symbol() != " " {
                    assert!(cell.modifier.contains(Modifier::BOLD), "bold at {c}");
                    assert_eq!(cell.fg, palette.accent.fg.unwrap(), "accent at {c}");
                }
            }
        }
    }

    #[test]
    fn at_rest_the_brackets_take_the_border_color_and_the_label_its_own() {
        let palette = Palette::for_account("gruvbox", true);
        let terminal = drawn(&palette, 60, 1, None);
        let buffer = terminal.backend().buffer();
        let rows = text(&terminal);
        let cells = |word: &str| {
            let x = column(&rows, 0, word);
            let label = buffer.cell((x + 2, 0)).unwrap().clone();
            (buffer.cell((x, 0)).unwrap().clone(), label)
        };
        let (bracket, label) = cells("[ Create");
        assert_eq!(bracket.fg, palette.border.fg.unwrap());
        assert_eq!(label.fg, palette.base.fg.unwrap());
        assert!(!label.modifier.contains(Modifier::BOLD));
        let (bracket, label) = cells("[ Log out");
        assert_eq!(bracket.fg, palette.border.fg.unwrap());
        assert_eq!(label.fg, palette.error.fg.unwrap(), "danger");
        for cell in buffer.content() {
            assert!(!cell.modifier.contains(Modifier::REVERSED));
            assert_eq!(cell.bg, buffer.cell((0, 0)).unwrap().bg, "no fill");
        }
    }

    #[test]
    fn a_focused_danger_button_takes_the_error_color() {
        let palette = Palette::for_account("gruvbox", true);
        let terminal = drawn(&palette, 60, 1, Some(2));
        let buffer = terminal.backend().buffer();
        let x = column(&text(&terminal), 0, "[ Log out");
        for c in [x, x + 2, x + 10] {
            let cell = buffer.cell((c, 0)).unwrap();
            assert_eq!(cell.fg, palette.error.fg.unwrap(), "cell {c}");
            assert!(cell.modifier.contains(Modifier::BOLD), "cell {c}");
        }
    }

    #[test]
    fn a_single_button_spans_its_brackets_and_a_disabled_one_mutes() {
        let palette = Palette::for_account("gruvbox", true);
        let rest = spans(&palette, &Button::new("Change"), false);
        let shown: String = rest.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(shown, "[ Change ]");
        assert!(rest.iter().all(|s| s.style.bg.is_none()), "no fill");
        let disabled = spans(&palette, &Button::new("Resend").disabled(true), false);
        assert_eq!(disabled[1].style.fg, palette.muted.fg);
        assert_eq!(disabled[0].style.fg, palette.border.fg);
        assert_eq!(row_width(&row()), 43);
    }
}
