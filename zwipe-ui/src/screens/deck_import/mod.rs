//! Import: a pasted decklist or an Archidekt link into one board of a deck, adding to it or replacing it, with what came in and what did not.
//!
//! The client keeps a [`State`] while the screen is up and sends [`Action::Open`] with the deck's id when it opens. The model holds the From, Mode and Board picks, the import in flight and its result, and decides the outcome toast through [`crate::deck::form::import_outcome`]. The text being typed stays in the client, in its text area or input, and comes in on [`Action::Submit`].

#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::deck::{ImportDeckCardsResult, ImportMode};
use zwipe_core::domain::user::hints::OneTimeHint;

use crate::deck::form::{self, ImportTone};
use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// The screen's title.
pub const TITLE: &str = "Import";

/// The label on the chip row that picks the [`Source`].
pub const FROM: &str = "From";

/// The label on the chip row that picks the [`ImportMode`].
pub const MODE: &str = "Mode";

/// The label on the chip row that picks the [`Board`].
pub const BOARD: &str = "Board";

/// The explainer's words before the board in [`ImportMode::Add`].
pub const ADD_BEFORE: &str = "Adds imported cards into the ";

/// The explainer's words after the board in [`ImportMode::Add`].
pub const ADD_AFTER: &str = ". Existing cards stay but take new quantities if applicable.";

/// The explainer's words before the board in [`ImportMode::Replace`].
pub const REPLACE_BEFORE: &str = "Removes all existing cards in the ";

/// The explainer's words after the board in [`ImportMode::Replace`].
pub const REPLACE_AFTER: &str = ", replacing with imported cards.";

/// The label over the decklist box, for [`Source::Text`].
pub const TEXT_LABEL: &str = "Paste decklist";

/// The decklist box's placeholder: three lines, one card each.
pub const TEXT_PLACEHOLDER: &str = "1 Krenko, Mob Boss\n1 Urza, Lord High Artificer\n1 Sol Ring";

/// The label over the link input, for [`Source::Archidekt`].
pub const URL_LABEL: &str = "Paste an Archidekt deck URL";

/// The link input's placeholder.
pub const URL_PLACEHOLDER: &str = "https://archidekt.com/decks/...";

/// The label over the cards that came in.
pub const IMPORTED: &str = "Imported";

/// The label over the lines that did not resolve.
pub const FAILED: &str = "Failed";

/// The Back button, to the deck.
pub const BACK: &str = "Back";

/// The submit button in [`ImportMode::Add`].
pub const IMPORT: &str = "Import";

/// The submit button in [`ImportMode::Replace`].
pub const REPLACE: &str = "Replace";

/// The confirm dialog's Cancel, which keeps the board as it is.
pub const CANCEL: &str = "Cancel";

/// The confirm dialog's title before a [`ImportMode::Replace`] import into `board`.
#[must_use]
pub fn replace_title(board: Board) -> String {
    format!("Replace the {}?", board.word())
}

/// The confirm dialog's line before a [`ImportMode::Replace`] import into `board`. The import
/// also clears the deck's undo history, so nothing brings the old cards back.
#[must_use]
pub fn replace_body(board: Board) -> String {
    format!(
        "Every card on the {} comes off and the imported cards go in their place. You can't undo this.",
        board.word()
    )
}

/// The submit button while an import is out.
pub const IMPORTING: &str = "Importing...";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The deck view for this deck, from Back.
    Deck(Uuid),
    /// The deck list, in place of a screen whose deck is gone.
    DeckList,
}

/// Where the cards come from.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    /// A pasted decklist.
    #[default]
    Text,
    /// An Archidekt deck link.
    Archidekt,
}

impl Source {
    /// Every source, in chip order.
    pub const ALL: [Self; 2] = [Self::Text, Self::Archidekt];

    /// The chip's words.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Archidekt => "Archidekt",
        }
    }
}

/// Every mode, in chip order.
pub const MODES: [ImportMode; 2] = [ImportMode::Add, ImportMode::Replace];

/// The Mode chip's words for `mode`.
#[must_use]
pub fn mode_label(mode: ImportMode) -> &'static str {
    match mode {
        ImportMode::Add => "Add",
        ImportMode::Replace => "Replace",
    }
}

/// The board the cards go to.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Board {
    /// The main deck. Sends no board, so a pasted list's section headers still sort its cards.
    #[default]
    Main,
    /// The maybeboard.
    Maybe,
    /// The sideboard.
    Side,
}

impl Board {
    /// Every board, in chip order.
    pub const ALL: [Self; 3] = [Self::Main, Self::Maybe, Self::Side];

    /// The chip's words.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Main => "Main",
            Self::Maybe => "Maybe",
            Self::Side => "Side",
        }
    }

    /// The board sent with the import: none for [`Board::Main`].
    #[must_use]
    pub fn wire(self) -> Option<&'static str> {
        match self {
            Self::Main => None,
            Self::Maybe => Some("maybeboard"),
            Self::Side => Some("sideboard"),
        }
    }

    /// The board's name in the explainer line.
    #[must_use]
    pub fn word(self) -> &'static str {
        self.wire().unwrap_or("mainboard")
    }
}

/// The line under the chips: [`before`](Self::before), the board highlighted, then [`after`](Self::after).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Explainer {
    /// The words before the board.
    pub before: &'static str,
    /// The board's name, drawn highlighted.
    pub board: &'static str,
    /// The words after the board.
    pub after: &'static str,
}

/// What a user does on Import, and each reply the client hands back.
#[derive(Debug, Clone)]
pub enum Action {
    /// Opens the screen for this deck: offers the hint once per visit.
    Open(Uuid),
    /// Picks where the cards come from. The client keeps the text and the link it holds for each.
    PickSource(Source),
    /// Picks whether the import adds or replaces.
    PickMode(ImportMode),
    /// Picks the board the cards go to.
    PickBoard(Board),
    /// Sends the import. Does nothing while one is out, before [`Action::Open`], or when `input` trims to nothing. In [`ImportMode::Replace`] it asks first, with [`Effect::ConfirmReplace`], and holds `input` for [`Action::ConfirmReplace`].
    Submit {
        /// The active input as typed: the decklist for [`Source::Text`], the link for [`Source::Archidekt`].
        input: String,
    },
    /// The user confirmed the replace: sends the import held since [`Action::Submit`]. A dialog closed without it sends nothing, and the next [`Action::Submit`] holds its own input.
    ConfirmReplace,
    /// The import holding `ticket` went through.
    Imported {
        /// The ticket from the effect this answers.
        ticket: Ticket,
        /// What came in and what did not.
        result: ImportDeckCardsResult,
    },
    /// The import holding `ticket` failed. The client has already reported it and raised its error toast.
    Failed {
        /// The ticket from the effect this answers.
        ticket: Ticket,
    },
    /// The request holding `ticket` found no deck: the server no longer has it. The client has already reported it and raised its toast.
    Gone {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// Goes back to the deck.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Imports a pasted decklist; answers with [`Action::Imported`] or [`Action::Failed`] carrying `ticket`.
    ImportDeckCards {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck the cards go into.
        deck_id: Uuid,
        /// The decklist as typed.
        text: String,
        /// The board to send, from [`Board::wire`].
        board: Option<&'static str>,
        /// Whether the import adds or replaces.
        mode: ImportMode,
    },
    /// Imports an Archidekt deck; answers with [`Action::Imported`] or [`Action::Failed`] carrying `ticket`.
    ImportArchidektDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck the cards go into.
        deck_id: Uuid,
        /// The link as typed.
        url: String,
        /// The board to send, from [`Board::wire`].
        board: Option<&'static str>,
        /// Whether the import adds or replaces.
        mode: ImportMode,
    },
    /// Asks the user to confirm a replace: the client opens a dialog with [`replace_title`] and [`replace_body`] for this board, and answers with [`Action::ConfirmReplace`] if the user goes ahead.
    ConfirmReplace(Board),
    /// Clears this deck's undo history, after an import that brought cards in.
    ClearUndo(Uuid),
    /// Raises this toast.
    Toast(Toast),
    /// Goes to this screen.
    Navigate(Destination),
    /// Goes to this screen in place of the current one, so Back skips a screen with nothing left to show.
    Replace(Destination),
}

/// Where the import stands.
#[derive(Debug, Default, Clone)]
enum Phase {
    /// Nothing sent yet, or the last import failed.
    #[default]
    Idle,
    /// An import is out.
    Importing,
    /// The last import went through with this result.
    Done(ImportDeckCardsResult),
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// Import for one visit. Starts on Text, Add and Main with nothing sent; send [`Action::Open`] when the screen opens.
///
/// Not `PartialEq`: the result it holds is zwipe-core's `ImportDeckCardsResult`, which has none.
#[derive(Debug, Default, Clone)]
pub struct State {
    deck_id: Option<Uuid>,
    source: Source,
    mode: ImportMode,
    board: Board,
    phase: Phase,
    hint: HintOffer,
    tickets: Generation,
    /// The input a replace waits to send until the user confirms it.
    held: Option<String>,
}

impl State {
    /// The screen as it first draws: Text, Add and Main picked, nothing sent.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// A reply whose ticket a newer import has replaced changes nothing. The hint is offered once per visit, on the first [`Action::Open`].
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Open(deck_id) => {
                self.deck_id = Some(deck_id);
                if self.hint == HintOffer::Waiting {
                    self.hint = HintOffer::Offered;
                    return vec![Effect::OfferHint(OneTimeHint::Import)];
                }
            }
            Action::PickSource(source) => self.source = source,
            Action::PickMode(mode) => self.mode = mode,
            Action::PickBoard(board) => self.board = board,
            Action::Submit { input } => {
                if self.mode.is_replace() && self.deck_id.is_some() && self.can_submit(&input) {
                    self.held = Some(input);
                    return vec![Effect::ConfirmReplace(self.board)];
                }
                return self.submit(input).into_iter().collect();
            }
            Action::ConfirmReplace => {
                return self
                    .held
                    .take()
                    .and_then(|input| self.submit(input))
                    .into_iter()
                    .collect();
            }
            Action::Imported { ticket, result } => return self.imported(ticket, result),
            Action::Failed { ticket } => {
                if self.lands(ticket) {
                    self.phase = Phase::Idle;
                }
            }
            Action::Gone { ticket } => {
                if self.lands(ticket) {
                    self.phase = Phase::Idle;
                }
                return vec![Effect::Replace(Destination::DeckList)];
            }
            Action::Back => {
                return self
                    .deck_id
                    .map(|id| Effect::Navigate(Destination::Deck(id)))
                    .into_iter()
                    .collect();
            }
        }
        Vec::new()
    }

    /// Starts the import, clearing the last result, unless [`Self::can_submit`] says no.
    fn submit(&mut self, input: String) -> Option<Effect> {
        let deck_id = self.deck_id?;
        if !self.can_submit(&input) {
            return None;
        }
        self.phase = Phase::Importing;
        let ticket = self.tickets.next_ticket();
        let (board, mode) = (self.board.wire(), self.mode);
        Some(match self.source {
            Source::Text => Effect::ImportDeckCards {
                ticket,
                deck_id,
                text: input,
                board,
                mode,
            },
            Source::Archidekt => Effect::ImportArchidektDeck {
                ticket,
                deck_id,
                url: input,
                board,
                mode,
            },
        })
    }

    /// Shows `result`, clears the undo history when cards came in, and raises the outcome toast.
    fn imported(&mut self, ticket: Ticket, result: ImportDeckCardsResult) -> Vec<Effect> {
        if !self.lands(ticket) {
            return Vec::new();
        }
        let mut effects = Vec::new();
        if let Some(deck_id) = self.deck_id
            && !result.imported.is_empty()
        {
            effects.push(Effect::ClearUndo(deck_id));
        }
        effects.push(Effect::Toast(outcome_toast(&result)));
        self.phase = Phase::Done(result);
        effects
    }

    /// Whether a reply holding `ticket` answers the import still out.
    fn lands(&self, ticket: Ticket) -> bool {
        matches!(self.phase, Phase::Importing) && self.tickets.is_current(ticket)
    }

    /// The deck, once [`Action::Open`] has named it.
    #[must_use]
    pub fn deck_id(&self) -> Option<Uuid> {
        self.deck_id
    }

    /// The picked source.
    #[must_use]
    pub fn source(&self) -> Source {
        self.source
    }

    /// The picked mode.
    #[must_use]
    pub fn mode(&self) -> ImportMode {
        self.mode
    }

    /// The picked board.
    #[must_use]
    pub fn board(&self) -> Board {
        self.board
    }

    /// Whether an import is out.
    #[must_use]
    pub fn importing(&self) -> bool {
        matches!(self.phase, Phase::Importing)
    }

    /// The last import's result, until another starts. A failed import leaves none.
    #[must_use]
    pub fn result(&self) -> Option<&ImportDeckCardsResult> {
        match &self.phase {
            Phase::Done(result) => Some(result),
            Phase::Idle | Phase::Importing => None,
        }
    }

    /// Whether the submit button takes a press with `input` as the active input: no import out, and something left once trimmed.
    #[must_use]
    pub fn can_submit(&self, input: &str) -> bool {
        !self.importing() && !input.trim().is_empty()
    }

    /// The submit button's words.
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

    /// The line under the chips for the picked mode and board.
    #[must_use]
    pub fn explainer(&self) -> Explainer {
        let (before, after) = if self.mode.is_replace() {
            (REPLACE_BEFORE, REPLACE_AFTER)
        } else {
            (ADD_BEFORE, ADD_AFTER)
        };
        Explainer {
            before,
            board: self.board.word(),
            after,
        }
    }
}

/// The toast after an import: failed lines alone stay up longer, the rest are routine.
fn outcome_toast(result: &ImportDeckCardsResult) -> Toast {
    let (tone, text) = form::import_outcome(result.imported.len(), result.unresolved.len());
    match tone {
        ImportTone::Info => Toast::quick(ToastKind::Info, text),
        ImportTone::Success => Toast::quick(ToastKind::Success, text),
        ImportTone::Error => Toast::normal(ToastKind::Error, text),
    }
}
