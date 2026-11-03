//! Playstyle, the app's suggested commanders screen: what the user builds, read from every deck they own, over their top 10 suggested commanders and the staples across their decks. Reached from home's `p`.
//!
//! What the screen knows and decides (the held ranking and its loads, the profile's sections and charts, the Show and Staples chips, the create gate, the edits and their toasts) is zwipe-ui's [`State`]; this view adds the accordion over the model's sections, the focus and chip cursors, the revealed role and tag, the card rows, the More sheet and Clear dialog, and the `?` topics.
//!
//! The app's "Art" chip toggles row thumbnails; the terminal's card rows draw none, so the Show row leaves it out. The app's Swipe is an overlay whose right swipe goes through this model's create gate; here it is the commander swipe screen, which takes its own [`DeckLimit`], so the view keeps the deck count the model reads too.
//!
//! The app's staple rows offer a printing the model never swaps in (its `PickPrinting` reaches only suggestions), so the view keeps the printings picked on staple rows itself, until the next load lands.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::Instant;

use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::models::hints::HINT_SUGGESTED_COMMANDERS;
use zwipe_core::http::contracts::suggestions::{
    HttpCommanderSuggestion, HttpCommanderSuggestions, HttpStaple,
};
use zwipe_ui::deck::VERIFY_TO_CREATE;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::playstyle::{
    Action, CARDS_COLUMN, DECKS_COLUMN, Destination, Effect, HELD_SIZE, IMPACT_COLUMN, LIST_TITLE,
    ROLES_TITLE, STAPLES_TITLE, Section, State, Status, TAGS_TITLE,
};

use super::{Screen, deck_form, home, oracle_tags};
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

/// A section's "?" topic; Summary has none.
pub(crate) fn topic(section: Section) -> Option<InfoPopup> {
    match section {
        Section::ColorMap => Some(home::keys::color_map_topic()),
        Section::Summary => None,
        Section::Roles => Some(InfoPopup::new(
            ROLES_TITLE,
            &["How many of your decks have a tag in each role. Tap a role for its tags"],
        )),
        Section::Tags => Some(InfoPopup::new(
            TAGS_TITLE,
            &["Your 10 most impactful tags, highest first. Tap a tag to see what it means"],
        )),
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
            Self::Decks => DECKS_COLUMN,
            Self::Cards => CARDS_COLUMN,
            Self::Impact => IMPACT_COLUMN,
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

/// Starts a deck with `card` as its commander, the app's `create_with`, for a right swipe in the commander swipe.
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

/// Whether the signed-in user's email is verified; with no user only the server can refuse, as the app passes for no session.
fn verified(cx: &Context<'_>) -> bool {
    cx.shared
        .user
        .as_ref()
        .is_none_or(|u| u.email_verified_at.is_some())
}

/// The Playstyle screen's state: the model plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct PlaystyleView {
    pub(crate) model: State,
    /// When the screen opened, which drives the loading ghosts.
    pub(crate) opened: Instant,
    /// How many decks the user owns, once fetched, for the commander swipe's create gate. The model holds the same count for its own Create deck but does not show it.
    pub(crate) decks: Option<usize>,
    /// The newest ranking load sent, so a landed reply drops the staple printings.
    load: Option<Ticket>,
    /// Printings picked on staple rows, each in place of the staple sharing its oracle id.
    pub(crate) staple_printings: Vec<Card>,
    /// The Show chip under the cursor: 0 is All, then each identity.
    pub(crate) show_cursor: usize,
    pub(crate) type_cursor: usize,
    pub(crate) color_cursor: usize,
    /// The model's sections as last laid out, with which one is open.
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

impl Default for PlaystyleView {
    /// A view waiting on its first load.
    fn default() -> Self {
        Self {
            model: State::new(),
            opened: Instant::now(),
            decks: None,
            load: None,
            staple_printings: Vec::new(),
            show_cursor: 0,
            type_cursor: 0,
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
}

impl PlaystyleView {
    /// Whether the column of sections, list and staples shows.
    pub(crate) fn ready(&self) -> bool {
        self.model.status() == Status::Loaded
    }

    /// Lays the accordion out again when the model's sections changed, keeping whichever was open, or opening the one that opens on entry when `first`.
    fn rebuild_sections(&mut self, first: bool) {
        let sections = self.model.sections();
        if !first && sections == self.sections {
            return;
        }
        let open = if first {
            sections.iter().copied().find(|s| s.opens_on_entry())
        } else {
            self.accordion
                .open()
                .and_then(|i| self.sections.get(i))
                .copied()
        };
        let titles: Vec<&str> = sections.iter().map(|s| s.title()).collect();
        let mut accordion = Accordion::new(&titles);
        if let Some(i) = open.and_then(|k| sections.iter().position(|&s| s == k))
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

    /// The staples the Type and Color rows let through, each in the printing picked on its row.
    pub(crate) fn matching_staples(&self) -> Vec<HttpStaple> {
        let mut staples = self.model.matching_staples();
        for staple in &mut staples {
            let oracle_id = staple.card.scryfall_data.oracle_id;
            if let Some(printing) = self
                .staple_printings
                .iter()
                .find(|p| oracle_id.is_some() && p.scryfall_data.oracle_id == oracle_id)
            {
                staple.card = printing.clone();
            }
        }
        staples
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
                Section::Roles => {
                    stops.extend((0..self.model.role_bars().len()).map(Focus::Role));
                }
                Section::Tags => {
                    stops.push(Focus::TagHead);
                    stops.extend((0..self.model.tag_rows().len()).map(Focus::Tag));
                }
                Section::ColorMap | Section::Summary => {}
            }
        }
        stops.push(Focus::Show);
        if !self.model.rows().is_empty() {
            stops.push(Focus::List);
        }
        if self.model.shows_staples() {
            stops.push(Focus::Types);
            stops.push(Focus::Colors);
            if !self.model.matching_staples().is_empty() {
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
                let last = self.model.rows().len().saturating_sub(1);
                self.rows
                    .set_focused(if toward == Toward::Next { 0 } else { last });
            }
            Focus::Staples => {
                let last = self.model.matching_staples().len().saturating_sub(1);
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
            Focus::Header(section) => self.is_open(section).then(|| topic(section)).flatten(),
            Focus::Role(_) => topic(Section::Roles),
            Focus::TagHead => Some(self.column.topic()),
            Focus::Tag(_) => topic(Section::Tags),
            Focus::List => Some(list_topic()),
            Focus::Staples => Some(staples_topic()),
            Focus::Show | Focus::Types | Focus::Colors => None,
        }
    }

    /// The section whose header `[?]` takes the focus look: the one `?` explains from the focus.
    pub(crate) fn about_section(&self) -> Option<Section> {
        match self.focus {
            Focus::Header(section) => Some(section),
            Focus::Role(_) => Some(Section::Roles),
            Focus::Tag(_) => Some(Section::Tags),
            Focus::TagHead
            | Focus::List
            | Focus::Staples
            | Focus::Show
            | Focus::Types
            | Focus::Colors => None,
        }
    }

    /// Reveals the profile's tags under `role`, or hides them when they show.
    pub(crate) fn toggle_role(&mut self, role: &str) {
        if self.open_role.as_deref() == Some(role) {
            self.open_role = None;
            self.role_tags = RoleTags::default();
            return;
        }
        self.open_role = Some(role.to_owned());
        self.role_tags = RoleTags::new(self.model.role_tags(role));
    }

    /// Reveals a table tag's definition, or hides it when it shows.
    pub(crate) fn toggle_tag(&mut self, slug: &str) {
        if self.open_tag.as_deref() == Some(slug) {
            self.open_tag = None;
        } else {
            self.open_tag = Some(slug.to_owned());
        }
    }

    /// The create gate the commander swipe takes for the signed-in user.
    pub(crate) fn deck_limit(&self, cx: &Context<'_>) -> DeckLimit {
        DeckLimit::of(cx.shared.user.as_ref(), self.decks)
    }

    /// Refetches when the commander swipe closed, taking the identity its Filter left picked, and swaps in a printing picked from a row: into the staples when a staple is open, else into the held list.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        if let Some(closed) = cx.shared.caches.handoff.commander_swipe.take() {
            if closed.identity.as_deref() != self.model.identity() {
                self.show_cursor = 0;
                let pick = closed
                    .identity
                    .map_or(Action::ShowAll, Action::ToggleIdentity);
                dispatch(self, pick, cx);
            }
            dispatch(self, Action::SwipeClosed, cx);
        }
        let Some(printing) = cx.shared.caches.handoff.printing.take() else {
            return;
        };
        let id = printing.scryfall_data.id;
        if self.staple_rows.expanded().is_some() {
            let oracle_id = printing.scryfall_data.oracle_id;
            if oracle_id.is_some()
                && self
                    .model
                    .staples()
                    .iter()
                    .any(|s| s.card.scryfall_data.oracle_id == oracle_id)
            {
                self.staple_printings
                    .retain(|p| p.scryfall_data.oracle_id != oracle_id);
                self.staple_printings.push(printing);
                self.staple_rows.expand(id);
            }
        } else {
            dispatch(self, Action::PickPrinting(Box::new(printing)), cx);
            if self
                .model
                .entries()
                .iter()
                .any(|s| s.card.scryfall_data.id == id)
            {
                self.rows.expand(id);
            }
        }
    }

    /// Whether something is moving: the loading ghosts, or the color map's sweep.
    pub(crate) fn animating(&self) -> bool {
        let sweeping = self.is_open(Section::ColorMap)
            && self
                .map_opened
                .is_some_and(|t| t.elapsed() < crate::inbound::widgets::color_map::entrance());
        self.model.status() == Status::Loading || sweeping
    }
}

/// Applies `action` to the model, performs the effects it returns in order, lays the sections out again and keeps the focus on a stop that still exists. The first load to land opens the section that opens on entry, starts the map's sweep and puts the focus on the first header.
pub(crate) fn dispatch(view: &mut PlaystyleView, action: Action, cx: &mut Context<'_>) {
    let was_loading = view.model.status() == Status::Loading;
    let mut seed = None;
    for effect in view.model.update(action) {
        match effect {
            Effect::OfferHint(offer) => {
                hint::once(cx, offer.key());
            }
            Effect::FetchSuggestions(ticket) => {
                view.load = Some(ticket);
                cx.request(Request::Load { ticket }.into());
            }
            Effect::FetchDeckProfiles => cx.request(Request::DeckProfiles.into()),
            Effect::AddToMaybeboard { ticket, oracle_id } => {
                cx.request(Request::Maybeboard { ticket, oracle_id }.into());
            }
            Effect::DismissCommander { ticket, oracle_id } => {
                cx.request(Request::Dismiss { ticket, oracle_id }.into());
            }
            Effect::ClearDismissals(ticket) => cx.request(Request::ClearSkips { ticket }.into()),
            Effect::SeedCommander(card) => seed = Some(*card),
            Effect::Navigate(Destination::Previous) => cx.back(),
            Effect::Navigate(Destination::CreateDeck) => {
                if let Some(card) = seed.take() {
                    deck_form::open_create_with_commander(cx, card);
                }
            }
            Effect::Toast(toast) => cx.show(toast),
        }
    }
    let first = was_loading && view.model.status() != Status::Loading;
    view.rebuild_sections(first);
    if first && view.model.status() == Status::Loaded {
        view.map_opened = Some(Instant::now());
        view.focus = view
            .sections
            .first()
            .map_or(Focus::Show, |&section| Focus::Header(section));
    }
    view.settle_focus();
}

/// Starts a deck with `card` through the model's create gate, for a row's Create deck.
pub(crate) fn create(view: &mut PlaystyleView, card: &Card, cx: &mut Context<'_>) {
    let verified = verified(cx);
    dispatch(
        view,
        Action::Create {
            card: Box::new(card.clone()),
            verified,
        },
        cx,
    );
}

/// The screen's work, each request but the deck count carrying its ticket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// The ranking and profile.
    Load { ticket: Ticket },
    /// The user's decks, for the create gate.
    DeckProfiles,
    /// Saves a suggestion to the commander maybeboard by oracle id.
    Maybeboard { ticket: Ticket, oracle_id: Uuid },
    /// Hides a suggestion for a week by oracle id.
    Dismiss { ticket: Ticket, oracle_id: Uuid },
    /// Lifts every dismissal.
    ClearSkips { ticket: Ticket },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Load { .. } => "load",
            Self::DeckProfiles => "deck_profiles",
            Self::Maybeboard { .. } => "maybeboard",
            Self::Dismiss { .. } => "dismiss",
            Self::ClearSkips { .. } => "clear_skips",
        }
    }
}

/// The screen's results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded {
        ticket: Ticket,
        found: Box<HttpCommanderSuggestions>,
    },
    /// How many decks the user owns.
    DeckProfiles(usize),
    Saved {
        ticket: Ticket,
    },
    Dismissed {
        ticket: Ticket,
    },
    Cleared {
        ticket: Ticket,
    },
    /// A request failed; `message` is the error toast's text.
    Failed {
        ticket: Ticket,
        message: String,
    },
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

/// Opens Playstyle: the model offers the hint and fetches the ranking and the user's decks, and the tag catalog warms for the rows.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.shared.caches.handoff.commander_swipe = None;
    let mut view = PlaystyleView::default();
    dispatch(&mut view, Action::Load, cx);
    oracle_tags::ensure_catalog(cx);
    cx.push(Screen::Playstyle(Box::new(view)));
}

/// Shows the hint again, for `H`.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_SUGGESTED_COMMANDERS);
}

/// Runs the ranking edit `call`, its success as `done` and its failure with the message to toast.
async fn edit<T>(
    ticket: Ticket,
    call: impl Future<Output = Result<T, ClientError>>,
    done: Update,
) -> Option<screens::Update> {
    match call.await {
        Ok(_) => Some(done.into()),
        Err(e) => Some(screens::failure(&e, |message| {
            Update::Failed { ticket, message }.into()
        })),
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
                Err(e) => Some(screens::failure(&e, |message| {
                    Update::Failed { ticket, message }.into()
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
        Request::Maybeboard { ticket, oracle_id } => {
            edit(
                ticket,
                async {
                    client
                        .add_commander_maybeboard_card(oracle_id, &account.fresh().await?)
                        .await
                },
                Update::Saved { ticket },
            )
            .await
        }
        Request::Dismiss { ticket, oracle_id } => {
            edit(
                ticket,
                async {
                    client
                        .dismiss_commander(oracle_id, &account.fresh().await?)
                        .await
                },
                Update::Dismissed { ticket },
            )
            .await
        }
        Request::ClearSkips { ticket } => {
            edit(
                ticket,
                async {
                    client
                        .clear_commander_dismissals(&account.fresh().await?)
                        .await
                },
                Update::Cleared { ticket },
            )
            .await
        }
    }
}

/// Hands a reply to the model. A failure's error toast goes up even when the screen has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    if let Update::Failed { message, .. } = &update {
        cx.shared
            .toasts
            .push_for(ToastKind::Error, message.clone(), toast::NORMAL);
    }
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    let action = match update {
        Update::Loaded { ticket, found } => {
            if view.load == Some(ticket) {
                view.load = None;
                view.staple_printings.clear();
            }
            Action::Loaded { ticket, found }
        }
        Update::DeckProfiles(count) => {
            view.decks = Some(count);
            Action::DeckProfilesLoaded { count }
        }
        Update::Saved { ticket } => Action::Saved { ticket },
        Update::Dismissed { ticket } => Action::Dismissed { ticket },
        Update::Cleared { ticket } => Action::Cleared { ticket },
        Update::Failed { ticket, .. } => Action::Failed { ticket },
    };
    dispatch(view, action, cx);
}
