//! Hint dialogs: the app's `?` topics, opened on demand, and its one-time hints, shown the first time a user reaches something and never again once the server has marked them shown.
//!
//! The copy is zwipe-ui's, as typed spans: [`HintTopic`] for the topics, [`one_time`] for the one-time hints. This module draws it, each [`Role`] in its palette color.
//!
//! - [`topic`] opens a [`HintTopic`], such as `Email` beside the email row.
//! - [`once`] opens the one-time hint stored under a key the first time only: it checks the user's `hints_shown`, opens the dialog, marks the hint locally so it cannot show twice, and asks the server to mark it with `mark_hint_shown`. The user the server answers with replaces the shared one and is saved with the session.
//! - [`reopen`] opens a one-time hint again on demand, as the key bar's `? hint` does, without recording it.
//!
//! One dialog is up at a time, and it takes every key until it closes: Esc, Enter or Space close it, and j/k scroll a long one.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::{self, Line};
use ratatui::widgets::{Paragraph, Wrap};
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::hints::OneTimeHint;
pub(crate) use zwipe_ui::hint::HintTopic;
use zwipe_ui::hint::{Block, GOT_IT, HintContent, KeyIcon, Role, Span, one_time};

use super::{card_info, measure, panel};
use crate::domain::theme::Palette;
use crate::inbound::app::Context;
use crate::inbound::screens::Request;

/// The dialog is at most this wide inside its panel, in cells.
const WIDTH: u16 = 56;

/// The palette color a key chip or an emphasized word in `role` takes.
fn role_style(palette: &Palette, role: Role) -> Style {
    let swatch = |i: usize| palette.swatches.get(i).copied().unwrap_or(palette.accent);
    match role {
        Role::Palette1 => swatch(0),
        Role::Palette2 => swatch(1),
        Role::Palette3 => swatch(2),
        Role::Palette4 => swatch(3),
        Role::Palette5 => swatch(4),
        Role::Palette6 => swatch(5),
        Role::Primary => palette.accent,
        Role::Secondary => palette.secondary,
        Role::Tertiary => palette.tertiary,
        Role::Success => palette.add,
        Role::Error => palette.error,
        Role::Warning => palette.warning,
    }
}

/// The glyph a key chip's icon draws as.
fn icon_glyph(icon: KeyIcon) -> &'static str {
    match icon {
        KeyIcon::Light => "☀",
        KeyIcon::Dark => "☾",
    }
}

/// One block of hint copy as a line of spans: a bullet in the muted color, key chips reversed with a space each side, emphasized words bold in their color.
pub(crate) fn line(palette: &Palette, block: &Block) -> Line<'static> {
    let mut spans: Vec<text::Span<'static>> = Vec::new();
    if matches!(block, Block::Bullet(_)) {
        spans.push(bullet(palette));
    }
    spans.extend(block_spans(palette, block));
    Line::from(spans)
}

/// The muted bullet that starts a hint's list line.
fn bullet(palette: &Palette) -> text::Span<'static> {
    text::Span::styled(card_info::BULLET, palette.muted)
}

/// One block of hint copy as `width` cells wide lines: a paragraph as one line for the dialog to wrap, a bullet broken between words with its continuations hung under its text.
fn block_lines(palette: &Palette, block: &Block, width: usize) -> Vec<Line<'static>> {
    match block {
        Block::Bullet(_) => {
            card_info::bulleted(bullet(palette), block_spans(palette, block), width)
        }
        Block::Line(_) => vec![line(palette, block)],
    }
}

/// A block's spans with no bullet: key chips reversed with a space each side, emphasized words bold in their color.
fn block_spans(palette: &Palette, block: &Block) -> Vec<text::Span<'static>> {
    let mut spans: Vec<text::Span<'static>> = Vec::new();
    for span in block.spans() {
        spans.push(match span {
            Span::Text(text) => text::Span::raw(text.clone()),
            Span::Key { label, role, icon } => {
                let label = match icon {
                    Some(icon) => format!(" {} {label} ", icon_glyph(*icon)),
                    None => format!(" {label} "),
                };
                text::Span::styled(label, role_style(palette, *role).reversed().bold())
            }
            Span::Emph { text, role } => {
                text::Span::styled(text.clone(), role_style(palette, *role).bold())
            }
        });
    }
    spans
}

/// The dialog as it reads, one block per line, styling dropped and runs of spaces closed up.
#[cfg(test)]
pub(crate) fn text(content: &HintContent) -> String {
    let palette = Palette::terminal();
    content
        .blocks
        .iter()
        .map(|block| {
            line(&palette, block)
                .to_string()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
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
        let width = WIDTH.min(f.area().width.saturating_sub(4));
        let lines: Vec<Line<'static>> = self
            .content
            .blocks
            .iter()
            .flat_map(|block| block_lines(palette, block, usize::from(width)))
            .collect();
        let body = Paragraph::new(lines).wrap(Wrap { trim: false });
        let body_height = measure::height_of(body.line_count(width));
        let room = f.area().height.saturating_sub(6);
        let shown = body_height.min(room);
        let scroll = self.scroll.min(body_height.saturating_sub(shown));
        let inner = panel::dialog(f, palette, f.area(), WIDTH, shown.saturating_add(4));
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

/// The app's copy for the one-time hint stored under `key`, or `None` for a key with no fixed copy.
fn content_for(key: &str) -> Option<HintContent> {
    OneTimeHint::from_key(key).and_then(one_time::content_for)
}

/// Opens the one-time hint `key` the first time only, with the app's copy for it. Returns whether it opened; a key with no copy of its own never does.
pub(crate) fn once(cx: &mut Context<'_>, key: &str) -> bool {
    content_for(key).is_some_and(|content| once_with(cx, key, content))
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

/// Opens the one-time hint `key` again, as the key bar's `? hint` does, without recording it.
pub(crate) fn reopen(cx: &mut Context<'_>, key: &str) {
    if let Some(content) = content_for(key) {
        open(cx, content);
    }
}

/// Whether `code` closes an open hint. Every other key is swallowed while it is up.
pub(crate) fn dismisses(code: KeyCode) -> bool {
    matches!(code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' '))
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;
    use zwipe_core::domain::user::hints::HINT_PROFILE;

    use super::*;
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
        let content = HintContent {
            title: "Swiping".to_owned(),
            blocks: vec![Block::Line(vec![Span::Text("Right adds.".to_owned())])],
        };
        assert!(once_with(&mut cx, "swipe", content));
        topic(&mut cx, HintTopic::Email);
        assert!(screen_text(&app).contains("Where password resets and verification go"));
    }

    #[test]
    fn a_wrapped_bullet_hangs_under_its_text_and_keeps_its_key_chips_whole() {
        for topic in [HintTopic::ColorMap, HintTopic::DrawOdds] {
            let mut app = App::new(Palette::terminal());
            app.shared.hint = Some(HintDialog::new(topic.content()));
            for (width, height) in [(200, 50), (120, 40), (80, 24), (60, 20)] {
                let rows = crate::test_support::rows(&app, width, height);
                crate::test_support::assert_bullets_hang(&rows);
                if topic == HintTopic::DrawOdds {
                    let text = rows.join("\n");
                    assert!(
                        text.contains(" - "),
                        "the - chip is whole at {width}x{height}"
                    );
                }
            }
        }
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

    #[test]
    fn bullets_chips_and_colored_words_take_their_roles_colors() {
        let palette = Palette::for_account("gruvbox", true);
        let block = Block::Bullet(vec![
            Span::Key {
                label: "Decks".to_owned(),
                role: Role::Palette2,
                icon: None,
            },
            Span::Text(" holds your ".to_owned()),
            Span::Emph {
                text: "decks".to_owned(),
                role: Role::Warning,
            },
            Span::Text(" and ".to_owned()),
            Span::Key {
                label: "dark".to_owned(),
                role: Role::Error,
                icon: Some(KeyIcon::Dark),
            },
        ]);
        let line = line(&palette, &block);
        assert_eq!(line.to_string(), "•  Decks  holds your decks and  ☾ dark ");
        let chip = &line.spans[1];
        assert_eq!(chip.style.fg, palette.swatches[1].fg);
        assert!(chip.style.add_modifier.contains(Modifier::REVERSED));
        assert_eq!(line.spans[3].style.fg, palette.warning.fg);
        assert_eq!(line.spans[5].style.fg, palette.error.fg);
    }
}
