//! Hint dialogs: the app's `?` topics, opened on demand, and its one-time hints, shown the first time a user reaches something and never again once the server has marked them shown.
//!
//! - [`topic`] opens a [`HintTopic`], such as `Email` beside the email row.
//! - [`once`] opens the one-time hint stored under a key the first time only: it checks the user's `hints_shown`, opens the dialog, marks the hint locally so it cannot show twice, and asks the server to mark it with `mark_hint_shown`. The user the server answers with replaces the shared one and is saved with the session.
//! - [`reopen`] opens a one-time hint again on demand, as a header `?` does, without recording it.
//!
//! One dialog is up at a time, and it takes every key until it closes: Esc, Enter or Space close it, and j/k scroll a long one.

pub(crate) mod keys;
pub(crate) mod markup;
pub(crate) mod topics;

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use zwipe_core::domain::user::User;

pub(crate) use self::topics::HintTopic;
use super::measure::height_of;
use super::panel;
use crate::domain::theme::Palette;
use crate::inbound::app::Context;
use crate::inbound::screens::Request;

/// The dialog is at most this wide inside its panel, in cells.
const WIDTH: u16 = 56;

/// The dismiss button every hint ends with.
const GOT_IT: &str = "Got it";

/// What a hint dialog says: its title, and its blocks in hint markup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HintContent {
    pub(crate) title: String,
    pub(crate) blocks: Vec<String>,
}

/// An open hint dialog and how far its body is scrolled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HintDialog {
    pub(crate) content: HintContent,
    pub(crate) scroll: u16,
}

impl HintDialog {
    pub(crate) fn new(content: HintContent) -> Self {
        Self { content, scroll: 0 }
    }

    /// Scrolls with j/k or the arrows. Returns whether the key closes the dialog.
    pub(crate) fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Down | KeyCode::Char('j') => self.scroll = self.scroll.saturating_add(1),
            KeyCode::Up | KeyCode::Char('k') => self.scroll = self.scroll.saturating_sub(1),
            other => return dismisses(other),
        }
        false
    }

    /// Draws the hint centered over the screen: title, rule, body, rule, and the `Got it` button.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette) {
        let lines: Vec<Line<'static>> = self
            .content
            .blocks
            .iter()
            .map(|block| markup::line(palette, block))
            .collect();
        let body = Paragraph::new(lines).wrap(Wrap { trim: false });
        let width = WIDTH.min(f.area().width.saturating_sub(4));
        let body_height = height_of(body.line_count(width));
        let room = f.area().height.saturating_sub(6);
        let shown = body_height.min(room);
        let scroll = self.scroll.min(body_height.saturating_sub(shown));
        let inner = panel::draw(f, palette, f.area(), WIDTH, shown.saturating_add(4));
        let [title_area, top_rule, body_area, bottom_rule, button_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(shown),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        f.render_widget(
            Line::from(self.content.title.clone()).bold().centered(),
            title_area,
        );
        panel::rule(f, palette, top_rule);
        f.render_widget(body.scroll((scroll, 0)), body_area);
        panel::rule(f, palette, bottom_rule);
        f.render_widget(
            Line::from(format!(" {GOT_IT} "))
                .style(palette.accent.bold().reversed())
                .centered(),
            button_area,
        );
    }
}

/// Whether the hint stored under `key` has yet to be shown to `user`. Never before sign-in.
pub(crate) fn due(user: Option<&User>, key: &str) -> bool {
    user.is_some_and(|user| !user.has_seen_hint(key))
}

/// Opens `content` over the screen, replacing a hint already up.
pub(crate) fn open(cx: &mut Context<'_>, content: HintContent) {
    cx.shared.hint = Some(HintDialog::new(content));
}

/// Opens the `?` topic `topic`.
pub(crate) fn topic(cx: &mut Context<'_>, topic: HintTopic) {
    open(cx, topic.content());
}

/// Opens the one-time hint `key` the first time only, with the app's copy for it. Returns whether it opened; a key with no copy of its own never does.
pub(crate) fn once(cx: &mut Context<'_>, key: &str) -> bool {
    keys::for_key(key).is_some_and(|content| once_with(cx, key, content))
}

/// Opens `content` as the one-time hint `key`: when it is due and no other hint is up, opens it, marks it shown on the user at once, and asks the server to remember. Returns whether it opened.
pub(crate) fn once_with(cx: &mut Context<'_>, key: &str, content: HintContent) -> bool {
    if !due(cx.shared.user.as_ref(), key) || cx.shared.hint.is_some() {
        return false;
    }
    if let Some(user) = &mut cx.shared.user {
        user.hints_shown.insert(key.to_owned(), true);
    }
    open(cx, content);
    cx.request(Request::MarkHintShown(key.to_owned()));
    true
}

/// Opens the one-time hint `key` again, as a header `?` does, without recording it.
pub(crate) fn reopen(cx: &mut Context<'_>, key: &str) {
    if let Some(content) = keys::for_key(key) {
        open(cx, content);
    }
}

/// Shows a one-time hint with a plain body, for copy that is not in the app's table.
pub(crate) fn show(cx: &mut Context<'_>, key: &str, title: &str, body: &str) -> bool {
    once_with(
        cx,
        key,
        HintContent {
            title: title.to_owned(),
            blocks: vec![body.to_owned()],
        },
    )
}

/// Whether `code` closes an open hint. Every other key is swallowed while it is up.
pub(crate) fn dismisses(code: KeyCode) -> bool {
    matches!(code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' '))
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::user::hints::HINT_PROFILE;

    use super::*;
    use crate::domain::theme::Palette;
    use crate::inbound::app::App;
    use crate::test_support::{render, screen_text, user};

    #[test]
    fn a_hint_shows_once_and_asks_the_server_to_remember() {
        let mut app = App::new(Palette::terminal());
        let mut cx = Context::new(&mut app.shared);
        assert!(!once(&mut cx, HINT_PROFILE), "not before sign-in");

        cx.shared.user = Some(user(true));
        assert!(once(&mut cx, HINT_PROFILE));
        cx.shared.hint = None;
        assert!(!once(&mut cx, HINT_PROFILE), "only once");
        assert!(!once(&mut cx, "no_such_hint"), "no copy, no hint");
        assert!(cx.shared.user.as_ref().unwrap().has_seen_hint(HINT_PROFILE));
        reopen(&mut cx, HINT_PROFILE);
        let effects = cx.into_effects();
        assert_eq!(
            effects.requests,
            [Request::MarkHintShown(HINT_PROFILE.to_owned())],
            "reopening records nothing"
        );
        let screen = screen_text(&app);
        assert!(screen.contains("Your account"));
        assert!(screen.contains("Got it"));
        assert!(screen.contains("├"), "ruled above and below the body");
    }

    #[test]
    fn a_plain_hint_and_a_topic_open_over_the_screen() {
        let mut app = App::new(Palette::terminal());
        let mut cx = Context::new(&mut app.shared);
        cx.shared.user = Some(user(true));
        assert!(show(&mut cx, "swipe", "Swiping", "Right adds."));
        topic(&mut cx, HintTopic::Email);
        assert!(screen_text(&app).contains("Where password resets and verification go"));
    }

    #[test]
    fn a_seen_hint_is_not_due_and_only_enter_esc_or_space_close_it() {
        let mut seen = user(true);
        seen.hints_shown.insert("swipe".to_owned(), true);
        assert!(!due(Some(&seen), "swipe"));
        assert!(due(Some(&user(true)), "swipe"));
        let mut dialog = HintDialog::new(HintTopic::Activity.content());
        assert!(!dialog.handle_key(KeyCode::Char('j')));
        assert_eq!(dialog.scroll, 1);
        assert!(!dialog.handle_key(KeyCode::Char('l')));
        assert!(dialog.handle_key(KeyCode::Enter));
    }

    #[test]
    fn a_long_hint_fits_a_tiny_terminal() {
        let mut app = App::new(Palette::terminal());
        app.shared.hint = Some(HintDialog::new(HintTopic::RoleRates.content()));
        render(&app, 20, 6);
        render(&app, 1, 1);
    }
}
