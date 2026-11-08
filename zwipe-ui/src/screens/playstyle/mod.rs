//! Playstyle: what the user builds, read from every deck they own, over their top 10 suggested commanders with the reasons each ranked, and the staples across their decks.
//!
//! The client keeps a [`State`] while the screen is up and sends [`Action::Load`] when it opens. The ranking is fetched [`HELD_SIZE`] deep and held; the list shows the first [`LIST_SIZE`] that pass the Show chip, and a row that leaves fills from the held list, fetching again only when fewer than ten remain or a load is still out. The numbers and filters come from [`crate::playstyle`] and the Create check from [`crate::deck::create_gate`].
//!
//! Which profile sections are open, the revealed role and tag, the Art chip, the expanded row and whether the Clear skips dialog is up stay in the client. So does the Swipe overlay: its swipes save and skip on their own, and the client sends [`Action::SwipeClosed`] when it closes and [`Action::Create`] for a right swipe.

#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::{self, Color};
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::suggestions::{
    HttpBuilderProfile, HttpCommanderSuggestion, HttpCommanderSuggestions, HttpStaple,
    HttpSuggestionReasons,
};

use crate::activity::Bar;
use crate::colors::ColorCounts;
use crate::deck;
use crate::generation::{Generation, Ticket};
use crate::playstyle::{self, ProfileSummary, SharedTags, StapleColor, TagRow};
pub use crate::playstyle::{HELD_SIZE, LIST_SIZE};
use crate::toast::{Toast, ToastKind};

/// The screen's title, also the hint's.
pub const TITLE: &str = "Playstyle";

/// The line in place of the screen when the ranking failed to load.
pub const LOAD_FAILED: &str = "Couldn't load your suggestions";

/// The line in place of the screen when the user has no deck with cards to read from.
pub const COLD_START: &str = "No playstyle yet";

/// The line in place of the list when the server suggested nothing.
pub const NO_SUGGESTIONS: &str = "No suggestions right now";

/// The suggestions list's group header.
pub const LIST_TITLE: &str = "Suggested commanders";

/// The Summary section's title.
pub const SUMMARY_TITLE: &str = "Summary";

/// The Colors section's title.
pub const COLORS_TITLE: &str = "Colors";

/// The card roles chart's section title.
pub const ROLES_TITLE: &str = "Card roles";

/// The oracle tags table's section title.
pub const TAGS_TITLE: &str = "Oracle tags";

/// The Summary row with the deck count.
pub const SUMMARY_DECKS: &str = "Decks";

/// The Summary row with the most common color set.
pub const SUMMARY_TOP_COLORS: &str = "Top colors";

/// The Summary row with the color in the most decks.
pub const SUMMARY_TOP_COLOR: &str = "Top color";

/// The oracle tags table's first column.
pub const TAG_COLUMN: &str = "Tag";

/// The oracle tags table's deck count column.
pub const DECKS_COLUMN: &str = "Decks";

/// The oracle tags table's card count column.
pub const CARDS_COLUMN: &str = "Cards";

/// The oracle tags table's impact column.
pub const IMPACT_COLUMN: &str = "Impact";

/// The Staples panel's group header.
pub const STAPLES_TITLE: &str = "Staples";

/// The line in place of the staples when the Type and Color rows match nothing.
pub const NO_STAPLES: &str = "No staples match";

/// The Show row's label.
pub const SHOW: &str = "Show";

/// The chip on each chip row that picks everything.
pub const ALL: &str = "All";

/// The Show chip that toggles the rows' art.
pub const ART: &str = "Art";

/// The Staples Type row's label.
pub const TYPE: &str = "Type";

/// The Staples Color row's label.
pub const COLOR: &str = "Color";

/// The chip that folds a suggestion's reasons, also their heading where they sit open.
pub const WHY: &str = "Why this commander?";

/// The lead-in of a suggestion's shared tags bullet.
pub const SHARES_TAGS: &str = "Shares Oracle tags with your decks";

/// A row's button that starts a deck with its commander.
pub const CREATE_DECK: &str = "Create deck";

/// A row's button that saves its commander to the commander maybeboard.
pub const MAYBEBOARD: &str = "Maybeboard";

/// A row's button that skips its commander for a week.
pub const REMOVE: &str = "Remove";

/// The button that opens the suggested swipe.
pub const SWIPE: &str = "Swipe";

/// The button that asks to clear the skips, and the title of the dialog it opens.
pub const CLEAR_SKIPS: &str = "Clear skips";

/// The Clear skips dialog's line.
pub const CLEAR_BODY: &str =
    "Commanders you skipped will be suggested again. This can't be undone.";

/// The Clear skips dialog's button that closes it without clearing.
pub const CANCEL: &str = "Cancel";

/// The Clear skips dialog's button that clears.
pub const CLEAR: &str = "Clear";

/// The toast after Maybeboard saves a suggestion.
pub const ADDED: &str = "Added to your commander maybeboard";

/// The toast after Remove skips a suggestion.
pub const SKIPPED: &str = "Skipped for a week";

/// The toast after Clear skips.
pub const SKIPS_CLEARED: &str = "Skips cleared";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The screen the user came from, from Back: Home's Playstyle panel or the deck list's More menu.
    Previous,
    /// The Create deck form, after [`Effect::SeedCommander`].
    CreateDeck,
}

/// One of the profile's sections, in the order they draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    /// A bar per color of the user's cards, colorless last.
    Colors,
    /// The deck count, the top colors and the top color.
    Summary,
    /// The chart of card roles across the decks.
    Roles,
    /// The table of oracle tags across the decks.
    Tags,
}

impl Section {
    /// Every section, top to bottom.
    pub const ALL: [Self; 4] = [Self::Colors, Self::Summary, Self::Roles, Self::Tags];

    /// The section's header, also its accordion key in the app.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Colors => COLORS_TITLE,
            Self::Summary => SUMMARY_TITLE,
            Self::Roles => ROLES_TITLE,
            Self::Tags => TAGS_TITLE,
        }
    }

    /// Whether the section opens with the screen. Only Colors does.
    #[must_use]
    pub fn opens_on_entry(self) -> bool {
        self == Self::Colors
    }
}

/// Where the ranking stands, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The first load is out: the sections, the Show row and the list ghost.
    Loading,
    /// A load failed with nothing held: [`LOAD_FAILED`] alone.
    Failed,
    /// No deck has cards to read from: [`COLD_START`] alone, and no Swipe.
    ColdStart,
    /// The sections, the Show row, the list and the staples.
    Loaded,
}

/// The reasons under one suggestion, from [`reasons`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reasons {
    /// The oracle tags the commander shares with the user's decks, grouped by role; `None` when it shares none, which drops the [`SHARES_TAGS`] bullet.
    pub tags: Option<SharedTags>,
    /// One bullet per color the user builds in, from [`playstyle::color_lines`].
    pub colors: Vec<String>,
}

/// A suggestion's reasons against `deck_count` decks, or `None` when it has neither a shared tag nor a color line, so the block draws nothing at all.
#[must_use]
pub fn reasons(reasons: &HttpSuggestionReasons, deck_count: u32) -> Option<Reasons> {
    let colors = playstyle::color_lines(&reasons.colors, deck_count);
    let tags = (!reasons.tags.is_empty()).then(|| playstyle::group_shared_tags(&reasons.tags));
    (tags.is_some() || !colors.is_empty()).then_some(Reasons { tags, colors })
}

/// What a user does on Playstyle, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Fetches the ranking and the user's decks and offers the hint, as the screen does when it opens.
    Load,
    /// The ranking arrived for the fetch holding `ticket`.
    Loaded {
        /// The ticket from the [`Effect::FetchSuggestions`] this answers.
        ticket: Ticket,
        /// The ranking, the deck count and the profile.
        found: Box<HttpCommanderSuggestions>,
    },
    /// The user's decks arrived, for the create gate. A failure sends nothing, and the gate counts no deck.
    DeckProfilesLoaded {
        /// How many decks the user owns.
        count: usize,
    },
    /// The request holding `ticket` failed. The client has already reported it and raised its error toast.
    Failed {
        /// The ticket from the effect this answers.
        ticket: Ticket,
    },
    /// Picks the Show row's All chip.
    ShowAll,
    /// Picks the Show chip for this identity, or All when it is already picked.
    ToggleIdentity(Vec<Color>),
    /// Picks the Staples Type row's All chip.
    TypeAll,
    /// Picks the Staples Type chip for this type, or All when it is already picked.
    ToggleType(CardType),
    /// Picks the Staples Color row's All chip.
    ColorAll,
    /// Picks the Staples Color chip, or All when it is already picked.
    ToggleColor(StapleColor),
    /// Starts a deck with this commander, through the create gate: a row's Create deck or a right swipe.
    Create {
        /// The commander, in the printing on screen.
        card: Box<Card>,
        /// Whether the user's email is verified. A client with no session passes `true`, so only the server can refuse.
        verified: bool,
    },
    /// Saves the suggestion with this oracle id to the commander maybeboard.
    Maybeboard(Uuid),
    /// The save holding `ticket` went through.
    Saved {
        /// The ticket from the [`Effect::AddToMaybeboard`] this answers.
        ticket: Ticket,
    },
    /// Skips the suggestion with this oracle id for a week.
    Dismiss(Uuid),
    /// The skip holding `ticket` went through.
    Dismissed {
        /// The ticket from the [`Effect::DismissCommander`] this answers.
        ticket: Ticket,
    },
    /// Clears every skip, from the Clear skips dialog.
    ClearSkips,
    /// The clear holding `ticket` went through.
    Cleared {
        /// The ticket from the [`Effect::ClearDismissals`] this answers.
        ticket: Ticket,
    },
    /// Swaps in a printing picked from a row, on the suggestion sharing its oracle id. A card with no oracle id, or one no suggestion shares, changes nothing. The client keeps the row expanded under the new printing's id.
    PickPrinting(Box<Card>),
    /// The Swipe overlay closed. Its swipes may have saved or skipped commanders, so the ranking fetches again.
    SwipeClosed,
    /// Goes back to the screen before.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches the ranking [`HELD_SIZE`] deep; answers with [`Action::Loaded`] or [`Action::Failed`] carrying this ticket.
    FetchSuggestions(Ticket),
    /// Fetches the user's decks, quietly; answers with [`Action::DeckProfilesLoaded`], or nothing on failure.
    FetchDeckProfiles,
    /// Saves a commander to the commander maybeboard; answers with [`Action::Saved`] or [`Action::Failed`].
    AddToMaybeboard {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The commander's oracle id.
        oracle_id: Uuid,
    },
    /// Skips a commander for a week; answers with [`Action::Dismissed`] or [`Action::Failed`].
    DismissCommander {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The commander's oracle id.
        oracle_id: Uuid,
    },
    /// Clears every skip; answers with [`Action::Cleared`] or [`Action::Failed`].
    ClearDismissals(Ticket),
    /// Hands this commander to Create deck, which opens with it in the commander slot.
    SeedCommander(Box<Card>),
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// A request that changes the ranking, waiting for its reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edit {
    Maybeboard(Uuid),
    Dismiss(Uuid),
    ClearSkips,
}

/// A reply that an [`Edit`] went through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Landed {
    Saved,
    Dismissed,
    Cleared,
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// Playstyle for one visit. Starts loading with nothing held and every chip on All; send [`Action::Load`] to start the fetches.
///
/// Every request takes a fresh [`Ticket`] from one [`Generation`], so a ticket names exactly one request. A load's reply lands only while it is the newest load, and a load still out when a row drops is sent again, since its reply left before the drop and could bring the row back. A refetch with the ranking on screen is silent.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    /// Until the first load's reply lands.
    loading: bool,
    /// A load failed while nothing was held; only a landed load clears it.
    failed: bool,
    /// The held ranking, best fit first.
    entries: Vec<HttpCommanderSuggestion>,
    /// Decks that fed the profile; 0 is the cold start.
    deck_count: u32,
    profile: Option<HttpBuilderProfile>,
    /// The newest load, while it is out.
    load_waiting: Option<Ticket>,
    /// The user's decks, for the create gate, once fetched.
    deck_profiles: Option<usize>,
    /// The Show chip in effect; `None` is All.
    identity: Option<Vec<Color>>,
    /// The Staples Type chip in effect; `None` is All.
    staple_type: Option<CardType>,
    /// The Staples Color chip in effect; `None` is All.
    staple_color: Option<StapleColor>,
    edits: Vec<(Ticket, Edit)>,
    hint: HintOffer,
    tickets: Generation,
}

impl Default for State {
    fn default() -> Self {
        Self {
            loading: true,
            failed: false,
            entries: Vec::new(),
            deck_count: 0,
            profile: None,
            load_waiting: None,
            deck_profiles: None,
            identity: None,
            staple_type: None,
            staple_color: None,
            edits: Vec::new(),
            hint: HintOffer::Waiting,
            tickets: Generation::default(),
        }
    }
}

impl State {
    /// The screen as it first draws: loading, nothing held, every chip on All.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// The hint is offered once per visit, on the first [`Action::Load`]. A load reply whose ticket a newer load has replaced changes nothing. The Staples chips go back to All whenever the Staples panel stops drawing, as the app's panel does when it unmounts.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        let effects = self.apply(action);
        if !self.shows_staples() {
            self.staple_type = None;
            self.staple_color = None;
        }
        effects
    }

    fn apply(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load => self.load(),
            Action::Loaded { ticket, found } => {
                if self.settle_load(ticket) {
                    let HttpCommanderSuggestions {
                        suggestions,
                        deck_count,
                        profile,
                    } = *found;
                    self.deck_count = deck_count;
                    self.profile = profile;
                    self.entries = suggestions;
                    self.failed = false;
                }
                Vec::new()
            }
            Action::DeckProfilesLoaded { count } => {
                self.deck_profiles = Some(count);
                Vec::new()
            }
            Action::Failed { ticket } => {
                if self.settle_load(ticket) {
                    self.failed = self.entries.is_empty();
                } else {
                    self.edits.retain(|(t, _)| *t != ticket);
                }
                Vec::new()
            }
            Action::ShowAll => {
                self.identity = None;
                Vec::new()
            }
            Action::ToggleIdentity(identity) => {
                self.identity = toggled(self.identity.as_ref(), identity);
                Vec::new()
            }
            Action::TypeAll => {
                self.staple_type = None;
                Vec::new()
            }
            Action::ToggleType(card_type) => {
                self.staple_type = toggled(self.staple_type.as_ref(), card_type);
                Vec::new()
            }
            Action::ColorAll => {
                self.staple_color = None;
                Vec::new()
            }
            Action::ToggleColor(color) => {
                self.staple_color = toggled(self.staple_color.as_ref(), color);
                Vec::new()
            }
            Action::Create { card, verified } => {
                match deck::create_gate(verified, self.deck_profiles.unwrap_or(0)) {
                    Ok(()) => vec![
                        Effect::SeedCommander(card),
                        Effect::Navigate(Destination::CreateDeck),
                    ],
                    Err(warning) => vec![Effect::Toast(warning)],
                }
            }
            Action::Maybeboard(oracle_id) => self.edit(Edit::Maybeboard(oracle_id)),
            Action::Saved { ticket } => self.land(ticket, Landed::Saved),
            Action::Dismiss(oracle_id) => self.edit(Edit::Dismiss(oracle_id)),
            Action::Dismissed { ticket } => self.land(ticket, Landed::Dismissed),
            Action::ClearSkips => self.edit(Edit::ClearSkips),
            Action::Cleared { ticket } => self.land(ticket, Landed::Cleared),
            Action::PickPrinting(card) => {
                let target = card.scryfall_data.oracle_id;
                if target.is_some()
                    && let Some(slot) = self
                        .entries
                        .iter_mut()
                        .find(|s| s.card.scryfall_data.oracle_id == target)
                {
                    slot.card = *card;
                }
                Vec::new()
            }
            Action::SwipeClosed => vec![self.fetch()],
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
        }
    }

    fn load(&mut self) -> Vec<Effect> {
        let mut effects = Vec::new();
        if self.hint == HintOffer::Waiting {
            self.hint = HintOffer::Offered;
            effects.push(Effect::OfferHint(OneTimeHint::SuggestedCommanders));
        }
        effects.push(self.fetch());
        effects.push(Effect::FetchDeckProfiles);
        effects
    }

    /// Sends a load whose reply replaces any still out.
    fn fetch(&mut self) -> Effect {
        let ticket = self.tickets.next_ticket();
        self.load_waiting = Some(ticket);
        Effect::FetchSuggestions(ticket)
    }

    /// Ends the load holding `ticket` when it is the newest. Returns whether it was.
    fn settle_load(&mut self, ticket: Ticket) -> bool {
        if self.load_waiting != Some(ticket) {
            return false;
        }
        self.load_waiting = None;
        self.loading = false;
        true
    }

    /// Sends the request for `edit` and waits for its reply.
    fn edit(&mut self, edit: Edit) -> Vec<Effect> {
        let ticket = self.tickets.next_ticket();
        let effect = match edit {
            Edit::Maybeboard(oracle_id) => Effect::AddToMaybeboard { ticket, oracle_id },
            Edit::Dismiss(oracle_id) => Effect::DismissCommander { ticket, oracle_id },
            Edit::ClearSkips => Effect::ClearDismissals(ticket),
        };
        self.edits.push((ticket, edit));
        vec![effect]
    }

    /// Lands the reply to the edit holding `ticket`, when it is waiting and is the kind `landed` answers.
    fn land(&mut self, ticket: Ticket, landed: Landed) -> Vec<Effect> {
        let Some(at) = self.edits.iter().position(|(t, _)| *t == ticket) else {
            return Vec::new();
        };
        let (_, edit) = self.edits.remove(at);
        match (edit, landed) {
            (Edit::Maybeboard(oracle_id), Landed::Saved) => {
                self.drop_row(oracle_id, Toast::quick(ToastKind::Success, ADDED))
            }
            (Edit::Dismiss(oracle_id), Landed::Dismissed) => {
                self.drop_row(oracle_id, Toast::quick(ToastKind::Info, SKIPPED))
            }
            (Edit::ClearSkips, Landed::Cleared) => vec![
                Effect::Toast(Toast::quick(ToastKind::Info, SKIPS_CLEARED)),
                self.fetch(),
            ],
            (edit, _) => {
                self.edits.insert(at, (ticket, edit));
                Vec::new()
            }
        }
    }

    /// Drops the suggestion with `oracle_id`, so the next held one of the Show identity moves up, then raises `toast`. The ranking fetches again first when the filtered list runs short of [`LIST_SIZE`] or a load from before the drop is still out.
    fn drop_row(&mut self, oracle_id: Uuid, toast: Toast) -> Vec<Effect> {
        self.entries
            .retain(|s| s.card.scryfall_data.oracle_id != Some(oracle_id));
        let mut effects = Vec::new();
        if self.filtered().len() < LIST_SIZE || self.load_waiting.is_some() {
            effects.push(self.fetch());
        }
        effects.push(Effect::Toast(toast));
        effects
    }

    /// Where the ranking stands.
    #[must_use]
    pub fn status(&self) -> Status {
        if self.loading {
            Status::Loading
        } else if self.failed {
            Status::Failed
        } else if self.deck_count == 0 {
            Status::ColdStart
        } else {
            Status::Loaded
        }
    }

    /// The line in place of the whole screen: [`LOAD_FAILED`] or [`COLD_START`]. `None` while loading or once loaded.
    #[must_use]
    pub fn message(&self) -> Option<&'static str> {
        match self.status() {
            Status::Failed => Some(LOAD_FAILED),
            Status::ColdStart => Some(COLD_START),
            Status::Loading | Status::Loaded => None,
        }
    }

    /// Whether Swipe shows: once a load has counted a deck. Hidden while the first load is out.
    #[must_use]
    pub fn shows_swipe(&self) -> bool {
        self.deck_count != 0
    }

    /// Decks that fed the profile, what the reasons and staples count against.
    #[must_use]
    pub fn deck_count(&self) -> u32 {
        self.deck_count
    }

    /// The profile the ranking reads, once a load has brought one.
    #[must_use]
    pub fn profile(&self) -> Option<&HttpBuilderProfile> {
        self.profile.as_ref()
    }

    /// The profile sections that draw, top to bottom: Colors and Summary whenever there is a profile, Card roles when a role has decks, Oracle tags when a tag does.
    #[must_use]
    pub fn sections(&self) -> Vec<Section> {
        if self.profile.is_none() {
            return Vec::new();
        }
        let has_roles = !self.role_bars().is_empty();
        let has_tags = !self.tag_rows().is_empty();
        Section::ALL
            .into_iter()
            .filter(|section| match section {
                Section::Colors | Section::Summary => true,
                Section::Roles => has_roles,
                Section::Tags => has_tags,
            })
            .collect()
    }

    /// The Colors section's counts, or `None` when there is no profile or no deck has a card, where the client draws Home's empty line instead.
    #[must_use]
    pub fn colors(&self) -> Option<ColorCounts> {
        self.profile
            .as_ref()?
            .color_map
            .as_ref()
            .map(ColorCounts::from)
            .filter(ColorCounts::has_cards)
    }

    /// The Summary section's rows.
    #[must_use]
    pub fn summary(&self) -> Option<ProfileSummary> {
        self.profile.as_ref().map(playstyle::profile_summary)
    }

    /// The Card roles chart's bars, from [`playstyle::role_bars`].
    #[must_use]
    pub fn role_bars(&self) -> Vec<Bar> {
        self.profile
            .as_ref()
            .map(|p| playstyle::role_bars(&p.roles, &p.tags))
            .unwrap_or_default()
    }

    /// The oracle tags under `role` that its bar reveals, highest score first.
    #[must_use]
    pub fn role_tags(&self, role: &str) -> Vec<String> {
        self.profile
            .as_ref()
            .map(|p| playstyle::role_tags(&p.tags, role))
            .unwrap_or_default()
    }

    /// The Oracle tags table's rows, from [`playstyle::tag_rows`].
    #[must_use]
    pub fn tag_rows(&self) -> Vec<TagRow> {
        self.profile
            .as_ref()
            .map(|p| playstyle::tag_rows(&p.tags))
            .unwrap_or_default()
    }

    /// The held ranking, best fit first, unnarrowed.
    #[must_use]
    pub fn entries(&self) -> &[HttpCommanderSuggestion] {
        &self.entries
    }

    /// The Show chips after All: each identity among the held ranking plus the one picked, so a chip stays after its last commander goes.
    #[must_use]
    pub fn identities(&self) -> Vec<Vec<Color>> {
        colors::identities_present(
            self.entries
                .iter()
                .map(suggestion_colors)
                .chain(self.identity.as_deref()),
        )
    }

    /// The Show chip picked; `None` is All.
    #[must_use]
    pub fn identity(&self) -> Option<&[Color]> {
        self.identity.as_deref()
    }

    /// The held ranking narrowed to the Show identity, in held order: what Swipe deals.
    #[must_use]
    pub fn filtered(&self) -> Vec<HttpCommanderSuggestion> {
        playstyle::of_identity(&self.entries, suggestion_colors, self.identity.as_deref())
    }

    /// The rows the list shows: the first [`LIST_SIZE`] of [`Self::filtered`].
    #[must_use]
    pub fn rows(&self) -> Vec<HttpCommanderSuggestion> {
        playstyle::shown(&self.filtered())
    }

    /// The line in place of the list: [`NO_SUGGESTIONS`] when nothing is held, else the Show identity's empty line when no row passes it. `None` when rows show.
    #[must_use]
    pub fn list_message(&self) -> Option<String> {
        if self.entries.is_empty() {
            Some(NO_SUGGESTIONS.to_owned())
        } else if self.rows().is_empty() {
            Some(playstyle::empty_identity_line(
                self.identity.as_deref().unwrap_or_default(),
            ))
        } else {
            None
        }
    }

    /// Every staple the profile sent, in the most decks first.
    #[must_use]
    pub fn staples(&self) -> &[HttpStaple] {
        self.profile.as_ref().map_or(&[], |p| p.staples.as_slice())
    }

    /// Whether the Staples panel draws: the screen is loaded and there is a staple.
    #[must_use]
    pub fn shows_staples(&self) -> bool {
        self.status() == Status::Loaded && !self.staples().is_empty()
    }

    /// The Staples Type chips after All.
    #[must_use]
    pub fn staple_types(&self) -> Vec<CardType> {
        playstyle::staple_types_present(self.staples(), playstyle::staple_type_line)
    }

    /// The Staples Color chips after All.
    #[must_use]
    pub fn staple_colors(&self) -> Vec<StapleColor> {
        playstyle::staple_colors_present(self.staples(), playstyle::staple_identity)
    }

    /// The Staples Type chip picked; `None` is All.
    #[must_use]
    pub fn staple_type(&self) -> Option<CardType> {
        self.staple_type
    }

    /// The Staples Color chip picked; `None` is All.
    #[must_use]
    pub fn staple_color(&self) -> Option<StapleColor> {
        self.staple_color
    }

    /// The first [`LIST_SIZE`] staples the Type and Color chips let through.
    #[must_use]
    pub fn matching_staples(&self) -> Vec<HttpStaple> {
        playstyle::matching_staples(
            self.staples(),
            playstyle::staple_type_line,
            playstyle::staple_identity,
            self.staple_type,
            self.staple_color,
        )
    }

    /// [`NO_STAPLES`] when the Type and Color chips let no staple through, else `None`.
    #[must_use]
    pub fn staples_message(&self) -> Option<&'static str> {
        self.matching_staples().is_empty().then_some(NO_STAPLES)
    }
}

/// A suggestion's colors, the identity the Show chips read.
fn suggestion_colors(suggestion: &HttpCommanderSuggestion) -> &[Color] {
    &suggestion.card.scryfall_data.color_identity
}

/// A single-select chip row's pick after tapping `tapped`: All when it is already picked, else `tapped`.
fn toggled<T: PartialEq>(current: Option<&T>, tapped: T) -> Option<T> {
    if current == Some(&tapped) {
        None
    } else {
        Some(tapped)
    }
}
