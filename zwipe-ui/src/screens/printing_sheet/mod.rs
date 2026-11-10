//! The printing sheet: every printing of a card, one shown at a time, to pick one for the host or only to look. Add cards, the deck cards view, Remove cards and the screens with card rows open it over themselves.
//!
//! The client keeps a [`State`] for as long as the sheet is mounted, so the printings fetched for one card come back without a refetch the next time it opens on that card. Which printing shows is the client's: the app's carousel, the web's arrows and the terminal's cursor each keep their own index and hand it in (`at`) on the actions and views that need it. The model decides what loads, which printing the sheet opens on, whether Save may fire, and what each way out toasts.

#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::Card;

use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// The sheet's header.
pub const TITLE: &str = "Printings";

/// The toast after Save when the host names none of its own.
pub const SAVED: &str = "Printing saved";

/// The toast on leaving with another printing showing but not saved.
pub const UNCHANGED: &str = "Printing unchanged";

/// The button that closes the sheet.
pub const BACK: &str = "Back";

/// The button that hands the showing printing to the host.
pub const SAVE: &str = "Save";

/// The body when the fetch failed, beside its Retry.
pub const LOAD_FAILED: &str = "Couldn't load printings";

/// Whether the sheet can hand a printing back.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Save picks the showing printing for the host.
    #[default]
    Pick,
    /// Browse only: no Save, and leaving never toasts. Remove cards opens it this way, since its delete is keyed on the printing already on the deck.
    ReadOnly,
}

/// What the sheet draws in its body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The fetch is out: the skeleton.
    Loading,
    /// Nothing to show: not opened yet, or the card has no oracle id.
    Empty,
    /// The fetch failed or timed out: [`LOAD_FAILED`] with a Retry, which
    /// sends [`Action::Retry`].
    Failed,
    /// One printing: its image and info, no carousel.
    Single,
    /// Several printings: the carousel, its dots and the showing printing's info.
    Several,
}

/// What a user does on the sheet, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// The sheet opens over a card. Reuses the printings already fetched for its oracle id, else fetches them; a card with no oracle id changes nothing.
    Open {
        /// The printing the card is in now, which the sheet opens on.
        printing_id: Uuid,
        /// The card's oracle id, whose printings the sheet lists.
        oracle_id: Option<Uuid>,
    },
    /// The printings arrived for the fetch holding `ticket`.
    Loaded {
        /// The ticket from the [`Effect::FetchPrintings`] this answers.
        ticket: Ticket,
        /// Every printing of the card.
        printings: Vec<Card>,
    },
    /// The fetch holding `ticket` failed. The client has already reported it and raised its error toast.
    Failed {
        /// The ticket from the [`Effect::FetchPrintings`] this answers.
        ticket: Ticket,
    },
    /// Retry, under a failed fetch: sends it again.
    Retry,
    /// The Back button, with printing `at` showing.
    Back {
        /// The index of the showing printing.
        at: usize,
    },
    /// The backdrop or the system back gesture, with printing `at` showing.
    Dismiss {
        /// The index of the showing printing.
        at: usize,
    },
    /// Save, with printing `at` showing. Does nothing unless [`State::can_save`] holds.
    Save {
        /// The index of the showing printing.
        at: usize,
    },
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Fetches every printing of `oracle_id` and answers with [`Action::Loaded`] or [`Action::Failed`] carrying `ticket`.
    FetchPrintings {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The card whose printings to fetch.
        oracle_id: Uuid,
    },
    /// A new list arrived: lay it out showing the printing at this index.
    Show(usize),
    /// The list is the one already laid out: go back to the printing at this index without animating.
    Rewind(usize),
    /// Hands this printing to the host, which saves or swaps it.
    Pick(Box<Card>),
    /// Raises this toast.
    Toast(Toast),
    /// Closes the sheet.
    Close,
}

/// The printings and where the sheet opened, for as long as the sheet is mounted.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    mode: Mode,
    saved_message: &'static str,
    printings: Vec<Card>,
    /// The oracle id `printings` belong to, once a fetch for it landed.
    cached_for: Option<Uuid>,
    /// The oracle id and printing the newest fetch went out for.
    pending: Option<(Uuid, Uuid)>,
    /// The oracle id and printing of a fetch that failed, which Retry sends
    /// again.
    failed: Option<(Uuid, Uuid)>,
    loading: bool,
    initial: usize,
    generation: Generation,
}

impl Default for State {
    fn default() -> Self {
        Self::new(Mode::Pick, SAVED)
    }
}

impl State {
    /// A sheet that has not opened yet: nothing loading, nothing listed. `saved_message` is the toast after Save; Add cards says "Printing swapped" because it only reskins the card in its stack.
    #[must_use]
    pub fn new(mode: Mode, saved_message: &'static str) -> Self {
        Self {
            mode,
            saved_message,
            printings: Vec::new(),
            cached_for: None,
            pending: None,
            failed: None,
            loading: false,
            initial: 0,
            generation: Generation::default(),
        }
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// A reply whose ticket a newer fetch has replaced changes nothing.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Open {
                printing_id,
                oracle_id,
            } => self.open(printing_id, oracle_id),
            Action::Loaded { ticket, printings } => self.loaded(ticket, printings),
            Action::Failed { ticket } => {
                if self.generation.is_current(ticket) {
                    self.loading = false;
                    self.failed = self.pending.take();
                }
                Vec::new()
            }
            Action::Retry => match self.failed.take() {
                Some((oracle_id, printing_id)) => self.fetch(oracle_id, printing_id),
                None => Vec::new(),
            },
            Action::Back { at } => self.leave(at, ToastKind::Info),
            Action::Dismiss { at } => self.leave(at, ToastKind::Warning),
            Action::Save { at } => {
                if !self.can_save(at) {
                    return Vec::new();
                }
                let Some(card) = self.printings.get(at) else {
                    return Vec::new();
                };
                vec![
                    Effect::Pick(Box::new(card.clone())),
                    Effect::Toast(Toast::quick(ToastKind::Info, self.saved_message)),
                    Effect::Close,
                ]
            }
        }
    }

    fn open(&mut self, printing_id: Uuid, oracle_id: Option<Uuid>) -> Vec<Effect> {
        let Some(oracle_id) = oracle_id else {
            return Vec::new();
        };
        if self.cached_for == Some(oracle_id) {
            self.initial = position(&self.printings, printing_id);
            return vec![Effect::Rewind(self.initial)];
        }
        self.fetch(oracle_id, printing_id)
    }

    /// Sends the fetch for `oracle_id`, to open on `printing_id`.
    fn fetch(&mut self, oracle_id: Uuid, printing_id: Uuid) -> Vec<Effect> {
        self.loading = true;
        self.failed = None;
        self.pending = Some((oracle_id, printing_id));
        vec![Effect::FetchPrintings {
            ticket: self.generation.next_ticket(),
            oracle_id,
        }]
    }

    fn loaded(&mut self, ticket: Ticket, printings: Vec<Card>) -> Vec<Effect> {
        if !self.generation.is_current(ticket) {
            return Vec::new();
        }
        let Some((oracle_id, printing_id)) = self.pending.take() else {
            return Vec::new();
        };
        self.initial = position(&printings, printing_id);
        self.printings = printings;
        self.cached_for = Some(oracle_id);
        self.loading = false;
        vec![Effect::Show(self.initial)]
    }

    /// Closes the sheet, toasting [`UNCHANGED`] as `kind` when a different printing shows and Save could have kept it.
    fn leave(&self, at: usize, kind: ToastKind) -> Vec<Effect> {
        let mut effects = Vec::new();
        if self.mode == Mode::Pick && self.has_changed(at) {
            effects.push(Effect::Toast(Toast::quick(kind, UNCHANGED)));
        }
        effects.push(Effect::Close);
        effects
    }

    /// What the body draws.
    #[must_use]
    pub fn status(&self) -> Status {
        if self.loading {
            return Status::Loading;
        }
        if self.failed.is_some() {
            return Status::Failed;
        }
        match self.printings.len() {
            0 => Status::Empty,
            1 => Status::Single,
            _ => Status::Several,
        }
    }

    /// Every printing listed, in the server's order.
    #[must_use]
    pub fn printings(&self) -> &[Card] {
        &self.printings
    }

    /// The index the sheet opened on: the card's own printing, or the first when the list lacks it.
    #[must_use]
    pub fn initial(&self) -> usize {
        self.initial
    }

    /// The printing at `at`, if the list reaches it.
    #[must_use]
    pub fn visible(&self, at: usize) -> Option<&Card> {
        self.printings.get(at)
    }

    /// Whether printing `at` is not the one the sheet opened on.
    #[must_use]
    pub fn has_changed(&self, at: usize) -> bool {
        !self.printings.is_empty() && at != self.initial
    }

    /// Whether the sheet has a Save button at all.
    #[must_use]
    pub fn shows_save(&self) -> bool {
        self.mode == Mode::Pick
    }

    /// Whether Save fires with printing `at` showing: in [`Mode::Pick`], on a printing other than the one the sheet opened on.
    #[must_use]
    pub fn can_save(&self, at: usize) -> bool {
        self.shows_save() && self.has_changed(at) && self.visible(at).is_some()
    }
}

/// Where `printing_id` sits in `printings`, or the first slot when it is missing.
fn position(printings: &[Card], printing_id: Uuid) -> usize {
    printings
        .iter()
        .position(|p| p.scryfall_data.id == printing_id)
        .unwrap_or(0)
}
