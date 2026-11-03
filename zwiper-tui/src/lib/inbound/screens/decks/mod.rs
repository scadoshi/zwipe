//! The deck list: Group by, the Show filter with its tag fold, the decks in collapsible groups, Create and the More sheet.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]; this view adds the focus in each chip row and the list, and whether the More sheet is up.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_core::domain::deck::DeckProfile;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::deck::list::{DeckGroup, GroupBy};
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::deck_list::{
    Action, Destination, Effect, MoreItem, ShowChip, State, Status,
};

use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::menu::{Menu, MenuItem};
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The part of the screen that takes the arrow keys.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    GroupBy,
    Show,
    /// The tag row under Show, while it is open.
    Tags,
    #[default]
    List,
}

/// A line in the list: a group's header or a deck.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Row {
    Header { group: DeckGroup, collapsed: bool },
    Deck(Box<DeckProfile>),
}

/// The deck list's state: the model plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct DeckList {
    pub(crate) model: State,
    pub(crate) focus: Focus,
    /// The focused chip in the Group by, Show and tag rows.
    pub(crate) group_focus: usize,
    pub(crate) show_focus: usize,
    pub(crate) tag_focus: usize,
    /// The focused list row.
    pub(crate) selected: usize,
    /// The More sheet, while open.
    pub(crate) more: Option<Menu>,
    /// Whether a list with a deck has placed the selection yet; the first one selects its first deck rather than its group's header.
    pub(crate) placed: bool,
}

impl Default for DeckList {
    fn default() -> Self {
        Self {
            model: State::new(),
            focus: Focus::default(),
            group_focus: 0,
            show_focus: 0,
            tag_focus: 0,
            selected: 0,
            more: None,
            placed: false,
        }
    }
}

impl DeckList {
    /// The list's rows: each group's header, then its decks unless it is collapsed. None while a message stands in for the list.
    pub(crate) fn rows(&self) -> Vec<Row> {
        if self.model.message().is_some() {
            return Vec::new();
        }
        let mut rows = Vec::new();
        for group in self.model.groups() {
            let collapsed = self.model.is_collapsed(&group.key());
            let decks = group.decks.clone();
            rows.push(Row::Header { group, collapsed });
            if !collapsed {
                rows.extend(decks.into_iter().map(|d| Row::Deck(Box::new(d))));
            }
        }
        rows
    }

    /// The areas the focus can move through, top to bottom.
    fn areas(&self) -> Vec<Focus> {
        if !self.model.chips_shown() {
            return vec![Focus::List];
        }
        let mut areas = vec![Focus::GroupBy, Focus::Show];
        if self.model.tags_open() && !self.model.tags().is_empty() {
            areas.push(Focus::Tags);
        }
        areas.push(Focus::List);
        areas
    }

    /// Moves the focus to the next area down, or the previous up, stopping at the ends.
    pub(crate) fn step_area(&mut self, toward: Toward) {
        let areas = self.areas();
        let here = areas.iter().position(|a| *a == self.focus).unwrap_or(0);
        let next = toward.clamped(here, areas.len());
        if let Some(area) = areas.get(next) {
            self.focus = *area;
        }
    }

    /// Keeps every focus index in range after the rows or chips change.
    pub(crate) fn clamp(&mut self) {
        if !self.areas().contains(&self.focus) {
            self.focus = Focus::List;
        }
        self.show_focus = self
            .show_focus
            .min(self.model.show_chips().len().saturating_sub(1));
        self.tag_focus = self
            .tag_focus
            .min(self.model.tags().len().saturating_sub(1));
        self.selected = self.selected.min(self.rows().len().saturating_sub(1));
    }

    /// The action the focused Show chip takes.
    pub(crate) fn show_action(&self) -> Option<Action> {
        self.model
            .show_chips()
            .get(self.show_focus)
            .map(ShowChip::action)
    }

    /// The action the focused tag in the tag row takes.
    pub(crate) fn tag_action(&self) -> Option<Action> {
        self.model
            .tags()
            .into_iter()
            .nth(self.tag_focus)
            .map(|(key, _)| Action::ToggleTag(key))
    }

    /// The action the Group by chip at `index` takes.
    pub(crate) fn group_by_action(index: usize) -> Option<Action> {
        GroupBy::ALL.get(index).copied().map(Action::PickGroupBy)
    }
}

/// Shows the hint again, whether or not it has been seen.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, OneTimeHint::DeckList.key());
}

/// The More sheet, its items the model's.
pub(crate) fn more_menu() -> Menu {
    Menu::new(
        zwipe_ui::screens::deck_list::MORE_TITLE,
        MoreItem::ALL
            .iter()
            .map(|item| MenuItem::new(item.label()))
            .collect(),
    )
}

/// Whether the session's user may pass the Create check: a verified email, or no user to ask, so only the server can refuse.
pub(crate) fn verified(cx: &Context<'_>) -> bool {
    cx.shared
        .user
        .as_ref()
        .is_none_or(|u| u.email_verified_at.is_some())
}

/// Applies `action` to the model, brings the focus in line with it, and performs the effects it returns, in order.
pub(crate) fn dispatch(list: &mut DeckList, action: Action, cx: &mut Context<'_>) {
    let effects = list.model.update(action);
    if !list.placed && list.model.status() == Status::Loaded {
        list.placed = true;
        list.selected = list
            .rows()
            .iter()
            .position(|r| matches!(r, Row::Deck(_)))
            .unwrap_or(0);
    }
    // A reload keeps the focus where it was, for the list that comes back.
    if list.model.status() != Status::Loading {
        list.clamp();
    }
    for effect in effects {
        match effect {
            Effect::RefreshUser => cx.request(Request::RefreshUser.into()),
            Effect::FetchDecks(ticket) => {
                cx.shared.status.clear();
                cx.request(Request::Fetch { ticket }.into());
            }
            Effect::OfferHint(offer) => {
                hint::once(cx, offer.key());
            }
            Effect::Navigate(Destination::Home) => cx.back(),
            Effect::Navigate(Destination::Deck(id)) => screens::deck::open(cx, id),
            Effect::Navigate(Destination::CreateDeck) => screens::deck_form::open_create(cx),
            Effect::Navigate(Destination::CommanderMaybeboard) => {
                screens::commander_maybeboard::open(cx);
            }
            Effect::Navigate(Destination::Playstyle) => screens::playstyle::open(cx),
            Effect::Toast(toast) => cx.show(toast),
        }
    }
}

/// The deck list's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// Refreshes the user, so the Create check reads a current verification. A failure only logs.
    RefreshUser,
    /// Fetches the decks for the load holding `ticket`.
    Fetch { ticket: Ticket },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(self) -> &'static str {
        match self {
            Self::RefreshUser => "refresh_user",
            Self::Fetch { .. } => "load",
        }
    }
}

/// The deck list's results, each fetch's carrying its ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    /// The user as the server has it now.
    User(Box<User>),
    Loaded {
        ticket: Ticket,
        decks: Vec<DeckProfile>,
    },
    Failed {
        ticket: Ticket,
        error: String,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Decks(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Decks(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut DeckList> {
    match screen {
        Screen::Decks(list) => Some(list),
        _ => None,
    }
}

/// Opens the deck list over the current screen, fetches the decks and refreshes the user.
pub(crate) fn open(cx: &mut Context<'_>) {
    let mut list = DeckList::default();
    dispatch(&mut list, Action::Load, cx);
    cx.push(Screen::Decks(Box::new(list)));
}

/// Fetches the decks again, keeping how they are folded and narrowed.
pub(crate) fn reload(list: &mut DeckList, cx: &mut Context<'_>) {
    dispatch(list, Action::Load, cx);
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    match request {
        Request::RefreshUser => {
            match async { client.fetch_user(&account.fresh().await?).await }.await {
                Ok(user) => Some(Update::User(Box::new(user)).into()),
                Err(_) => None,
            }
        }
        Request::Fetch { ticket } => Some(
            match async { client.fetch_deck_profiles(&account.fresh().await?).await }.await {
                Ok(decks) => Update::Loaded { ticket, decks }.into(),
                Err(e) => screens::failure(&e, |error| Update::Failed { ticket, error }.into()),
            },
        ),
    }
}

/// Hands a reply to the model. A fresh user lands in the session whether or not the list is still up, and a failure raises its toast even when the screen has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::User(user) => {
            cx.shared.username = Some(user.username.to_string());
            cx.shared.user = Some(*user);
            return;
        }
        Update::Loaded { ticket, decks } => Action::Loaded { ticket, decks },
        Update::Failed { ticket, error } => {
            cx.toast(ToastKind::Error, error);
            Action::Failed { ticket }
        }
    };
    if let Some(list) = nav.find_mut(pick) {
        dispatch(list, action, cx);
    }
}

/// The ticket the latest deck fetch went out with.
#[cfg(test)]
pub(crate) fn last_ticket(pending: &[screens::Request]) -> Ticket {
    pending
        .iter()
        .rev()
        .find_map(|r| match r {
            screens::Request::Decks(Request::Fetch { ticket }) => Some(*ticket),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no deck fetch went out: {pending:?}"))
}
