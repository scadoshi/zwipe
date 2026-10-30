//! Bar charts drawn from zwipe-core's deck metrics: vertical columns, and horizontal rows of filled blocks.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, Block};

use crate::domain::theme::Palette;

/// The block a horizontal bar is drawn with.
const FILL: char = '█';

/// One column of a vertical chart.
#[derive(Debug)]
pub(super) struct Column {
    pub(super) label: String,
    pub(super) count: usize,
    pub(super) style: Style,
}

/// A titled vertical bar chart, each column labeled below and counted above.
pub(super) fn columns(
    f: &mut Frame<'_>,
    palette: &Palette,
    title: &str,
    columns: &[Column],
    area: Rect,
) {
    let count = u16::try_from(columns.len()).unwrap_or(u16::MAX).max(1);
    let inner_width = area.width.saturating_sub(2);
    let width = (inner_width / count).saturating_sub(1).clamp(1, 7);
    let bars: Vec<Bar<'_>> = columns
        .iter()
        .map(|c| {
            let value = u64::try_from(c.count).unwrap_or(u64::MAX);
            Bar::with_label(c.label.clone(), value)
                .style(c.style)
                .text_value(if c.count > 0 {
                    c.count.to_string()
                } else {
                    String::new()
                })
        })
        .collect();
    let chart = BarChart::vertical(bars)
        .block(
            Block::bordered()
                .title(format!(" {title} "))
                .border_style(palette.border),
        )
        .bar_width(width)
        .bar_gap(1)
        .label_style(palette.muted)
        .value_style(palette.base.reversed());
    f.render_widget(chart, area);
}

/// A row of a horizontal chart: `label`, a bar filled to `pct` percent of `width`, then `tail`.
pub(super) fn row(
    label: Span<'static>,
    label_width: usize,
    pct: u32,
    width: usize,
    bar: Style,
    tail: Vec<Span<'static>>,
) -> Line<'static> {
    let filled = width.saturating_mul(usize::try_from(pct.min(100)).unwrap_or_default()) / 100;
    let label_text = format!("{:<label_width$} ", label.content);
    let mut spans = vec![
        Span::styled(label_text, label.style),
        Span::styled(FILL.to_string().repeat(filled), bar),
        Span::raw(" ".repeat(width.saturating_sub(filled).saturating_add(1))),
    ];
    spans.extend(tail);
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_fills_its_share_and_pads_to_the_same_width() {
        let full = row(
            Span::raw("ramp"),
            6,
            100,
            10,
            Style::new(),
            vec![Span::raw("4")],
        );
        let half = row(
            Span::raw("draw"),
            6,
            50,
            10,
            Style::new(),
            vec![Span::raw("2")],
        );
        let none = row(
            Span::raw("wipe"),
            6,
            0,
            10,
            Style::new(),
            vec![Span::raw("0")],
        );
        assert_eq!(full.to_string(), "ramp   ██████████ 4");
        assert_eq!(half.to_string(), "draw   █████      2");
        assert_eq!(none.to_string(), "wipe              0");
    }
}
