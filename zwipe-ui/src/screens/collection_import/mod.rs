//! Collection: importing a ManaBox or Archidekt collection export, adding it to the collection or
//! replacing the collection with it, then what landed and the rows that didn't.
//!
//! The client keeps a [`State`] while the screen is up. Reading the picked file stays in the
//! client, which hands its name and text in on [`Action::PickFile`]; the model holds the Mode
//! pick, the file, the import in flight and its summary. A replace asks first.

#[cfg(test)]
mod tests;

use zwipe_core::domain::collection::CollectionSource;
use zwipe_core::domain::collection::limits::MAX_BODY_BYTES;
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::http::contracts::collection::{HttpCollectionImportSummary, UnmatchedReason};

use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// The screen's title, and its entry in the web nav.
pub const TITLE: &str = "Collection";

/// The label on the chip row that picks the [`ImportMode`].
pub const MODE: &str = "Mode";

/// The label chip beside the file field.
pub const FILE: &str = "File";

/// The file field before a file is picked.
pub const FILE_PLACEHOLDER: &str = "Choose a .csv export";

/// The file types the picker offers.
pub const ACCEPT: &str = ".csv,text/csv";

/// The line under the chips: which exports the import takes.
pub const SOURCES: &str = "Takes a collection export from ManaBox or Archidekt.";

/// The explainer in [`ImportMode::Add`].
pub const ADD_EXPLAINER: &str =
    "Adds the export's cards to your collection. Copies of a card you already have add up.";

/// The explainer in [`ImportMode::Replace`].
pub const REPLACE_EXPLAINER: &str =
    "Makes your collection exactly the export. Cards it doesn't list come out.";

/// The submit in [`ImportMode::Add`].
pub const IMPORT: &str = "Import";

/// The submit in [`ImportMode::Replace`], and the confirm dialog's button.
pub const REPLACE: &str = "Replace";

/// The submit while an import is out.
pub const IMPORTING: &str = "Importing…";

/// The confirm dialog's Cancel, which keeps the collection as it is.
pub const CANCEL: &str = "Cancel";

/// The confirm dialog's title before a replace.
pub const REPLACE_TITLE: &str = "Replace your collection?";

/// The confirm dialog's line before a replace.
pub const REPLACE_BODY: &str = "Every card in your collection comes out and the export's cards go in its place. You can't undo this.";

/// The label over the import's counts.
pub const IMPORTED: &str = "Imported";

/// The label over the rows that didn't match a card.
pub const NOT_MATCHED: &str = "Not matched";

/// The toast for a file past the import's size cap; nothing is sent.
pub const TOO_BIG: &str = "That file is too big to import";

/// The toast for a file the browser couldn't read.
pub const UNREADABLE: &str = "Couldn't read that file";

/// What a user does on Collection, and each reply the client hands back.
#[derive(Debug, Clone)]
pub enum Action {
    /// Picks whether the import adds or replaces.
    PickMode(ImportMode),
    /// A file was picked and read: its name and its text. A file whose request would pass the
    /// size cap is turned down with a toast and clears the last pick.
    PickFile {
        /// The file's name, shown in the field.
        name: String,
        /// The file's text.
        text: String,
    },
    /// A file past the size cap was picked; the client didn't read it. It clears the last pick.
    PickedTooBig,
    /// The browser couldn't read the picked file. It clears the last pick.
    Unreadable,
    /// Sends the import. Does nothing without a file or while one is out. In
    /// [`ImportMode::Replace`] it asks first, with [`Effect::ConfirmReplace`].
    Submit,
    /// The user confirmed the replace: sends it.
    ConfirmReplace,
    /// The import holding `ticket` went through.
    Imported {
        /// The ticket from the effect this answers.
        ticket: Ticket,
        /// What it did.
        summary: HttpCollectionImportSummary,
    },
    /// The import holding `ticket` failed. The client has already reported it and raised its
    /// error toast, which for a file the server turned down says why.
    Failed {
        /// The ticket from the effect this answers.
        ticket: Ticket,
    },
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Imports the export; answers with [`Action::Imported`] or [`Action::Failed`] carrying
    /// `ticket`.
    ImportCollection {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The file's text.
        csv: String,
        /// Whether the import adds or replaces.
        mode: ImportMode,
    },
    /// Asks the user to confirm a replace: the client opens a dialog with [`REPLACE_TITLE`] and
    /// [`REPLACE_BODY`], and answers with [`Action::ConfirmReplace`] if the user goes ahead.
    ConfirmReplace,
    /// Raises this toast.
    Toast(Toast),
    /// The collection changed: the client drops what it kept of it (the summary that decides
    /// whether "In my collection" shows), so the next screen that needs it asks again.
    CollectionChanged,
}

/// The picked file.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Picked {
    name: String,
    text: String,
}

/// Where the import stands.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
enum Phase {
    /// Nothing sent yet, or the last import failed.
    #[default]
    Idle,
    /// An import is out.
    Importing,
    /// The last import went through with this summary.
    Done(HttpCollectionImportSummary),
}

/// Collection for one visit. Starts on Add with no file.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct State {
    mode: ImportMode,
    file: Option<Picked>,
    phase: Phase,
    tickets: Generation,
}

impl State {
    /// The screen as it first draws: Add picked, no file.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order. A reply whose ticket a
    /// newer import has replaced changes nothing.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::PickMode(mode) => self.mode = mode,
            // A refused pick replaces the last one too, so Import never sends a file the field
            // no longer names.
            Action::PickFile { name, text } => {
                if body_bytes(&text) > MAX_BODY_BYTES {
                    self.file = None;
                    return vec![Effect::Toast(Toast::normal(ToastKind::Error, TOO_BIG))];
                }
                self.file = Some(Picked { name, text });
            }
            Action::PickedTooBig => {
                self.file = None;
                return vec![Effect::Toast(Toast::normal(ToastKind::Error, TOO_BIG))];
            }
            Action::Unreadable => {
                self.file = None;
                return vec![Effect::Toast(Toast::normal(ToastKind::Error, UNREADABLE))];
            }
            Action::Submit => {
                if !self.can_submit() {
                    return Vec::new();
                }
                if self.mode.is_replace() {
                    return vec![Effect::ConfirmReplace];
                }
                return self.send().into_iter().collect();
            }
            Action::ConfirmReplace => {
                if self.can_submit() && self.mode.is_replace() {
                    return self.send().into_iter().collect();
                }
            }
            Action::Imported { ticket, summary } => {
                if self.lands(ticket) {
                    let toast = outcome_toast(&summary);
                    self.phase = Phase::Done(summary);
                    // The file went in; sending it again would add its copies twice.
                    self.file = None;
                    return vec![Effect::Toast(toast), Effect::CollectionChanged];
                }
            }
            Action::Failed { ticket } => {
                if self.lands(ticket) {
                    self.phase = Phase::Idle;
                }
            }
        }
        Vec::new()
    }

    /// Starts the import with the picked file.
    fn send(&mut self) -> Option<Effect> {
        let csv = self.file.as_ref()?.text.clone();
        self.phase = Phase::Importing;
        Some(Effect::ImportCollection {
            ticket: self.tickets.next_ticket(),
            csv,
            mode: self.mode,
        })
    }

    /// Whether a reply holding `ticket` answers the import still out.
    fn lands(&self, ticket: Ticket) -> bool {
        self.phase == Phase::Importing && self.tickets.is_current(ticket)
    }

    /// The picked mode.
    #[must_use]
    pub fn mode(&self) -> ImportMode {
        self.mode
    }

    /// The picked file's name.
    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.file.as_ref().map(|f| f.name.as_str())
    }

    /// Whether Submit would send: a file is picked and no import is out. A finished import clears
    /// the file, so each pick sends once.
    #[must_use]
    pub fn can_submit(&self) -> bool {
        self.file.is_some() && self.phase != Phase::Importing
    }

    /// Whether an import is out.
    #[must_use]
    pub fn importing(&self) -> bool {
        self.phase == Phase::Importing
    }

    /// The submit's label: [`IMPORTING`] while one is out, else [`REPLACE`] or [`IMPORT`].
    #[must_use]
    pub fn submit_label(&self) -> &'static str {
        if self.importing() {
            IMPORTING
        } else if self.mode.is_replace() {
            REPLACE
        } else {
            IMPORT
        }
    }

    /// The line under the chips for the picked mode.
    #[must_use]
    pub fn explainer(&self) -> &'static str {
        if self.mode.is_replace() {
            REPLACE_EXPLAINER
        } else {
            ADD_EXPLAINER
        }
    }

    /// The last import's summary.
    #[must_use]
    pub fn summary(&self) -> Option<&HttpCollectionImportSummary> {
        match &self.phase {
            Phase::Done(summary) => Some(summary),
            Phase::Idle | Phase::Importing => None,
        }
    }
}

/// Room in the request for everything but the file's text: the braces, the field names and the
/// mode.
const BODY_OVERHEAD: usize = 64;

/// The request body's size for a file of `text`: its bytes, one more for each character JSON
/// escapes with a backslash, five more for any other control character, and [`BODY_OVERHEAD`].
fn body_bytes(text: &str) -> usize {
    let escapes: usize = text
        .chars()
        .map(|c| match c {
            '"' | '\\' | '\n' | '\r' | '\t' | '\u{8}' | '\u{c}' => 1,
            c if c.is_control() && u32::from(c) < 0x20 => 5,
            _ => 0,
        })
        .sum();
    text.len()
        .saturating_add(escapes)
        .saturating_add(BODY_OVERHEAD)
}

/// The toast after an import: a success when every row matched, news with the count when some
/// didn't.
#[must_use]
pub fn outcome_toast(summary: &HttpCollectionImportSummary) -> Toast {
    match summary.unmatched_total {
        0 => Toast::quick(ToastKind::Success, "Collection imported"),
        1 => Toast::normal(ToastKind::Info, "Imported; 1 row didn't match a card"),
        n => Toast::normal(
            ToastKind::Info,
            format!("Imported; {n} rows didn't match a card"),
        ),
    }
}

/// The export's app, as the counts name it.
#[must_use]
pub fn source_name(source: CollectionSource) -> &'static str {
    match source {
        CollectionSource::ManaBox => "ManaBox",
        CollectionSource::Archidekt => "Archidekt",
        CollectionSource::Unknown => "export",
    }
}

/// The import's counts, one row each as `(value, what it counts)`.
#[must_use]
pub fn count_rows(summary: &HttpCollectionImportSummary) -> Vec<(String, String)> {
    vec![
        (
            summary.rows.to_string(),
            format!("{} rows read", source_name(summary.source)),
        ),
        (summary.added.to_string(), "added".to_owned()),
        (summary.updated.to_string(), "updated".to_owned()),
        (summary.removed.to_string(), "removed".to_owned()),
        (
            summary.cards.to_string(),
            "cards in your collection".to_owned(),
        ),
    ]
}

/// Why a row didn't match, in a few words.
#[must_use]
pub fn reason_words(reason: UnmatchedReason) -> &'static str {
    match reason {
        UnmatchedReason::UnknownPrinting => "no such printing",
        UnmatchedReason::AmbiguousName => "more than one printing",
        UnmatchedReason::MissingName => "no card given",
        UnmatchedReason::BadQuantity => "quantity out of range",
        UnmatchedReason::Unknown => "not matched",
    }
}

/// The rows that didn't match, one each as `(line and name, why)`: `Line 4: Forest (zzz)`.
#[must_use]
pub fn unmatched_rows(summary: &HttpCollectionImportSummary) -> Vec<(String, &'static str)> {
    summary
        .unmatched
        .iter()
        .map(|row| {
            let name = row.name.as_deref().unwrap_or("no name");
            let set = row
                .set
                .as_deref()
                .map_or_else(String::new, |set| format!(" ({set})"));
            (
                format!("Line {}: {name}{set}", row.line),
                reason_words(row.reason),
            )
        })
        .collect()
}

/// The line under the unmatched list when the reply listed only some of them.
#[must_use]
pub fn more_unmatched(summary: &HttpCollectionImportSummary) -> Option<String> {
    let listed = u32::try_from(summary.unmatched.len()).unwrap_or(u32::MAX);
    let more = summary.unmatched_total.saturating_sub(listed);
    (more > 0).then(|| format!("And {more} more not shown"))
}
