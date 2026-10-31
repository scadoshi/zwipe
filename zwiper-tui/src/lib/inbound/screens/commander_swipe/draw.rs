//! The commander swipe: the title, the card on top as text with its reasons under it in Suggested, then the swipe keys. The Suggested Filter draws as a bottom sheet holding the Show chips.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget, Wrap};
use zwipe_core::domain::card::Card;

use super::CommanderSwipeView;
use crate::domain::playstyle;
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::card_sheets::filter::draw as filter_sheet;
use crate::inbound::screens::playstyle::draw::{Reasons, reasons_lines, show_line, wrap_at_gaps};
use crate::inbound::widgets::card_motion::{self, CardMotion, Phase};
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
    let now = Instant::now();
    let motion = view.motion.as_ref().filter(|m| m.running(now));
    // While a card comes back after an undo, the card it covers rests under it.
    let resting = match motion.map(CardMotion::phase) {
        Some(Phase::Enter) => view.under(),
        _ => view.current(),
    };
    let paragraph = |lines| Paragraph::new(lines).wrap(Wrap { trim: false });
    match resting {
        Some(card) => f.render_widget(paragraph(card_lines(palette, view, card)), body),
        None if view.current().is_none() => f.render_widget(
            paragraph(body_lines(palette, view, usize::from(body.width))),
            body,
        ),
        None => {}
    }
    if let Some(motion) = motion {
        let moving = match motion.phase() {
            Phase::Exit(card) => Some(card),
            Phase::Enter => view.current(),
        };
        if let Some(card) = moving
            && let Some(placement) = motion.placement(now, body, body)
        {
            let text = paragraph(card_lines(palette, view, card));
            card_motion::draw(f.buffer_mut(), &placement, palette.base, |buf| {
                text.render(body, buf);
            });
        }
    }
    f.render_widget(action_bar::styled_line(&key_hints(palette, view)), keys);
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
    if let Some(sheet) = &view.sheet {
        filter_sheet::draw(f, shared, sheet, f.area());
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
            return vec![Line::styled(
                playstyle::empty_identity_line(identity),
                palette.muted,
            )];
        }
        return skeleton::rows(palette, GHOST_ROWS, width);
    };
    card_lines(palette, view, card)
}

/// A card as text with its rarity, and its reasons when it is the Suggested card on top.
fn card_lines(palette: &Palette, view: &CommanderSwipeView, card: &Card) -> Vec<Line<'static>> {
    let data = &card.scryfall_data;
    let mut lines = card_text::lines(palette, data);
    lines.push(Line::styled(data.rarity.to_long_name(), palette.muted));
    let on_top = view
        .current()
        .is_some_and(|top| top.scryfall_data.id == data.id);
    if on_top && let (Some(suggestion), Some(pile)) = (view.current_suggestion(), view.suggested())
    {
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

/// The swipe keys in the app's swipe colors (add, skip, maybeboard, undo), then the Filter with a dot while it narrows the stack, Refresh, the hint and Back.
fn key_hints(
    palette: &Palette,
    view: &CommanderSwipeView,
) -> Vec<(&'static str, &'static str, Style)> {
    let mut keys = if view.current().is_some() {
        action_bar::swipes(
            palette,
            ("→", "add"),
            ("←", "skip"),
            Some(("↑", "maybeboard")),
            ("↓", "undo"),
        )
    } else {
        vec![("↓", "undo", palette.tertiary)]
    };
    let filter = if view.filter_dot() {
        "Filter •"
    } else {
        "Filter"
    };
    keys.extend(
        [
            ("f", filter),
            ("r", "Refresh"),
            ("?", "Hint"),
            ("Esc", "Back"),
        ]
        .into_iter()
        .map(|(k, a)| (k, a, palette.muted)),
    );
    keys
}
