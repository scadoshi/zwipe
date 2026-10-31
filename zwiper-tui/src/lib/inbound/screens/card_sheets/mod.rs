//! The sheets the Add, deck cards and Remove screens share: the card filter sheet, the card details sheet, and the printing sheet, with the lists and printings they fetch.
//!
//! These are not screens. A host holds one in its state, hands it keys while it is open, and draws it over itself. Their fetched lists land in the shared stores, so an answer is kept even when the host has closed.

pub(crate) mod details;
pub(crate) mod filter;
pub(crate) mod printing;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_role::CardRoleView;
use zwipe_core::domain::card::oracle_tag::OracleTag;

use super::Screen;
use crate::domain::stores::catalog::Kind;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The sheets' work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Fetches one picker list.
    Catalog(Kind),
    /// Fetches every printing of a card.
    Printings { oracle_id: Uuid },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Catalog(..) => "catalog",
            Self::Printings { .. } => "printings",
        }
    }
}

/// The sheets' results.
#[derive(Debug)]
pub(crate) enum Update {
    /// A word list, or `None` when it failed.
    Words(Kind, Option<Vec<String>>),
    Roles(Option<Vec<CardRoleView>>),
    Tags(Option<Vec<OracleTag>>),
    Printings {
        oracle_id: Uuid,
        printings: Result<Vec<Card>, String>,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::CardSheets(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::CardSheets(update)
    }
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let update = match request {
        Request::Catalog(kind) => match kind {
            Kind::CardRoles => Update::Roles(client.fetch_card_roles().await.ok()),
            Kind::OracleTags => Update::Tags(client.fetch_oracle_tags().await.ok()),
            Kind::Keywords => Update::Words(kind, client.fetch_keywords().await.ok()),
            Kind::OracleWords => Update::Words(kind, client.fetch_oracle_words().await.ok()),
            Kind::CardTypes => Update::Words(kind, client.fetch_card_types().await.ok()),
            Kind::Artists => Update::Words(kind, client.fetch_artists().await.ok()),
            Kind::Sets => Update::Words(kind, client.fetch_sets().await.ok()),
        },
        Request::Printings { oracle_id } => Update::Printings {
            oracle_id,
            printings: client
                .fetch_printings(oracle_id)
                .await
                .map_err(|e| crate::inbound::screens::shown(&e)),
        },
    };
    Some(update.into())
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let catalog = &mut cx.shared.cards.catalog;
    match update {
        Update::Words(kind, words) => match kind {
            Kind::Keywords => catalog.keywords.settle(words),
            Kind::OracleWords => catalog.oracle_words.settle(words),
            Kind::CardTypes => catalog.card_types.settle(words),
            Kind::Artists => catalog.artists.settle(words),
            Kind::Sets => catalog.sets.settle(words),
            Kind::CardRoles | Kind::OracleTags => {}
        },
        Update::Roles(roles) => catalog.card_roles.settle(roles),
        Update::Tags(tags) => catalog.oracle_tags.settle(tags),
        Update::Printings {
            oracle_id,
            printings,
        } => match printings {
            Ok(printings) => {
                cx.shared.cards.printings = Some((oracle_id, printings.clone()));
                if let Some(sheet) = open_printing_sheet(nav.top_mut()) {
                    sheet.loaded(oracle_id, printings);
                }
            }
            Err(message) => {
                cx.toast(ToastKind::Error, message);
                if let Some(sheet) = open_printing_sheet(nav.top_mut()) {
                    sheet.failed(oracle_id);
                }
            }
        },
    }
}

/// The printing sheet open on the screen on top, if any.
fn open_printing_sheet(screen: &mut Screen) -> Option<&mut printing::PrintingSheet> {
    match screen {
        Screen::Swipe(view) => view.printing.as_mut(),
        Screen::DeckCards(view) => view.printing.as_mut(),
        Screen::RemoveCards(view) => view.printing.as_mut(),
        _ => None,
    }
}
