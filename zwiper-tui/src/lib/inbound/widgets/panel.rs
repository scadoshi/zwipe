//! The frame every box in the app draws with, the label on its top border, a centered dialog, and the full-bleed rules that divide a box.
//!
//! Every box, on the grid or over other content, is `bordered`: square corners and the theme's background filled to its outer edge, border cells included, so it sits on the grid or over a screen as one solid rectangle. A terminal cell holds one background, so a rounded corner glyph could never fill cleanly to the box's corner.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Padding};

use super::measure::BORDERS;
use crate::domain::theme::Palette;

/// The frame of every box: a border on every side with square corners, filled with the theme's background through its border cells. Its text draws in the theme's text color unless a span says otherwise. A box drawn over other content clears its area first with a `Clear`.
pub(crate) fn bordered(palette: &Palette) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Plain)
        .style(palette.base)
}

/// The style of every label on a box's top border: panel, section, sheet and chart titles and text field labels alike, in the third accent as the app's form labels.
pub(crate) fn label_style(palette: &Palette) -> Style {
    palette.tertiary
}

/// `text` as a label for a box's top border, a space each side, in [`label_style`].
pub(crate) fn title_span(palette: &Palette, text: &str) -> Span<'static> {
    Span::styled(format!(" {text} "), label_style(palette))
}

/// `text` as a box's title on its top border, a space each side, in [`label_style`]; the caller aligns it.
pub(crate) fn title(palette: &Palette, text: &str) -> Line<'static> {
    Line::from(title_span(palette, text))
}

/// The area of a centered panel whose inside is `width` by `height`, with room for its border and padding.
fn centered_area(area: Rect, width: u16, height: u16) -> Rect {
    area.centered(
        Constraint::Max(width.saturating_add(BORDERS).saturating_add(BORDERS)),
        Constraint::Length(height.saturating_add(BORDERS)),
    )
}

/// The frame of a centered panel, in the border color and padded a cell each side.
fn panel_block(block: Block<'static>, palette: &Palette) -> Block<'static> {
    block
        .border_style(palette.border)
        .padding(Padding::horizontal(1))
}

/// Clears and frames a centered dialog of the given inner size over whatever is under it, the grid or a screen, and returns the area inside its border and padding.
pub(crate) fn dialog(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    width: u16,
    height: u16,
) -> Rect {
    let outer = centered_area(area, width, height);
    let block = panel_block(bordered(palette), palette);
    let inner = block.inner(outer);
    f.render_widget(Clear, outer);
    f.render_widget(block, outer);
    inner
}

/// Draws a full-bleed rule along `row`, a row inside a panel's one-cell padding, meeting the border on both sides.
pub(crate) fn rule(f: &mut Frame<'_>, palette: &Palette, row: Rect) {
    if row.height == 0 {
        return;
    }
    let left = row.x.saturating_sub(2);
    let width = row
        .width
        .saturating_add(4)
        .min(f.area().right().saturating_sub(left));
    let dashes = "─".repeat(usize::from(width.saturating_sub(2)));
    let area = Rect {
        x: left,
        y: row.y,
        width,
        height: 1,
    };
    f.render_widget(
        Line::from(format!("├{dashes}┤")).style(palette.border),
        area,
    );
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::{Constraint, Layout};

    use super::*;

    #[test]
    fn a_box_title_matches_a_text_field_label() {
        use crate::inbound::widgets::text_input::TextInput;

        let palette = Palette::for_account("gruvbox", true);
        let mut terminal = Terminal::new(TestBackend::new(30, 6)).unwrap();
        terminal
            .draw(|f| {
                let [field, boxed] = Layout::vertical([Constraint::Length(3); 2]).areas(f.area());
                TextInput::default().draw(f, &palette, field, "Email", false);
                f.render_widget(bordered(&palette).title(title(&palette, "Decklist")), boxed);
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let style_of = |text: &str| {
            let (x, y) = crate::test_support::find_all(buffer, text)[0];
            let cell = &buffer[(x, y)];
            (cell.fg, cell.bg, cell.modifier)
        };
        assert_eq!(style_of("Decklist"), style_of("Email"));
    }

    #[test]
    fn a_rule_meets_the_border_on_both_sides() {
        let mut terminal = Terminal::new(TestBackend::new(20, 5)).unwrap();
        terminal
            .draw(|f| {
                let palette = Palette::terminal();
                let inner = dialog(f, &palette, f.area(), 10, 3);
                let [_, middle, _] = Layout::vertical([Constraint::Length(1); 3]).areas(inner);
                rule(f, &palette, middle);
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let row: String = (0..20)
            .map(|x| buffer.cell((x, 2)).unwrap().symbol())
            .collect();
        assert_eq!(row.trim(), "├────────────┤");
    }
}
