//! The card details sheet the eye key opens: the card's details under its name, with Flip for a card with more faces and Printings where the host allows it.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use zwipe_core::domain::card::Card;

use crate::domain::theme::Palette;
use crate::inbound::widgets::card_info;
use crate::inbound::widgets::sheet::{Sheet, SheetKey};

/// What a key did to the sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DetailsOutcome {
    Open,
    Closed,
    /// Printings was asked for.
    Printings,
}

/// The details sheet over one card.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DetailsSheet {
    pub(crate) card: Card,
    pub(crate) face: usize,
    /// Whether the footer offers Printings.
    pub(crate) printings: bool,
    scroll: usize,
}

impl DetailsSheet {
    pub(crate) fn new(card: Card, printings: bool) -> Self {
        Self {
            card,
            face: 0,
            printings,
            scroll: 0,
        }
    }

    fn sheet(&self, palette: &Palette) -> Sheet {
        let mut sheet = Sheet::new(
            &self.card.scryfall_data.name,
            card_info::details_lines(palette, &self.card, self.face),
        )
        .with_footer("Esc", "back");
        if card_info::face_count(&self.card) > 1 {
            sheet = sheet.with_footer("t", "flip");
        }
        if self.printings {
            sheet = sheet.with_footer("p", "printings");
        }
        sheet.scroll = self.scroll;
        sheet
    }

    /// t flips, p opens Printings, the arrows scroll, Esc closes.
    pub(crate) fn handle_key(&mut self, code: KeyCode, palette: &Palette) -> DetailsOutcome {
        match code {
            KeyCode::Char('t') if card_info::face_count(&self.card) > 1 => {
                self.face = self.face.saturating_add(1) % card_info::face_count(&self.card);
            }
            KeyCode::Char('p') if self.printings => return DetailsOutcome::Printings,
            _ => {
                let mut sheet = self.sheet(palette);
                if sheet.handle_key(code) == SheetKey::Close {
                    return DetailsOutcome::Closed;
                }
                self.scroll = sheet.scroll;
            }
        }
        DetailsOutcome::Open
    }

    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette, area: Rect) {
        self.sheet(palette).draw(f, palette, area);
    }
}
