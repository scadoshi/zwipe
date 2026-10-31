//! One-time hints: a dialog shown the first time a user reaches something, never again once the server has marked it shown.
//!
//! [`show`] checks the user's `hints_shown`, opens the dialog over whatever screen is up, marks the hint locally so it cannot show twice, and asks the server to mark it with `mark_hint_shown`. The user the server answers with replaces the shared one.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use zwipe_core::domain::user::User;

use super::measure::height_of;
use super::{action_bar, panel};
use crate::domain::theme::Palette;
use crate::inbound::app::Context;
use crate::inbound::screens::Request;

/// The dialog is at most this wide inside its panel, in cells.
const WIDTH: u16 = 56;

/// An open hint: the key it is stored under, and what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HintDialog {
    pub(crate) key: String,
    pub(crate) title: String,
    pub(crate) body: String,
}

/// Whether the hint stored under `key` has yet to be shown to `user`. Never before sign-in.
pub(crate) fn due(user: Option<&User>, key: &str) -> bool {
    user.is_some_and(|user| !user.has_seen_hint(key))
}

/// Shows the hint once: opens it when it is due, marks it shown on the user at once, and asks the server to remember. Returns whether it opened.
pub(crate) fn show(cx: &mut Context<'_>, key: &str, title: &str, body: &str) -> bool {
    if !due(cx.shared.user.as_ref(), key) || cx.shared.hint.is_some() {
        return false;
    }
    if let Some(user) = &mut cx.shared.user {
        user.hints_shown.insert(key.to_owned(), true);
    }
    cx.shared.hint = Some(HintDialog {
        key: key.to_owned(),
        title: title.to_owned(),
        body: body.to_owned(),
    });
    cx.request(Request::MarkHintShown(key.to_owned()));
    true
}

/// Whether `code` closes an open hint. Every other key is swallowed while it is up.
pub(crate) fn dismisses(code: KeyCode) -> bool {
    matches!(code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' '))
}

impl HintDialog {
    /// Draws the hint centered over the screen.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette) {
        let body = Paragraph::new(self.body.clone()).wrap(Wrap { trim: true });
        let body_height = height_of(body.line_count(WIDTH));
        let inner = panel::draw(f, palette, f.area(), WIDTH, body_height.saturating_add(4));
        let [title_area, body_area, _, keys_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(body_height.saturating_add(1)),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        f.render_widget(
            Line::from(self.title.clone()).style(palette.accent).bold(),
            title_area,
        );
        f.render_widget(body, body_area);
        f.render_widget(action_bar::line(palette, &[("Enter", "got it")]), keys_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::theme::Palette;
    use crate::inbound::app::App;
    use crate::test_support::{screen_text, user};

    #[test]
    fn a_hint_shows_once_and_asks_the_server_to_remember() {
        let mut app = App::new(Palette::terminal());
        let mut cx = Context::new(&mut app.shared);
        assert!(
            !show(&mut cx, "swipe", "Swiping", "Right adds."),
            "not before sign-in"
        );

        cx.shared.user = Some(user(true));
        assert!(show(&mut cx, "swipe", "Swiping", "Right adds."));
        assert!(
            !show(&mut cx, "swipe", "Swiping", "Right adds."),
            "only once"
        );
        assert!(cx.shared.user.as_ref().unwrap().has_seen_hint("swipe"));
        let effects = cx.into_effects();
        assert_eq!(
            effects.requests,
            [Request::MarkHintShown("swipe".to_owned())]
        );
        assert!(screen_text(&app).contains("Right adds."));
    }

    #[test]
    fn a_seen_hint_is_not_due_and_only_enter_esc_or_space_close_it() {
        let mut seen = user(true);
        seen.hints_shown.insert("swipe".to_owned(), true);
        assert!(!due(Some(&seen), "swipe"));
        assert!(due(Some(&user(true)), "swipe"));
        assert!(dismisses(KeyCode::Enter));
        assert!(!dismisses(KeyCode::Char('l')));
    }
}
