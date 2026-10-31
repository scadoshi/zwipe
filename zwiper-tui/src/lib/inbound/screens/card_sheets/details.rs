//! The card details sheet the eye key opens: the card's details under its name, with Flip for a card with more faces and Printings where the host allows it.
//!
//! Tab and `BackTab` walk the card's oracle tags, the app's `OtagDef` reveals: the open tag's definition shows under the details, and `x` opens its example cards.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use zwipe_core::domain::card::Card;

use crate::domain::theme::Palette;
use crate::inbound::app::{Context, Shared};
use crate::inbound::screens::home::cache::Catalog;
use crate::inbound::screens::oracle_tags;
use crate::inbound::widgets::card_info;
use crate::inbound::widgets::otag::{self, RoleTags};
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
    /// The card's oracle tags, role by role then the rest, with at most one definition open.
    pub(crate) tags: RoleTags,
}

impl DetailsSheet {
    pub(crate) fn new(card: Card, printings: bool) -> Self {
        let tags = RoleTags::new(card_info::oracle_tags(&card));
        Self {
            card,
            face: 0,
            printings,
            scroll: 0,
            tags,
        }
    }

    fn sheet(&self, palette: &Palette, catalog: &Catalog) -> Sheet {
        let mut lines = card_info::details_lines(palette, &self.card, self.face);
        if let Some(slug) = self.tags.open_tag() {
            lines.push(Line::default());
            lines.push(Line::from(otag::chip(palette, slug, /* open */ true)));
            lines.extend(otag::def_lines(palette, catalog, slug));
        }
        let mut sheet = Sheet::new(&self.card.scryfall_data.name, lines).with_footer("Esc", "back");
        if card_info::face_count(&self.card) > 1 {
            sheet = sheet.with_footer("t", "flip");
        }
        if self.printings {
            sheet = sheet.with_footer("p", "printings");
        }
        if !self.tags.tags().is_empty() {
            sheet = sheet.with_footer("Tab", "tags");
        }
        sheet.scroll = self.scroll;
        sheet
    }

    /// t flips, p opens Printings, Tab and `BackTab` walk the oracle tags and `x` opens the open one's examples, the arrows scroll, Esc closes.
    pub(crate) fn handle_key(&mut self, code: KeyCode, cx: &mut Context<'_>) -> DetailsOutcome {
        if matches!(code, KeyCode::Tab | KeyCode::BackTab) {
            oracle_tags::ensure_catalog(cx);
        }
        if self.tags.handle_key(code, cx) {
            return DetailsOutcome::Open;
        }
        match code {
            KeyCode::Char('t') if card_info::face_count(&self.card) > 1 => {
                self.face = self
                    .face
                    .saturating_add(1)
                    .checked_rem(card_info::face_count(&self.card))
                    .unwrap_or(0);
            }
            KeyCode::Char('p') if self.printings => return DetailsOutcome::Printings,
            _ => {
                let mut sheet = self.sheet(&cx.shared.palette, &cx.shared.caches.oracle_tags);
                if sheet.handle_key(code) == SheetKey::Close {
                    return DetailsOutcome::Closed;
                }
                self.scroll = sheet.scroll;
            }
        }
        DetailsOutcome::Open
    }

    pub(crate) fn draw(&self, f: &mut Frame<'_>, shared: &Shared, area: Rect) {
        let palette = &shared.palette;
        self.sheet(palette, &shared.caches.oracle_tags)
            .draw(f, palette, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::theme::Palette;
    use crate::inbound::app::{App, Navigation};
    use crate::inbound::screens::{self, Screen};
    use crate::test_support::card;

    /// Llanowar Elves with ramp under its role and one other tag.
    fn tagged() -> Card {
        let mut card = card("Llanowar Elves");
        let profile = &mut card.card_profile;
        profile.card_roles = vec!["ramp".to_owned()];
        profile
            .oracle_tags_by_role
            .insert("ramp".to_owned(), vec!["mana-dork".to_owned()]);
        profile.other_oracle_tags = vec!["elf-tribal".to_owned(), "mana-dork".to_owned()];
        card
    }

    #[test]
    fn tab_walks_the_cards_tags_and_x_opens_the_open_ones_examples() {
        let mut app = App::new(Palette::terminal());
        let mut sheet = DetailsSheet::new(tagged(), false);
        assert_eq!(
            sheet.tags.tags(),
            ["mana-dork", "elf-tribal"],
            "role tags first, each once"
        );

        let mut cx = Context::new(&mut app.shared);
        assert_eq!(
            sheet.handle_key(KeyCode::Char('x'), &mut cx),
            DetailsOutcome::Open
        );
        assert_eq!(
            sheet.handle_key(KeyCode::Tab, &mut cx),
            DetailsOutcome::Open
        );
        assert_eq!(
            sheet.handle_key(KeyCode::Tab, &mut cx),
            DetailsOutcome::Open
        );
        assert_eq!(sheet.tags.open_tag(), Some("elf-tribal"));
        assert_eq!(
            sheet.handle_key(KeyCode::Char('x'), &mut cx),
            DetailsOutcome::Open
        );
        let effects = cx.into_effects();
        assert!(
            effects
                .requests
                .contains(&screens::oracle_tags::Request::Catalog.into()),
            "Tab warms the catalog for the definitions"
        );
        assert!(matches!(
            effects.navigation.as_slice(),
            [Navigation::Push(Screen::OtagExamples(view))] if view.slug == "elf-tribal"
        ));

        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(60, 30)).unwrap();
        terminal
            .draw(|f| sheet.draw(f, &app.shared, f.area()))
            .unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(text.contains("No description yet") && text.contains("Examples"));
        assert!(text.contains("Tab tags"));

        let mut bear = card("Bear");
        bear.card_profile.card_roles.clear();
        bear.card_profile.other_oracle_tags.clear();
        let mut plain = DetailsSheet::new(bear, false);
        let mut cx = Context::new(&mut app.shared);
        assert_eq!(
            plain.handle_key(KeyCode::Tab, &mut cx),
            DetailsOutcome::Open
        );
        assert_eq!(plain.tags.open_tag(), None);
    }
}
