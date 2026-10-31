//! Playstyle, the app's suggested commanders screen: what the user builds, read from every deck they own, over their top 10 suggested commanders and the staples across their decks. Reached from home's `p`.
//!
//! The ranking is fetched 100 deep on every open and the list shows the first 10 that pass the Show row. Above the list, one accordion of the Color map (open on entry, drawn at home's seed), Summary, Card roles and Oracle tags sections sums up the profile the ranking reads. An expanded suggestion offers Create deck, Maybeboard and Remove; a row that leaves fills from the held list, and the screen refetches only when fewer than ten remain or a load is still out. Below the list, the staples show the first 10 that match their own Type and Color rows.
//!
//! Each load takes a ticket and only the newest one's reply lands, so a refetch started by a dropped row or by Clear skips replaces a reply still in flight. A refetch with rows on screen is silent; only the first load shows the skeleton.
//!
//! The app's "Art" chip toggles row thumbnails; the terminal's card rows draw none, so the Show row leaves it out.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::Instant;

use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::models::hints::HINT_SUGGESTED_COMMANDERS;
use zwipe_core::http::contracts::suggestions::{
    HttpBuilderProfile, HttpCommanderSuggestion, HttpCommanderSuggestions, HttpStaple,
};

use super::{Screen, deck_form, failure, home, oracle_tags};
use crate::domain::activity::Bar;
use crate::domain::playstyle::{
    self, HELD_SIZE, LIST_SIZE, StapleColor, TagRow, identities_present, of_identity,
};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::card_rows::CardRows;
use crate::inbound::widgets::collapsible::Accordion;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::info::InfoPopup;
use crate::inbound::widgets::otag::RoleTags;
use crate::inbound::widgets::sheet::Sheet;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// The screen's title and its hint's.
pub(crate) const TITLE: &str = "Playstyle";

/// Said when the first load failed.
pub(crate) const LOAD_FAILED: &str = "Could not load your suggestions";

/// Said when the user has no deck to read from, and in the Color map section when the map has nothing on it.
pub(crate) const COLD_START: &str = "Build a deck to see your playstyle";

/// Said when the server suggested nothing.
pub(crate) const NO_SUGGESTIONS: &str = "No suggestions right now";

/// Said when the Staples rows match nothing.
pub(crate) const NO_STAPLES: &str = "No staples match";

/// The list's group header.
pub(crate) const LIST_TITLE: &str = "Suggested commanders";

/// The Staples panel's group header.
pub(crate) const STAPLES_TITLE: &str = "Staples";

/// The chip that folds a suggestion's reasons.
pub(crate) const WHY: &str = "Why this commander?";

/// The lead-in of the shared tags bullet.
pub(crate) const SHARES_TAGS: &str = "Shares oracle tags with your decks";

/// The More sheet's title.
pub(crate) const MORE_TITLE: &str = "More actions";

/// The More sheet's one action, and its dialog's title.
pub(crate) const CLEAR_SKIPS: &str = "Clear skips";

/// The Clear skips dialog's body.
pub(crate) const CLEAR_BODY: &str =
    "Commanders you skipped will be suggested again. This can't be undone.";

/// The toast after a suggestion is saved to the maybeboard.
pub(crate) const ADDED: &str = "Added to your commander maybeboard";

/// The toast after a suggestion is removed.
pub(crate) const SKIPPED_FOR_A_WEEK: &str = "Skipped for a week";

/// The toast after Clear skips.
pub(crate) const SKIPS_CLEARED: &str = "Skips cleared";

/// The warning when an unverified account already has its one deck.
pub(crate) const VERIFY_TO_CREATE: &str = "Verify your email to create more than 1 deck";

/// One of the profile's sections, in the accordion's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    ColorMap,
    Summary,
    Roles,
    Tags,
}

impl Section {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::ColorMap => "Color map",
            Self::Summary => "Summary",
            Self::Roles => "Card roles",
            Self::Tags => "Oracle tags",
        }
    }

    /// The section's "?" topic; Summary has none.
    pub(crate) fn topic(self) -> Option<InfoPopup> {
        match self {
            Self::ColorMap => Some(home::keys::color_map_topic()),
            Self::Summary => None,
            Self::Roles => Some(InfoPopup::new(
                "Card roles",
                &["How many of your decks have a tag in each role. Tap a role for its tags"],
            )),
            Self::Tags => Some(InfoPopup::new(
                "Oracle tags",
                &["Your 10 most impactful tags, highest first. Tap a tag to see what it means"],
            )),
        }
    }
}

/// A number column of the Oracle tags table, each with its own "?".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Column {
    #[default]
    Decks,
    Cards,
    Impact,
}

impl Column {
    pub(crate) const ALL: [Self; 3] = [Self::Decks, Self::Cards, Self::Impact];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Decks => "Decks",
            Self::Cards => "Cards",
            Self::Impact => "Impact",
        }
    }

    pub(crate) fn topic(self) -> InfoPopup {
        let line = match self {
            Self::Decks => {
                "How many of your decks have this tag. The more decks, the higher its impact"
            }
            Self::Cards => {
                "How many cards carry this tag across those decks. The more cards, the higher its impact"
            }
            Self::Impact => {
                "How much more your decks lean on this tag than Magic cards do overall. Your top tag scores 100"
            }
        };
        InfoPopup::new(self.label(), &[line])
    }
}

/// The list's "?" topic.
pub(crate) fn list_topic() -> InfoPopup {
    InfoPopup::new(
        LIST_TITLE,
        &[
            "Commanders you haven't built, ranked against all your decks, best fit first",
            "Tap a row to see why it fits. Swipe goes further down the same ranking",
        ],
    )
}

/// The Staples group's "?" topic.
pub(crate) fn staples_topic() -> InfoPopup {
    InfoPopup::new(
        STAPLES_TITLE,
        &[
            "The cards in the most of your decks, with the share of your decks that run each",
            "Basic lands are left out. Tap a Type or Color chip to narrow it",
        ],
    )
}

/// What has the keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    /// A profile section's header.
    Header(Section),
    /// A row of the Card roles chart.
    Role(usize),
    /// The Oracle tags table's header, where h/l pick the column whose "?" shows.
    TagHead,
    /// A row of the Oracle tags table.
    Tag(usize),
    /// The Show chips.
    Show,
    /// The suggestion rows, which move their own focus.
    List,
    /// The Staples Type chips.
    Types,
    /// The Staples Color chips.
    Colors,
    /// The staple rows, which move their own focus.
    Staples,
}

/// What covers the screen.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Overlay {
    /// A "?" topic.
    Info(InfoPopup),
    /// The More actions sheet.
    More(Sheet),
    /// The Clear skips dialog, over the More sheet.
    Clear(AlertDialog),
}

/// Whether the user may start another deck: an unverified account stops at one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeckLimit {
    /// Another deck may be created.
    Open,
    /// The account is unverified and already has a deck.
    Reached,
}

impl DeckLimit {
    /// The limit for `user` owning `decks` decks; `None` while the count is still out, which lets creation through as the app does.
    pub(crate) fn of(user: Option<&User>, decks: Option<usize>) -> Self {
        let unverified = user.is_some_and(|u| u.email_verified_at.is_none());
        if unverified && decks.is_some_and(|n| n > 0) {
            Self::Reached
        } else {
            Self::Open
        }
    }
}

/// Starts a deck with `card` as its commander, the app's `create_with`, shared by Playstyle's Create deck, a right swipe in the commander swipe, and the Commander maybeboard.
///
/// At the deck limit it warns, as the app does, for the long toast. Otherwise it opens Create seeded with `card` as the commander in the Commander format, the app's `CreateDeckCommanderSeed` and `Router::CreateDeck`. The printing on screen is the one the deck gets.
pub(crate) fn create_deck_with(cx: &mut Context<'_>, card: &Card, limit: DeckLimit) {
    if limit == DeckLimit::Reached {
        cx.shared
            .toasts
            .push_for(ToastKind::Warning, VERIFY_TO_CREATE.to_owned(), toast::LONG);
        return;
    }
    deck_form::open_create_with_commander(cx, card.clone());
}

/// A suggestion's colors, what the Show chips read.
pub(crate) fn suggestion_colors(suggestion: &HttpCommanderSuggestion) -> &[Color] {
    &suggestion.card.scryfall_data.color_identity
}

/// The Playstyle screen's state.
#[derive(Debug)]
pub(crate) struct PlaystyleView {
    /// When the screen opened, which drives the loading ghosts.
    pub(crate) opened: Instant,
    /// Until the first reply lands.
    pub(crate) is_loading: bool,
    /// The last load failed with nothing held.
    pub(crate) load_failed: bool,
    /// The held ranking, best fit first, 100 deep.
    pub(crate) entries: Vec<HttpCommanderSuggestion>,
    pub(crate) deck_count: u32,
    pub(crate) profile: Option<HttpBuilderProfile>,
    /// The newest load's ticket; only its reply lands.
    pub(crate) ticket: u64,
    /// Whether the newest load is still out.
    pub(crate) load_out: bool,
    /// How many decks the user owns, once fetched, for the create gate.
    pub(crate) decks: Option<usize>,
    /// The Show chip in effect; `None` is All.
    pub(crate) identity: Option<Vec<Color>>,
    /// The Show chip under the cursor: 0 is All, then each identity.
    pub(crate) show_cursor: usize,
    pub(crate) staple_type: Option<CardType>,
    pub(crate) type_cursor: usize,
    pub(crate) staple_color: Option<StapleColor>,
    pub(crate) color_cursor: usize,
    /// The sections present, in order, with which one is open.
    pub(crate) accordion: Accordion,
    pub(crate) sections: Vec<Section>,
    pub(crate) focus: Focus,
    /// When the color map first drew, which drives its sweep.
    pub(crate) map_opened: Option<Instant>,
    /// The role whose tags are revealed, and those tags.
    pub(crate) open_role: Option<String>,
    pub(crate) role_tags: RoleTags,
    /// The table tag whose definition is revealed.
    pub(crate) open_tag: Option<String>,
    /// The table column whose "?" the header shows.
    pub(crate) column: Column,
    pub(crate) rows: CardRows,
    pub(crate) staple_rows: CardRows,
    /// The scryfall id of the suggestion whose reasons are unfolded.
    pub(crate) why_open: Option<Uuid>,
    pub(crate) overlay: Option<Overlay>,
}

impl PlaystyleView {
    /// A view waiting on its first load.
    pub(crate) fn loading() -> Self {
        Self {
            opened: Instant::now(),
            is_loading: true,
            load_failed: false,
            entries: Vec::new(),
            deck_count: 0,
            profile: None,
            ticket: 0,
            load_out: false,
            decks: None,
            identity: None,
            show_cursor: 0,
            staple_type: None,
            type_cursor: 0,
            staple_color: None,
            color_cursor: 0,
            accordion: Accordion::new(&[]),
            sections: Vec::new(),
            focus: Focus::Show,
            map_opened: None,
            open_role: None,
            role_tags: RoleTags::default(),
            open_tag: None,
            column: Column::default(),
            rows: CardRows::default(),
            staple_rows: CardRows::default(),
            why_open: None,
            overlay: None,
        }
    }

    /// No deck to read from: the screen is one line and has no Swipe.
    pub(crate) fn cold_start(&self) -> bool {
        self.deck_count == 0
    }

    /// Whether the column of sections, list and staples shows.
    pub(crate) fn ready(&self) -> bool {
        !self.is_loading && !self.load_failed && !self.cold_start()
    }

    /// Takes a reply: the counts, the profile and the held list, keeping the open section and opening Color map the first time.
    pub(crate) fn land(&mut self, found: HttpCommanderSuggestions) {
        let first = self.is_loading;
        self.deck_count = found.deck_count;
        self.profile = found.profile;
        self.entries = found.suggestions;
        self.load_failed = false;
        self.is_loading = false;
        self.load_out = false;
        self.rebuild_sections(first);
        if first {
            self.map_opened = Some(Instant::now());
            self.focus = match self.sections.first() {
                Some(&section) => Focus::Header(section),
                None => Focus::Show,
            };
        }
        self.settle_focus();
    }

    /// The sections the profile has, keeping whichever was open, or Color map on the first reply.
    fn rebuild_sections(&mut self, first: bool) {
        let open = self
            .accordion
            .open()
            .and_then(|i| self.sections.get(i))
            .copied();
        let mut sections = Vec::new();
        if let Some(profile) = &self.profile {
            sections.push(Section::ColorMap);
            sections.push(Section::Summary);
            if !playstyle::role_bars(&profile.roles, &profile.tags).is_empty() {
                sections.push(Section::Roles);
            }
            if !playstyle::tag_rows(&profile.tags).is_empty() {
                sections.push(Section::Tags);
            }
        }
        let titles: Vec<&str> = sections.iter().map(|s| s.title()).collect();
        let mut accordion = Accordion::new(&titles);
        let keep = if first { Some(Section::ColorMap) } else { open };
        if let Some(i) = keep.and_then(|k| sections.iter().position(|&s| s == k))
            && let Some(section) = accordion.sections.get_mut(i)
        {
            section.open = true;
        }
        self.accordion = accordion;
        self.sections = sections;
    }

    pub(crate) fn is_open(&self, section: Section) -> bool {
        self.sections
            .iter()
            .position(|&s| s == section)
            .and_then(|i| self.accordion.sections.get(i))
            .is_some_and(|s| s.open)
    }

    /// Opens `section` and closes the rest, or closes it when it is open.
    pub(crate) fn toggle(&mut self, section: Section) {
        let Some(i) = self.sections.iter().position(|&s| s == section) else {
            return;
        };
        self.accordion.focus = i;
        self.accordion.toggle_focused();
    }

    /// The identities among the held list plus the chosen one, so a chip stays after its last commander goes.
    pub(crate) fn identities(&self) -> Vec<Vec<Color>> {
        identities_present(
            self.entries
                .iter()
                .map(suggestion_colors)
                .chain(self.identity.as_deref()),
        )
    }

    /// The held list narrowed by the Show chip, in held order.
    pub(crate) fn filtered(&self) -> Vec<HttpCommanderSuggestion> {
        of_identity(&self.entries, suggestion_colors, self.identity.as_deref())
    }

    /// The rows the list shows: the first ten filtered.
    pub(crate) fn shown(&self) -> Vec<HttpCommanderSuggestion> {
        playstyle::shown(&self.filtered())
    }

    pub(crate) fn staples(&self) -> &[HttpStaple] {
        self.profile.as_ref().map_or(&[], |p| p.staples.as_slice())
    }

    pub(crate) fn staple_types(&self) -> Vec<CardType> {
        playstyle::staple_types_present(self.staples(), playstyle::staple_type_line)
    }

    pub(crate) fn staple_colors(&self) -> Vec<StapleColor> {
        playstyle::staple_colors_present(self.staples(), playstyle::staple_identity)
    }

    /// The staples the Type and Color rows let through, the first ten.
    pub(crate) fn matching_staples(&self) -> Vec<HttpStaple> {
        playstyle::matching_staples(
            self.staples(),
            playstyle::staple_type_line,
            playstyle::staple_identity,
            self.staple_type,
            self.staple_color,
        )
    }

    pub(crate) fn role_bars(&self) -> Vec<Bar> {
        self.profile
            .as_ref()
            .map(|p| playstyle::role_bars(&p.roles, &p.tags))
            .unwrap_or_default()
    }

    pub(crate) fn tag_rows(&self) -> Vec<TagRow> {
        self.profile
            .as_ref()
            .map(|p| playstyle::tag_rows(&p.tags))
            .unwrap_or_default()
    }

    /// Everything the focus can land on, top to bottom.
    pub(crate) fn stops(&self) -> Vec<Focus> {
        let mut stops = Vec::new();
        if !self.ready() {
            return stops;
        }
        for &section in &self.sections {
            stops.push(Focus::Header(section));
            if !self.is_open(section) {
                continue;
            }
            match section {
                Section::Roles => stops.extend((0..self.role_bars().len()).map(Focus::Role)),
                Section::Tags => {
                    stops.push(Focus::TagHead);
                    stops.extend((0..self.tag_rows().len()).map(Focus::Tag));
                }
                Section::ColorMap | Section::Summary => {}
            }
        }
        stops.push(Focus::Show);
        if !self.shown().is_empty() {
            stops.push(Focus::List);
        }
        if !self.staples().is_empty() {
            stops.push(Focus::Types);
            stops.push(Focus::Colors);
            if !self.matching_staples().is_empty() {
                stops.push(Focus::Staples);
            }
        }
        stops
    }

    /// Moves the focus one stop `toward` the next or the previous, entering a group of rows at its near end. Returns whether it moved.
    pub(crate) fn step(&mut self, toward: Toward) -> bool {
        let stops = self.stops();
        let Some(at) = stops.iter().position(|&s| s == self.focus) else {
            if let Some(&first) = stops.first() {
                self.focus = first;
            }
            return true;
        };
        let Some(&focus) = toward.neighbor(at).and_then(|i| stops.get(i)) else {
            return false;
        };
        match focus {
            Focus::List => {
                let last = self.shown().len().saturating_sub(1);
                self.rows
                    .set_focused(if toward == Toward::Next { 0 } else { last });
            }
            Focus::Staples => {
                let last = self.matching_staples().len().saturating_sub(1);
                self.staple_rows
                    .set_focused(if toward == Toward::Next { 0 } else { last });
            }
            _ => {}
        }
        self.focus = focus;
        true
    }

    /// Keeps the focus on a stop that still exists after the lists or sections changed: the nearest row of the same chart, else the section's header, else the first stop.
    pub(crate) fn settle_focus(&mut self) {
        let stops = self.stops();
        if stops.contains(&self.focus) {
            return;
        }
        self.focus = match self.focus {
            Focus::Role(_) => stops
                .iter()
                .rev()
                .find(|s| matches!(s, Focus::Role(_)))
                .copied()
                .unwrap_or(Focus::Header(Section::Roles)),
            Focus::Tag(_) | Focus::TagHead => stops
                .iter()
                .rev()
                .find(|s| matches!(s, Focus::Tag(_) | Focus::TagHead))
                .copied()
                .unwrap_or(Focus::Header(Section::Tags)),
            Focus::List => Focus::Show,
            Focus::Staples => Focus::Colors,
            other => other,
        };
        if !stops.contains(&self.focus) {
            self.focus = stops.first().copied().unwrap_or(Focus::Show);
        }
    }

    /// The "?" topic for what has the focus, if it has one.
    pub(crate) fn topic(&self) -> Option<InfoPopup> {
        match self.focus {
            Focus::Header(section) => self.is_open(section).then(|| section.topic()).flatten(),
            Focus::Role(_) => Section::Roles.topic(),
            Focus::TagHead => Some(self.column.topic()),
            Focus::Tag(_) => Section::Tags.topic(),
            Focus::List => Some(list_topic()),
            Focus::Staples => Some(staples_topic()),
            Focus::Show | Focus::Types | Focus::Colors => None,
        }
    }

    /// Reveals the profile's tags under `role`, or hides them when they show.
    pub(crate) fn toggle_role(&mut self, role: &str) {
        if self.open_role.as_deref() == Some(role) {
            self.open_role = None;
            self.role_tags = RoleTags::default();
            return;
        }
        let tags = self
            .profile
            .as_ref()
            .map(|p| playstyle::role_tags(&p.tags, role))
            .unwrap_or_default();
        self.open_role = Some(role.to_owned());
        self.role_tags = RoleTags::new(tags);
    }

    /// Reveals a table tag's definition, or hides it when it shows.
    pub(crate) fn toggle_tag(&mut self, slug: &str) {
        if self.open_tag.as_deref() == Some(slug) {
            self.open_tag = None;
        } else {
            self.open_tag = Some(slug.to_owned());
        }
    }

    /// Picks the Show chip at `index` (0 is All); picking the selected identity again returns to All.
    pub(crate) fn pick_identity(&mut self, index: usize) {
        let Some(chosen) = index.checked_sub(1) else {
            self.identity = None;
            self.show_cursor = 0;
            self.settle_focus();
            return;
        };
        let Some(identity) = self.identities().into_iter().nth(chosen) else {
            return;
        };
        if self.identity.as_ref() == Some(&identity) {
            self.identity = None;
            self.show_cursor = 0;
        } else {
            self.identity = Some(identity);
        }
        self.settle_focus();
    }

    /// Picks the Type chip at `index` (0 is All); the selected type again is All.
    pub(crate) fn pick_type(&mut self, index: usize) {
        let picked = index
            .checked_sub(1)
            .and_then(|i| self.staple_types().into_iter().nth(i));
        self.staple_type = if picked == self.staple_type {
            None
        } else {
            picked
        };
        if self.staple_type.is_none() {
            self.type_cursor = 0;
        }
        self.settle_focus();
    }

    /// Picks the Color chip at `index` (0 is All); the selected color again is All.
    pub(crate) fn pick_color(&mut self, index: usize) {
        let picked = index
            .checked_sub(1)
            .and_then(|i| self.staple_colors().into_iter().nth(i));
        self.staple_color = if picked == self.staple_color {
            None
        } else {
            picked
        };
        if self.staple_color.is_none() {
            self.color_cursor = 0;
        }
        self.settle_focus();
    }

    /// Starts a new load whose reply replaces any still out.
    pub(crate) fn refetch(&mut self, cx: &mut Context<'_>) {
        self.ticket = self.ticket.wrapping_add(1);
        self.load_out = true;
        cx.request(
            Request::Load {
                ticket: self.ticket,
            }
            .into(),
        );
    }

    /// Drops the row with `oracle_id`; the next held suggestion of that identity moves up, and the server is asked again only when the filtered list runs short of ten or a load that predates the drop is still out.
    pub(crate) fn drop_row(&mut self, oracle_id: Uuid, cx: &mut Context<'_>) {
        self.entries
            .retain(|s| s.card.scryfall_data.oracle_id != Some(oracle_id));
        if self.filtered().len() < LIST_SIZE || self.load_out {
            self.refetch(cx);
        }
        self.settle_focus();
    }

    /// The create gate for the signed-in user.
    pub(crate) fn deck_limit(&self, cx: &Context<'_>) -> DeckLimit {
        DeckLimit::of(cx.shared.user.as_ref(), self.decks)
    }

    /// Refetches when the commander swipe closed, taking the identity its Filter left picked, and swaps in a printing picked from a row: into the staples when a staple is open, else into the held list.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        if let Some(closed) = cx.shared.caches.handoff.commander_swipe.take() {
            if closed.identity != self.identity {
                self.show_cursor = 0;
            }
            self.identity = closed.identity;
            self.refetch(cx);
            self.settle_focus();
        }
        if self.staple_rows.expanded().is_some() {
            if let Some(profile) = &mut self.profile {
                self.staple_rows
                    .take_printing(cx, &mut profile.staples, |s| &mut s.card);
            }
        } else {
            self.rows
                .take_printing(cx, &mut self.entries, |s| &mut s.card);
        }
    }

    /// Whether something is moving: the loading ghosts, or the color map's sweep.
    pub(crate) fn animating(&self) -> bool {
        let sweeping = self.is_open(Section::ColorMap)
            && self
                .map_opened
                .is_some_and(|t| t.elapsed() < crate::inbound::widgets::color_map::entrance());
        self.is_loading || sweeping
    }
}

/// The screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// The ranking and profile, under a ticket.
    Load { ticket: u64 },
    /// The user's decks, for the create gate.
    DeckProfiles,
    /// Saves a suggestion to the commander maybeboard by oracle id.
    Maybeboard(Uuid),
    /// Hides a suggestion for a week by oracle id.
    Dismiss(Uuid),
    /// Lifts every dismissal.
    ClearSkips,
}

/// The screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded {
        ticket: u64,
        found: Box<HttpCommanderSuggestions>,
    },
    LoadFailed {
        ticket: u64,
        message: String,
    },
    /// How many decks the user owns.
    DeckProfiles(usize),
    Maybeboarded(Uuid),
    Dismissed(Uuid),
    SkipsCleared,
    /// A row action or Clear skips failed with this message.
    ActionFailed(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Playstyle(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Playstyle(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut PlaystyleView> {
    match screen {
        Screen::Playstyle(view) => Some(view),
        _ => None,
    }
}

/// Opens Playstyle: fetches the ranking, the user's decks and the tag catalog, and shows the hint the first time.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.shared.caches.handoff.commander_swipe = None;
    let mut view = PlaystyleView::loading();
    view.refetch(cx);
    cx.push(Screen::Playstyle(Box::new(view)));
    cx.request(Request::DeckProfiles.into());
    oracle_tags::ensure_catalog(cx);
    hint::once(cx, HINT_SUGGESTED_COMMANDERS);
}

/// Shows the hint again, for `H`.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_SUGGESTED_COMMANDERS);
}

/// Runs `call` for an action, its success as `done` and its failure as a message.
async fn act<T>(
    call: impl Future<Output = Result<T, ClientError>>,
    done: Update,
) -> Option<screens::Update> {
    match call.await {
        Ok(_) => Some(done.into()),
        Err(e) => Some(failure(&e, |message| Update::ActionFailed(message).into())),
    }
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    match request {
        Request::Load { ticket } => {
            let reply = async {
                client
                    .commander_suggestions(HELD_SIZE, &account.fresh().await?)
                    .await
            }
            .await;
            match reply {
                Ok(found) => Some(
                    Update::Loaded {
                        ticket,
                        found: Box::new(found),
                    }
                    .into(),
                ),
                Err(e) => Some(failure(&e, |message| {
                    Update::LoadFailed { ticket, message }.into()
                })),
            }
        }
        Request::DeckProfiles => {
            match async { client.fetch_deck_profiles(&account.fresh().await?).await }.await {
                Ok(profiles) => Some(Update::DeckProfiles(profiles.len()).into()),
                Err(ClientError::Unauthorized(_)) => Some(screens::Update::SessionEnded),
                // Quiet, as the app's `run_quiet`: the gate only lets creation through.
                Err(_) => None,
            }
        }
        Request::Maybeboard(oracle_id) => {
            act(
                async {
                    client
                        .add_commander_maybeboard_card(oracle_id, &account.fresh().await?)
                        .await
                },
                Update::Maybeboarded(oracle_id),
            )
            .await
        }
        Request::Dismiss(oracle_id) => {
            act(
                async {
                    client
                        .dismiss_commander(oracle_id, &account.fresh().await?)
                        .await
                },
                Update::Dismissed(oracle_id),
            )
            .await
        }
        Request::ClearSkips => {
            act(
                async {
                    client
                        .clear_commander_dismissals(&account.fresh().await?)
                        .await
                },
                Update::SkipsCleared,
            )
            .await
        }
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let view = nav.find_mut(pick);
    match update {
        Update::Loaded { ticket, found } => {
            if let Some(view) = view.filter(|v| v.ticket == ticket) {
                view.land(*found);
            }
        }
        Update::LoadFailed { ticket, message } => {
            cx.shared
                .toasts
                .push_for(ToastKind::Error, message, toast::NORMAL);
            if let Some(view) = view.filter(|v| v.ticket == ticket) {
                view.load_failed = view.entries.is_empty();
                view.is_loading = false;
                view.load_out = false;
                view.settle_focus();
            }
        }
        Update::DeckProfiles(count) => {
            if let Some(view) = view {
                view.decks = Some(count);
            }
        }
        Update::Maybeboarded(oracle_id) => {
            if let Some(view) = view {
                view.drop_row(oracle_id, cx);
            }
            cx.shared
                .toasts
                .push_for(ToastKind::Success, ADDED.to_owned(), toast::QUICK);
        }
        Update::Dismissed(oracle_id) => {
            if let Some(view) = view {
                view.drop_row(oracle_id, cx);
            }
            cx.shared
                .toasts
                .push_for(ToastKind::Info, SKIPPED_FOR_A_WEEK.to_owned(), toast::QUICK);
        }
        Update::SkipsCleared => {
            cx.shared
                .toasts
                .push_for(ToastKind::Info, SKIPS_CLEARED.to_owned(), toast::QUICK);
            if let Some(view) = view {
                view.refetch(cx);
            }
        }
        Update::ActionFailed(message) => {
            cx.shared
                .toasts
                .push_for(ToastKind::Error, message, toast::NORMAL);
        }
    }
}
