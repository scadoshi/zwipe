//! The printing sheet: every printing of a card, one shown at a time, to pick one for the host or only to look. Add cards, the deck cards view, Remove cards and the screens with card rows open it over themselves.
//!
//! The client keeps a [`State`] for as long as the sheet is mounted, so the printings fetched for one card come back without a refetch the next time it opens on that card. Which printing shows is the client's: the app's carousel, the web's arrows and the terminal's cursor each keep their own index and hand it in (`at`) on the actions and views that need it. The model decides what loads, which printing the sheet opens on, whether Save may fire, and what each way out toasts.

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::hash::BuildHasher;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// The sheet's name. Its title bar shows the card's name; a screen reader hears [`label`].
pub const TITLE: &str = "Printings";

/// What a screen reader calls the sheet open on the card named `card_name`.
#[must_use]
pub fn label(card_name: &str) -> String {
    format!("{TITLE} of {card_name}")
}

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

/// The chip that narrows the printings to those in the user's collection.
pub const ONLY_MINE: &str = "Only mine";

/// Whether the sheet can hand a printing back.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Save picks the showing printing for the host.
    #[default]
    Pick,
    /// Browse only: no Save, and leaving never toasts. Every host with nothing to keep a pick opens it this way: Remove cards, whose delete is keyed on the printing already on the deck, a staple, Activity and Examples.
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
    /// The user's owned printings of the card `oracle_id` arrived. A client fetches them only
    /// when the user has a collection, so without them [`ONLY_MINE`] never shows.
    OwnedLoaded {
        /// The card they belong to.
        oracle_id: Uuid,
        /// The owned printings' ids.
        owned: Vec<Uuid>,
    },
    /// [`ONLY_MINE`] turned on or off, with printing `at` showing. Does nothing unless
    /// [`State::offers_only_mine`] holds.
    OnlyMine {
        /// Whether it is on.
        on: bool,
        /// The index of the showing printing.
        at: usize,
    },
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
    /// Every printing the fetch returned, in the server's order.
    fetched: Vec<Card>,
    /// The printings listed: [`shown_printings`] of `fetched` for the printing the sheet opened on.
    printings: Vec<Card>,
    /// The oracle id `printings` belong to, once a fetch for it landed.
    cached_for: Option<Uuid>,
    /// The oracle id and printing the newest fetch went out for.
    pending: Option<(Uuid, Uuid)>,
    /// The oracle id and printing of a fetch that failed, which Retry sends
    /// again.
    failed: Option<(Uuid, Uuid)>,
    loading: bool,
    /// Where the opening printing sits in `printings`; `None` when [`ONLY_MINE`] hides it, so
    /// every printing listed differs from it.
    initial: Option<usize>,
    /// The printing the sheet opened on.
    opened: Option<Uuid>,
    /// The user's owned printings, with the card they belong to.
    owned: Option<(Uuid, HashSet<Uuid>)>,
    /// Whether [`ONLY_MINE`] is on.
    only_mine: bool,
    /// Where turning [`ONLY_MINE`] on placed the sheet when it hides the opening printing: leaving
    /// from there says nothing, since the user picked nothing.
    placed: Option<usize>,
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
            fetched: Vec::new(),
            printings: Vec::new(),
            cached_for: None,
            pending: None,
            failed: None,
            loading: false,
            initial: None,
            opened: None,
            owned: None,
            only_mine: false,
            placed: None,
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
            Action::OwnedLoaded { oracle_id, owned } => {
                self.owned = Some((oracle_id, owned.into_iter().collect()));
                Vec::new()
            }
            Action::OnlyMine { on, at } => {
                if on == self.only_mine || !self.offers_only_mine() {
                    return Vec::new();
                }
                let showing = self.visible(at).map(|card| card.scryfall_data.id);
                self.only_mine = on;
                self.relist();
                let at = showing
                    .and_then(|id| self.printings.iter().position(|p| p.scryfall_data.id == id))
                    .or(self.initial)
                    .unwrap_or(0);
                self.placed = self.initial.is_none().then_some(at);
                vec![Effect::Show(at)]
            }
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
        self.opened = Some(printing_id);
        self.only_mine = false;
        if self.cached_for == Some(oracle_id) {
            self.relist();
            return vec![Effect::Rewind(self.initial())];
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
        self.opened = Some(printing_id);
        self.fetched = printings;
        self.cached_for = Some(oracle_id);
        self.loading = false;
        self.relist();
        vec![Effect::Show(self.initial())]
    }

    /// Closes the sheet, toasting [`UNCHANGED`] as `kind` when a different printing shows and Save could have kept it.
    fn leave(&self, at: usize, kind: ToastKind) -> Vec<Effect> {
        let mut effects = Vec::new();
        if self.mode == Mode::Pick && self.has_moved(at) {
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

    /// The printings listed, in the server's order: [`shown_printings`] of the fetch.
    #[must_use]
    pub fn printings(&self) -> &[Card] {
        &self.printings
    }

    /// The index the sheet opened on: the card's own printing, or the first when the list lacks it.
    #[must_use]
    pub fn initial(&self) -> usize {
        self.initial.unwrap_or(0)
    }

    /// Whether [`ONLY_MINE`] shows: the user owns a printing of this card among those listed.
    #[must_use]
    pub fn offers_only_mine(&self) -> bool {
        self.opened.is_some() && offers_only_mine(&self.fetched, self.owned_here())
    }

    /// Whether [`ONLY_MINE`] is on.
    #[must_use]
    pub fn only_mine(&self) -> bool {
        self.only_mine
    }

    /// The owned printings when they belong to the card the sheet lists.
    fn owned_here(&self) -> Option<&HashSet<Uuid>> {
        match (&self.owned, self.cached_for) {
            (Some((oracle_id, owned)), Some(listed)) if *oracle_id == listed => Some(owned),
            _ => None,
        }
    }

    /// Lists the fetch again for the opening printing, narrowed to the owned printings while
    /// [`ONLY_MINE`] is on, and finds where the opening printing sits.
    fn relist(&mut self) {
        let Some(keep) = self.opened else {
            return;
        };
        let owned = if self.only_mine {
            self.owned_here()
        } else {
            None
        };
        let printings = listed(&self.fetched, keep, owned);
        let at = printings.iter().position(|p| p.scryfall_data.id == keep);
        // The full list always holds the opening printing; when it doesn't, the first slot
        // stands for it, as it always has.
        self.initial = if self.only_mine {
            at
        } else {
            Some(at.unwrap_or(0))
        };
        self.placed = None;
        self.printings = printings;
    }

    /// The printing at `at`, if the list reaches it.
    #[must_use]
    pub fn visible(&self, at: usize) -> Option<&Card> {
        self.printings.get(at)
    }

    /// Whether printing `at` is not the one the sheet opened on.
    #[must_use]
    pub fn has_changed(&self, at: usize) -> bool {
        !self.printings.is_empty() && Some(at) != self.initial
    }

    /// Whether the user moved to printing `at` from where the sheet put them: off the opening
    /// printing, or, when [`ONLY_MINE`] hides it, off where turning it on placed them.
    fn has_moved(&self, at: usize) -> bool {
        match self.initial {
            Some(_) => self.has_changed(at),
            None => !self.printings.is_empty() && Some(at) != self.placed,
        }
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

/// The printings of `all` a sheet lists, in their order: those with an image URL, plus `keep`
/// (the printing a deck has, which shows as its stock card when it has none). A card with no
/// image on any printing keeps them all.
#[must_use]
pub fn shown_printings(all: &[Card], keep: Uuid) -> Vec<Card> {
    let has_image = |p: &Card| {
        p.scryfall_data
            .primary_image_url(ImageSize::Large)
            .is_some()
    };
    if !all.iter().any(has_image) {
        return all.to_vec();
    }
    all.iter()
        .filter(|p| has_image(p) || p.scryfall_data.id == keep)
        .cloned()
        .collect()
}

/// The printings a sheet lists for `keep`: [`shown_printings`], or with `owned` every owned
/// printing of `all`, an imageless one too (it shows as its stock card), so a printing the user
/// owns is never hidden from [`ONLY_MINE`].
#[must_use]
pub fn listed<S: BuildHasher>(
    all: &[Card],
    keep: Uuid,
    owned: Option<&HashSet<Uuid, S>>,
) -> Vec<Card> {
    match owned {
        Some(owned) => all
            .iter()
            .filter(|p| owned.contains(&p.scryfall_data.id))
            .cloned()
            .collect(),
        None => shown_printings(all, keep),
    }
}

/// Whether a sheet offers [`ONLY_MINE`]: `owned` holds a printing of the card among `all`.
#[must_use]
pub fn offers_only_mine<S: BuildHasher>(all: &[Card], owned: Option<&HashSet<Uuid, S>>) -> bool {
    owned.is_some_and(|owned| all.iter().any(|p| owned.contains(&p.scryfall_data.id)))
}
