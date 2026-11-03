//! Import: a pasted decklist or an Archidekt link into one board of the deck, adding to it or replacing it, with what came in and what did not.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]; this view adds the text area and the link box, which keep their own values when From switches, and the focus.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCardsResult;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::generation::Ticket;
use zwipe_ui::hint::{Block, HintContent, Span, one_time};
use zwipe_ui::screens::deck_import::{Action, Destination, Effect, Source, State};

use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::text_area::TextArea;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The control the keys go to.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    From,
    Mode,
    Board,
    #[default]
    Input,
}

impl Focus {
    const ORDER: [Self; 4] = [Self::From, Self::Mode, Self::Board, Self::Input];

    /// The next or the previous control, wrapping.
    fn step(self, toward: Toward) -> Self {
        let here = Self::ORDER.iter().position(|f| *f == self).unwrap_or(0);
        let next = toward.wrapped(here, Self::ORDER.len());
        Self::ORDER.get(next).copied().unwrap_or_default()
    }
}

/// The Import screen's state: the model plus the two inputs and the focus.
#[derive(Debug)]
pub(crate) struct ImportView {
    pub(crate) model: State,
    pub(crate) text: TextArea,
    pub(crate) url: TextInput,
    pub(crate) focus: Focus,
}

impl Default for ImportView {
    fn default() -> Self {
        Self {
            model: State::new(),
            text: TextArea::default(),
            url: TextInput::default(),
            focus: Focus::Input,
        }
    }
}

impl ImportView {
    /// The active input as typed: the decklist for Text, the link for Archidekt.
    pub(crate) fn input(&self) -> String {
        match self.model.source() {
            Source::Text => self.text.text(),
            Source::Archidekt => self.url.value().to_owned(),
        }
    }

    /// Whether the submit key takes a press now.
    pub(crate) fn can_submit(&self) -> bool {
        self.model.can_submit(&self.input())
    }

    pub(crate) fn step_focus(&mut self, toward: Toward) {
        self.focus = self.focus.step(toward);
    }
}

/// Sends the active input to the model's submit, which refuses while an import is out or the input is blank.
pub(crate) fn submit(view: &mut ImportView, cx: &mut Context<'_>) {
    let input = view.input();
    dispatch(view, Action::Submit { input }, cx);
}

/// Applies `action` to the model and performs the effects it returns, in order.
pub(crate) fn dispatch(view: &mut ImportView, action: Action, cx: &mut Context<'_>) {
    for effect in view.model.update(action) {
        match effect {
            Effect::OfferHint(offer) => {
                if let Some(content) = hint_content() {
                    hint::once_with(cx, offer.key(), content);
                }
            }
            Effect::ImportDeckCards {
                ticket,
                deck_id,
                text,
                board,
                mode,
            } => cx.request(
                Request::Text {
                    ticket,
                    deck_id,
                    text,
                    board,
                    mode,
                }
                .into(),
            ),
            Effect::ImportArchidektDeck {
                ticket,
                deck_id,
                url,
                board,
                mode,
            } => cx.request(
                Request::Archidekt {
                    ticket,
                    deck_id,
                    url,
                    board,
                    mode,
                }
                .into(),
            ),
            Effect::ClearUndo(deck_id) => cx.shared.cards.undo.clear(deck_id),
            Effect::Toast(toast) => cx.show(toast),
            Effect::Navigate(Destination::Deck(_)) => cx.back(),
        }
    }
}

/// An example decklist, one card a line as the paste box reads it, shown at the end of the import hint.
pub(crate) const EXAMPLE: [&str; 3] = [
    "1 Krenko, Mob Boss",
    "1 Urza, Lord High Artificer",
    "1 Sol Ring",
];

/// The app's import hint with [`EXAMPLE`] under it, since the paste box shows nothing while empty.
pub(crate) fn hint_content() -> Option<HintContent> {
    let mut content = one_time::content_for(OneTimeHint::Import)?;
    content.blocks.push(Block::Line(vec![Span::Text(
        "A pasted list reads one card a line:".to_owned(),
    )]));
    content.blocks.extend(
        EXAMPLE
            .iter()
            .map(|line| Block::Line(vec![Span::Text(format!("  {line}"))])),
    );
    Some(content)
}

/// Shows the import hint again.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    if let Some(content) = hint_content() {
        hint::open(cx, content);
    }
}

/// Import's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    Text {
        ticket: Ticket,
        deck_id: Uuid,
        text: String,
        /// The board sent, none for Main.
        board: Option<&'static str>,
        mode: ImportMode,
    },
    Archidekt {
        ticket: Ticket,
        deck_id: Uuid,
        url: String,
        /// The board sent, none for Main.
        board: Option<&'static str>,
        mode: ImportMode,
    },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Text { .. } => "text",
            Self::Archidekt { .. } => "archidekt",
        }
    }

    /// The ticket the request went out with.
    fn ticket(&self) -> Ticket {
        match self {
            Self::Text { ticket, .. } | Self::Archidekt { ticket, .. } => *ticket,
        }
    }
}

/// Import's results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    Imported {
        ticket: Ticket,
        result: ImportDeckCardsResult,
    },
    Failed {
        ticket: Ticket,
        error: String,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::DeckImport(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::DeckImport(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut ImportView> {
    match screen {
        Screen::DeckImport(view) => Some(view),
        _ => None,
    }
}

/// Opens Import for `deck_id` and shows its hint once.
pub(crate) fn open(cx: &mut Context<'_>, deck_id: Uuid) {
    let mut view = ImportView::default();
    dispatch(&mut view, Action::Open(deck_id), cx);
    cx.push(Screen::DeckImport(Box::new(view)));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let ticket = request.ticket();
    let result = async {
        let session = account.fresh().await?;
        match request {
            Request::Text {
                deck_id,
                text,
                board,
                mode,
                ..
            } => {
                client
                    .import_deck_cards(deck_id, &text, board, mode, &session)
                    .await
            }
            Request::Archidekt {
                deck_id,
                url,
                board,
                mode,
                ..
            } => {
                client
                    .import_archidekt_deck(deck_id, &url, board, mode, &session)
                    .await
            }
        }
    }
    .await;
    Some(outcome(ticket, result))
}

/// What an import's answer means for the screen. A list over the server's line cap (422) or body limit (413) comes back as the client's user message, which Import toasts as an error, as the app does.
fn outcome(ticket: Ticket, result: Result<ImportDeckCardsResult, ClientError>) -> screens::Update {
    match result {
        Ok(result) => Update::Imported { ticket, result }.into(),
        Err(e) => screens::failure(&e, |error| Update::Failed { ticket, error }.into()),
    }
}

/// Hands a reply to the model. A failure raises its toast even when the screen has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::Imported { ticket, result } => Action::Imported { ticket, result },
        Update::Failed { ticket, error } => {
            cx.toast(ToastKind::Error, error);
            Action::Failed { ticket }
        }
    };
    if let Some(view) = nav.find_mut(pick) {
        dispatch(view, action, cx);
    }
}
