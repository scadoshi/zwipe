//! The deck list: the user's decks folded by Group by and narrowed by Show, with Create and the list's one-time hint.
//!
//! Group by, Show, the tag fold and the collapsed groups last one visit: the client keeps a [`State`] while the screen is up and drops it on leaving. The sort outlives the visit: the client starts the screen with [`State::with_sort`] and keeps what [`Effect::RememberSort`] hands it. Folding and narrowing come from [`crate::deck::list`] and the Create check from [`crate::deck::create_gate`].

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::deck::DeckProfile;
use zwipe_core::domain::user::hints::OneTimeHint;

use crate::deck;
use crate::deck::list::{self, DeckGroup, GroupBy, Show, Sort, SortBy};
use crate::generation::{Generation, Ticket};
use crate::toast::Toast;

/// The screen's title.
pub const TITLE: &str = "Decks";

/// The Group by row's label.
pub const GROUP_BY: &str = "Group by";

/// The Sort row's label.
pub const SORT: &str = "Sort";

/// The Show row's label.
pub const SHOW: &str = "Show";

/// The Show chip that unpicks everything.
pub const SHOW_ALL: &str = "All";

/// The Show chip that folds the tag row open and closed.
pub const TAGS: &str = "Tags";

/// The line in place of the list when the decks failed to load.
pub const LOAD_FAILED: &str = "Couldn't load decks";

/// The line in place of the list when the user has no deck.
pub const NO_DECKS: &str = "No decks";

/// The line above the groups when Show lets no deck through.
pub const NO_MATCH: &str = "No decks match";

/// The button that starts a deck.
pub const CREATE: &str = "Create";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Home, from Back.
    Home,
    /// The deck view for the deck with this id.
    Deck(Uuid),
    /// The Create deck form.
    CreateDeck,
}

/// Where the deck fetch stands, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The fetch is out: the chip rows and list ghost.
    Loading,
    /// The fetch failed: [`LOAD_FAILED`], no chip rows.
    Failed,
    /// The user has no deck: [`NO_DECKS`], no chip rows.
    Empty,
    /// At least one deck: the chip rows and the groups.
    Loaded,
}

/// A chip in the Show row, in the order [`State::show_chips`] lists them.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShowChip {
    /// [`SHOW_ALL`], picked while nothing else is.
    All,
    /// One of the five colors.
    Color(Color),
    /// [`TAGS`], the fold for the tag row.
    TagFold,
    /// A picked tag, kept in the Show row while the tag row is folded.
    Picked {
        /// The tag's key, as [`Action::ToggleTag`] takes it.
        key: String,
        /// The tag's label.
        label: String,
    },
}

impl ShowChip {
    /// The action pressing the chip takes.
    #[must_use]
    pub fn action(&self) -> Action {
        match self {
            Self::All => Action::ShowAll,
            Self::Color(color) => Action::ToggleColor(*color),
            Self::TagFold => Action::ToggleTagFold,
            Self::Picked { key, .. } => Action::ToggleTag(key.clone()),
        }
    }
}

/// What a user does on the deck list, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Fetches the decks and refreshes the user, as the screen does when it opens. Group by, Show and the folds stay as they are.
    Load,
    /// The decks arrived for the fetch holding `ticket`.
    Loaded {
        /// The ticket from the [`Effect::FetchDecks`] this answers.
        ticket: Ticket,
        /// The user's decks in any order.
        decks: Vec<DeckProfile>,
    },
    /// The fetch holding `ticket` failed. The client has already reported it and raised its error toast.
    Failed {
        /// The ticket from the [`Effect::FetchDecks`] this answers.
        ticket: Ticket,
    },
    /// Picks a Group by mode. Collapsed groups stay collapsed.
    PickGroupBy(GroupBy),
    /// Sorts on this key, keeping the direction.
    PickSort(SortBy),
    /// Turns the sort the other way.
    FlipSortDirection,
    /// Unpicks every color and tag.
    ShowAll,
    /// Picks or unpicks a color.
    ToggleColor(Color),
    /// Picks or unpicks a tag by its key.
    ToggleTag(String),
    /// Folds the tag row open or closed.
    ToggleTagFold,
    /// Collapses or expands the group under this key ([`DeckGroup::key`]).
    ToggleGroup(String),
    /// Opens the deck with this id.
    OpenDeck(Uuid),
    /// Starts a deck, through the Create check.
    Create {
        /// Whether the user's email is verified. A client with no session passes `true`, so only the server can refuse.
        verified: bool,
    },
    /// Goes back to Home.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Fetches the user and stores them in the session, so the Create check reads a current verification. A failure only logs.
    RefreshUser,
    /// Fetches the user's decks and answers with [`Action::Loaded`] or [`Action::Failed`] carrying this ticket.
    FetchDecks(Ticket),
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
    /// Keeps this sort for the next visit's [`State::with_sort`].
    RememberSort(Sort),
}

/// The decks as the last fetch left them.
#[derive(Debug, Default, Clone, PartialEq)]
enum Decks {
    #[default]
    Loading,
    Failed,
    Empty,
    /// Never empty: an empty reply is [`Decks::Empty`].
    Loaded(Vec<DeckProfile>),
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// The deck list for one visit. Starts loading with nothing picked; send [`Action::Load`] to start the fetch.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    decks: Decks,
    group_by: GroupBy,
    sort: Sort,
    show: Show,
    tags_open: bool,
    collapsed: BTreeSet<String>,
    hint: HintOffer,
    generation: Generation,
}

impl State {
    /// The screen as it first draws: loading, nothing picked, nothing collapsed.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The screen as it first draws, sorted by `sort`: the one a past visit left.
    #[must_use]
    pub fn with_sort(sort: Sort) -> Self {
        Self {
            sort,
            ..Self::default()
        }
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// A reply whose ticket a newer [`Action::Load`] has replaced changes nothing. The hint is offered once per visit, on the first reply with a deck: it teaches the chip rows, which an empty list hides.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load => {
                self.decks = Decks::Loading;
                let ticket = self.generation.next_ticket();
                return vec![Effect::RefreshUser, Effect::FetchDecks(ticket)];
            }
            Action::Loaded { ticket, decks } => {
                if !self.generation.is_current(ticket) {
                    return Vec::new();
                }
                if decks.is_empty() {
                    self.decks = Decks::Empty;
                    return Vec::new();
                }
                self.decks = Decks::Loaded(decks);
                if self.hint == HintOffer::Waiting {
                    self.hint = HintOffer::Offered;
                    return vec![Effect::OfferHint(OneTimeHint::DeckList)];
                }
            }
            Action::Failed { ticket } => {
                if self.generation.is_current(ticket) {
                    self.decks = Decks::Failed;
                }
            }
            Action::PickGroupBy(by) => self.group_by = by,
            Action::PickSort(by) => {
                self.sort.by = by;
                return vec![Effect::RememberSort(self.sort)];
            }
            Action::FlipSortDirection => {
                self.sort.direction = self.sort.direction.flipped();
                return vec![Effect::RememberSort(self.sort)];
            }
            Action::ShowAll => self.show.clear(),
            Action::ToggleColor(color) => self.show.toggle_color(color),
            Action::ToggleTag(key) => self.show.toggle_tag(&key),
            Action::ToggleTagFold => self.tags_open = !self.tags_open,
            Action::ToggleGroup(key) => {
                if !self.collapsed.remove(&key) {
                    self.collapsed.insert(key);
                }
            }
            Action::OpenDeck(id) => return vec![Effect::Navigate(Destination::Deck(id))],
            Action::Create { verified } => {
                return vec![match deck::create_gate(verified, self.decks().len()) {
                    Ok(()) => Effect::Navigate(Destination::CreateDeck),
                    Err(warning) => Effect::Toast(warning),
                }];
            }
            Action::Back => return vec![Effect::Navigate(Destination::Home)],
        }
        Vec::new()
    }

    /// Where the fetch stands.
    #[must_use]
    pub fn status(&self) -> Status {
        match self.decks {
            Decks::Loading => Status::Loading,
            Decks::Failed => Status::Failed,
            Decks::Empty => Status::Empty,
            Decks::Loaded(_) => Status::Loaded,
        }
    }

    /// The decks as the client handed them in, empty until they load.
    #[must_use]
    pub fn decks(&self) -> &[DeckProfile] {
        match &self.decks {
            Decks::Loaded(decks) => decks,
            Decks::Loading | Decks::Failed | Decks::Empty => &[],
        }
    }

    /// The picked Group by mode.
    #[must_use]
    pub fn group_by(&self) -> GroupBy {
        self.group_by
    }

    /// The list's order: the picked sort, or Name in its direction while the decks can't sort on the picked key (see [`Self::sort_keys`]).
    #[must_use]
    pub fn sort(&self) -> Sort {
        self.sort.within(&self.sort_keys())
    }

    /// The Sort row's key chips: Name, Size and Format, then Last changed and Price when the decks carry them.
    #[must_use]
    pub fn sort_keys(&self) -> Vec<SortBy> {
        SortBy::offered(self.decks())
    }

    /// The picked colors and tags.
    #[must_use]
    pub fn show(&self) -> &Show {
        &self.show
    }

    /// Whether the tag row is folded open.
    #[must_use]
    pub fn tags_open(&self) -> bool {
        self.tags_open
    }

    /// Whether the group under `key` is collapsed.
    #[must_use]
    pub fn is_collapsed(&self, key: &str) -> bool {
        self.collapsed.contains(key)
    }

    /// Whether the Group by, Sort and Show rows show: only once a list with a deck has loaded.
    #[must_use]
    pub fn chips_shown(&self) -> bool {
        self.status() == Status::Loaded
    }

    /// Every distinct tag across the decks as a key and a label, sorted by label: the tag row.
    #[must_use]
    pub fn tags(&self) -> Vec<(String, String)> {
        list::all_tags(self.decks())
    }

    /// The picked tags the Show row keeps in view while the tag row is folded, in the tag row's order. Empty while it is open.
    #[must_use]
    pub fn folded_picks(&self) -> Vec<(String, String)> {
        if self.tags_open {
            return Vec::new();
        }
        self.tags()
            .into_iter()
            .filter(|(key, _)| self.show.tags.contains(key))
            .collect()
    }

    /// The Show row: All, the five colors, then the tag fold and [`Self::folded_picks`] when any deck has a tag.
    #[must_use]
    pub fn show_chips(&self) -> Vec<ShowChip> {
        let mut chips = vec![ShowChip::All];
        chips.extend(Color::all().map(ShowChip::Color));
        if !self.tags().is_empty() {
            chips.push(ShowChip::TagFold);
            chips.extend(
                self.folded_picks()
                    .into_iter()
                    .map(|(key, label)| ShowChip::Picked { key, label }),
            );
        }
        chips
    }

    /// Whether a Show chip is picked: All while nothing is, a color or tag while it is, the fold while the tag row is open.
    #[must_use]
    pub fn is_picked(&self, chip: &ShowChip) -> bool {
        match chip {
            ShowChip::All => self.show.is_all(),
            ShowChip::Color(color) => self.show.colors.contains(color),
            ShowChip::TagFold => self.tags_open,
            ShowChip::Picked { key, .. } => self.show.tags.contains(key),
        }
    }

    /// The line above or in place of the list: [`LOAD_FAILED`], [`NO_DECKS`], or [`NO_MATCH`] when Show lets no deck through. `None` while loading or when decks show.
    #[must_use]
    pub fn message(&self) -> Option<&'static str> {
        match &self.decks {
            Decks::Loading => None,
            Decks::Failed => Some(LOAD_FAILED),
            Decks::Empty => Some(NO_DECKS),
            Decks::Loaded(decks) => {
                (!decks.iter().any(|d| self.show.matches(d))).then_some(NO_MATCH)
            }
        }
    }

    /// The decks Show lets through, in [`Self::sort`]'s order, in their groups. Empty until decks load.
    ///
    /// With nothing let through, [`GroupBy::None`] still gives its one "All" group with no decks under [`NO_MATCH`], as the app draws it; the other modes give no group.
    #[must_use]
    pub fn groups(&self) -> Vec<DeckGroup> {
        match &self.decks {
            Decks::Loaded(decks) => {
                list::group(&list::shown(decks, &self.show, self.sort()), self.group_by)
            }
            Decks::Loading | Decks::Failed | Decks::Empty => Vec::new(),
        }
    }
}
