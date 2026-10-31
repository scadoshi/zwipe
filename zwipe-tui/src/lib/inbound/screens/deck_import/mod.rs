//! Import: a pasted decklist or an Archidekt link into one board of the deck, adding to it or replacing it, with what came in and what did not.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCardsResult;

use super::{Screen, failure};
use crate::domain::deck::form::{self, ImportTone};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hint::{self, HintDialog};
use crate::inbound::widgets::text_area::TextArea;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The key the server stores the import hint under.
pub(crate) const HINT_KEY: &str = "import";

const HINT_TITLE: &str = "Importing cards";

const HINT_BODY: &str = "• Paste a decklist, or switch From to an Archidekt deck link\n• Add keeps your current cards. Replace swaps the board's contents\n• Board picks where the cards go\n• Importing clears the deck's undo history";

/// Where the cards come from.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Source {
    #[default]
    Text,
    Archidekt,
}

/// The board the cards go to. Main sends no board, so a pasted list's section headers still apply.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    #[default]
    Main,
    Maybe,
    Side,
}

impl Target {
    pub(crate) const ALL: [Self; 3] = [Self::Main, Self::Maybe, Self::Side];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Main => "Main",
            Self::Maybe => "Maybe",
            Self::Side => "Side",
        }
    }

    /// The board sent to the server: none for Main.
    pub(crate) fn wire(self) -> Option<&'static str> {
        match self {
            Self::Main => None,
            Self::Maybe => Some("maybeboard"),
            Self::Side => Some("sideboard"),
        }
    }

    /// The board's name in the explainer line.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Main => "mainboard",
            Self::Maybe => "maybeboard",
            Self::Side => "sideboard",
        }
    }
}

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

    /// The next control, or the previous with `back`, wrapping.
    fn step(self, back: bool) -> Self {
        let here = Self::ORDER.iter().position(|f| *f == self).unwrap_or(0);
        let next = if back {
            here.checked_sub(1)
                .unwrap_or(Self::ORDER.len().saturating_sub(1))
        } else {
            here.saturating_add(1) % Self::ORDER.len()
        };
        Self::ORDER.get(next).copied().unwrap_or_default()
    }
}

/// The Import screen. The text and the link keep their own values when From switches.
#[derive(Debug)]
pub(crate) struct ImportView {
    pub(crate) deck_id: Uuid,
    pub(crate) source: Source,
    pub(crate) mode: ImportMode,
    pub(crate) target: Target,
    pub(crate) text: TextArea,
    pub(crate) url: TextInput,
    pub(crate) focus: Focus,
    pub(crate) loading: bool,
    /// The last import's result, cleared when another starts.
    pub(crate) result: Option<ImportDeckCardsResult>,
}

impl ImportView {
    pub(crate) fn opening(deck_id: Uuid) -> Self {
        Self {
            deck_id,
            source: Source::Text,
            mode: ImportMode::Add,
            target: Target::Main,
            text: TextArea::default().with_placeholder(&[
                "1 Krenko, Mob Boss",
                "1 Urza, Lord High Artificer",
                "1 Sol Ring",
            ]),
            url: TextInput::default().with_placeholder("https://archidekt.com/decks/..."),
            focus: Focus::Input,
            loading: false,
            result: None,
        }
    }

    /// Whether the active input is blank once trimmed.
    pub(crate) fn blank(&self) -> bool {
        match self.source {
            Source::Text => self.text.is_blank(),
            Source::Archidekt => self.url.value().trim().is_empty(),
        }
    }

    /// The submit button's label.
    pub(crate) fn submit_label(&self) -> &'static str {
        if self.loading {
            "Importing..."
        } else if self.mode.is_replace() {
            "Replace"
        } else {
            "Import"
        }
    }

    pub(crate) fn step_focus(&mut self, back: bool) {
        self.focus = self.focus.step(back);
    }

    /// The explainer line's two parts around the board word.
    pub(crate) fn explainer(&self) -> (&'static str, &'static str) {
        if self.mode.is_replace() {
            (
                "Removes all existing cards in the ",
                ", replacing with imported cards.",
            )
        } else {
            (
                "Adds imported cards into the ",
                ". Existing cards stay but take new quantities if applicable.",
            )
        }
    }

    /// Sends the import unless one is under way or the input is blank.
    pub(crate) fn submit(&mut self, cx: &mut Context<'_>) {
        if self.loading || self.blank() {
            return;
        }
        self.loading = true;
        self.result = None;
        let request = match self.source {
            Source::Text => Request::Text {
                deck_id: self.deck_id,
                text: self.text.text(),
                target: self.target,
                mode: self.mode,
            },
            Source::Archidekt => Request::Archidekt {
                deck_id: self.deck_id,
                url: self.url.value().to_owned(),
                target: self.target,
                mode: self.mode,
            },
        };
        cx.request(request.into());
    }
}

/// Shows the import hint again.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    cx.shared.hint = Some(HintDialog {
        key: HINT_KEY.to_owned(),
        title: HINT_TITLE.to_owned(),
        body: HINT_BODY.to_owned(),
    });
}

/// Stand-in for clearing the deck's undo history, which package D's undo store owns. Wire it to that store's clear for `deck_id`.
pub(crate) fn clear_undo_history_stub(_deck_id: Uuid) {}

/// Import's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    Text {
        deck_id: Uuid,
        text: String,
        target: Target,
        mode: ImportMode,
    },
    Archidekt {
        deck_id: Uuid,
        url: String,
        target: Target,
        mode: ImportMode,
    },
}

/// Import's results.
#[derive(Debug)]
pub(crate) enum Update {
    Imported(ImportDeckCardsResult),
    Failed(String),
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
    cx.push(Screen::DeckImport(Box::new(ImportView::opening(deck_id))));
    hint::show(cx, HINT_KEY, HINT_TITLE, HINT_BODY);
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let result = async {
        let session = account.fresh().await?;
        match request {
            Request::Text {
                deck_id,
                text,
                target,
                mode,
            } => {
                client
                    .import_deck_cards(deck_id, &text, target.wire(), mode, &session)
                    .await
            }
            Request::Archidekt {
                deck_id,
                url,
                target,
                mode,
            } => {
                client
                    .import_archidekt_deck(deck_id, &url, target.wire(), mode, &session)
                    .await
            }
        }
    }
    .await;
    Some(match result {
        Ok(result) => Update::Imported(result).into(),
        Err(e) => failure(&e, |m| Update::Failed(m).into()),
    })
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    view.loading = false;
    match update {
        Update::Imported(result) => {
            let (tone, message) =
                form::import_outcome(result.imported.len(), result.unresolved.len());
            let kind = match tone {
                ImportTone::Info => ToastKind::Info,
                ImportTone::Error => ToastKind::Error,
                ImportTone::Success => ToastKind::Success,
            };
            cx.toast(kind, message);
            if !result.imported.is_empty() {
                clear_undo_history_stub(view.deck_id);
            }
            view.result = Some(result);
        }
        Update::Failed(message) => cx.toast(ToastKind::Error, message),
    }
}
