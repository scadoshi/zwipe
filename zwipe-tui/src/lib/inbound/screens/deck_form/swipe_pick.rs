//! The swipe picker for a command-zone slot: one card at a time in popularity order, right to pick it, left to skip, up to save it to the commander maybeboard, down to undo.

use std::time::Instant;

use crossterm::event::KeyCode;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::Format;

use super::{DeckForm, Overlay, Request};
use crate::domain::deck::form::{self, SWIPE_PAGE, Slot};
use crate::domain::filter;
use crate::domain::stack::Direction;
use crate::inbound::app::Context;
use crate::inbound::screens::card_sheets;
use crate::inbound::screens::card_sheets::filter::{FilterSheet, SheetConfig, SheetOutcome};
use crate::inbound::widgets::card_motion::CardMotion;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::info::InfoPopup;
use crate::inbound::widgets::toast::ToastKind;

/// The key the server stores the swipe picker's hint under.
pub(crate) const HINT_KEY: &str = "swipe_select";

/// A new page is fetched once this many cards or fewer are left.
const PREFETCH_AT: usize = 5;

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
    /// Counts refreshes so a page for an older pile is dropped.
    pub(crate) generation: u64,
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
            generation: 0,
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
        if self.loading || self.exhausted || !self.can_search() || left > PREFETCH_AT {
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
                generation: self.generation,
                query: Box::new(query),
            }
            .into(),
        );
    }

    /// Takes a page: new cards join the pile, and an empty page ends it.
    pub(crate) fn take_page(&mut self, generation: u64, cards: Vec<Card>, cx: &mut Context<'_>) {
        if generation != self.generation {
            return;
        }
        self.loading = false;
        self.offset = self.offset.saturating_add(SWIPE_PAGE);
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
        self.generation = self.generation.saturating_add(1);
        self.cards.clear();
        self.index = 0;
        self.history.clear();
        self.motion = None;
        self.offset = 0;
        self.loading = false;
        self.exhausted = false;
        self.fetch_more(cx);
    }

    /// The hint's bullets, worded for this slot.
    pub(crate) fn hint_body(&self) -> String {
        let noun = self.noun();
        format!(
            "• Swipe right picks that {noun}\n• Swipe left skips it\n• Swipe up saves it to your commander maybeboard\n• Swipe down undoes the last swipe\nMost-played first. Filter narrows by color or anything else"
        )
    }
}

/// Opens the swipe picker for `slot` and deals the first page, showing its hint once.
pub(crate) fn open(form: &mut DeckForm, slot: Slot, cx: &mut Context<'_>) {
    let mut picker = SwipePicker::new(form, slot);
    picker.fetch_more(cx);
    let body = picker.hint_body();
    form.overlay = Some(Overlay::Swipe(Box::new(picker)));
    hint::show(cx, HINT_KEY, "Swipe select", &body);
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
            return Some(Overlay::Info(InfoPopup::new(
                "Swipe select",
                &picker
                    .hint_body()
                    .lines()
                    .map(|l| l.trim_start_matches("• "))
                    .collect::<Vec<_>>(),
            )));
        }
        KeyCode::F(5) => {
            picker.refresh(cx);
            cx.toast(ToastKind::Info, "Stack refreshed".to_owned());
        }
        KeyCode::F(3) => picker.open_filter(cx),
        KeyCode::Right | KeyCode::Char('l') => {
            if let Some(card) = picker.current().cloned() {
                cx.toast(
                    ToastKind::Success,
                    format!("{} selected", capitalized(picker.noun())),
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
                cx.toast(ToastKind::Info, "Skipped".to_owned());
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
                    Swiped::Skipped => cx.toast(ToastKind::Info, "Undid skip".to_owned()),
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
