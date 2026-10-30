//! The app's bottom row of buttons, drawn as key hints: each key in bold, its action muted.

use ratatui::style::Stylize;
use ratatui::text::{Line, Span};

use crate::domain::theme::Palette;

/// Key hints for `keys`, each a key and what it does.
pub(crate) fn line(palette: &Palette, keys: &[(&str, &str)]) -> Line<'static> {
    let spans: Vec<Span<'static>> = keys
        .iter()
        .flat_map(|&(key, action)| {
            [
                key.to_owned().bold(),
                Span::styled(format!(" {action}  "), palette.muted),
            ]
        })
        .collect();
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;

    use super::*;

    #[test]
    fn each_key_is_bold_and_its_action_muted() {
        let palette = Palette::terminal();
        let line = line(&palette, &[("Enter", "open"), ("q", "quit")]);
        assert_eq!(line.to_string(), "Enter open  q quit  ");
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[1].style, palette.muted);
    }
}
