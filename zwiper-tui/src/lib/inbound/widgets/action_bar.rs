//! The app's bottom row of buttons: drawn as key hints, each key in bold and its action muted, or as buttons the focus moves across.

use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};

use crate::domain::theme::Palette;

/// One button in a row of [`buttons`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Button<'label> {
    pub(crate) label: &'label str,
    /// In the error color, like the app's `Log out`.
    pub(crate) danger: bool,
    /// Muted, and not to be pressed.
    pub(crate) disabled: bool,
}

impl<'label> Button<'label> {
    pub(crate) fn new(label: &'label str) -> Self {
        Self {
            label,
            danger: false,
            disabled: false,
        }
    }

    pub(crate) fn danger(self) -> Self {
        Self {
            danger: true,
            ..self
        }
    }

    pub(crate) fn disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }
}

/// A row of buttons, centered by the caller: each label padded, the one at `focused` reversed.
pub(crate) fn buttons(
    palette: &Palette,
    row: &[Button<'_>],
    focused: Option<usize>,
) -> Line<'static> {
    let spans: Vec<Span<'static>> = row
        .iter()
        .enumerate()
        .flat_map(|(i, button)| {
            let base = if button.disabled {
                palette.muted
            } else if button.danger {
                palette.error
            } else {
                palette.accent
            };
            let style = if focused == Some(i) {
                base.bold().reversed()
            } else {
                base
            };
            [
                Span::styled(format!(" {} ", button.label), style),
                Span::raw(" "),
            ]
        })
        .collect();
    Line::from(spans)
}

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
    fn the_focused_button_reverses_and_a_disabled_one_mutes() {
        let palette = Palette::for_account("gruvbox", true);
        let row = buttons(
            &palette,
            &[
                Button::new("Back"),
                Button::new("Log out").danger(),
                Button::new("More").disabled(true),
            ],
            Some(1),
        );
        assert_eq!(row.to_string(), " Back   Log out   More  ");
        assert!(row.spans[2].style.add_modifier.contains(Modifier::REVERSED));
        assert_eq!(row.spans[2].style.fg, palette.error.fg);
        assert_eq!(row.spans[4].style, palette.muted);
    }

    #[test]
    fn each_key_is_bold_and_its_action_muted() {
        let palette = Palette::terminal();
        let line = line(&palette, &[("Enter", "open"), ("q", "quit")]);
        assert_eq!(line.to_string(), "Enter open  q quit  ");
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[1].style, palette.muted);
    }
}
