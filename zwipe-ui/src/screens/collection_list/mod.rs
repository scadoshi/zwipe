//! Collection's list: the user's collection a page at a time, searched by card name, each row's
//! copies changed with a stepper, and the whole collection exported as a ManaBox file.
//!
//! Pages come in order, one out at a time, the next asked for as the list nears its end; a
//! newer search starts over from the first page and drops any page still out. Typing searches
//! once it rests for [`timing::QUICK_ADD_DEBOUNCE`].
//!
//! A step shows the new count at once and saves the row once steps rest for
//! [`timing::QUANTITY_FLUSH`], so a burst of taps is one write. A row has one write out at a
//! time; a step made meanwhile goes when it lands. 0 removes the row once the write lands, and the
//! next page starts one row earlier for it; a page already out when a row goes is asked again
//! from there. A failed write puts the saved count back and replays the steps made since. Edits are kept apart from the pages, so a new search
//! never drops one still waiting to save. A row holding a finish, condition or language this
//! build doesn't know can't be edited, since the server refuses those values.
//!
//! The model holds the list and the edits; the client fetches, saves, runs the timers, draws the
//! rows and saves the export's file. Import is its own model, [`super::collection_import`].

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::time::Duration;

use uuid::Uuid;
use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionQuantity, CollectionSearch, Finish, SetQuantity,
};
use zwipe_core::http::contracts::collection::{
    CollectionCard, HttpCollectionExport, HttpCollectionPage, HttpSetCollectionCard,
    HttpSetCollectionCardReply,
};

use crate::generation::{Generation, Ticket};
use crate::timing;
use crate::toast::{Toast, ToastKind};

/// The line for a collection with no cards.
pub const EMPTY: &str = "No cards in your collection yet. Import a ManaBox or Archidekt export.";

/// The line for a search no card matches.
pub const NO_MATCH: &str = "No cards in your collection match";

/// The line when the first page fails.
pub const FAILED: &str = "Couldn't load your collection";

/// The toast when a later page fails; scrolling on asks again.
pub const MORE_FAILED: &str = "Couldn't load more cards";

/// The toast when a row's write fails and its count goes back.
pub const SAVE_FAILED: &str = "Couldn't save that change";

/// The toast when the export fails.
pub const EXPORT_FAILED: &str = "Couldn't export your collection";

/// What the list shows.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The first page is out.
    #[default]
    Loading,
    /// The rows.
    List,
    /// The collection has no cards: [`EMPTY`].
    Empty,
    /// The search matches nothing: [`NO_MATCH`].
    NoMatch,
    /// The first page failed: [`FAILED`], with Retry.
    Failed,
}

/// One row of the collection, as a write names it: a printing in one finish, condition and
/// language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowKey {
    /// The printing's Scryfall id.
    pub printing: Uuid,
    /// The finish.
    pub finish: Finish,
    /// The condition, `None` when not given.
    pub condition: Option<CardCondition>,
    /// The language, `None` when not given.
    pub language: Option<CardLanguage>,
}

impl RowKey {
    /// The key of `card`'s row.
    #[must_use]
    pub fn of(card: &CollectionCard) -> Self {
        Self {
            printing: card.card.scryfall_data.id,
            finish: card.finish,
            condition: card.condition,
            language: card.language,
        }
    }

    /// Whether every value is one the server takes back: none is this build's `Unknown`.
    #[must_use]
    pub fn editable(self) -> bool {
        self.finish != Finish::Unknown
            && self.condition != Some(CardCondition::Unknown)
            && self.language != Some(CardLanguage::Unknown)
    }
}

/// A row as the list shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// What a write names.
    pub key: RowKey,
    /// The row as the server last sent it.
    pub card: CollectionCard,
    /// The copies shown: the stepper's count, saved or not.
    pub copies: i32,
    /// Whether the stepper works on it ([`RowKey::editable`]).
    pub editable: bool,
}

/// Every tap, key and reply the list takes.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// The screen opened: loads the first page.
    Open,
    /// A page arrived for the fetch holding `ticket`.
    PageLoaded {
        /// The [`Effect::FetchPage`] ticket.
        ticket: Ticket,
        /// The page.
        page: Box<HttpCollectionPage>,
    },
    /// The fetch holding `ticket` failed.
    PageFailed {
        /// The [`Effect::FetchPage`] ticket.
        ticket: Ticket,
    },
    /// The list is near its end: the next page, if there is one.
    NearEnd,
    /// Retry after the first page failed.
    Retry,
    /// The search box now holds this text.
    Type(String),
    /// A search's wait is over; the [`Effect::SearchAfter`] ticket.
    Search(Ticket),
    /// The stepper on a row: `delta` copies, +1 or -1 for a tap.
    Step {
        /// The row.
        key: RowKey,
        /// The change.
        delta: i32,
    },
    /// A row's steps have rested; the [`Effect::FlushAfter`] ticket.
    Flush(Ticket),
    /// The write holding `ticket` landed.
    Saved {
        /// The [`Effect::SetCopies`] ticket.
        ticket: Ticket,
        /// The row as it is now.
        reply: Box<HttpSetCollectionCardReply>,
    },
    /// The write holding `ticket` failed.
    SaveFailed {
        /// The [`Effect::SetCopies`] ticket.
        ticket: Ticket,
    },
    /// The Export button.
    Export,
    /// The export holding `ticket` arrived.
    Exported {
        /// The [`Effect::Export`] ticket.
        ticket: Ticket,
        /// The file.
        export: HttpCollectionExport,
    },
    /// The export holding `ticket` failed.
    ExportFailed {
        /// The [`Effect::Export`] ticket.
        ticket: Ticket,
    },
}

/// What an action asks the client to do, in order.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Fetches the page from `offset` for `search`; answers with [`Action::PageLoaded`] or
    /// [`Action::PageFailed`] carrying `ticket`.
    FetchPage {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The search, empty for every row.
        search: CollectionSearch,
        /// Rows before the page.
        offset: u32,
    },
    /// Waits `wait`, then answers with [`Action::Search`] carrying `ticket`.
    SearchAfter {
        /// The ticket to hand back.
        ticket: Ticket,
        /// [`timing::QUICK_ADD_DEBOUNCE`].
        wait: Duration,
    },
    /// Waits `wait`, then answers with [`Action::Flush`] carrying `ticket`.
    FlushAfter {
        /// The ticket to hand back.
        ticket: Ticket,
        /// [`timing::QUANTITY_FLUSH`].
        wait: Duration,
    },
    /// Gives a row its copies; answers with [`Action::Saved`] or [`Action::SaveFailed`]
    /// carrying `ticket`.
    SetCopies {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The row and its copies.
        set: HttpSetCollectionCard,
    },
    /// The collection changed: the client drops what it kept of it (the summary that decides
    /// whether "In my collection" shows).
    CollectionChanged,
    /// Fetches the export; answers with [`Action::Exported`] or [`Action::ExportFailed`]
    /// carrying `ticket`.
    Export {
        /// The ticket to hand back.
        ticket: Ticket,
    },
    /// Saves a file for the user.
    Download {
        /// Its name.
        file_name: String,
        /// Its text.
        text: String,
    },
    /// Raises this toast.
    Toast(Toast),
}

/// A row's change not yet saved, or being saved.
#[derive(Debug, Clone, Copy)]
struct Edit {
    /// The count the user asked for.
    shown: i32,
    /// The count the server holds.
    saved: i32,
    /// The rest before the next write.
    wait: Option<Ticket>,
    /// The write out.
    writing: Option<Ticket>,
    /// The count the write out sent, to replay the steps made since if it fails.
    written: i32,
}

/// The list for one visit to Collection.
#[derive(Debug, Clone, Default)]
pub struct State {
    status: Status,
    rows: Vec<Row>,
    total: i64,
    next_offset: Option<u32>,
    /// The page out, with nothing else asked until it settles.
    page_out: Option<Ticket>,
    /// The offset the page out was asked from.
    page_offset: u32,
    /// Rows removed since the page out was asked for, which shift its rows on the server.
    removed_while_out: u32,
    /// The box's text.
    text: String,
    /// The search the pages are for.
    search: CollectionSearch,
    search_wait: Option<Ticket>,
    edits: HashMap<RowKey, Edit>,
    exporting: Option<Ticket>,
    tickets: Generation,
}

impl State {
    /// A list not yet opened.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// What the list shows.
    #[must_use]
    pub fn status(&self) -> Status {
        self.status
    }

    /// The rows loaded so far, in the server's order.
    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// Rows the search matches across every page.
    #[must_use]
    pub fn total(&self) -> i64 {
        self.total
    }

    /// The search box's text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether an export is out, which disables Export.
    #[must_use]
    pub fn exporting(&self) -> bool {
        self.exporting.is_some()
    }

    /// Takes one action and answers with what the client should do.
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Open => self.restart(),
            Action::PageLoaded { ticket, page } => {
                if self.page_out != Some(ticket) {
                    return Vec::new();
                }
                if self.removed_while_out > 0 {
                    // The server may have cut the page before or after a removal; asked again
                    // now, from where the removals leave it, it can't skip or repeat a row.
                    let offset = self.page_offset.saturating_sub(self.removed_while_out);
                    return vec![self.fetch(offset)];
                }
                self.land(*page);
                Vec::new()
            }
            Action::PageFailed { ticket } => self.page_failed(ticket),
            Action::NearEnd => match self.next_offset {
                Some(offset) if self.page_out.is_none() && self.status == Status::List => {
                    vec![self.fetch(offset)]
                }
                Some(_) | None => Vec::new(),
            },
            Action::Retry => {
                if self.status == Status::Failed {
                    self.restart()
                } else {
                    Vec::new()
                }
            }
            Action::Type(text) => {
                self.text = text;
                let ticket = self.tickets.next_ticket();
                self.search_wait = Some(ticket);
                vec![Effect::SearchAfter {
                    ticket,
                    wait: timing::QUICK_ADD_DEBOUNCE,
                }]
            }
            Action::Search(ticket) => self.search(ticket),
            Action::Step { key, delta } => self.step(key, delta),
            Action::Flush(ticket) => self.flush(ticket),
            Action::Saved { ticket, reply } => self.saved(ticket, &reply),
            Action::SaveFailed { ticket } => self.save_failed(ticket),
            Action::Export => {
                if self.exporting.is_some() {
                    return Vec::new();
                }
                let ticket = self.tickets.next_ticket();
                self.exporting = Some(ticket);
                vec![Effect::Export { ticket }]
            }
            Action::Exported { ticket, export } => {
                if self.exporting != Some(ticket) {
                    return Vec::new();
                }
                self.exporting = None;
                vec![Effect::Download {
                    file_name: export.file_name,
                    text: export.csv,
                }]
            }
            Action::ExportFailed { ticket } => {
                if self.exporting != Some(ticket) {
                    return Vec::new();
                }
                self.exporting = None;
                vec![Effect::Toast(Toast::normal(
                    ToastKind::Error,
                    EXPORT_FAILED,
                ))]
            }
        }
    }

    /// Drops the pages and asks for the first one for the current search.
    fn restart(&mut self) -> Vec<Effect> {
        self.rows.clear();
        self.total = 0;
        self.next_offset = None;
        self.status = Status::Loading;
        vec![self.fetch(0)]
    }

    fn fetch(&mut self, offset: u32) -> Effect {
        let ticket = self.tickets.next_ticket();
        self.page_out = Some(ticket);
        self.page_offset = offset;
        self.removed_while_out = 0;
        Effect::FetchPage {
            ticket,
            search: self.search.clone(),
            offset,
        }
    }

    /// Adds a page's rows, each showing a count waiting to save over the server's.
    fn land(&mut self, page: HttpCollectionPage) {
        self.page_out = None;
        self.total = page.total;
        self.next_offset = page.next_offset;
        for card in page.cards {
            let key = RowKey::of(&card);
            if self.rows.iter().any(|r| r.key == key) {
                continue;
            }
            let copies = self
                .edits
                .get(&key)
                .map_or_else(|| card.quantity.get(), |edit| edit.shown);
            self.rows.push(Row {
                key,
                card,
                copies,
                editable: key.editable(),
            });
        }
        self.status = if !self.rows.is_empty() {
            Status::List
        } else if self.search.is_empty() {
            Status::Empty
        } else {
            Status::NoMatch
        };
    }

    fn page_failed(&mut self, ticket: Ticket) -> Vec<Effect> {
        if self.page_out != Some(ticket) {
            return Vec::new();
        }
        self.page_out = None;
        if self.rows.is_empty() {
            self.status = Status::Failed;
            return Vec::new();
        }
        vec![Effect::Toast(Toast::normal(ToastKind::Error, MORE_FAILED))]
    }

    fn search(&mut self, ticket: Ticket) -> Vec<Effect> {
        if self.search_wait != Some(ticket) {
            return Vec::new();
        }
        self.search_wait = None;
        if let Ok(search) = CollectionSearch::new(&self.text) {
            self.search = search;
            self.restart()
        } else {
            self.rows.clear();
            self.total = 0;
            self.next_offset = None;
            self.page_out = None;
            self.status = Status::NoMatch;
            Vec::new()
        }
    }

    fn step(&mut self, key: RowKey, delta: i32) -> Vec<Effect> {
        let Some(row) = self.rows.iter_mut().find(|r| r.key == key && r.editable) else {
            return Vec::new();
        };
        let edit = self.edits.entry(key).or_insert(Edit {
            shown: row.copies,
            saved: row.copies,
            wait: None,
            writing: None,
            written: row.copies,
        });
        edit.shown = edit
            .shown
            .saturating_add(delta)
            .clamp(0, CollectionQuantity::MAX);
        row.copies = edit.shown;
        let ticket = self.tickets.next_ticket();
        edit.wait = Some(ticket);
        vec![Effect::FlushAfter {
            ticket,
            wait: timing::QUANTITY_FLUSH,
        }]
    }

    fn flush(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some((&key, edit)) = self
            .edits
            .iter_mut()
            .find(|(_, edit)| edit.wait == Some(ticket))
        else {
            return Vec::new();
        };
        edit.wait = None;
        if edit.writing.is_some() {
            return Vec::new();
        }
        if edit.shown == edit.saved {
            self.edits.remove(&key);
            return Vec::new();
        }
        self.write(key).into_iter().collect()
    }

    /// Sends `key`'s shown count.
    fn write(&mut self, key: RowKey) -> Option<Effect> {
        let ticket = self.tickets.next_ticket();
        let edit = self.edits.get_mut(&key)?;
        let quantity = SetQuantity::new(edit.shown).ok()?;
        edit.writing = Some(ticket);
        edit.written = edit.shown;
        Some(Effect::SetCopies {
            ticket,
            set: HttpSetCollectionCard {
                scryfall_data_id: key.printing,
                finish: key.finish,
                condition: key.condition,
                language: key.language,
                quantity,
            },
        })
    }

    fn saved(&mut self, ticket: Ticket, reply: &HttpSetCollectionCardReply) -> Vec<Effect> {
        let Some((&key, edit)) = self
            .edits
            .iter_mut()
            .find(|(_, edit)| edit.writing == Some(ticket))
        else {
            return Vec::new();
        };
        edit.writing = None;
        edit.saved = reply.card.as_ref().map_or(0, |card| card.quantity.get());
        let edit = *edit;
        let mut effects = vec![Effect::CollectionChanged];
        if let Some(card) = &reply.card
            && let Some(row) = self.rows.iter_mut().find(|r| r.key == key)
        {
            row.card = card.clone();
        }
        if edit.shown != edit.saved {
            if edit.wait.is_none() {
                effects.extend(self.write(key));
            }
            return effects;
        }
        if edit.wait.is_none() {
            self.edits.remove(&key);
        }
        if reply.card.is_none() {
            self.remove_row(key);
        }
        effects
    }

    /// Takes a removed row out of the list; the rows after it each sit one earlier on the
    /// server, so the next page starts one earlier too.
    fn remove_row(&mut self, key: RowKey) {
        let before = self.rows.len();
        self.rows.retain(|r| r.key != key);
        if self.rows.len() < before {
            if self.page_out.is_some() {
                self.removed_while_out = self.removed_while_out.saturating_add(1);
            }
            self.total = self.total.saturating_sub(1);
            self.next_offset = self.next_offset.map(|n| n.saturating_sub(1));
            if self.rows.is_empty() && self.next_offset.is_none() {
                self.status = if self.search.is_empty() {
                    Status::Empty
                } else {
                    Status::NoMatch
                };
            }
        }
    }

    /// Puts the saved count back, then replays the steps made while the write was out: sent at
    /// once, or by the flush still waiting on them.
    fn save_failed(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some((&key, edit)) = self
            .edits
            .iter_mut()
            .find(|(_, edit)| edit.writing == Some(ticket))
        else {
            return Vec::new();
        };
        edit.writing = None;
        let since = edit.shown.saturating_sub(edit.written);
        edit.shown = edit
            .saved
            .saturating_add(since)
            .clamp(0, CollectionQuantity::MAX);
        let edit = *edit;
        if let Some(row) = self.rows.iter_mut().find(|r| r.key == key) {
            row.copies = edit.shown;
        }
        let mut effects = vec![Effect::Toast(Toast::normal(ToastKind::Error, SAVE_FAILED))];
        if edit.wait.is_some() {
            return effects;
        }
        if edit.shown == edit.saved {
            self.edits.remove(&key);
        } else {
            effects.extend(self.write(key));
        }
        effects
    }
}
