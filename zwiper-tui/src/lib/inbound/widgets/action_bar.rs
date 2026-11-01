//! The app's bottom row of key hints, each key in bold and its action muted. Buttons live in [`crate::inbound::widgets::button`].

use std::mem;

use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span, Text};

use crate::domain::theme::Palette;

/// Key hints for `keys`, each a key and what it does.
pub(crate) fn line(palette: &Palette, keys: &[(&str, &str)]) -> Line<'static> {
    let styled: Vec<(&str, &str, Style)> = keys
        .iter()
        .map(|&(key, action)| (key, action, palette.muted))
        .collect();
    styled_line(&styled)
}

/// Key hints whose actions each take their own style, like the swipe verbs in the colors of the app's swipe hints.
pub(crate) fn styled_line(keys: &[(&str, &str, Style)]) -> Line<'static> {
    let spans: Vec<Span<'static>> = keys
        .iter()
        .flat_map(|&(key, action, style)| {
            [
                key.to_owned().bold(),
                Span::styled(format!(" {action}  "), style),
            ]
        })
        .collect();
    Line::from(spans)
}

/// Which key opens a screen's hint dialog, for [`with_help`] to name in the bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hint {
    /// `?` opens the hint, so the bar shows `? hint`.
    Question,
    /// `H` opens the hint, on the screens whose `?` explains the focused section instead, so the bar shows `H hint`.
    ShiftH,
    /// The screen has no hint, or none while a field takes the letters.
    Absent,
}

impl Hint {
    /// The key that opens the hint, if there is one.
    fn key(self) -> Option<&'static str> {
        match self {
            Self::Question => Some("?"),
            Self::ShiftH => Some("H"),
            Self::Absent => None,
        }
    }
}

/// `bar` with the F1 Help & feedback hint, every screen's in the same spot: just before the run of Esc and `q` hints nearest the end, or last when the bar has no such run past its first hint.
///
/// The screen's hint key, when it has one, goes just before `F1 help` as `? hint` (or `H hint`). Any hint the bar names itself is dropped first, so it shows once under one label.
pub(crate) fn with_help(palette: &Palette, bar: Line<'static>, hint: Hint) -> Line<'static> {
    let Line {
        style,
        alignment,
        spans,
    } = bar;
    let names_hint = |group: &Vec<Span<'static>>| {
        group
            .get(1)
            .is_some_and(|span| span.content.trim().eq_ignore_ascii_case(HINT_ACTION))
    };
    let mut groups = hints(spans);
    groups.retain(|group| !names_hint(group));
    let is_exit = |group: &Vec<Span<'static>>| {
        group
            .first()
            .is_some_and(|span| matches!(span.content.as_ref(), "Esc" | "q"))
    };
    let run_start = groups.iter().rposition(is_exit).map(|last| {
        let mut start = last;
        while start > 0 && groups.get(start.saturating_sub(1)).is_some_and(is_exit) {
            start = start.saturating_sub(1);
        }
        start
    });
    let at = match run_start {
        Some(start) if start > 0 => start,
        _ => {
            if let Some(last) = groups.last_mut().and_then(|group| group.last_mut())
                && !last.content.ends_with(GAP)
            {
                last.content = format!("{}{GAP}", last.content).into();
            }
            groups.len()
        }
    };
    let help = vec![
        "F1".bold(),
        Span::styled(format!(" help{GAP}"), palette.muted),
    ];
    groups.insert(at, help);
    if let Some(key) = hint.key() {
        groups.insert(
            at,
            vec![
                key.bold(),
                Span::styled(format!(" {HINT_ACTION}{GAP}"), palette.muted),
            ],
        );
    }
    let line = Line::from(groups.into_iter().flatten().collect::<Vec<_>>()).style(style);
    match alignment {
        Some(alignment) => line.alignment(alignment),
        None => line,
    }
}

/// What the bar calls the hint `?` opens.
const HINT_ACTION: &str = "hint";

/// The gap that closes each key hint in a [`line`](fn@line) or [`styled_line`], and where [`wrap`] may break a row.
const GAP: &str = "  ";

/// `bar` broken into rows no wider than `width`, each break falling after a hint's closing gap so a key never parts from its action. A bar that fits stays one row, every row keeps the bar's alignment and style, and no row ends in a gap. A single hint wider than `width` takes a row of its own and is cut there.
pub(crate) fn wrap(bar: Line<'static>, width: u16) -> Text<'static> {
    let Line {
        style,
        alignment,
        spans,
    } = bar;
    let width = usize::from(width);
    let mut rows: Vec<Line<'static>> = Vec::new();
    let mut row: Vec<Span<'static>> = Vec::new();
    let mut row_width = 0_usize;
    for hint in hints(spans) {
        let hint_width: usize = hint.iter().map(Span::width).sum();
        let trailing = hint.last().map_or(0, |span| {
            span.content.chars().rev().take_while(|c| *c == ' ').count()
        });
        let shown = hint_width.saturating_sub(trailing);
        if !row.is_empty() && row_width.saturating_add(shown) > width {
            rows.push(Line::from(mem::take(&mut row)));
            row_width = 0;
        }
        row_width = row_width.saturating_add(hint_width);
        row.extend(hint);
    }
    if !row.is_empty() || rows.is_empty() {
        rows.push(Line::from(row));
    }
    let rows: Vec<Line<'static>> = rows
        .into_iter()
        .map(|mut row| {
            // The last hint's gap would push a centered row off its middle, or past the edge.
            if let Some(last) = row.spans.last_mut() {
                last.content = last.content.trim_end().to_owned().into();
            }
            let row = row.style(style);
            match alignment {
                Some(alignment) => row.alignment(alignment),
                None => row,
            }
        })
        .collect();
    Text::from(rows)
}

/// `bar` with each hint's inner spaces made non-breaking, for keys inside text a [`Paragraph`](ratatui::widgets::Paragraph) wraps, so a wrap falls only in the gap between hints. Reads the same on screen.
pub(crate) fn unbreakable(bar: Line<'static>) -> Line<'static> {
    let glue = |text: &str| text.replace(' ', NBSP);
    let spans = bar
        .spans
        .into_iter()
        .map(|mut span| {
            let content = match span.content.strip_suffix(GAP) {
                Some(hint) => format!("{}{GAP}", glue(hint)),
                None => glue(&span.content),
            };
            span.content = content.into();
            span
        })
        .collect::<Vec<_>>();
    Line { spans, ..bar }
}

/// A space a wrap never breaks at.
const NBSP: &str = "\u{a0}";

/// The rows `bar` takes once [`wrap`] breaks it at `width`.
pub(crate) fn height(bar: &Line<'static>, width: u16) -> u16 {
    u16::try_from(wrap(bar.clone(), width).lines.len()).unwrap_or(u16::MAX)
}

/// The spans of a bar grouped into hints, each group ending with the span that carries a hint's closing [`GAP`].
fn hints(spans: Vec<Span<'static>>) -> Vec<Vec<Span<'static>>> {
    let mut groups = Vec::new();
    let mut group = Vec::new();
    for span in spans {
        let closes = span.content.ends_with(GAP);
        group.push(span);
        if closes {
            groups.push(mem::take(&mut group));
        }
    }
    if !group.is_empty() {
        groups.push(group);
    }
    groups
}

/// The four swipe hints, each verb in its direction's color from the app's swipe hint dialogs: right success, left error, up warning, down tertiary. A swipe with no verb is left out.
pub(crate) fn swipes<'keys>(
    palette: &Palette,
    right: (&'keys str, &'keys str),
    left: (&'keys str, &'keys str),
    up: Option<(&'keys str, &'keys str)>,
    down: (&'keys str, &'keys str),
) -> Vec<(&'keys str, &'keys str, Style)> {
    let mut keys = vec![
        (right.0, right.1, palette.add),
        (left.0, left.1, palette.error),
    ];
    if let Some((key, verb)) = up {
        keys.push((key, verb, palette.warning));
    }
    keys.push((down.0, down.1, palette.tertiary));
    keys
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

    #[test]
    fn a_bar_that_fits_stays_one_row() {
        let palette = Palette::terminal();
        let bar = line(&palette, &[("Enter", "open"), ("q", "quit")]);
        assert_eq!(height(&bar, 18), 1, "the closing gap may run past the edge");
        assert_eq!(height(&bar, 80), 1);
    }

    #[test]
    fn a_long_bar_wraps_between_hints_and_keeps_its_alignment() {
        let palette = Palette::terminal();
        let bar = line(
            &palette,
            &[
                ("l", "add"),
                ("h", "skip"),
                ("m", "maybeboard"),
                ("?", "hint"),
                ("Esc", "back"),
            ],
        )
        .centered();
        let rows = wrap(bar.clone(), 24);
        let text: Vec<String> = rows.lines.iter().map(Line::to_string).collect();
        assert_eq!(
            text,
            ["l add  h skip", "m maybeboard  ? hint", "Esc back"],
            "no key parts from its action"
        );
        assert!(rows.lines.iter().all(|row| row.alignment == bar.alignment));
        assert_eq!(height(&bar, 24), 3);
    }

    #[test]
    fn a_hint_wider_than_the_bar_takes_its_own_row() {
        let palette = Palette::terminal();
        let bar = line(
            &palette,
            &[("a", "b"), ("Enter", "something long"), ("q", "quit")],
        );
        let text: Vec<String> = wrap(bar, 8).lines.iter().map(Line::to_string).collect();
        assert_eq!(text, ["a b", "Enter something long", "q quit"]);
    }

    #[test]
    fn an_unbreakable_bar_wraps_in_a_paragraph_only_between_hints() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;
        use ratatui::widgets::{Paragraph, Wrap};

        let palette = Palette::terminal();
        let bar = unbreakable(line(
            &palette,
            &[("i", "Image"), ("c", "Create deck"), ("r", "Remove")],
        ));
        let mut terminal = Terminal::new(TestBackend::new(16, 3)).unwrap();
        terminal
            .draw(|f| {
                f.render_widget(Paragraph::new(bar).wrap(Wrap { trim: false }), f.area());
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let rows: Vec<String> = (0..3)
            .map(|y| {
                (0..16)
                    .map(|x| buffer.cell((x, y)).unwrap().symbol())
                    .collect::<String>()
                    .replace(NBSP, " ")
                    .trim_end()
                    .to_owned()
            })
            .collect();
        assert_eq!(rows, ["i Image", "c Create deck", "r Remove"]);
    }

    #[test]
    fn help_goes_before_the_closing_back_and_quit_hints() {
        let palette = Palette::terminal();
        let shown = |keys: &[(&str, &str)]| {
            with_help(&palette, line(&palette, keys), Hint::Absent).to_string()
        };
        assert_eq!(
            shown(&[("c", "Create"), ("Esc", "Back"), ("q", "quit")]),
            "c Create  F1 help  Esc Back  q quit  "
        );
        assert_eq!(
            shown(&[("Tab", "next"), ("Esc", "Back"), ("F2", "Save")]),
            "Tab next  F1 help  Esc Back  F2 Save  ",
            "a hint after Esc stays after it"
        );
        assert_eq!(
            shown(&[("Esc", "Back"), ("Tab", "next")]),
            "Esc Back  Tab next  F1 help  ",
            "a bar that opens with Esc takes help last"
        );
        let centered = with_help(
            &palette,
            line(&palette, &[("q", "quit")]).centered(),
            Hint::Absent,
        );
        assert_eq!(centered.to_string(), "q quit  F1 help  ");
        assert_eq!(centered.alignment, Some(ratatui::layout::Alignment::Center));
    }

    #[test]
    fn help_after_a_hint_with_no_gap_still_wraps_between_hints() {
        let palette = Palette::terminal();
        let mut bar = line(&palette, &[("Tab", "next")]);
        bar.spans.push(Span::raw("F2 "));
        bar.spans.push(Span::raw("Import"));
        let bar = with_help(&palette, bar, Hint::Absent);
        assert_eq!(bar.to_string(), "Tab next  F2 Import  F1 help  ");
        let text: Vec<String> = wrap(bar, 20).lines.iter().map(Line::to_string).collect();
        assert_eq!(text, ["Tab next  F2 Import", "F1 help"]);
    }

    #[test]
    fn a_screen_with_a_hint_names_it_once_just_before_help() {
        let palette = Palette::terminal();
        let shown = |keys: &[(&str, &str)], hint: Hint| {
            with_help(&palette, line(&palette, keys), hint).to_string()
        };
        assert_eq!(
            shown(&[("c", "Create"), ("Esc", "Back")], Hint::Question),
            "c Create  ? hint  F1 help  Esc Back  "
        );
        assert_eq!(
            shown(
                &[("?", "Hint"), ("c", "Create"), ("Esc", "Back")],
                Hint::Question
            ),
            "c Create  ? hint  F1 help  Esc Back  ",
            "a hint the bar lists itself moves to its spot"
        );
        assert_eq!(
            shown(&[("Esc", "Back"), ("?", "hint")], Hint::Question),
            "Esc Back  ? hint  F1 help  "
        );
        assert_eq!(
            shown(
                &[("?", "About"), ("H", "Hint"), ("Esc", "Back")],
                Hint::ShiftH
            ),
            "? About  H hint  F1 help  Esc Back  ",
            "a ? that opens something else keeps its own name"
        );
        assert_eq!(
            shown(
                &[("c", "Create"), ("?", "hint"), ("Esc", "Back")],
                Hint::Absent
            ),
            "c Create  F1 help  Esc Back  ",
            "no hint while a field takes the letters"
        );
    }

    #[test]
    fn an_empty_bar_is_one_empty_row() {
        assert_eq!(height(&Line::default(), 10), 1);
    }
}
