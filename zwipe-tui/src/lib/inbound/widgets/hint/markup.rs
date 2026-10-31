//! The little markup hint copy is written in, so the tables read like the app's.
//!
//! A block starting with `• ` is a bullet; any other block is a line of its own. Inside a block, `[]` and `{}` read as written, `[Label]` is a key chip in `--palette-2`, `[Label|role]` a key chip in that role's color, and `{word|role}` a bold word in that role's color. The roles are `p1` to `p6`, `primary`, `secondary`, `tertiary`, `success`, `error` and `warning`.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::domain::theme::Palette;

/// Starts each bullet.
const BULLET: &str = "• ";

/// The color a key chip or a colored word takes.
fn role(palette: &Palette, name: &str) -> Style {
    let swatch = |i: usize| palette.swatches.get(i).copied().unwrap_or(palette.accent);
    match name {
        "p1" => swatch(0),
        "p3" => swatch(2),
        "p4" => swatch(3),
        "p5" => swatch(4),
        "p6" => swatch(5),
        "primary" => palette.accent,
        "secondary" => palette.secondary,
        "tertiary" => palette.tertiary,
        "success" => palette.add,
        "error" => palette.error,
        "warning" => palette.warning,
        _ => swatch(1),
    }
}

/// Splits `token` into its label and role, the role empty when it has none.
fn split(token: &str) -> (&str, &str) {
    token.split_once('|').unwrap_or((token, ""))
}

/// One block of hint copy as a line of spans.
pub(crate) fn line(palette: &Palette, block: &str) -> Line<'static> {
    let (bullet, text) = block
        .strip_prefix(BULLET)
        .map_or((false, block), |rest| (true, rest));
    let mut spans: Vec<Span<'static>> = Vec::new();
    if bullet {
        spans.push(Span::styled(BULLET, palette.muted));
    }
    let mut rest = text;
    while let Some(start) = rest.find(['[', '{']) {
        let (before, from) = rest.split_at(start);
        if !before.is_empty() {
            spans.push(Span::raw(before.to_owned()));
        }
        let close = if from.starts_with('[') { ']' } else { '}' };
        let Some(end) = from.find(close) else {
            spans.push(Span::raw(from.to_owned()));
            rest = "";
            break;
        };
        let inner = from.get(1..end).unwrap_or_default();
        if inner.is_empty() {
            // `[]` and `{}` are literal, as in the password symbols.
            spans.push(Span::raw(from.get(..1).unwrap_or_default().to_owned()));
            rest = from.get(1..).unwrap_or_default();
            continue;
        }
        let (label, name) = split(inner);
        let style = role(palette, name);
        if close == ']' {
            spans.push(Span::styled(format!(" {label} "), style.reversed().bold()));
        } else {
            spans.push(Span::styled(label.to_owned(), style.bold()));
        }
        rest = from.get(end.saturating_add(1)..).unwrap_or_default();
    }
    if !rest.is_empty() {
        spans.push(Span::raw(rest.to_owned()));
    }
    Line::from(spans)
}

/// The text a block reads as, without its styling: chips keep a space on each side, colored words read plain.
#[cfg(test)]
pub(crate) fn plain(block: &str) -> String {
    line(&Palette::terminal(), block).to_string()
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;

    use super::*;

    #[test]
    fn bullets_chips_and_colored_words_take_their_styles() {
        let palette = Palette::for_account("gruvbox", true);
        let line = line(
            &palette,
            "• [Decks] holds your {decks|warning} and [More|error]",
        );
        assert_eq!(line.to_string(), "•  Decks  holds your decks and  More ");
        let chip = &line.spans[1];
        assert_eq!(chip.style.fg, palette.swatches[1].fg);
        assert!(chip.style.add_modifier.contains(Modifier::REVERSED));
        assert_eq!(line.spans[3].style.fg, palette.warning.fg);
        assert_eq!(line.spans[5].style.fg, palette.error.fg);
    }

    #[test]
    fn a_plain_line_and_an_unclosed_bracket_read_as_written() {
        assert_eq!(plain("Line: no bullet"), "Line: no bullet");
        assert_eq!(plain("half [open"), "half [open");
        assert_eq!(plain("[-] and [+]"), " -  and  + ");
        assert_eq!(
            plain("from ~!@#$%^&*()_+=[]{}\\/?|:;<>,."),
            "from ~!@#$%^&*()_+=[]{}\\/?|:;<>,."
        );
    }
}
