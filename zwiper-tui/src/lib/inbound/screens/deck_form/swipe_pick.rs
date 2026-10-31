//! The swipe picker for a command-zone slot: one card at a time in popularity order, right to pick it, left to skip, up to save it to the commander maybeboard, down to undo.

use std::time::Instant;

use crossterm::event::KeyCode;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::Format;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::deck::form::{self, Slot};
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::generation::{Generation, Ticket};
use zwipe_ui::hint::{Block, HintContent, SwipeSelectMode, one_time};
use zwipe_ui::stack::Direction;
use zwipe_ui::{filter, timing};

use super::{DeckForm, Overlay, Request};
use crate::inbound::app::Context;
use crate::inbound::screens::card_sheets;
use crate::inbound::screens::card_sheets::filter::{FilterSheet, SheetOutcome};
use crate::inbound::widgets::card_motion::CardMotion;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::info::InfoPopup;
use crate::inbound::widgets::toast::{self, ToastKind};

/// What a swipe did, so down can undo it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Swiped {
    Skipped,
    /// Saved to the commander maybeboard, by oracle id.
    Saved(uuid::Uuid),
}

/// The swipe picker's pile and where it is in it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SwipePicker {
    pub(crate) slot: Slot,
    pub(crate) format: Option<Format>,
    /// The commander's card, for a signature spell's colors.
    pub(crate) commander: Option<Card>,
    pub(crate) cards: Vec<Card>,
    /// The card up now.
    pub(crate) index: usize,
    pub(crate) history: Vec<Swiped>,
    /// Where the next page starts.
    pub(crate) offset: u32,
    pub(crate) loading: bool,
    /// An empty page came back, so there are no more.
    pub(crate) exhausted: bool,
    /// The pile generation: each refresh takes a new ticket, so a page for an older pile is dropped. A next page continues it.
    pub(crate) generation: Generation,
    /// The user's filter the slot's rule goes on top of.
    pub(crate) filter: CardQueryBuilder,
    /// The card filter sheet, while it is open over the pile.
    pub(crate) sheet: Option<FilterSheet>,
    /// The swiped card flying off, or the undone one coming back. A pick closes the picker, so it has no exit.
    pub(crate) motion: Option<CardMotion<Card>>,
}

impl SwipePicker {
    /// The picker for `slot`, with nothing loaded yet and the slot's last applied filter.
    pub(crate) fn new(form: &DeckForm, slot: Slot) -> Self {
        Self {
            slot,
            format: form.format,
            commander: form.commander().cloned(),
            cards: Vec::new(),
            index: 0,
            history: Vec::new(),
            offset: 0,
            loading: false,
            exhausted: false,
            generation: Generation::default(),
            filter: form.card(slot).swipe_filter.clone(),
            sheet: None,
            motion: None,
        }
    }

    /// Whether a card is in motion, so the loop redraws.
    pub(crate) fn animating(&self) -> bool {
        self.motion
            .as_ref()
            .is_some_and(|m| m.running(Instant::now()))
    }

    /// The card under the one up now, which a card coming back after an undo covers.
    pub(crate) fn under(&self) -> Option<&Card> {
        self.cards.get(self.index.checked_add(1)?)
    }

    /// Sends `card` off by `direction`, replacing any motion still running.
    fn exit(&mut self, direction: Direction, card: Card) {
        self.motion = Some(CardMotion::exit(direction, card, Instant::now()));
    }

    /// Whether the Filter key shows its dot: any user filter or an explicit sort.
    pub(crate) fn filter_dot(&self) -> bool {
        filter::shows_dot(&self.filter)
    }

    /// Opens the card filter sheet over the picker's filter, without the format section.
    pub(crate) fn open_filter(&mut self, cx: &mut Context<'_>) {
        let config = SheetConfig {
            show_format: false,
            ..SheetConfig::default()
        };
        self.sheet = Some(FilterSheet::open(cx, &self.filter, config));
    }

    /// Whether the slot has what it needs to search: a format for the commander, a commander for the signature spell.
    pub(crate) fn can_search(&self) -> bool {
        match self.slot {
            Slot::Commander => self.format.is_some(),
            Slot::SignatureSpell => self.commander.is_some(),
            Slot::Partner | Slot::Background => true,
        }
    }

    /// The card up now.
    pub(crate) fn current(&self) -> Option<&Card> {
        self.cards.get(self.index)
    }

    /// What the picker calls its card, such as "commander".
    pub(crate) fn noun(&self) -> &'static str {
        self.slot.noun(self.format)
    }

    /// Asks for the next page when the slot can search and the pile is running low.
    pub(crate) fn fetch_more(&mut self, cx: &mut Context<'_>) {
        let left = self.cards.len().saturating_sub(self.index);
        if self.loading || self.exhausted || !self.can_search() || left > timing::LOAD_MORE_AT {
            return;
        }
        let Ok(query) = form::swipe_query(
            &self.filter,
            self.slot,
            self.format,
            self.commander.as_ref(),
            self.offset,
        ) else {
            return;
        };
        self.loading = true;
        cx.request(
            Request::SwipePage {
                generation: self.generation.current(),
                query: Box::new(query),
            }
            .into(),
        );
    }

    /// Takes a page: new cards join the pile, and an empty page ends it.
    pub(crate) fn take_page(&mut self, generation: Ticket, cards: Vec<Card>, cx: &mut Context<'_>) {
        if !self.generation.is_current(generation) {
            return;
        }
        self.loading = false;
        self.offset = self.offset.saturating_add(timing::PAGE_SIZE);
        if cards.is_empty() {
            self.exhausted = true;
            return;
        }
        for card in cards {
            if !self
                .cards
                .iter()
                .any(|c| c.scryfall_data.id == card.scryfall_data.id)
            {
                self.cards.push(card);
            }
        }
        self.fetch_more(cx);
    }

    /// Starts the pile over from the top.
    pub(crate) fn refresh(&mut self, cx: &mut Context<'_>) {
        self.generation.next_ticket();
        self.cards.clear();
        self.index = 0;
        self.history.clear();
        self.motion = None;
        self.offset = 0;
        self.loading = false;
        self.exhausted = false;
        self.fetch_more(cx);
    }

    /// The app's Swipe select hint, worded for this slot.
    pub(crate) fn hint(&self) -> HintContent {
        one_time::swipe_select(SwipeSelectMode::Picking(self.noun()))
    }
}

/// Opens the swipe picker for `slot` and deals the first page, showing its hint once.
pub(crate) fn open(form: &mut DeckForm, slot: Slot, cx: &mut Context<'_>) {
    let mut picker = SwipePicker::new(form, slot);
    picker.fetch_more(cx);
    let content = picker.hint();
    form.overlay = Some(Overlay::Swipe(Box::new(picker)));
    hint::once_with(cx, OneTimeHint::SwipeSelect.key(), content);
}

/// The noun with its first letter capitalized, for "Commander selected".
fn capitalized(noun: &str) -> String {
    let mut chars = noun.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// Keys in the swipe picker. Returns the picker while it stays open.
pub(crate) fn handle(
    form: &mut DeckForm,
    mut picker: Box<SwipePicker>,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<Overlay> {
    if let Some(sheet) = &mut picker.sheet {
        match card_sheets::filter::keys::handle(sheet, code, cx) {
            SheetOutcome::Open => {}
            SheetOutcome::Closed => picker.sheet = None,
            SheetOutcome::Applied(filter) => {
                picker.sheet = None;
                form.card_mut(picker.slot).swipe_filter = (*filter).clone();
                picker.filter = *filter;
                picker.refresh(cx);
            }
        }
        return Some(Overlay::Swipe(picker));
    }
    match code {
        KeyCode::Esc => return None,
        KeyCode::F(6) => {
            let lines: Vec<String> = picker.hint().blocks.iter().map(Block::plain).collect();
            let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
            return Some(Overlay::Info(InfoPopup::new("Swipe select", &lines)));
        }
        KeyCode::F(5) => {
            picker.refresh(cx);
            cx.toast_for(ToastKind::Info, "Stack refreshed".to_owned(), toast::QUICK);
        }
        KeyCode::F(3) => picker.open_filter(cx),
        KeyCode::Right | KeyCode::Char('l') => {
            if let Some(card) = picker.current().cloned() {
                cx.toast_for(
                    ToastKind::Success,
                    format!("{} selected", capitalized(picker.noun())),
                    toast::QUICK,
                );
                form.pick_card(picker.slot, card, cx);
                return None;
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if let Some(card) = picker.current().cloned() {
                picker.exit(Direction::Left, card);
                picker.index = picker.index.saturating_add(1);
                picker.history.push(Swiped::Skipped);
                cx.toast_for(ToastKind::Info, "Skipped".to_owned(), toast::QUICK);
                picker.fetch_more(cx);
            }
        }
        KeyCode::Up | KeyCode::Char('k')
            if matches!(picker.slot, Slot::Commander | Slot::Partner) =>
        {
            if let Some(card) = picker.current().cloned() {
                match card.scryfall_data.oracle_id {
                    Some(oracle_id) => {
                        picker.exit(Direction::Up, card);
                        cx.request(Request::MaybeboardAdd(oracle_id).into());
                        picker.index = picker.index.saturating_add(1);
                        picker.history.push(Swiped::Saved(oracle_id));
                        picker.fetch_more(cx);
                    }
                    None => cx.toast(ToastKind::Error, "This card can't be saved".to_owned()),
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => match picker.history.pop() {
            None => cx.toast(ToastKind::Info, "Nothing to undo".to_owned()),
            Some(swiped) => {
                picker.index = picker.index.saturating_sub(1);
                let side = match swiped {
                    Swiped::Skipped => Direction::Left,
                    Swiped::Saved(_) => Direction::Up,
                };
                picker.motion = Some(CardMotion::enter(side, Instant::now()));
                match swiped {
                    Swiped::Skipped => {
                        cx.toast_for(ToastKind::Info, "Undid skip".to_owned(), toast::QUICK);
                    }
                    Swiped::Saved(oracle_id) => {
                        cx.request(Request::MaybeboardRemove(oracle_id).into());
                    }
                }
            }
        },
        _ => {}
    }
    Some(Overlay::Swipe(picker))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nouns_capitalize_for_the_selected_toast() {
        assert_eq!(capitalized("signature spell"), "Signature spell");
        assert_eq!(capitalized(""), "");
    }
}
