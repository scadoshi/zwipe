//! The printing sheet: every printing of a card in a carousel, to swap the one a deck uses, or on Remove only to browse.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Wrap};
use uuid::Uuid;
use zwipe_core::domain::card::Card;

use super::Request;
use crate::domain::theme::Palette;
use crate::inbound::app::Context;
use crate::inbound::widgets::card_info;
use crate::inbound::widgets::toast::{self, ToastKind};

/// What a key did to the sheet.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PrintingOutcome {
    Open,
    Closed,
    /// Save picked this printing; the sheet closed.
    Saved(Box<Card>),
}

/// The printing sheet over a card.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PrintingSheet {
    /// The printing in use when the sheet opened.
    pub(crate) card: Card,
    /// Browse only: no Save, no "Printing unchanged".
    pub(crate) read_only: bool,
    /// The toast on Save.
    pub(crate) saved_message: &'static str,
    /// Every printing, or `None` while they load.
    pub(crate) printings: Option<Vec<Card>>,
    /// The printing shown.
    pub(crate) index: usize,
    /// The printing in use, within `printings`.
    initial: usize,
}

impl PrintingSheet {
    /// Opens over `card`: reuses the printings fetched last when they are this card's, else fetches them. A card without an oracle ID loads nothing.
    pub(crate) fn open(
        cx: &mut Context<'_>,
        card: Card,
        read_only: bool,
        saved_message: &'static str,
    ) -> Self {
        let mut sheet = Self {
            card,
            read_only,
            saved_message,
            printings: None,
            index: 0,
            initial: 0,
        };
        let Some(oracle_id) = sheet.card.scryfall_data.oracle_id else {
            return sheet;
        };
        match &cx.shared.cards.printings {
            Some((cached, printings)) if *cached == oracle_id => {
                sheet.loaded(oracle_id, printings.clone());
            }
            _ => cx.request(Request::Printings { oracle_id }.into()),
        }
        sheet
    }

    /// Takes the fetched printings and shows the one in use.
    pub(crate) fn loaded(&mut self, oracle_id: Uuid, printings: Vec<Card>) {
        if self.card.scryfall_data.oracle_id != Some(oracle_id) {
            return;
        }
        let current = self.card.scryfall_data.id;
        self.initial = printings
            .iter()
            .position(|c| c.scryfall_data.id == current)
            .unwrap_or_default();
        self.index = self.initial;
        self.printings = Some(printings);
    }

    /// Stops loading after a failed fetch.
    pub(crate) fn failed(&mut self, oracle_id: Uuid) {
        if self.card.scryfall_data.oracle_id == Some(oracle_id) {
            self.printings = Some(Vec::new());
        }
    }

    /// Whether the carousel moved off the printing in use.
    pub(crate) fn has_changed(&self) -> bool {
        self.printings.as_ref().is_some_and(|p| !p.is_empty()) && self.index != self.initial
    }

    /// The printing shown.
    pub(crate) fn visible(&self) -> Option<&Card> {
        self.printings.as_ref()?.get(self.index)
    }

    /// Left and right move the carousel, s saves, Esc closes.
    pub(crate) fn handle_key(&mut self, code: KeyCode, cx: &mut Context<'_>) -> PrintingOutcome {
        let count = self.printings.as_ref().map_or(0, Vec::len);
        match code {
            KeyCode::Left | KeyCode::Char('h') => self.index = self.index.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('l') => {
                self.index = self.index.saturating_add(1).min(count.saturating_sub(1));
            }
            KeyCode::Esc | KeyCode::Char('b') => {
                if !self.read_only && self.has_changed() {
                    cx.toast_for(
                        ToastKind::Info,
                        "Printing unchanged".to_owned(),
                        toast::QUICK,
                    );
                }
                return PrintingOutcome::Closed;
            }
            KeyCode::Char('s') if !self.read_only && self.has_changed() => {
                if let Some(card) = self.visible().cloned() {
                    cx.toast_for(ToastKind::Info, self.saved_message.to_owned(), toast::QUICK);
                    return PrintingOutcome::Saved(Box::new(card));
                }
            }
            _ => {}
        }
        PrintingOutcome::Open
    }

    /// Draws the sheet over the bottom of `area`.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, palette: &Palette, area: Rect) {
        let mut body: Vec<Line<'static>> = Vec::new();
        match (&self.printings, self.visible()) {
            (None, _) => body.push(Line::from("Loading printings...").style(palette.muted)),
            (Some(_), Some(card)) => {
                let count = self.printings.as_ref().map_or(0, Vec::len);
                if count > 1 {
                    body.push(
                        Line::from(format!("‹ {}/{count} ›", self.index.saturating_add(1)))
                            .style(palette.accent)
                            .centered(),
                    );
                    body.push(Line::default());
                }
                body.extend(card_info::info_lines(palette, card));
            }
            (Some(_), None) => body.push(Line::from("No printings").style(palette.muted)),
        }
        let mut keys = vec![("Esc", "back"), ("h/l", "browse")];
        if !self.read_only {
            keys.push(("s", "save"));
        }
        let save_style = if self.has_changed() {
            palette.accent
        } else {
            palette.muted
        };
        let mut footer = crate::inbound::widgets::action_bar::line(palette, &keys);
        if let Some(last) = footer.spans.last_mut()
            && !self.read_only
        {
            last.style = save_style;
        }
        let height = u16::try_from(body.len())
            .unwrap_or(u16::MAX)
            .saturating_add(2)
            .min(area.height);
        let [_, sheet] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(height)]).areas(area);
        f.render_widget(Clear, sheet);
        f.render_widget(
            Paragraph::new(body).wrap(Wrap { trim: true }).block(
                Block::bordered()
                    .title(Line::from(" Printings ").bold())
                    .title_bottom(footer)
                    .border_style(palette.accent)
                    .style(palette.base)
                    .padding(Padding::horizontal(1)),
            ),
            sheet,
        );
    }
}
