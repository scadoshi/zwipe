//! Create deck and Edit deck: one form in two modes, with every field, the name check, the knock-on clears, the four command-zone searches, the format, deck tag and oracle tag pickers, and the save.
//!
//! The client keeps a [`State`] while the screen is up and sends [`Action::Load`] once with [`Open::Create`] (and the commander the maybeboard seeded, if any) or [`Open::Edit`] with the deck's id. The values, the request and the knock-on clears come from [`crate::deck::form`], the picker drafts, caps and toasts from [`crate::deck::pickers`]. Text inputs, the picker grids and search boxes, the swipe overlays and which one is up stay in the client.

#[cfg(test)]
mod tests;

use std::time::Duration;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::commander_eligibility::{self, PartnerKind};
use zwipe_core::domain::deck::requests::update_deck_profile::InvalidUpdateDeckProfile;
use zwipe_core::domain::deck::{
    self, DeckName, DeckOtherTag, DeckProfile, DeckTagView, Format, PowerLevel,
};
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::deck::{HttpCreateDeckProfile, HttpUpdateDeckProfile};
use zwipe_core::text::sentence_case;

use crate::deck::form::{self, Cascade, FormValues, Slot, Step};
use crate::deck::pickers::{self, Draft, PartnerAutofill, Picker, TagPicker, Tapped};
use crate::generation::{Generation, Ticket};
use crate::hint::HintTopic;
use crate::timing;
use crate::toast::{Toast, ToastKind};

/// Create's title.
pub const CREATE_TITLE: &str = "Create deck";

/// Edit's title.
pub const EDIT_TITLE: &str = "Edit deck";

/// The line in place of the form when Edit's deck failed to load.
pub const LOAD_FAILED: &str = "Could not load deck";

/// The Back button.
pub const BACK: &str = "Back";

/// The save button, Create's and Edit's; Edit shows it only while something changed.
pub const SAVE: &str = "Save";

/// The save button while the save is out.
pub const SAVING: &str = "Saving...";

/// The title of the dialog that asks before leaving the form with unsaved changes.
pub const LEAVE_TITLE: &str = "Leave without saving?";

/// The leave dialog's body.
pub const LEAVE_BODY: &str = "Your changes to this deck will be lost.";

/// The leave dialog's button that keeps the user on the form.
pub const STAY: &str = "Stay";

/// The leave dialog's button that goes on and drops the changes.
pub const LEAVE: &str = "Leave";

/// The first section's title: name, format, command zone and power level.
pub const PROFILE: &str = "Profile";

/// The second section's title: land and price targets.
pub const BUDGET: &str = "Budget";

/// The third section's title: deck, oracle and other tags.
pub const TAGS: &str = "Tags";

/// The name field's label.
pub const DECK_NAME: &str = "Deck name";

/// The format field's label.
pub const FORMAT: &str = "Format";

/// The power level field's label.
pub const POWER_LEVEL: &str = "Power level";

/// The land target field's label.
pub const LAND_TARGET: &str = "Land target";

/// The price target field's label.
pub const PRICE_TARGET: &str = "Price target";

/// The deck tag field's label.
pub const DECK_TAGS: &str = "Deck tags";

/// The oracle tag field's label.
pub const ORACLE_TAGS: &str = "Oracle tags";

/// The other tag field's label.
pub const OTHER_TAGS: &str = "Other tags";

/// What an empty field shows, and every input's placeholder.
pub const NOT_SET: &str = "Not set";

/// A command-zone field's chip that narrows its search to cards that fit the slot.
pub const FILTER: &str = "Filter";

/// A command-zone field's chip that opens the swipe picker.
pub const SWIPE: &str = "Swipe";

/// A command-zone search's one chip while it waits or runs.
pub const SEARCHING: &str = "Searching...";

/// A command-zone search's one chip when it found nothing.
pub const NO_RESULTS: &str = "No results";

/// The toast after closing the deck tag picker adds oracle tags the deck tags seed.
pub const ORACLE_TAGS_SEEDED: &str = "Oracle tags derived from your deck tags";

/// Cards the named partner search asks for.
const PARTNER_RESULT_LIMIT: u32 = 5;

/// How the screen opens, for [`Action::Load`].
#[derive(Debug, Clone, PartialEq)]
pub enum Open {
    /// A new deck, starting from the commander the commander maybeboard seeded, if any.
    Create {
        /// The seeded commander. It comes in with the format set to Commander and no knock-on clear or partner autofill.
        seed: Option<Box<Card>>,
    },
    /// The saved deck with this id.
    Edit(Uuid),
}

/// Which form this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// A new deck: nothing to load, Save always shows.
    Create,
    /// The saved deck with this id: Save shows only while something changed.
    Edit(Uuid),
}

impl Mode {
    /// The screen's title.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Create => CREATE_TITLE,
            Self::Edit(_) => EDIT_TITLE,
        }
    }

    /// The one-time hint the screen offers on arrival, which its `?` reopens.
    #[must_use]
    pub fn hint(self) -> OneTimeHint {
        match self {
            Self::Create => OneTimeHint::CreateDeck,
            Self::Edit(_) => OneTimeHint::EditDeck,
        }
    }
}

/// Where Edit's deck fetch stands, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The deck is on its way: the form ghosts.
    Loading,
    /// The deck failed to load: [`LOAD_FAILED`] in place of the form.
    Failed,
    /// The form draws. Create is always here.
    Ready,
}

/// What floats over a command-zone field while its search shows.
#[derive(Debug, Clone, PartialEq)]
pub enum Results {
    /// The search waits out its debounce or is out: one [`SEARCHING`] chip.
    Searching,
    /// The search found nothing: one [`NO_RESULTS`] chip.
    Empty,
    /// The cards found, in the server's order, each a chip.
    Found(Vec<Card>),
}

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Wherever the user came from, Create's Back.
    Previous,
    /// The deck view for the deck with this id: Edit's Back, and after either save.
    Deck(Uuid),
    /// The deck list, in place of a screen whose deck is gone.
    DeckList,
}

/// What a user does on the form, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Opens the form, once, as the screen mounts.
    Load(Open),
    /// Edit's deck arrived for the fetch holding `ticket`.
    Loaded {
        /// The ticket from the [`Effect::FetchDeck`] this answers.
        ticket: Ticket,
        /// The deck's profile.
        profile: Box<DeckProfile>,
    },
    /// One of Edit's command-zone cards arrived for the fetch holding `ticket`.
    CardLoaded {
        /// The ticket from the [`Effect::FetchCard`] this answers.
        ticket: Ticket,
        /// The card.
        card: Box<Card>,
    },
    /// The request holding `ticket` failed. The client has already reported it, and raised its error toast unless it was a search.
    Failed {
        /// The ticket from the effect this answers.
        ticket: Ticket,
    },
    /// The request holding `ticket` found no deck: the server no longer has it. The client has already reported it and raised its toast.
    Gone {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// The server's deck tag catalog, whenever the client's copy changes. Closing the deck tag picker seeds oracle tags from it; an empty catalog seeds nothing.
    Catalog(Vec<DeckTagView>),
    /// The name field reads this.
    SetName(String),
    /// Clears the format from the form's format field, with its knock-on clears.
    ClearFormat,
    /// A command-zone input reads `text`, from typing. The card stays until the input is tapped or cleared.
    Type {
        /// The slot.
        slot: Slot,
        /// The input's text.
        text: String,
    },
    /// Taps a command-zone input: its card, its text and its search go.
    TapInput(Slot),
    /// The × beside a command-zone input: its text and its search go, the card stays.
    ClearSearch(Slot),
    /// The × on a command-zone label row: the card, its text and its search go.
    ClearSlot(Slot),
    /// Turns a command-zone field's Filter on or off. It narrows the next search; a search already shown stays.
    ToggleFilter(Slot),
    /// Picks a card from a command-zone field's results.
    PickResult {
        /// The slot.
        slot: Slot,
        /// The card.
        card: Box<Card>,
    },
    /// Picks a card from a command-zone field's swipe picker. The results stay as they were.
    PickSwiped {
        /// The slot.
        slot: Slot,
        /// The card.
        card: Box<Card>,
    },
    /// The debounce for the search holding `ticket` is over.
    Debounced {
        /// The ticket from the [`Effect::Debounce`] this answers.
        ticket: Ticket,
    },
    /// The search holding `ticket` found these cards.
    Searched {
        /// The ticket from the [`Effect::SearchCards`] this answers.
        ticket: Ticket,
        /// The cards found.
        cards: Vec<Card>,
    },
    /// The named partner search started for the commander `picked` found these cards. A failed search sends nothing.
    PartnerSearched {
        /// The printing id of the commander the search started for.
        picked: Uuid,
        /// The partner the commander names.
        mate: String,
        /// The cards found.
        cards: Vec<Card>,
    },
    /// Picks or unpicks a power level.
    TogglePowerLevel(PowerLevel),
    /// The power level's ×.
    ClearPowerLevel,
    /// A land target stepper button.
    StepLand(Step),
    /// The land target's ×: back to Not set.
    ClearLand,
    /// The price box reads `text`; anything but digits and dots is dropped.
    TypePrice(String),
    /// The price target's ×.
    ClearPrice,
    /// Picks the price target's currency.
    PickCurrency(PriceCurrency),
    /// Picks or unpicks an other tag.
    ToggleOtherTag(DeckOtherTag),
    /// The other tags' ×.
    ClearOtherTags,
    /// Opens a picker on the form's current value.
    OpenPicker(Picker),
    /// Taps a format chip in the format picker: picks it, or clears it when it is the format picked. Either way the knock-on clears run.
    TapFormat(Format),
    /// Taps a tag chip in a tag picker: adds or removes it, or warns at the cap.
    ToggleTag {
        /// The picker.
        picker: TagPicker,
        /// The tag's slug.
        slug: String,
    },
    /// Removes one tag, from its chip's × on the form.
    RemoveTag {
        /// The field.
        picker: TagPicker,
        /// The tag's slug.
        slug: String,
    },
    /// Empties a tag list, from the × on its form field or its picker.
    ClearTags(TagPicker),
    /// The oracle tag dictionary's Use hands this slug back.
    AdoptOracleTag(String),
    /// The open picker's Save: keeps the pick. Closing the deck tag picker reseeds the oracle tags.
    SavePicker,
    /// The open picker's Back, or the OS back gesture over it: puts back what it opened on, saying so when that throws a change away.
    BackPicker,
    /// The Save button.
    Save,
    /// The save holding `ticket` made the deck with this id.
    Created {
        /// The ticket from the [`Effect::CreateDeck`] this answers.
        ticket: Ticket,
        /// The new deck's id.
        deck_id: Uuid,
    },
    /// The save holding `ticket` went through.
    Updated {
        /// The ticket from the [`Effect::UpdateDeck`] this answers.
        ticket: Ticket,
    },
    /// Back.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches Edit's deck; answers with [`Action::Loaded`] or [`Action::Failed`].
    FetchDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck's id.
        deck_id: Uuid,
    },
    /// Fetches one of Edit's saved command-zone cards; answers with [`Action::CardLoaded`] or [`Action::Failed`].
    FetchCard {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The slot the card fills, for the client's error report.
        slot: Slot,
        /// The card's printing id.
        card_id: Uuid,
    },
    /// Waits `wait`, then answers with [`Action::Debounced`] carrying `ticket`.
    Debounce {
        /// The ticket to hand back.
        ticket: Ticket,
        /// How long to wait: [`timing::COMMAND_ZONE_SEARCH_DEBOUNCE`].
        wait: Duration,
    },
    /// Searches cards, quietly; answers with [`Action::Searched`] or [`Action::Failed`].
    SearchCards {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The search, from [`form::search_query`].
        query: Box<CardQuery>,
    },
    /// Searches the partner a "Partner with" commander names, quietly; answers with [`Action::PartnerSearched`], or nothing on failure.
    FindPartner {
        /// The printing id of the commander picked.
        picked: Uuid,
        /// The partner's name.
        mate: String,
        /// The search.
        query: Box<CardQuery>,
    },
    /// Creates the deck; answers with [`Action::Created`] or [`Action::Failed`].
    CreateDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The request, from [`FormValues::create_request`].
        request: Box<HttpCreateDeckProfile>,
    },
    /// Saves Edit's changes; answers with [`Action::Updated`] or [`Action::Failed`].
    UpdateDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck's id.
        deck_id: Uuid,
        /// The request, from [`FormValues::update_request`].
        request: Box<HttpUpdateDeckProfile>,
    },
    /// Goes to this screen.
    Navigate(Destination),
    /// Goes to this screen in place of the current one, so Back skips a screen with nothing left to show.
    Replace(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// One command-zone field's input and search.
#[derive(Debug, Clone, PartialEq)]
struct Search {
    /// What the input shows: the card's name, or the text typed.
    display: String,
    /// The text typed, which the search runs on.
    query: String,
    filter_on: bool,
    /// Whether the results float under the input.
    open: bool,
    /// The search waits out its debounce or is out.
    searching: bool,
    results: Vec<Card>,
    /// The debounce or search that may still land.
    waiting: Option<Ticket>,
}

impl Default for Search {
    fn default() -> Self {
        Self {
            display: String::new(),
            query: String::new(),
            filter_on: true,
            open: false,
            searching: false,
            results: Vec::new(),
            waiting: None,
        }
    }
}

/// The four command-zone fields' searches.
#[derive(Debug, Default, Clone, PartialEq)]
struct Searches {
    commander: Search,
    partner: Search,
    background: Search,
    signature_spell: Search,
}

impl Searches {
    fn get(&self, slot: Slot) -> &Search {
        match slot {
            Slot::Commander => &self.commander,
            Slot::Partner => &self.partner,
            Slot::Background => &self.background,
            Slot::SignatureSpell => &self.signature_spell,
        }
    }

    fn get_mut(&mut self, slot: Slot) -> &mut Search {
        match slot {
            Slot::Commander => &mut self.commander,
            Slot::Partner => &mut self.partner,
            Slot::Background => &mut self.background,
            Slot::SignatureSpell => &mut self.signature_spell,
        }
    }

    /// The slot whose debounce or search holds `ticket`.
    fn holding(&self, ticket: Ticket) -> Option<Slot> {
        Slot::ALL
            .into_iter()
            .find(|&slot| self.get(slot).waiting == Some(ticket))
    }
}

/// The commander and signature spell as the format picker opened on them, which its Back puts back. The partner and background are not kept, so a format change that cleared them leaves them cleared.
#[derive(Debug, Clone, PartialEq)]
struct ZoneSnapshot {
    commander: Option<Card>,
    commander_display: String,
    signature_spell: Option<Card>,
    signature_spell_display: String,
}

/// The picker that is open and what it opened on.
#[derive(Debug, Clone, PartialEq)]
enum OpenPicker {
    Format {
        draft: Draft<Option<Format>>,
        zone: Box<ZoneSnapshot>,
    },
    Tags {
        picker: TagPicker,
        draft: Draft<Vec<String>>,
    },
}

/// A request that may still land, besides the searches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Request {
    Deck,
    Card(Slot),
    Save,
}

/// The deck form for one visit. Starts as an empty Create form; send [`Action::Load`] to open it.
///
/// Every request takes a fresh [`Ticket`] from one [`Generation`], so a ticket names exactly one request, and a reply lands only while its request is still waiting. A command-zone field's new text drops its older search.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    mode: Mode,
    status: Status,
    values: FormValues,
    /// Edit's values as saved, each command-zone card filling in as it arrives; Create's as the form opened. A save that went through moves it to the values saved.
    saved: FormValues,
    searches: Searches,
    name_touched: bool,
    name_error: Option<String>,
    picker: Option<OpenPicker>,
    /// The oracle tags the deck tags seeded last, which the next reseed swaps out.
    applied_seed: Vec<String>,
    catalog: Vec<DeckTagView>,
    saving: bool,
    waiting: Vec<(Ticket, Request)>,
    tickets: Generation,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    /// An empty Create form.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: Mode::Create,
            status: Status::Ready,
            values: FormValues::default(),
            saved: FormValues::default(),
            searches: Searches::default(),
            name_touched: false,
            name_error: None,
            picker: None,
            applied_seed: Vec::new(),
            catalog: Vec::new(),
            saving: false,
            waiting: Vec::new(),
            tickets: Generation::default(),
        }
    }

    /// Applies `action` and returns what the client must do, in order.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load(open) => self.load(open),
            Action::Loaded { ticket, profile } => self.loaded(ticket, &profile),
            Action::CardLoaded { ticket, card } => self.card_loaded(ticket, *card),
            Action::Failed { ticket } => {
                self.failed(ticket);
                Vec::new()
            }
            Action::Gone { ticket } => {
                self.failed(ticket);
                vec![Effect::Replace(Destination::DeckList)]
            }
            Action::Catalog(catalog) => {
                self.catalog = catalog;
                Vec::new()
            }
            Action::SetName(name) => {
                self.values.name = name;
                self.check_name();
                Vec::new()
            }
            Action::ClearFormat => {
                self.set_format(None);
                Vec::new()
            }
            Action::Debounced { ticket } => self.debounced(ticket),
            Action::Searched { ticket, cards } => {
                if let Some(slot) = self.searches.holding(ticket) {
                    let search = self.searches.get_mut(slot);
                    search.waiting = None;
                    search.searching = false;
                    search.open = true;
                    search.results = cards;
                }
                Vec::new()
            }
            Action::PartnerSearched {
                picked,
                mate,
                cards,
            } => self.partner_searched(picked, &mate, &cards),
            Action::OpenPicker(picker) => {
                self.open_picker(picker);
                Vec::new()
            }
            Action::TapFormat(format) => {
                let picked = (self.values.format != Some(format)).then_some(format);
                self.set_format(picked);
                Vec::new()
            }
            tag @ (Action::ToggleTag { .. }
            | Action::RemoveTag { .. }
            | Action::ClearTags(_)
            | Action::AdoptOracleTag(_)) => self.tag(tag),
            Action::SavePicker => self.save_picker(),
            Action::BackPicker => self.back_picker(),
            Action::Save => self.save(),
            Action::Created { ticket, deck_id } => {
                if self.take(ticket) != Some(Request::Save) {
                    return Vec::new();
                }
                self.saved = self.values.clone();
                vec![Effect::Navigate(Destination::Deck(deck_id))]
            }
            Action::Updated { ticket } => {
                if self.take(ticket) != Some(Request::Save) {
                    return Vec::new();
                }
                self.saving = false;
                self.saved = self.values.clone();
                match self.mode {
                    Mode::Edit(deck_id) => vec![Effect::Navigate(Destination::Deck(deck_id))],
                    Mode::Create => Vec::new(),
                }
            }
            Action::Back => {
                if self.saving {
                    return Vec::new();
                }
                vec![Effect::Navigate(match self.mode {
                    Mode::Create => Destination::Previous,
                    Mode::Edit(deck_id) => Destination::Deck(deck_id),
                })]
            }
            command_zone @ (Action::Type { .. }
            | Action::TapInput(_)
            | Action::ClearSearch(_)
            | Action::ClearSlot(_)
            | Action::ToggleFilter(_)
            | Action::PickResult { .. }
            | Action::PickSwiped { .. }) => self.command_zone(command_zone),
            field => {
                self.field(field);
                Vec::new()
            }
        }
    }

    /// Which form this is.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Where Edit's deck fetch stands. Create is always [`Status::Ready`].
    #[must_use]
    pub fn status(&self) -> Status {
        self.status
    }

    /// Every field's value as the form shows it.
    #[must_use]
    pub fn values(&self) -> &FormValues {
        &self.values
    }

    /// The line under the name field: the name check's error once the field has held text, or after a save it refused.
    #[must_use]
    pub fn name_error(&self) -> Option<&str> {
        self.name_error.as_deref()
    }

    /// Whether `slot`'s field shows, from the format and the commander.
    #[must_use]
    pub fn shows(&self, slot: Slot) -> bool {
        self.values.shows(slot)
    }

    /// `slot`'s label, which names the commander Oathbreaker where the format has a signature spell.
    #[must_use]
    pub fn label(&self, slot: Slot) -> &'static str {
        slot.label(self.values.format)
    }

    /// The `?` topic beside `slot`'s label.
    #[must_use]
    pub fn hint(&self, slot: Slot) -> HintTopic {
        slot.hint(self.values.format)
    }

    /// What `slot`'s input shows: the card's name, or the text typed.
    #[must_use]
    pub fn display(&self, slot: Slot) -> &str {
        &self.searches.get(slot).display
    }

    /// Whether `slot`'s input holds typed text, which shows the × beside it.
    #[must_use]
    pub fn has_query(&self, slot: Slot) -> bool {
        !self.searches.get(slot).query.is_empty()
    }

    /// Whether `slot`'s Filter is on.
    #[must_use]
    pub fn filter_on(&self, slot: Slot) -> bool {
        self.searches.get(slot).filter_on
    }

    /// What floats under `slot`'s input, if anything.
    #[must_use]
    pub fn results(&self, slot: Slot) -> Option<Results> {
        let search = self.searches.get(slot);
        if !search.open {
            None
        } else if search.searching {
            Some(Results::Searching)
        } else if search.results.is_empty() {
            Some(Results::Empty)
        } else {
            Some(Results::Found(search.results.clone()))
        }
    }

    /// The open picker, if any.
    #[must_use]
    pub fn picker(&self) -> Option<Picker> {
        match &self.picker {
            Some(OpenPicker::Format { .. }) => Some(Picker::Format),
            Some(OpenPicker::Tags { picker, .. }) => Some((*picker).into()),
            None => None,
        }
    }

    /// Whether the open picker still holds what it opened on, which keeps its Save off. True with no picker open.
    #[must_use]
    pub fn picker_unchanged(&self) -> bool {
        match &self.picker {
            Some(OpenPicker::Format { draft, .. }) => draft.is_unchanged(&self.values.format),
            Some(OpenPicker::Tags { picker, draft }) => draft.is_unchanged(self.tag_list(*picker)),
            None => true,
        }
    }

    /// The tags in `picker`'s field.
    #[must_use]
    pub fn tags(&self, picker: TagPicker) -> &[String] {
        self.tag_list(picker)
    }

    /// Whether a save is out, which turns Back and Save off.
    #[must_use]
    pub fn is_saving(&self) -> bool {
        self.saving
    }

    /// Whether Edit's form differs from the deck as saved. Create counts as changed.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        match self.mode {
            Mode::Create => true,
            Mode::Edit(_) => self.values.update_request(&self.saved).is_some(),
        }
    }

    /// Whether leaving now would lose something: the form differs from the deck as saved (Create: from the form as it opened), or the open picker holds a pick not yet kept. False while Edit's deck loads or failed to load, and once a save went through.
    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        if self.status != Status::Ready {
            return false;
        }
        let changed = match self.mode {
            Mode::Create => self.values != self.saved,
            Mode::Edit(_) => self.values.update_request(&self.saved).is_some(),
        };
        changed || !self.picker_unchanged()
    }

    /// The save button's text, or `None` when it hides: Edit with nothing changed.
    #[must_use]
    pub fn save_label(&self) -> Option<&'static str> {
        if !self.has_changes() {
            None
        } else if self.saving {
            Some(SAVING)
        } else {
            Some(SAVE)
        }
    }

    fn load(&mut self, open: Open) -> Vec<Effect> {
        match open {
            Open::Create { seed } => {
                self.mode = Mode::Create;
                self.status = Status::Ready;
                if let Some(card) = seed {
                    self.values.format = Some(Format::Commander);
                    self.searches
                        .commander
                        .display
                        .clone_from(&card.scryfall_data.name);
                    self.values.commander = Some(*card);
                }
                self.saved = self.values.clone();
                vec![Effect::OfferHint(self.mode.hint())]
            }
            Open::Edit(deck_id) => {
                self.mode = Mode::Edit(deck_id);
                self.status = Status::Loading;
                let ticket = self.request(Request::Deck);
                vec![
                    Effect::OfferHint(self.mode.hint()),
                    Effect::FetchDeck { ticket, deck_id },
                ]
            }
        }
    }

    fn loaded(&mut self, ticket: Ticket, profile: &DeckProfile) -> Vec<Effect> {
        if self.take(ticket) != Some(Request::Deck) {
            return Vec::new();
        }
        self.status = Status::Ready;
        let loaded = FormValues::saved(profile);
        self.values = FormValues {
            commander: self.values.commander.take(),
            partner: self.values.partner.take(),
            background: self.values.background.take(),
            signature_spell: self.values.signature_spell.take(),
            ..loaded.clone()
        };
        self.saved = FormValues {
            commander: self.saved.commander.take(),
            partner: self.saved.partner.take(),
            background: self.saved.background.take(),
            signature_spell: self.saved.signature_spell.take(),
            ..loaded
        };
        self.applied_seed = deck::seed_oracle_tags(&profile.tags);
        self.check_name();
        self.filters_on();
        let ids = [
            (Slot::Commander, profile.commander_id),
            (Slot::Partner, profile.partner_commander_id),
            (Slot::Background, profile.background_id),
            (Slot::SignatureSpell, profile.signature_spell_id),
        ];
        ids.into_iter()
            .filter_map(|(slot, id)| id.map(|card_id| (slot, card_id)))
            .map(|(slot, card_id)| Effect::FetchCard {
                ticket: self.request(Request::Card(slot)),
                slot,
                card_id,
            })
            .collect()
    }

    fn card_loaded(&mut self, ticket: Ticket, card: Card) -> Vec<Effect> {
        let Some(Request::Card(slot)) = self.take(ticket) else {
            return Vec::new();
        };
        *self.saved.slot_mut(slot) = Some(card.clone());
        self.searches
            .get_mut(slot)
            .display
            .clone_from(&card.scryfall_data.name);
        if slot == Slot::Commander {
            let cascade = self.values.set_commander(Some(card));
            self.apply(&cascade);
        } else {
            *self.values.slot_mut(slot) = Some(card);
        }
        Vec::new()
    }

    fn failed(&mut self, ticket: Ticket) {
        if let Some(slot) = self.searches.holding(ticket) {
            let search = self.searches.get_mut(slot);
            search.waiting = None;
            search.searching = false;
            search.open = false;
            return;
        }
        match self.take(ticket) {
            Some(Request::Deck) => self.status = Status::Failed,
            Some(Request::Save) => self.saving = false,
            Some(Request::Card(_)) | None => {}
        }
    }

    /// The form fields with no request or toast behind them.
    fn field(&mut self, action: Action) {
        match action {
            Action::TogglePowerLevel(level) => {
                self.values.power_level = (self.values.power_level != Some(level)).then_some(level);
            }
            Action::ClearPowerLevel => self.values.power_level = None,
            Action::StepLand(step) => {
                self.values.land_target = Some(form::step_land_target(
                    self.values.land_target,
                    self.values.format,
                    step,
                ));
            }
            Action::ClearLand => self.values.land_target = None,
            Action::TypePrice(text) => {
                self.values.price_text = text.chars().filter(|&c| form::price_char(c)).collect();
            }
            Action::ClearPrice => self.values.price_text.clear(),
            Action::PickCurrency(currency) => self.values.currency = currency,
            Action::ToggleOtherTag(tag) => {
                let tags = &mut self.values.other_tags;
                if let Some(at) = tags.iter().position(|t| *t == tag) {
                    tags.remove(at);
                } else {
                    tags.push(tag);
                }
            }
            Action::ClearOtherTags => self.values.other_tags.clear(),
            _ => {}
        }
    }

    /// The tag fields and the tag pickers' chips.
    fn tag(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::ToggleTag { picker, slug } => {
                match picker.toggle(self.tags_mut(picker), &slug) {
                    Tapped::AtCap => vec![Effect::Toast(picker.cap_toast())],
                    Tapped::Added | Tapped::Removed => Vec::new(),
                }
            }
            Action::RemoveTag { picker, slug } => {
                self.tags_mut(picker).retain(|s| *s != slug);
                Vec::new()
            }
            Action::ClearTags(picker) => {
                self.tags_mut(picker).clear();
                Vec::new()
            }
            Action::AdoptOracleTag(slug) => {
                let adopted = pickers::adopt_oracle_tag(&mut self.values.oracle_tags, slug);
                vec![Effect::Toast(adopted.toast())]
            }
            _ => Vec::new(),
        }
    }

    /// The command-zone inputs, chips and picks.
    fn command_zone(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Type { slot, text } => {
                let search = self.searches.get_mut(slot);
                search.display.clone_from(&text);
                search.query = text;
                self.query_changed(slot)
            }
            Action::TapInput(slot) => {
                let search = self.searches.get_mut(slot);
                search.query.clear();
                search.display.clear();
                self.empty_slot(slot);
                self.query_changed(slot)
            }
            Action::ClearSearch(slot) => {
                let search = self.searches.get_mut(slot);
                search.query.clear();
                search.display.clear();
                search.open = false;
                self.query_changed(slot)
            }
            Action::ClearSlot(slot) => {
                self.empty_slot(slot);
                let search = self.searches.get_mut(slot);
                search.display.clear();
                search.query.clear();
                search.open = false;
                self.query_changed(slot)
            }
            Action::ToggleFilter(slot) => {
                let search = self.searches.get_mut(slot);
                search.filter_on = !search.filter_on;
                Vec::new()
            }
            Action::PickResult { slot, card } => {
                self.searches.get_mut(slot).open = false;
                self.pick(slot, *card)
            }
            Action::PickSwiped { slot, card } => self.pick(slot, *card),
            _ => Vec::new(),
        }
    }

    /// Fills `slot` with `card`. A "Partner with" commander starts the search for its partner.
    fn pick(&mut self, slot: Slot, card: Card) -> Vec<Effect> {
        self.searches
            .get_mut(slot)
            .display
            .clone_from(&card.scryfall_data.name);
        if slot != Slot::Commander {
            *self.values.slot_mut(slot) = Some(card);
            return Vec::new();
        }
        let named = match commander_eligibility::partner_kind(&card) {
            Some(PartnerKind::Named(mate)) => Some((card.scryfall_data.id, mate)),
            _ => None,
        };
        let cascade = self.values.set_commander(Some(card));
        self.apply(&cascade);
        let Some((picked, mate)) = named else {
            return Vec::new();
        };
        let mut builder = CardQueryBuilder::with_name_contains(&mate);
        builder.set_limit(PARTNER_RESULT_LIMIT);
        let Ok(query) = builder.build() else {
            return Vec::new();
        };
        vec![Effect::FindPartner {
            picked,
            mate,
            query: Box::new(query),
        }]
    }

    fn partner_searched(&mut self, picked: Uuid, mate: &str, cards: &[Card]) -> Vec<Effect> {
        let found = match pickers::partner_autofill(
            cards,
            mate,
            picked,
            self.values.commander.as_ref(),
            self.values.partner.as_ref(),
        ) {
            PartnerAutofill::Fill(found) => found.clone(),
            PartnerAutofill::NoMatch | PartnerAutofill::Stale => return Vec::new(),
        };
        let name = found.scryfall_data.name.clone();
        self.searches.partner.display.clone_from(&name);
        self.values.partner = Some(found);
        vec![Effect::Toast(pickers::partner_found_toast(&name))]
    }

    /// Empties `slot`'s card; the commander goes with its knock-on clears.
    fn empty_slot(&mut self, slot: Slot) {
        if slot == Slot::Commander {
            let cascade = self.values.set_commander(None);
            self.apply(&cascade);
        } else {
            *self.values.slot_mut(slot) = None;
        }
    }

    /// Starts or drops `slot`'s search after its text changed. Text shorter than the slot's minimum closes the results; longer text shows Searching while the debounce runs.
    fn query_changed(&mut self, slot: Slot) -> Vec<Effect> {
        let ticket = self.tickets.next_ticket();
        let search = self.searches.get_mut(slot);
        if search.query.len() < slot.min_chars() {
            search.waiting = None;
            search.open = false;
            search.searching = false;
            return Vec::new();
        }
        search.waiting = Some(ticket);
        search.searching = true;
        search.open = true;
        vec![Effect::Debounce {
            ticket,
            wait: timing::COMMAND_ZONE_SEARCH_DEBOUNCE,
        }]
    }

    /// Sends the search for the debounce holding `ticket`, with the Filter, format and commander as they are now.
    fn debounced(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some(slot) = self.searches.holding(ticket) else {
            return Vec::new();
        };
        let search = self.searches.get(slot);
        let Ok(query) = form::search_query(
            slot,
            &search.query,
            search.filter_on,
            self.values.format,
            self.values.commander.as_ref(),
        ) else {
            return Vec::new();
        };
        vec![Effect::SearchCards {
            ticket,
            query: Box::new(query),
        }]
    }

    /// Picks or clears the format, emptying each slot the change clears.
    fn set_format(&mut self, format: Option<Format>) {
        let cascade = match format {
            Some(format) => self.values.pick_format(format),
            None => self.values.clear_format(),
        };
        self.apply(&cascade);
    }

    /// Resets what the fields show for the slots a knock-on clear emptied.
    fn apply(&mut self, cascade: &Cascade) {
        for &slot in &cascade.cleared {
            self.searches.get_mut(slot).display.clear();
        }
        for &slot in &cascade.reset {
            let search = self.searches.get_mut(slot);
            search.display.clear();
            search.query.clear();
            search.open = false;
            let _short_query_starts_nothing = self.query_changed(slot);
        }
        if cascade.filters_on {
            self.filters_on();
        }
    }

    /// Turns every Filter back on, as any format change does.
    fn filters_on(&mut self) {
        for slot in Slot::ALL {
            self.searches.get_mut(slot).filter_on = true;
        }
    }

    fn check_name(&mut self) {
        if !self.values.name.is_empty() {
            self.name_touched = true;
        }
        if self.name_touched {
            self.name_error = DeckName::new(self.values.name.as_str())
                .err()
                .map(|e| sentence_case(&e.to_string()));
        }
    }

    fn tag_list(&self, picker: TagPicker) -> &Vec<String> {
        match picker {
            TagPicker::DeckTags => &self.values.tags,
            TagPicker::OracleTags => &self.values.oracle_tags,
        }
    }

    fn tags_mut(&mut self, picker: TagPicker) -> &mut Vec<String> {
        match picker {
            TagPicker::DeckTags => &mut self.values.tags,
            TagPicker::OracleTags => &mut self.values.oracle_tags,
        }
    }

    fn open_picker(&mut self, picker: Picker) {
        self.picker = Some(match picker {
            Picker::Format => OpenPicker::Format {
                draft: Draft::format(self.values.format),
                zone: Box::new(ZoneSnapshot {
                    commander: self.values.commander.clone(),
                    commander_display: self.searches.commander.display.clone(),
                    signature_spell: self.values.signature_spell.clone(),
                    signature_spell_display: self.searches.signature_spell.display.clone(),
                }),
            },
            Picker::DeckTags => self.tag_draft(TagPicker::DeckTags),
            Picker::OracleTags => self.tag_draft(TagPicker::OracleTags),
        });
    }

    fn tag_draft(&self, picker: TagPicker) -> OpenPicker {
        OpenPicker::Tags {
            picker,
            draft: Draft::tags(picker, self.tags(picker).to_vec()),
        }
    }

    fn save_picker(&mut self) -> Vec<Effect> {
        match self.picker.take() {
            Some(OpenPicker::Tags {
                picker: TagPicker::DeckTags,
                ..
            }) => self.reseed(),
            Some(_) | None => Vec::new(),
        }
    }

    fn back_picker(&mut self) -> Vec<Effect> {
        let mut effects = Vec::new();
        match self.picker.take() {
            Some(OpenPicker::Format { draft, zone }) => {
                effects.extend(draft.back_toast(&self.values.format).map(Effect::Toast));
                let ZoneSnapshot {
                    commander,
                    commander_display,
                    signature_spell,
                    signature_spell_display,
                } = *zone;
                self.values.format = draft.into_snapshot();
                self.values.commander = commander;
                self.searches.commander.display = commander_display;
                self.values.signature_spell = signature_spell;
                self.searches.signature_spell.display = signature_spell_display;
                self.filters_on();
            }
            Some(OpenPicker::Tags { picker, draft }) => {
                effects.extend(draft.back_toast(self.tag_list(picker)).map(Effect::Toast));
                *self.tags_mut(picker) = draft.into_snapshot();
                if picker == TagPicker::DeckTags {
                    effects.extend(self.reseed());
                }
            }
            None => {}
        }
        effects
    }

    /// Swaps the oracle tags the deck tags seeded last for the ones they seed now, keeping manual picks, and says so when any were added.
    fn reseed(&mut self) -> Vec<Effect> {
        let seed = deck::seed_oracle_tags_from_catalog(&self.values.tags, &self.catalog);
        let (tags, added) =
            form::reseed_oracle_tags(&self.values.oracle_tags, &self.applied_seed, &seed);
        self.values.oracle_tags = tags;
        self.applied_seed = seed;
        if added {
            vec![Effect::Toast(Toast::normal(
                ToastKind::Info,
                ORACLE_TAGS_SEEDED,
            ))]
        } else {
            Vec::new()
        }
    }

    fn save(&mut self) -> Vec<Effect> {
        if self.saving || self.status != Status::Ready {
            return Vec::new();
        }
        if let Err(e) = DeckName::new(self.values.name.as_str()) {
            self.name_error = Some(sentence_case(&e.to_string()));
            return Vec::new();
        }
        self.saving = true;
        match self.mode {
            Mode::Create => {
                let request = Box::new(self.values.create_request());
                let ticket = self.request(Request::Save);
                vec![Effect::CreateDeck { ticket, request }]
            }
            Mode::Edit(deck_id) => {
                let Some(request) = self.values.update_request(&self.saved) else {
                    self.saving = false;
                    return vec![Effect::Toast(Toast::normal(
                        ToastKind::Error,
                        InvalidUpdateDeckProfile::NoUpdates.to_string(),
                    ))];
                };
                let ticket = self.request(Request::Save);
                vec![Effect::UpdateDeck {
                    ticket,
                    deck_id,
                    request: Box::new(request),
                }]
            }
        }
    }

    /// Takes a ticket for `request` and puts it on the waiting list.
    fn request(&mut self, request: Request) -> Ticket {
        let ticket = self.tickets.next_ticket();
        self.waiting.push((ticket, request));
        ticket
    }

    /// Takes the request holding `ticket` off the waiting list, if it is there.
    fn take(&mut self, ticket: Ticket) -> Option<Request> {
        let at = self.waiting.iter().position(|(t, _)| *t == ticket)?;
        Some(self.waiting.remove(at).1)
    }
}
