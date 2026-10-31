//! The commander swipe: the title, the card on top as text with its reasons under it in Suggested, then the swipe keys. The Suggested Filter draws as a bottom sheet holding the Show chips.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use super::CommanderSwipeView;
use crate::domain::playstyle::empty_identity_line;
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::playstyle::draw::{Reasons, reasons_lines, show_line, wrap_at_gaps};
use crate::inbound::widgets::sheet::Sheet;
use crate::inbound::widgets::{action_bar, card_text, skeleton};

/// Ghost rows standing in for a card's text.
const GHOST_ROWS: usize = 8;

/// The cells a bottom sheet takes from the width: its two borders and one cell of padding each side.
const SHEET_FRAME: usize = 4;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &CommanderSwipeView) {
    let palette = &shared.palette;
    let area = f.area().inner(Margin::new(1, 0));
    let [title, body, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);
    f.render_widget(
        Line::styled(view.title(), palette.accent).bold().centered(),
        title,
    );
    let body = body.inner(Margin::new(1, 1));
    f.render_widget(
        Paragraph::new(body_lines(palette, view, usize::from(body.width)))
            .wrap(Wrap { trim: false }),
        body,
    );
    f.render_widget(action_bar::line(palette, &key_hints(view)), keys);
    if let (Some(filter), Some(pile)) = (view.filter, view.suggested()) {
        let chips = show_line(
            palette,
            &view.identities(),
            pile.identity.as_deref(),
            filter.cursor,
            true,
        );
        let room = usize::from(f.area().width).saturating_sub(SHEET_FRAME);
        Sheet::new("Filter", wrap_at_gaps(chips, room))
            .with_footer("h/l", "Chip")
            .with_footer("Enter", "Pick")
            .with_footer("Esc", "Back")
            .draw(f, palette, f.area());
    }
}

/// The card on top and its reasons; a line when the Suggested filter matches nothing; ghost rows while a page loads or past the last card.
fn body_lines(palette: &Palette, view: &CommanderSwipeView, width: usize) -> Vec<Line<'static>> {
    let Some(card) = view.current() else {
        let empty_pile = view
            .suggested()
            .filter(|_| view.cards.is_empty())
            .and_then(|pile| pile.identity.as_deref());
        if let Some(identity) = empty_pile.filter(|_| !view.loading()) {
            return vec![Line::styled(empty_identity_line(identity), palette.muted)];
        }
        return skeleton::rows(palette, GHOST_ROWS, width);
    };
    let data = &card.scryfall_data;
    let mut lines = card_text::lines(palette, data);
    lines.push(Line::styled(data.rarity.to_long_name(), palette.muted));
    if let (Some(suggestion), Some(pile)) = (view.current_suggestion(), view.suggested()) {
        let reasons = reasons_lines(
            palette,
            &suggestion.reasons,
            pile.deck_count,
            Reasons::Heading,
        );
        if !reasons.is_empty() {
            lines.push(Line::default());
            lines.extend(reasons);
        }
    }
    lines
}

/// The swipe keys, the Filter with a dot while an identity is picked, Refresh, the hint and Back.
fn key_hints(view: &CommanderSwipeView) -> Vec<(&'static str, &'static str)> {
    let mut keys = Vec::new();
    if view.current().is_some() {
        keys.extend([("→/l", "Pick"), ("↑/k", "Save"), ("←/h", "Skip")]);
    }
    keys.push(("↓/j", "Undo"));
    if let Some(pile) = view.suggested() {
        keys.push((
            "f",
            if pile.identity.is_some() {
                "Filter •"
            } else {
                "Filter"
            },
        ));
    }
    keys.push(("r", "Refresh"));
    keys.push(("?", "Hint"));
    keys.push(("Esc", "Back"));
    keys
}
