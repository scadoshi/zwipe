//! One chip drawn on its own: the screens build their chip rows, label chips and chips that are only shown from it.

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

/// Chips laid out in lines no wider than `width`, each line starting with `indent`.
pub(crate) fn wrap(indent: &str, spans: Vec<Span<'static>>, width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut current: Vec<Span<'static>> = vec![Span::raw(indent.to_owned())];
    let mut used = indent.chars().count();
    let start = used;
    for span in spans {
        let len = span.width().saturating_add(1);
        if used > start && used.saturating_add(len) > width {
            lines.push(Line::from(std::mem::replace(
                &mut current,
                vec![Span::raw(indent.to_owned())],
            )));
            used = start;
        }
        current.push(span);
        current.push(Span::raw(" "));
        used = used.saturating_add(len);
    }
    if current.len() > 1 {
        lines.push(Line::from(current));
    }
    lines
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

    #[test]
    fn wrapping_starts_a_new_line_when_the_next_chip_will_not_fit() {
        let chips = ["Aggro", "Tokens", "Draw"]
            .iter()
            .map(|l| pill(l, Style::new(), PillState::default()))
            .collect();
        let lines: Vec<String> = wrap("  ", chips, 20)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(lines, ["   Aggro   Tokens  ", "   Draw  "]);
        assert_eq!(wrap("  ", Vec::new(), 20), Vec::<Line<'static>>::new());
    }
}
