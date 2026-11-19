//! Collection's list: the user's collection a page at a time, searched by card name, each row's
//! copies changed with a stepper, and the whole collection exported as a ManaBox file.
//!
//! Pages come in order by cursor, one out at a time, the next asked for as the list nears its end;
//! a newer search starts over from the first page and drops any page still out. Typing searches
//! once it rests for [`timing::QUICK_ADD_DEBOUNCE`].
//!
//! A step shows the new count at once and saves the row once steps rest for
//! [`timing::QUANTITY_FLUSH`], so a burst of taps is one write. A row has one write out at a
//! time; a step made meanwhile goes when it lands. 0 removes the row once the write lands; the
//! next page follows its cursor, which a removal doesn't move, and a row already listed isn't
//! listed twice. A failed write puts the saved count back and replays the steps made since. Edits
//! are kept apart from the pages, so a new search never drops one still waiting to save. A row
//! holding a finish, condition or language this build doesn't know can't be edited, since the
//! server refuses those values.
//!
//! The model holds the list and the edits; the client fetches, saves, runs the timers, draws the
//! rows and saves the export's file, with [`EXPORTED`] as it starts. Import is its own model,
//! [`super::collection_import`].

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

use uuid::Uuid;
use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionCursor, CollectionQuantity, CollectionSearch, Finish,
    SetQuantity,
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

/// The toast when the export's file starts saving.
pub const EXPORTED: &str = "Collection exported";

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

    /// A stable id for the row, for a client that keys rows by id: the printing's own for a
    /// nonfoil row with no condition or language given, else the printing's with its finish,
    /// condition and language mixed into the top bits, so each row of one printing has its own.
    #[must_use]
    pub fn id(self) -> Uuid {
        let mut hasher = DefaultHasher::new();
        (self.finish, self.condition, self.language).hash(&mut hasher);
        let plain = (Finish::Nonfoil, None::<CardCondition>, None::<CardLanguage>);
        let mut plain_hasher = DefaultHasher::new();
        plain.hash(&mut plain_hasher);
        let mix = hasher.finish() ^ plain_hasher.finish();
        Uuid::from_u128(self.printing.as_u128() ^ (u128::from(mix) << 64))
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
    /// An import changed the collection under the list: reads it again from the first page.
    Imported,
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
    /// Fetches the page after `after` for `search`; answers with [`Action::PageLoaded`] or
    /// [`Action::PageFailed`] carrying `ticket`.
    FetchPage {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The search, empty for every row.
        search: CollectionSearch,
        /// The last page's `next`; `None` for the first page.
        after: Option<CollectionCursor>,
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
    /// whether From offers Collection).
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
    next: Option<CollectionCursor>,
    /// The page out, with nothing else asked until it settles.
    page_out: Option<Ticket>,
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

    /// Whether a page follows the ones loaded, for the list's Show more.
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.next.is_some()
    }

    /// Whether an export is out, which disables Export.
    #[must_use]
    pub fn exporting(&self) -> bool {
        self.exporting.is_some()
    }

    /// Takes one action and answers with what the client should do.
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Open | Action::Imported => self.restart(),
            Action::PageLoaded { ticket, page } => {
                if self.page_out != Some(ticket) {
                    return Vec::new();
                }
                self.land(*page);
                Vec::new()
            }
            Action::PageFailed { ticket } => self.page_failed(ticket),
            Action::NearEnd => match self.next.clone() {
                Some(after) if self.page_out.is_none() && self.status == Status::List => {
                    vec![self.fetch(Some(after))]
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
                vec![
                    Effect::Download {
                        file_name: export.file_name,
                        text: export.csv,
                    },
                    Effect::Toast(Toast::quick(ToastKind::Success, EXPORTED)),
                ]
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
        self.next = None;
        self.status = Status::Loading;
        vec![self.fetch(None)]
    }

    fn fetch(&mut self, after: Option<CollectionCursor>) -> Effect {
        let ticket = self.tickets.next_ticket();
        self.page_out = Some(ticket);
        Effect::FetchPage {
            ticket,
            search: self.search.clone(),
            after,
        }
    }

    /// Adds a page's rows, each showing a count waiting to save over the server's.
    fn land(&mut self, page: HttpCollectionPage) {
        self.page_out = None;
        self.total = page.total;
        self.next = page.next;
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
            self.next = None;
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

    /// Takes a removed row out of the list.
    fn remove_row(&mut self, key: RowKey) {
        let before = self.rows.len();
        self.rows.retain(|r| r.key != key);
        if self.rows.len() < before {
            self.total = self.total.saturating_sub(1);
            if self.rows.is_empty() && self.next.is_none() {
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

/// What a row shows in place of its count: the count first, then its finish, condition and
/// language, leaving out nonfoil, English and anything not given or this build doesn't know.
#[must_use]
pub fn row_tags(row: &Row) -> Vec<String> {
    let mut tags = vec![row.copies.to_string()];
    match row.key.finish {
        Finish::Foil => tags.push("Foil".to_owned()),
        Finish::Etched => tags.push("Etched".to_owned()),
        Finish::Nonfoil | Finish::Unknown => {}
    }
    if let Some(condition) = row.key.condition.and_then(condition_short) {
        tags.push(condition.to_owned());
    }
    if let Some(language) = row.key.language
        && !matches!(language, CardLanguage::En | CardLanguage::Unknown)
    {
        tags.push(language.as_str().to_uppercase());
    }
    tags
}

/// A condition's short grade, as collection apps print it; `None` for one this build doesn't
/// know.
fn condition_short(condition: CardCondition) -> Option<&'static str> {
    Some(match condition {
        CardCondition::Mint => "M",
        CardCondition::NearMint => "NM",
        CardCondition::Excellent => "EX",
        CardCondition::Good => "GD",
        CardCondition::LightlyPlayed => "LP",
        CardCondition::ModeratelyPlayed => "MP",
        CardCondition::Played => "PL",
        CardCondition::HeavilyPlayed => "HP",
        CardCondition::Poor => "PR",
        CardCondition::Damaged => "DMG",
        CardCondition::Unknown => return None,
    })
}
