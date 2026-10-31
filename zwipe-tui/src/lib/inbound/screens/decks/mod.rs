//! The deck list: Group by, the Show filter with its tag fold, the decks in collapsible groups, Create and the More sheet.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use zwipe_client::ClientError;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::deck::DeckProfile;
use zwipe_core::domain::user::User;

use super::{Screen, failure};
use crate::domain::deck::list::{self, DeckGroup, GroupBy, Show};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hint::{self, HintDialog};
use crate::inbound::widgets::menu::{Menu, MenuItem};
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The key the server stores the deck list's hint under.
pub(crate) const HINT_KEY: &str = "deck_list";

/// The deck list's hint, as the app words it.
const HINT_TITLE: &str = "Your decks";

/// The hint's bullets.
const HINT_BODY: &str = "• Tap a deck to open it\n• Group by folds the list into sections, Show narrows it by color or tag\n• Create starts a new deck";

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

/// A chip in the Show row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ShowChip {
    All,
    Color(Color),
    /// The fold that opens the tag row.
    Tags,
    /// A picked tag kept in view while the tag row is folded.
    Picked {
        key: String,
        label: String,
    },
}

/// A line in the list: a group's header or a deck.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Row {
    Header { group: DeckGroup, collapsed: bool },
    Deck(Box<DeckProfile>),
}

/// The user's decks and how they are folded and narrowed. Group by, Show, the tag fold and the collapsed groups last for one visit.
#[derive(Debug, Default)]
pub(crate) struct DeckList {
    pub(crate) decks: Vec<DeckProfile>,
    pub(crate) loading: bool,
    /// Why the last load failed.
    pub(crate) error: Option<String>,
    pub(crate) group_by: GroupBy,
    pub(crate) show: Show,
    /// Whether the tag row is open.
    pub(crate) tags_open: bool,
    /// Keys of the collapsed groups.
    pub(crate) collapsed: BTreeSet<String>,
    pub(crate) focus: Focus,
    /// The focused chip in the Group by, Show and tag rows.
    pub(crate) group_focus: usize,
    pub(crate) show_focus: usize,
    pub(crate) tag_focus: usize,
    /// The focused list row.
    pub(crate) selected: usize,
    /// The More sheet, while open.
    pub(crate) more: Option<Menu>,
    /// Whether the hint has had its one try this visit.
    pub(crate) hint_tried: bool,
}

impl DeckList {
    pub(crate) fn loading() -> Self {
        Self {
            loading: true,
            ..Self::default()
        }
    }

    /// Takes the loaded decks, keeping the focus in range. The first load selects the first deck rather than its group's header.
    pub(crate) fn set_decks(&mut self, decks: Vec<DeckProfile>) {
        self.decks = decks;
        self.loading = false;
        self.error = None;
        if !self.hint_tried {
            self.selected = self
                .rows()
                .iter()
                .position(|r| matches!(r, Row::Deck(_)))
                .unwrap_or(0);
        }
        self.clamp();
    }

    /// Whether the chip rows show: only once a non-empty list has loaded.
    pub(crate) fn chips_shown(&self) -> bool {
        !self.loading && self.error.is_none() && !self.decks.is_empty()
    }

    /// Every distinct tag across the decks, as a key and a label.
    pub(crate) fn tags(&self) -> Vec<(String, String)> {
        list::all_tags(&self.decks)
    }

    /// The Show row: All, the five colors, the Tags fold when any deck has a tag, then the picked tags while it is folded.
    pub(crate) fn show_chips(&self) -> Vec<ShowChip> {
        let mut chips = vec![ShowChip::All];
        chips.extend(Color::all().map(ShowChip::Color));
        let tags = self.tags();
        if !tags.is_empty() {
            chips.push(ShowChip::Tags);
            if !self.tags_open {
                chips.extend(
                    tags.into_iter()
                        .filter(|(key, _)| self.show.tags.contains(key))
                        .map(|(key, label)| ShowChip::Picked { key, label }),
                );
            }
        }
        chips
    }

    /// The decks that pass Show, in their groups.
    pub(crate) fn groups(&self) -> Vec<DeckGroup> {
        let shown = list::shown(&self.decks, &self.show);
        if shown.is_empty() {
            return Vec::new();
        }
        list::group(&shown, self.group_by)
    }

    /// The list's rows: each group's header, then its decks unless it is collapsed.
    pub(crate) fn rows(&self) -> Vec<Row> {
        let mut rows = Vec::new();
        for group in self.groups() {
            let collapsed = self.collapsed.contains(&group.key());
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
        if !self.chips_shown() {
            return vec![Focus::List];
        }
        let mut areas = vec![Focus::GroupBy, Focus::Show];
        if self.tags_open && !self.tags().is_empty() {
            areas.push(Focus::Tags);
        }
        areas.push(Focus::List);
        areas
    }

    /// Moves the focus to the next area down, or up with `back`.
    pub(crate) fn step_area(&mut self, back: bool) {
        let areas = self.areas();
        let here = areas.iter().position(|a| *a == self.focus).unwrap_or(0);
        let next = if back {
            here.checked_sub(1).unwrap_or(here)
        } else {
            here.saturating_add(1).min(areas.len().saturating_sub(1))
        };
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
            .min(self.show_chips().len().saturating_sub(1));
        self.tag_focus = self.tag_focus.min(self.tags().len().saturating_sub(1));
        self.selected = self.selected.min(self.rows().len().saturating_sub(1));
    }

    /// Presses the focused Show chip.
    pub(crate) fn press_show(&mut self) {
        match self.show_chips().get(self.show_focus).cloned() {
            Some(ShowChip::All) => self.show.clear(),
            Some(ShowChip::Color(color)) => self.show.toggle_color(color),
            Some(ShowChip::Tags) => self.tags_open = !self.tags_open,
            Some(ShowChip::Picked { key, .. }) => self.show.toggle_tag(&key),
            None => {}
        }
        self.clamp();
    }

    /// Toggles the focused tag in the tag row.
    pub(crate) fn press_tag(&mut self) {
        if let Some((key, _)) = self.tags().get(self.tag_focus) {
            self.show.toggle_tag(key);
        }
        self.clamp();
    }

    /// Picks the Group by mode at `index`, opening every group.
    pub(crate) fn pick_group_by(&mut self, index: usize) {
        if let Some(by) = GroupBy::ALL.get(index) {
            self.group_by = *by;
            self.collapsed.clear();
            self.clamp();
        }
    }

    /// Fetches the decks again, keeping how they are folded and narrowed.
    pub(crate) fn reload(&mut self, cx: &mut Context<'_>) {
        cx.shared.status.clear();
        self.loading = true;
        self.error = None;
        cx.request(Request::Load.into());
    }
}

/// Shows the hint again, whether or not it has been seen.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    cx.shared.hint = Some(HintDialog {
        key: HINT_KEY.to_owned(),
        title: HINT_TITLE.to_owned(),
        body: HINT_BODY.to_owned(),
    });
}

/// The More sheet.
pub(crate) fn more_menu() -> Menu {
    Menu::new(
        "More actions",
        vec![
            MenuItem::new("Commander maybeboard"),
            MenuItem::new("Playstyle"),
        ],
    )
}

/// Create, guarded the way the app guards it: an unverified user with a deck already gets a warning instead.
pub(crate) fn create(list: &DeckList, cx: &mut Context<'_>) {
    let unverified = cx
        .shared
        .user
        .as_ref()
        .is_some_and(|u| u.email_verified_at.is_none());
    if unverified && !list.decks.is_empty() {
        cx.toast(
            ToastKind::Warning,
            "Verify your email to create more than 1 deck".to_owned(),
        );
        return;
    }
    screens::deck_form::open_create(cx);
}

/// Stand-in for the Commander maybeboard screen, which package B builds. Wire it to that screen's `open`.
pub(crate) fn open_commander_maybeboard_stub(cx: &mut Context<'_>) {
    cx.toast(
        ToastKind::Info,
        "Commander maybeboard is not in the terminal yet".to_owned(),
    );
}

/// Stand-in for the Playstyle screen, which package B builds. Wire it to that screen's `open`.
pub(crate) fn open_playstyle_stub(cx: &mut Context<'_>) {
    cx.toast(
        ToastKind::Info,
        "Playstyle is not in the terminal yet".to_owned(),
    );
}

/// The deck list's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// Refreshes the user, so the Create guard sees a fresh verification, then fetches the decks.
    Load,
}

/// The deck list's results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded(Vec<DeckProfile>),
    /// The decks, with the user as the server has it now.
    LoadedWithUser(Vec<DeckProfile>, Box<User>),
    Failed(String),
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
    cx.shared.status.clear();
    cx.push(Screen::Decks(DeckList::loading()));
    cx.request(Request::Load.into());
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    match request {
        Request::Load => Some(
            match async {
                let session = account.fresh().await?;
                // A failed refresh only leaves the user as it was.
                let user = client.fetch_user(&session).await.ok();
                let decks = client.fetch_deck_profiles(&session).await?;
                Ok::<_, ClientError>((decks, user))
            }
            .await
            {
                Ok((decks, Some(user))) => Update::LoadedWithUser(decks, Box::new(user)).into(),
                Ok((decks, None)) => Update::Loaded(decks).into(),
                Err(e) => failure(&e, |m| Update::Failed(m).into()),
            },
        ),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let update = match update {
        Update::LoadedWithUser(decks, user) => {
            cx.shared.username = Some(user.username.to_string());
            cx.shared.user = Some(*user);
            Update::Loaded(decks)
        }
        other => other,
    };
    let Some(list) = nav.find_mut(pick) else {
        return;
    };
    match update {
        Update::Loaded(decks) | Update::LoadedWithUser(decks, _) => {
            list.set_decks(decks);
            if !list.decks.is_empty() && !list.hint_tried {
                list.hint_tried = true;
                hint::show(cx, HINT_KEY, HINT_TITLE, HINT_BODY);
            }
        }
        Update::Failed(error) => {
            list.loading = false;
            list.error = Some(error.clone());
            list.clamp();
            cx.toast(ToastKind::Error, error);
        }
    }
}
