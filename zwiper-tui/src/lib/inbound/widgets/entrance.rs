//! A screen's entrance, the app's `content-enter`: 0.3 s in, drawn faint for the first half since a terminal has no opacity.
//!
//! The app runs it on every new screen by itself; a screen can also run it over part of itself with [`draw`].

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

/// How long an entrance takes.
pub(crate) const DURATION: Duration = Duration::from_millis(300);

/// The first half of an entrance, drawn faint.
const FAINT: Duration = Duration::from_millis(150);

/// Whether an entrance `elapsed` in is still running.
pub(crate) fn animating(elapsed: Duration) -> bool {
    elapsed < DURATION
}

/// Draws `area` faint while the entrance is in its first half, `elapsed` in.
pub(crate) fn draw(f: &mut Frame<'_>, area: Rect, elapsed: Duration) {
    if elapsed < FAINT {
        f.buffer_mut()
            .set_style(area, Style::new().add_modifier(Modifier::DIM));
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn the_first_half_is_faint() {
        let dim = |elapsed| {
            let mut terminal = Terminal::new(TestBackend::new(4, 1)).unwrap();
            terminal.draw(|f| draw(f, f.area(), elapsed)).unwrap();
            terminal.backend().buffer().content()[0]
                .modifier
                .contains(Modifier::DIM)
        };
        assert!(dim(Duration::ZERO));
        assert!(!dim(DURATION / 2));
        assert!(animating(Duration::from_millis(299)) && !animating(DURATION));
    }
}
