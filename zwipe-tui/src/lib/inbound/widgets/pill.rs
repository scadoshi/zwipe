//! One chip drawn on its own, for rows a [`ChipRow`](super::chips::ChipRow) does not fit: mixed chips, label chips, and chips that are only shown.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::domain::theme::Palette;

/// How a chip is drawn.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PillState {
    /// Picked chips are reversed.
    pub(crate) picked: bool,
    /// The focused chip is underlined and bold.
    pub(crate) focused: bool,
}

/// `label` padded by a space each side in `style`, reversed when picked and underlined when focused.
pub(crate) fn pill(label: &str, style: Style, state: PillState) -> Span<'static> {
    let style = if state.picked {
        style.bold().reversed()
    } else {
        style
    };
    let style = if state.focused {
        style.underlined().bold()
    } else {
        style
    };
    Span::styled(format!(" {label} "), style)
}

/// A row's label chip, such as "Group by", in bold.
pub(crate) fn label(palette: &Palette, text: &str) -> Span<'static> {
    Span::styled(format!("{text} "), palette.accent.bold())
}

/// Chips in a line with a space between each.
pub(crate) fn row(spans: Vec<Span<'static>>) -> Line<'static> {
    let mut out = Vec::with_capacity(spans.len().saturating_mul(2));
    for span in spans {
        if !out.is_empty() {
            out.push(Span::raw(" "));
        }
        out.push(span);
    }
    Line::from(out)
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;

    use super::*;

    #[test]
    fn picked_reverses_and_focused_underlines() {
        let palette = Palette::terminal();
        let plain = pill("All", palette.muted, PillState::default());
        assert_eq!(plain.content, " All ");
        assert!(!plain.style.add_modifier.contains(Modifier::REVERSED));
        let picked = pill(
            "All",
            palette.muted,
            PillState {
                picked: true,
                focused: true,
            },
        );
        assert!(picked.style.add_modifier.contains(Modifier::REVERSED));
        assert!(picked.style.add_modifier.contains(Modifier::UNDERLINED));
        let line = row(vec![label(&palette, "Show"), plain]);
        assert_eq!(line.to_string(), "Show   All ");
    }
}
