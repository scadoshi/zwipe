//! The ZWIPE logo decoding in, as on the splash and the sign-in screens' hero, for any screen to draw.
//!
//! Driven by the time since its entrance started; a screen showing it reports itself animating while [`animating`] holds.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;
use zwipe_core::domain::logo;

use super::measure;
use crate::domain::decode;

/// The art that fits `width` by `height`: the full mark, the compact Z, or none.
pub(crate) fn fitting(width: u16, height: u16) -> Option<&'static str> {
    [logo::ZWIPE, logo::Z]
        .into_iter()
        .find(|art| size(art).0 <= width && size(art).1 <= height)
}

/// `art`'s width and height in cells.
pub(crate) fn size(art: &str) -> (u16, u16) {
    (
        art.lines().map(measure::width_of).max().unwrap_or(0),
        measure::height_of(art.lines().count()),
    )
}

/// Whether the decode is still running `elapsed` into it.
pub(crate) fn animating(elapsed: Duration) -> bool {
    elapsed < decode::DURATION
}

/// Draws `art` centered at the top of `area` as it looks `elapsed` into its decode, or settled when `elapsed` is `None`.
pub(crate) fn draw(
    f: &mut Frame<'_>,
    area: Rect,
    art: &str,
    style: Style,
    elapsed: Option<Duration>,
) {
    let frame = elapsed.map_or_else(|| art.to_owned(), |e| decode::at(art, e));
    let (width, height) = size(art);
    let lines: Vec<Line<'static>> = frame.lines().map(|l| Line::from(l.to_owned())).collect();
    let spot = area.centered_horizontally(Constraint::Length(width));
    let spot = Rect {
        height: height.min(spot.height),
        ..spot
    };
    f.render_widget(Paragraph::new(Text::from(lines).style(style)), spot);
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn the_logo_picks_what_fits_and_decodes_by_elapsed_time() {
        assert_eq!(fitting(80, 20), Some(logo::ZWIPE));
        assert_eq!(fitting(20, 8), Some(logo::Z));
        assert_eq!(fitting(5, 2), None);
        assert!(animating(Duration::ZERO) && !animating(decode::DURATION));
        let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap();
        terminal
            .draw(|f| {
                draw(
                    f,
                    f.area(),
                    logo::ZWIPE,
                    Style::new(),
                    Some(decode::DURATION),
                );
            })
            .unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(text.contains('▉'));
        let mut tiny = Terminal::new(TestBackend::new(2, 1)).unwrap();
        tiny.draw(|f| draw(f, f.area(), logo::ZWIPE, Style::new(), Some(Duration::ZERO)))
            .unwrap();
    }
}
