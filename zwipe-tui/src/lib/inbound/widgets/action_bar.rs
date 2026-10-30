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
