//! The Commanders screen's Suggested list: the merged commander ranking ([`deal`]) dealt a page at
//! a time under the screen's filter, with a row's Maybeboard and Remove, Clear skips, and Show
//! more once per deal.
//!
//! The client keeps a [`State`] while the screen is up. It sends [`Action::Load`] on mount and on
//! every change of the screen's filter, [`Action::Redeal`] when the list's Swipe select closes, and
//! the replies to the effects it performs. A filter change, a redeal and Clear skips deal again
//! from the top; Show more deals the next page, leaving out the rows already dealt, and offers
//! itself once per deal; a row that leaves tops the list back up to a page. Each deal takes a
//! fresh [`Ticket`], so only the newest one's reply lands.
//!
//! The builder profile and deck count the reasons count against, the Create check, the Swipe
//! overlay, the Clear skips dialog and which row is open stay in the client.

#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::http::contracts::suggestions::{
    HttpCommanderDeal, HttpCommanderDealPage, HttpCommanderSuggestion, RankedBy,
};

use crate::commander::deal::{self, Door, LIST_PAGE};
use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// The list's header.
pub const LIST_TITLE: &str = "Suggested";

/// The line in place of the list when its first page failed to load.
pub const LOAD_FAILED: &str = "Couldn't load your suggestions";

/// The line in place of the rows when the ranking dealt none and no filter is set.
pub const NO_SUGGESTIONS: &str = "No suggestions right now";

/// The line in place of the rows when the screen's filter matches none.
pub const NO_MATCH: &str = "No commanders match";

/// The button under the list that deals the next page.
pub const SHOW_MORE: &str = "Show more";

/// The toast once a row's Maybeboard saved it.
pub const SAVED: &str = "Added to your commander maybeboard";

/// The toast once a row's Remove skipped it.
pub const SKIPPED: &str = "Skipped for a week";

/// The toast once Clear skips went through.
pub const SKIPS_CLEARED: &str = "Skips cleared";

/// The Danger zone's row and the dialog's title.
pub const CLEAR_SKIPS: &str = "Clear skips";

/// The Clear skips dialog's line.
pub const CLEAR_SKIPS_LINE: &str =
    "Commanders you skipped will be suggested again. This can't be undone.";

/// What a user does on the list, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// The screen opened, or its filter changed to `filter`: the list deals from the top.
    Load(Box<CardQueryBuilder>),
    /// Deals again from the top under the same filter: the list's Swipe select closed, whose
    /// swipes saved and skipped commanders the list must not show any more.
    Redeal,
    /// The page asked for under `ticket` arrived, or failed (`Err`, already logged).
    Page {
        /// The ticket from the [`Effect::Deal`] this answers.
        ticket: Ticket,
        /// The page, or `Err(())` when the request failed.
        reply: Result<HttpCommanderDealPage, ()>,
    },
    /// Show more was tapped: the next page, and Show more goes until the next fresh deal.
    ShowMore,
    /// A row's Maybeboard: saves the commander.
    Maybeboard(Uuid),
    /// A row's Remove: skips the commander for a week.
    Dismiss(Uuid),
    /// The Clear skips dialog confirmed.
    ClearSkips,
    /// The save of this oracle id went through: its row leaves.
    Saved(Uuid),
    /// The skip of this oracle id went through: its row leaves.
    Dismissed(Uuid),
    /// Clear skips went through: the list deals again from the top.
    Cleared,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Asks the merged ranking for `request` and answers with [`Action::Page`].
    Deal {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The request.
        request: Box<HttpCommanderDeal>,
    },
    /// Saves this oracle id to the commander maybeboard and answers with [`Action::Saved`]; a
    /// failure raises the client's error toast and answers nothing.
    AddMaybeboard(Uuid),
    /// Skips this oracle id's commander for a week and answers with [`Action::Dismissed`]; a
    /// failure raises the client's error toast and answers nothing.
    Dismiss(Uuid),
    /// Lifts every skip and answers with [`Action::Cleared`]; a failure raises the client's error
    /// toast and answers nothing.
    ClearDismissals,
    /// Raises this toast.
    Toast(Toast),
}

/// Which page a deal out is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    /// The first page of a fresh deal, which replaces the list.
    First,
    /// A next page, which joins it.
    More,
}

/// Where the first page of the newest deal stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FirstPage {
    /// Not yet in.
    Loading,
    /// It failed with no rows to show.
    Failed,
    /// It landed, or failed while rows from before still show.
    Shown,
}

/// The Suggested list for as long as its screen is up. Starts loading, with no rows; send
/// [`Action::Load`] on mount.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    filter: CardQueryBuilder,
    entries: Vec<HttpCommanderSuggestion>,
    ranked_by: Option<RankedBy>,
    exhausted: bool,
    /// Whether Show more has dealt its page since the last fresh deal.
    more_used: bool,
    first: FirstPage,
    /// The newest deal out, if any.
    out: Option<(Ticket, Page)>,
    generation: Generation,
}

impl Default for State {
    fn default() -> Self {
        Self {
            filter: CardQueryBuilder::new(),
            entries: Vec::new(),
            ranked_by: None,
            exhausted: false,
            more_used: false,
            first: FirstPage::Loading,
            out: None,
            generation: Generation::default(),
        }
    }
}

impl State {
    /// The list as its screen first mounts: loading, no rows.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order. A page whose deal is no
    /// longer the newest changes nothing.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load(filter) => {
                self.filter = *filter;
                self.deal()
            }
            Action::Redeal | Action::Cleared => {
                let mut effects = Vec::new();
                if action == Action::Cleared {
                    effects.push(Effect::Toast(Toast::quick(ToastKind::Info, SKIPS_CLEARED)));
                }
                effects.extend(self.deal());
                effects
            }
            Action::Page { ticket, reply } => {
                self.land(ticket, reply);
                Vec::new()
            }
            Action::ShowMore => {
                if !self.offers_more() {
                    return Vec::new();
                }
                self.more_used = true;
                self.more()
            }
            Action::Maybeboard(oracle_id) => vec![Effect::AddMaybeboard(oracle_id)],
            Action::Dismiss(oracle_id) => vec![Effect::Dismiss(oracle_id)],
            Action::ClearSkips => vec![Effect::ClearDismissals],
            Action::Saved(oracle_id) => {
                let mut effects = self.drop_row(oracle_id);
                effects.push(Effect::Toast(Toast::quick(ToastKind::Success, SAVED)));
                effects
            }
            Action::Dismissed(oracle_id) => {
                let mut effects = self.drop_row(oracle_id);
                effects.push(Effect::Toast(Toast::quick(ToastKind::Info, SKIPPED)));
                effects
            }
        }
    }

    /// Deals the first page afresh under the filter; Show more is offered again.
    fn deal(&mut self) -> Vec<Effect> {
        let ticket = self.generation.next_ticket();
        self.more_used = false;
        let Some(request) = deal::next_page(Door::Suggested, &self.filter, &[], LIST_PAGE) else {
            self.out = None;
            return Vec::new();
        };
        self.out = Some((ticket, Page::First));
        vec![Effect::Deal {
            ticket,
            request: Box::new(request),
        }]
    }

    /// Deals the next page, leaving out the rows already dealt, unless a deal is out or the
    /// ranking has no more.
    fn more(&mut self) -> Vec<Effect> {
        if self.exhausted || self.out.is_some() {
            return Vec::new();
        }
        let held = deal::oracle_ids(&self.entries);
        let Some(request) = deal::next_page(Door::Suggested, &self.filter, &held, LIST_PAGE) else {
            self.exhausted = true;
            return Vec::new();
        };
        let ticket = self.generation.next_ticket();
        self.out = Some((ticket, Page::More));
        vec![Effect::Deal {
            ticket,
            request: Box::new(request),
        }]
    }

    /// Lands a page when its deal is the newest: the first replaces the list, a next one joins it.
    /// A first page that failed leaves the rows it had, failing only an empty list.
    fn land(&mut self, ticket: Ticket, reply: Result<HttpCommanderDealPage, ()>) {
        let Some((_, page)) = self.out.filter(|(out, _)| *out == ticket) else {
            return;
        };
        self.out = None;
        match (page, reply) {
            (Page::First, Ok(found)) => {
                self.exhausted = deal::is_last_page(found.suggestions.len(), LIST_PAGE);
                self.ranked_by = Some(found.ranked_by);
                self.entries = found.suggestions;
                self.first = FirstPage::Shown;
            }
            (Page::First, Err(())) => {
                self.first = if self.entries.is_empty() {
                    FirstPage::Failed
                } else {
                    FirstPage::Shown
                };
            }
            (Page::More, Ok(found)) => {
                self.exhausted = deal::is_last_page(found.suggestions.len(), LIST_PAGE);
                self.entries.extend(found.suggestions);
            }
            (Page::More, Err(())) => {}
        }
    }

    /// Takes `oracle_id`'s row off the list, topping it back up to a page when it runs short.
    fn drop_row(&mut self, oracle_id: Uuid) -> Vec<Effect> {
        self.entries
            .retain(|s| s.card.scryfall_data.oracle_id != Some(oracle_id));
        let short = u32::try_from(self.entries.len()).is_ok_and(|n| n < LIST_PAGE);
        if short { self.more() } else { Vec::new() }
    }

    /// The rows dealt so far, best fit first.
    #[must_use]
    pub fn entries(&self) -> &[HttpCommanderSuggestion] {
        &self.entries
    }

    /// The rows' cards, in order.
    #[must_use]
    pub fn cards(&self) -> Vec<Card> {
        self.entries.iter().map(|s| s.card.clone()).collect()
    }

    /// The line over the rows naming what the ranking is built from, where it says one.
    #[must_use]
    pub fn ranked_line(&self) -> Option<&'static str> {
        self.ranked_by.and_then(deal::ranked_line)
    }

    /// The line in place of the rows when the list holds none: [`NO_SUGGESTIONS`], or
    /// [`NO_MATCH`] while the filter is set.
    #[must_use]
    pub fn empty_line(&self) -> Option<&'static str> {
        if !self.entries.is_empty() {
            return None;
        }
        Some(if self.filter.is_empty() {
            NO_SUGGESTIONS
        } else {
            NO_MATCH
        })
    }

    /// Whether Show more shows under the list: it holds rows, the ranking has more to deal, and
    /// Show more has not dealt its page since the last fresh deal.
    #[must_use]
    pub fn offers_more(&self) -> bool {
        !self.entries.is_empty() && !self.exhausted && !self.more_used
    }

    /// Whether the first page is still out.
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.first == FirstPage::Loading
    }

    /// Whether a first page failed with nothing to show.
    #[must_use]
    pub fn load_failed(&self) -> bool {
        self.first == FirstPage::Failed
    }

    /// The filter the list deals under, which its Swipe starts from.
    #[must_use]
    pub fn filter(&self) -> &CardQueryBuilder {
        &self.filter
    }

    /// Swaps the printing of `picked`'s commander in place, so Create deck starts from it.
    /// Returns whether a row held that commander.
    pub fn swap_printing(&mut self, picked: &Card) -> bool {
        let target = picked.scryfall_data.oracle_id;
        if target.is_none() {
            return false;
        }
        let Some(slot) = self
            .entries
            .iter_mut()
            .find(|s| s.card.scryfall_data.oracle_id == target)
        else {
            return false;
        };
        slot.card = picked.clone();
        true
    }
}
