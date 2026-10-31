//! Expandable card rows, the app's `CardRow` list (spec 0.8): Activity's Most skipped, Playstyle's suggestions and staples, and the Commander maybeboard.
//!
//! # What a row shows
//!
//! Compact, one line: an arrow, the quantity (or the host's qty tags in its place), the name, the USD price and the color identity pips in WUBRG order. The terminal draws no art thumbnails, so the app's `ShowRowArt` toggle has nothing to switch here.
//!
//! Expanded, one row at a time, keyed by scryfall id: the card's text (cost, type line, rules, stats), its rarity and keywords, the classification (each role in `card_roles` with its tags from `oracle_tags_by_role`, then "Other tags" from `other_oracle_tags`), the open tag's definition, the host's footer lines, then the action keys `i Image`, `p Printing`, the host's extras, and `Tab Tags` when the card has tags.
//!
//! # Keys ([`CardRows::handle_key`])
//!
//! | Key | Does |
//! |---|---|
//! | j/k, Down/Up | move the focus a row (an expanded row stays open) |
//! | Enter | expand the focused row, or collapse it when it is the open one |
//! | Tab, BackTab | open the next or previous tag's definition in the expanded row; past either end closes it |
//! | x | open the examples for the open tag (`otag_examples::open`) |
//! | i | open the expanded card's image (`image_preview::open(cx, card, 0)`) |
//! | p | open the expanded card's printings to pick one (`printings::open` in `Mode::Pick`) |
//!
//! `handle_key` returns `false` for any other key, and for these when they have nothing to act on, so the host handles its own keys after it, such as the extras it listed, on [`CardRows::expanded_row`].
//!
//! # What a host does
//!
//! - Keeps a [`CardRows`] in its state and builds a [`CardRow`] for each card, borrowing it, whenever it draws or handles a key.
//! - Calls `oracle_tags::ensure_catalog` when it opens, so tag definitions have descriptions.
//! - In its arm of `Screen::on_return`, calls [`CardRows::take_printing`] with its own list, which swaps a printing picked with `p` into place (matched by oracle id) and keeps the row expanded under its new scryfall id. Nothing goes to the server.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::{Card, card_role};
use zwipe_core::domain::deck::deck_metrics;

use super::{action_bar, card_text, item_list, mana, otag};
use crate::domain::theme::Palette;
use crate::inbound::app::Context;
use crate::inbound::screens::home::cache::Catalog;
use crate::inbound::screens::printings::{self, Mode};
use crate::inbound::screens::{image_preview, otag_examples};

/// The label over the classification, as the app's.
pub(crate) const CARD_ROLES: &str = "Card roles";

/// The bucket for tags under no role, as the app's.
pub(crate) const OTHER_TAGS: &str = "Other tags";

/// One card as its host wants it shown: the quantity or tags leading the line, and what the expanded row adds.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CardRow<'card> {
    pub(crate) card: &'card Card,
    pub(crate) qty: u32,
    /// Shown in place of the quantity when not empty, such as a staple's deck share.
    pub(crate) qty_tags: Vec<String>,
    /// Lines under the expanded card, before its keys.
    pub(crate) footer: Vec<Line<'static>>,
    /// The host's own keys after Image and Printing, each a key and its label. The host handles them.
    pub(crate) actions: Vec<(String, String)>,
}

impl<'card> CardRow<'card> {
    /// A row for one copy of `card`, with nothing extra.
    pub(crate) fn new(card: &'card Card) -> Self {
        Self {
            card,
            qty: 1,
            qty_tags: Vec::new(),
            footer: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub(crate) fn with_qty(mut self, qty: u32) -> Self {
        self.qty = qty;
        self
    }

    pub(crate) fn with_qty_tags(mut self, tags: Vec<String>) -> Self {
        self.qty_tags = tags;
        self
    }

    pub(crate) fn with_footer(mut self, footer: Vec<Line<'static>>) -> Self {
        self.footer = footer;
        self
    }

    /// Adds a host key, such as `("c", "Create deck")`, to the expanded row's keys.
    pub(crate) fn with_action(mut self, key: &str, label: &str) -> Self {
        self.actions.push((key.to_owned(), label.to_owned()));
        self
    }

    fn id(&self) -> Uuid {
        self.card.scryfall_data.id
    }
}

/// The card's oracle tags in the order the expanded row shows them: each role's tags in `card_roles` order, then the other tags.
pub(crate) fn tags_of(card: &Card) -> Vec<String> {
    let profile = &card.card_profile;
    profile
        .card_roles
        .iter()
        .filter_map(|role| profile.oracle_tags_by_role.get(role))
        .flatten()
        .chain(&profile.other_oracle_tags)
        .cloned()
        .collect()
}

/// The places of the card's colors in WUBRG order.
fn pips(card: &Card) -> Vec<usize> {
    let all = Color::all();
    let mut pips: Vec<usize> = card
        .scryfall_data
        .color_identity
        .iter()
        .filter_map(|c| all.iter().position(|a| a == c))
        .collect();
    pips.sort_unstable();
    pips.dedup();
    pips
}

/// The rows' state: which is focused, which is expanded, and which tag of the expanded one has its definition open.
///
/// Rows are matched by position for the focus and by scryfall id for the expansion, so a host that reorders its list keeps the same card open.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CardRows {
    focused: usize,
    expanded: Option<Uuid>,
    /// An index into [`tags_of`] the expanded card.
    open_tag: Option<usize>,
}

impl CardRows {
    /// The focused row's place in the list.
    pub(crate) fn focused(&self) -> usize {
        self.focused
    }

    /// Focuses the row at `index`, for a host whose focus enters the rows from above.
    pub(crate) fn set_focused(&mut self, index: usize) {
        self.focused = index;
    }

    /// The scryfall id of the expanded card.
    pub(crate) fn expanded(&self) -> Option<Uuid> {
        self.expanded
    }

    /// Expands the card with this scryfall id, closing any open tag.
    pub(crate) fn expand(&mut self, id: Uuid) {
        self.expanded = Some(id);
        self.open_tag = None;
    }

    pub(crate) fn collapse(&mut self) {
        self.expanded = None;
        self.open_tag = None;
    }

    pub(crate) fn focused_row<'rows, 'card>(
        &self,
        rows: &'rows [CardRow<'card>],
    ) -> Option<&'rows CardRow<'card>> {
        rows.get(self.focused.min(rows.len().saturating_sub(1)))
    }

    pub(crate) fn expanded_row<'rows, 'card>(
        &self,
        rows: &'rows [CardRow<'card>],
    ) -> Option<&'rows CardRow<'card>> {
        let id = self.expanded?;
        rows.iter().find(|r| r.id() == id)
    }

    /// The slug of the expanded card's open tag.
    pub(crate) fn open_tag(&self, rows: &[CardRow<'_>]) -> Option<String> {
        let row = self.expanded_row(rows)?;
        tags_of(row.card).into_iter().nth(self.open_tag?)
    }

    /// Expands the focused row, or collapses it when it is the one open.
    pub(crate) fn toggle(&mut self, rows: &[CardRow<'_>]) {
        let Some(id) = self.focused_row(rows).map(CardRow::id) else {
            return;
        };
        if self.expanded == Some(id) {
            self.collapse();
        } else {
            self.expand(id);
        }
    }

    /// Moves, expands and acts on the rows; see the module doc for the keys. Returns whether the key was used.
    pub(crate) fn handle_key(
        &mut self,
        rows: &[CardRow<'_>],
        code: KeyCode,
        cx: &mut Context<'_>,
    ) -> bool {
        let last = rows.len().saturating_sub(1);
        self.focused = self.focused.min(last);
        match code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.focused = self.focused.saturating_add(1).min(last);
            }
            KeyCode::Up | KeyCode::Char('k') => self.focused = self.focused.saturating_sub(1),
            KeyCode::Enter => self.toggle(rows),
            KeyCode::Tab | KeyCode::BackTab => {
                let Some(row) = self.expanded_row(rows) else {
                    return false;
                };
                let count = tags_of(row.card).len();
                if count == 0 {
                    return false;
                }
                self.open_tag = if code == KeyCode::Tab {
                    otag::next_open(self.open_tag, count)
                } else {
                    otag::previous_open(self.open_tag, count)
                };
            }
            KeyCode::Char('x') => {
                let Some(slug) = self.open_tag(rows) else {
                    return false;
                };
                otag_examples::open(cx, &slug);
            }
            KeyCode::Char('i') => {
                let Some(row) = self.expanded_row(rows) else {
                    return false;
                };
                image_preview::open(cx, row.card.clone(), 0);
            }
            KeyCode::Char('p') => {
                let Some(row) = self.expanded_row(rows) else {
                    return false;
                };
                printings::open(cx, row.card.clone(), Mode::Pick);
            }
            _ => return false,
        }
        !rows.is_empty()
    }

    /// Takes a printing picked with `p` and swaps it into `items` in place of the card with the same oracle id, keeping that row expanded under the printing's scryfall id. `card_of` reaches the card inside one of the host's items. Returns whether a card was swapped.
    ///
    /// A host calls this from its arm of `Screen::on_return`. The handoff is emptied either way, so a printing never lands twice.
    pub(crate) fn take_printing<T>(
        &mut self,
        cx: &mut Context<'_>,
        items: &mut [T],
        mut card_of: impl FnMut(&mut T) -> &mut Card,
    ) -> bool {
        let Some(printing) = cx.shared.caches.handoff.printing.take() else {
            return false;
        };
        let Some(oracle_id) = printing.scryfall_data.oracle_id else {
            return false;
        };
        let id = printing.scryfall_data.id;
        let Some(slot) = items
            .iter_mut()
            .map(&mut card_of)
            .find(|c| c.scryfall_data.oracle_id == Some(oracle_id))
        else {
            return false;
        };
        *slot = printing;
        self.expanded = Some(id);
        true
    }

    /// One row's lines: the compact line, and the detail under it when it is the expanded one.
    pub(crate) fn row_lines(
        &self,
        palette: &Palette,
        catalog: &Catalog,
        row: &CardRow<'_>,
    ) -> Vec<Line<'static>> {
        let expanded = self.expanded == Some(row.id());
        let mut lines = vec![compact(palette, row, expanded)];
        if expanded {
            lines.extend(self.detail(palette, catalog, row));
        }
        lines
    }

    /// Every row's lines, one entry per row, for a host that lays the rows out itself.
    pub(crate) fn lines(
        &self,
        palette: &Palette,
        catalog: &Catalog,
        rows: &[CardRow<'_>],
    ) -> Vec<Vec<Line<'static>>> {
        rows.iter()
            .map(|row| self.row_lines(palette, catalog, row))
            .collect()
    }

    /// Draws the rows into `area`, paged so the focused row is in view, with a marker beside it. No rows draw nothing; the host draws its own empty state.
    pub(crate) fn draw(
        &self,
        f: &mut Frame<'_>,
        palette: &Palette,
        catalog: &Catalog,
        rows: &[CardRow<'_>],
        area: Rect,
    ) {
        let focused = self.focused.min(rows.len().saturating_sub(1));
        item_list::draw(
            f,
            palette,
            area,
            &self.lines(palette, catalog, rows),
            focused,
        );
    }

    /// The expanded card: its text, rarity and keywords, classification, open tag, the host's footer and the keys.
    fn detail(
        &self,
        palette: &Palette,
        catalog: &Catalog,
        row: &CardRow<'_>,
    ) -> Vec<Line<'static>> {
        let data = &row.card.scryfall_data;
        let mut lines = card_text::lines(palette, data);
        let mut facts = vec![Span::styled(data.rarity.to_long_name(), palette.muted)];
        if let Some(keywords) = data.keywords.as_ref().filter(|k| !k.is_empty()) {
            facts.push(Span::raw("  "));
            facts.push(Span::styled(keywords.join(", "), palette.accent));
        }
        lines.push(Line::from(facts));
        let tags = tags_of(row.card);
        lines.extend(self.classification(palette, row.card));
        if let Some(slug) = self.open_tag.and_then(|i| tags.get(i)) {
            lines.extend(otag::def_lines(palette, catalog, slug));
        }
        lines.extend(row.footer.iter().cloned());
        let mut keys: Vec<(&str, &str)> = vec![("i", "Image"), ("p", "Printing")];
        keys.extend(row.actions.iter().map(|(k, l)| (k.as_str(), l.as_str())));
        if !tags.is_empty() {
            keys.push(("Tab", "Tags"));
        }
        lines.push(action_bar::unbreakable(action_bar::line(palette, &keys)));
        lines
    }

    /// "Card roles", then a line per role with its tags as chips, then "Other tags"; nothing for a card with neither.
    fn classification(&self, palette: &Palette, card: &Card) -> Vec<Line<'static>> {
        let profile = &card.card_profile;
        if profile.card_roles.is_empty() && profile.other_oracle_tags.is_empty() {
            return Vec::new();
        }
        let mut groups: Vec<(String, &[String])> = profile
            .card_roles
            .iter()
            .map(|role| {
                let tags = profile
                    .oracle_tags_by_role
                    .get(role)
                    .map_or(&[][..], Vec::as_slice);
                (card_role::role_label(role), tags)
            })
            .collect();
        if !profile.other_oracle_tags.is_empty() {
            groups.push((OTHER_TAGS.to_owned(), &profile.other_oracle_tags));
        }
        let mut lines = vec![Line::styled(CARD_ROLES, palette.muted)];
        let mut index = 0_usize;
        for (label, tags) in groups {
            let mut spans = vec![Span::raw(label).bold()];
            if !tags.is_empty() {
                spans.push(Span::raw(":"));
            }
            for tag in tags {
                spans.push(Span::raw(" "));
                spans.push(otag::chip(palette, tag, self.open_tag == Some(index)));
                index = index.saturating_add(1);
            }
            lines.push(Line::from(spans));
        }
        lines
    }
}

/// The compact line: arrow, quantity or qty tags, name, price and pips.
fn compact(palette: &Palette, row: &CardRow<'_>, expanded: bool) -> Line<'static> {
    let data = &row.card.scryfall_data;
    let arrow = if expanded { "▾ " } else { "▸ " };
    let mut spans = vec![Span::styled(arrow, palette.muted)];
    if row.qty_tags.is_empty() {
        spans.push(Span::styled(format!("{} ", row.qty), palette.muted));
    } else {
        for tag in &row.qty_tags {
            spans.push(Span::styled(tag.clone(), palette.accent.bold()));
            spans.push(Span::raw(" "));
        }
    }
    spans.push(Span::raw(data.name.clone()).bold());
    if let Some(price) = deck_metrics::card_price(data, PriceCurrency::Usd) {
        spans.push(Span::styled(
            format!("  {}", PriceCurrency::Usd.format_amount(price)),
            palette.muted,
        ));
    }
    let pips = pips(row.card);
    if !pips.is_empty() {
        spans.push(Span::raw("  "));
        spans.extend(pips.into_iter().map(|pip| mana::pip(palette, pip)));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::inbound::event_loop::harness::Harness;
    use crate::inbound::screens::Screen;
    use crate::test_support::card;

    /// A green-blue card with two roles, one holding tags, and one other tag.
    fn classified(name: &str) -> Card {
        let mut card = card(name);
        card.scryfall_data.color_identity = vec![Color::Green, Color::Blue].into();
        card.scryfall_data.keywords = Some(vec!["Flying".to_owned()]);
        let profile = &mut card.card_profile;
        profile.card_roles = vec!["ramp".to_owned(), "card_draw".to_owned()];
        profile.oracle_tags_by_role = BTreeMap::from([(
            "ramp".to_owned(),
            vec!["mana-dork".to_owned(), "mana-ability".to_owned()],
        )]);
        profile.other_oracle_tags = vec!["elf-tribal".to_owned()];
        card
    }

    fn text(lines: &[Line<'static>]) -> String {
        lines
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            .replace('\u{a0}', " ")
    }

    fn screen(state: &CardRows, rows: &[CardRow<'_>], width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|f| {
                state.draw(f, &Palette::terminal(), &Catalog::Idle, rows, f.area());
            })
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect()
    }

    /// Presses `codes` on `state` over home, returning what each did.
    fn press(
        harness: &mut Harness,
        state: &mut CardRows,
        rows: &[CardRow<'_>],
        codes: &[KeyCode],
    ) -> Vec<bool> {
        let mut used = Vec::new();
        harness.open(|cx| {
            for &code in codes {
                used.push(state.handle_key(rows, code, cx));
            }
        });
        used
    }

    #[test]
    fn tags_run_in_role_order_then_the_others() {
        assert_eq!(
            tags_of(&classified("Elves")),
            ["mana-dork", "mana-ability", "elf-tribal"]
        );
        let mut plain = card("Plain");
        plain.card_profile.card_roles.clear();
        plain.card_profile.other_oracle_tags.clear();
        assert!(tags_of(&plain).is_empty(), "an unclassified card");
    }

    #[test]
    fn a_compact_row_shows_qty_name_price_and_pips() {
        let palette = Palette::terminal();
        let elves = classified("Llanowar Elves");
        let price = deck_metrics::card_price(&elves.scryfall_data, PriceCurrency::Usd)
            .map(|p| format!("  {}", PriceCurrency::Usd.format_amount(p)))
            .unwrap_or_default();
        let line = compact(&palette, &CardRow::new(&elves).with_qty(3), false);
        assert_eq!(line.to_string(), format!("▸ 3 Llanowar Elves{price}  UG"));
        let tagged = CardRow::new(&elves).with_qty_tags(vec!["42%".to_owned()]);
        assert!(
            compact(&palette, &tagged, true)
                .to_string()
                .starts_with("▾ 42% Llanowar")
        );
    }

    #[test]
    fn enter_expands_one_row_at_a_time_with_its_detail() {
        let cards = [classified("Llanowar Elves"), card("Sol Ring")];
        let rows: Vec<CardRow<'_>> = cards
            .iter()
            .map(|c| {
                CardRow::new(c)
                    .with_footer(vec![Line::from("Skipped under 3 commanders")])
                    .with_action("c", "Create deck")
            })
            .collect();
        let mut harness = Harness::on(Screen::Home(Box::default()));
        let mut state = CardRows::default();
        press(&mut harness, &mut state, &rows, &[KeyCode::Enter]);
        assert_eq!(state.expanded(), Some(cards[0].scryfall_data.id));
        let detail = text(&state.row_lines(&Palette::terminal(), &Catalog::Idle, &rows[0]));
        for expected in [
            "Common  Flying",
            "Card roles",
            "Ramp:  mana-dork   mana-ability ",
            "Card Draw",
            "Other tags:  elf-tribal ",
            "Skipped under 3 commanders\ni Image",
            "i Image  p Printing  c Create deck  Tab Tags",
        ] {
            assert!(detail.contains(expected), "missing {expected} in\n{detail}");
        }
        press(
            &mut harness,
            &mut state,
            &rows,
            &[KeyCode::Char('j'), KeyCode::Enter],
        );
        assert_eq!(state.expanded(), Some(cards[1].scryfall_data.id));
        assert_eq!(state.focused(), 1);
        press(&mut harness, &mut state, &rows, &[KeyCode::Enter]);
        assert_eq!(state.expanded(), None, "Enter on the open row closes it");
        press(
            &mut harness,
            &mut state,
            &rows,
            &[KeyCode::Down, KeyCode::Down],
        );
        assert_eq!(state.focused(), 1, "the focus stops at the last row");
    }

    #[test]
    fn tab_opens_each_tag_and_x_shows_its_examples() {
        let elves = classified("Llanowar Elves");
        let rows = [CardRow::new(&elves)];
        let mut harness = Harness::on(Screen::Home(Box::default()));
        let mut state = CardRows::default();
        assert_eq!(
            press(
                &mut harness,
                &mut state,
                &rows,
                &[KeyCode::Tab, KeyCode::Char('x')]
            ),
            [false, false],
            "nothing expanded, nothing to act on"
        );
        press(
            &mut harness,
            &mut state,
            &rows,
            &[KeyCode::Enter, KeyCode::Tab, KeyCode::Tab],
        );
        assert_eq!(state.open_tag(&rows).as_deref(), Some("mana-ability"));
        let detail = text(&state.row_lines(&Palette::terminal(), &Catalog::Idle, &rows[0]));
        assert!(detail.contains("No description yet\nx Examples"));
        press(
            &mut harness,
            &mut state,
            &rows,
            &[KeyCode::BackTab, KeyCode::BackTab],
        );
        assert_eq!(state.open_tag(&rows), None, "past the first closes it");
        press(
            &mut harness,
            &mut state,
            &rows,
            &[KeyCode::BackTab, KeyCode::Char('x')],
        );
        let Screen::OtagExamples(view) = harness.screen() else {
            panic!("no examples");
        };
        assert_eq!(view.slug, "elf-tribal");
    }

    #[test]
    fn i_and_p_open_the_expanded_card() {
        let elves = classified("Llanowar Elves");
        let rows = [CardRow::new(&elves)];
        let mut harness = Harness::on(Screen::Home(Box::default()));
        let mut state = CardRows::default();
        press(
            &mut harness,
            &mut state,
            &rows,
            &[KeyCode::Enter, KeyCode::Char('i')],
        );
        assert!(matches!(harness.screen(), Screen::ImagePreview(_)));
        harness.press(KeyCode::Esc);
        press(&mut harness, &mut state, &rows, &[KeyCode::Char('p')]);
        let Screen::Printings(view) = harness.screen() else {
            panic!("no printings");
        };
        assert_eq!(view.mode, Mode::Pick);
        assert_eq!(view.card.scryfall_data.id, elves.scryfall_data.id);
        assert_eq!(
            press(&mut harness, &mut state, &rows, &[KeyCode::Char('?')]),
            [false],
            "a key the rows do not use is the host's"
        );
    }

    #[test]
    fn a_picked_printing_swaps_in_and_stays_open() {
        let mut items = vec![(1, card("Sol Ring")), (2, card("Llanowar Elves"))];
        let mut printing = card("Llanowar Elves");
        printing.scryfall_data.oracle_id = items[1].1.scryfall_data.oracle_id;
        let mut state = CardRows::default();
        state.expand(items[1].1.scryfall_data.id);
        let mut harness = Harness::on(Screen::Home(Box::default()));
        harness.app.shared.caches.handoff.printing = Some(printing.clone());
        let mut swapped = Vec::new();
        harness.open(|cx| {
            swapped.push(state.take_printing(cx, &mut items, |item| &mut item.1));
            swapped.push(state.take_printing(cx, &mut items, |item| &mut item.1));
        });
        assert_eq!(swapped, [true, false], "a printing lands once");
        assert_eq!(items[1].1, printing);
        assert_eq!(state.expanded(), Some(printing.scryfall_data.id));
        assert!(harness.app.shared.caches.handoff.printing.is_none());
    }

    #[test]
    fn a_printing_for_a_card_not_in_the_list_is_dropped() {
        let mut items = vec![card("Sol Ring")];
        let mut state = CardRows::default();
        let mut harness = Harness::on(Screen::Home(Box::default()));
        harness.app.shared.caches.handoff.printing = Some(card("Stray"));
        let mut swapped = true;
        harness.open(|cx| swapped = state.take_printing(cx, &mut items, |c| c));
        assert!(!swapped);
        assert_eq!(state.expanded(), None);
        assert!(harness.app.shared.caches.handoff.printing.is_none());
    }

    #[test]
    fn rows_draw_paged_with_the_focus_marked_and_fit_tiny_areas() {
        let cards: Vec<Card> = (0..30).map(|i| card(&format!("Card {i}"))).collect();
        let rows: Vec<CardRow<'_>> = cards.iter().map(CardRow::new).collect();
        let mut state = CardRows::default();
        let mut harness = Harness::on(Screen::Home(Box::default()));
        press(&mut harness, &mut state, &rows, &[KeyCode::Char('j'); 12]);
        let drawn = screen(&state, &rows, 40, 10);
        assert!(drawn.contains("> ▸ 1 Card 12"), "{drawn}");
        assert!(!drawn.contains("Card 0 "));
        screen(&state, &rows, 8, 3);
        screen(&state, &rows, 1, 1);
        screen(&state, &[], 20, 5);
        press(&mut harness, &mut state, &rows, &[KeyCode::Enter]);
        screen(&state, &rows, 8, 3);
    }
}
