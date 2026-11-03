//! The sheets the Add, deck cards and Remove screens share: the card filter sheet, the card details sheet, and the printing sheet, with the lists and printings they fetch.
//!
//! These are not screens. A host holds one in its state, hands it keys while it is open, and draws it over itself. Their fetched lists land in the shared stores, so an answer is kept even when the host has closed; a list that fails while a filter sheet is open on top tells that sheet, which warns once per opening.

pub(crate) mod details;
pub(crate) mod filter;
pub(crate) mod printing;

use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_role::CardRoleView;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;
use zwipe_core::domain::deck::DeckTagView;
use zwipe_ui::stores::catalog::Kind;

use super::{Screen, deck_form};
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
    Reminders(Option<HashMap<String, String>>),
    Franchises(Option<Vec<UbFranchiseView>>),
    DeckTags(Option<Vec<DeckTagView>>),
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
            Kind::KeywordReminders => {
                Update::Reminders(client.fetch_keyword_reminders().await.ok())
            }
            Kind::UbFranchises => Update::Franchises(client.fetch_ub_franchises().await.ok()),
            Kind::DeckTags => Update::DeckTags(
                async { client.fetch_deck_tags(&account.fresh().await?).await }
                    .await
                    .ok(),
            ),
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
    let now = Utc::now();
    let list = !matches!(update, Update::Printings { .. });
    match update {
        Update::Words(kind, words) => match kind {
            Kind::Keywords => catalog.keywords.settle(words, now),
            Kind::OracleWords => catalog.oracle_words.settle(words, now),
            Kind::CardTypes => catalog.card_types.settle(words, now),
            Kind::Artists => catalog.artists.settle(words, now),
            Kind::Sets => catalog.sets.settle(words, now),
            Kind::KeywordReminders
            | Kind::CardRoles
            | Kind::OracleTags
            | Kind::UbFranchises
            | Kind::DeckTags => {}
        },
        Update::Roles(roles) => catalog.card_roles.settle(roles, now),
        Update::Tags(tags) => catalog.oracle_tags.settle(tags, now),
        Update::Reminders(reminders) => catalog.keyword_reminders.settle(reminders, now),
        Update::Franchises(franchises) => catalog.ub_franchises.settle(franchises, now),
        Update::DeckTags(tags) => catalog.deck_tags.settle(tags, now),
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
    if list
        && cx.shared.cards.catalog.any_public_failed()
        && let Some(sheet) = open_filter_sheet(nav.top_mut())
    {
        sheet.lists_failed(cx);
    }
}

/// The filter sheet open on the screen on top, if any.
fn open_filter_sheet(screen: &mut Screen) -> Option<&mut filter::FilterSheet> {
    match screen {
        Screen::Swipe(view) => view.sheet.as_mut(),
        Screen::DeckCards(view) => view.sheet.as_mut(),
        Screen::RemoveCards(view) => view.sheet.as_mut(),
        Screen::CommanderSwipe(view) => view.sheet.as_mut(),
        Screen::CommanderMaybeboard(view) => view.sheet.as_mut(),
        Screen::DeckForm(form) => match &mut form.overlay {
            Some(deck_form::Overlay::Swipe(picker)) => picker.sheet.as_mut(),
            _ => None,
        },
        _ => None,
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
