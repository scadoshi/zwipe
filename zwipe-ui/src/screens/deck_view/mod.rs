//! The deck view: one deck's profile, its sections and stats, the warnings with their fixes, and the deck's action buttons with their dialogs and requests.
//!
//! The client keeps a [`State`] while the screen is up and sends [`Action::Load`] with the deck's id when it opens or the id changes. Which sections show, the Budget figures, the share link and the clone prefill come from [`crate::deck::view`], the fixes from [`crate::deck::warning`] and the buy links from [`crate::buy`]. Which sections are open, the dialogs' visibility and the clone name being typed stay in the client; the Budget section's Buy row opens a retailer through [`Effect::OpenUrl`].

#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{Deck, DeckEntry, DeckProfile, DeckWarning};
use zwipe_core::domain::user::hints::OneTimeHint;

use crate::buy;
use crate::deck::view::{self, Done};
use crate::deck::warning::{self, Fix};
use crate::generation::{Generation, Ticket};
use crate::screens::deck_form;
use crate::toast::{Line, Toast, keep_one};

/// The screen's title.
pub const TITLE: &str = "Deck";

/// The line in place of the deck when its profile failed to load.
pub const LOAD_FAILED: &str = "Couldn't load deck";

/// The button that opens Edit deck.
pub const EDIT: &str = "Edit";

/// The button that opens the deck's cards.
pub const CARDS: &str = "Cards";

/// The title of the panel near the foot of the page that holds Import and Export.
pub const DATA: &str = "Data";

/// The title of the last panel, which holds Clear skips and Delete.
pub const DANGER_ZONE: &str = "Danger zone";

/// The Budget section's running total row.
pub const TOTAL_PRICE: &str = "Total price";

/// The Budget section's average price row.
pub const AVERAGE_CARD_PRICE: &str = "Average card price";

/// The average price row's value when no card has a price in the picked currency.
pub const NO_PRICE: &str = "N/A";

/// The Distributions section's average power and toughness row.
pub const AVERAGE_PT: &str = "Average P/T";

/// The Mana section's land count row.
pub const LANDS: &str = "Lands";

/// The Mana section's average mana value row.
pub const AVERAGE_MANA_VALUE: &str = "Average mana value";

/// The button that closes the Share dialog.
pub const CLOSE: &str = "Close";

/// The button that closes the Clone, Clear skips and Delete dialogs without acting.
pub const CANCEL: &str = "Cancel";

/// The Budget section's Buy row label, before the retailer buttons.
pub const BUY: &str = "Buy";

/// The Clone dialog's line above the name field.
pub const CLONE_BODY: &str = "Make an exact copy of your deck under a new name.";

/// The Clone dialog's name field placeholder.
pub const CLONE_PLACEHOLDER: &str = "New deck name";

/// The Clone dialog's button that clones.
pub const CLONE_SAVE: &str = "Save";

/// The Clone dialog's button while the clone is on its way.
pub const CLONE_BUSY: &str = "Cloning…";

/// The Share dialog's line while the deck has a link.
pub const SHARE_SHARED: &str = "Anyone with the link can view this deck. Stop sharing disables the link; sharing again creates a new one.";

/// The Share dialog's line while the deck has no link.
pub const SHARE_UNSHARED: &str =
    "Anyone with the link can view this deck. You can stop sharing at any time.";

/// The Share dialog's button that copies the live link.
pub const COPY_LINK: &str = "Copy link";

/// The Share dialog's button that disables the link.
pub const STOP_SHARE: &str = "Stop sharing";

/// The Share dialog's button that creates a link.
pub const CREATE_LINK: &str = "Create link";

/// The Clear skips dialog's line.
pub const CLEAR_SKIPS_BODY: &str = "Skipped cards come back on their own after a week, and removed cards after a month. Clearing brings them all back now. This can't be undone.";

/// The Clear skips dialog's button that clears.
pub const CLEAR: &str = "Clear";

/// The Delete dialog's line.
pub const DELETE_BODY: &str = "Are you sure you want to delete this deck?";

/// The Delete dialog's button that deletes.
pub const DELETE: &str = "Delete";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The screen that opened the deck, from Back. The client knows it; the
    /// deck list when it doesn't.
    Back,
    /// The deck list, in place of a deleted or gone deck.
    DeckList,
    /// Edit deck for this deck.
    EditDeck(Uuid),
    /// The cards of this deck.
    DeckCards(Uuid),
    /// Add cards for this deck, from the below-minimum warning.
    AddCards(Uuid),
    /// Import for this deck.
    ImportDeck(Uuid),
    /// Export for this deck.
    ExportDeck(Uuid),
    /// The deck view for this deck, after a clone.
    Deck(Uuid),
}

/// One section under the profile, in the order [`State::sections`] lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    /// The price target and running prices, with the currency chips.
    Budget,
    /// Deck, oracle and other tags.
    Tags,
    /// Average power and toughness, the type and role charts.
    Distributions,
    /// Lands, average mana value, curve, colors and mana fulfillment.
    Mana,
    /// The draw odds table.
    DrawOdds,
    /// The deck's warnings with their fixes.
    Warnings,
}

impl Section {
    /// Every section, top to bottom.
    pub const ALL: [Self; 6] = [
        Self::Budget,
        Self::Tags,
        Self::Distributions,
        Self::Mana,
        Self::DrawOdds,
        Self::Warnings,
    ];

    /// The section's title.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Budget => "Budget",
            Self::Tags => "Tags",
            Self::Distributions => "Distributions",
            Self::Mana => "Mana",
            Self::DrawOdds => "Draw odds",
            Self::Warnings => "Warnings",
        }
    }
}

/// One of the deck's action buttons, on the profile card and in the panels under the sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoreItem {
    /// Opens Import.
    ImportCards,
    /// Opens Export, once the deck has a card.
    ExportCards,
    /// Opens the Clone dialog.
    CloneDeck,
    /// Opens the Share dialog.
    ShareDeck,
    /// Opens the Clear skips dialog.
    ClearSkips,
    /// Opens the Delete dialog.
    DeleteDeck,
}

impl MoreItem {
    /// Every item, top to bottom.
    pub const ALL: [Self; 6] = [
        Self::ImportCards,
        Self::ExportCards,
        Self::CloneDeck,
        Self::ShareDeck,
        Self::ClearSkips,
        Self::DeleteDeck,
    ];

    /// The item's button text.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::ImportCards => "Import cards",
            Self::ExportCards => "Export cards",
            Self::CloneDeck => "Clone deck",
            Self::ShareDeck => "Share deck",
            Self::ClearSkips => "Clear skips",
            Self::DeleteDeck => "Delete deck",
        }
    }

    /// The item's label as a button on the deck view itself, where the deck is
    /// already the subject: `Clone`, not `Clone deck`.
    #[must_use]
    pub fn short_label(self) -> &'static str {
        match self {
            Self::ImportCards => "Import",
            Self::ExportCards => "Export",
            Self::CloneDeck => "Clone",
            Self::ShareDeck => "Share",
            Self::ClearSkips => "Clear",
            Self::DeleteDeck => "Delete",
        }
    }

    /// Whether the item is drawn in the danger color: Clear skips and Delete deck.
    #[must_use]
    pub fn is_danger(self) -> bool {
        matches!(self, Self::ClearSkips | Self::DeleteDeck)
    }
}

/// A retailer the Budget section's Buy row opens, with the deck's cards filled in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Retailer {
    /// `TCGplayer`'s mass entry.
    TcgPlayer,
    /// `Card Kingdom`'s deck builder.
    CardKingdom,
}

impl Retailer {
    /// Every retailer, in the order the Buy row shows them.
    pub const ALL: [Self; 2] = [Self::TcgPlayer, Self::CardKingdom];

    /// The retailer's button text.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::TcgPlayer => "TCGplayer \u{2197}",
            Self::CardKingdom => "Card Kingdom \u{2197}",
        }
    }
}

/// A dialog over the deck view. The client keeps whether each is up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dialog {
    /// A name for the copy.
    Clone,
    /// Create, copy or stop the share link.
    Share,
    /// Confirms clearing the deck's skips.
    ClearSkips,
    /// Confirms deleting the deck.
    Delete,
}

impl Dialog {
    /// The dialog's title.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Clone => "Clone deck",
            Self::Share => "Share deck",
            Self::ClearSkips => "Clear skips",
            Self::Delete => "Delete deck",
        }
    }
}

/// Where the profile fetch stands, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The profile hasn't arrived: the profile and stats skeletons.
    Loading,
    /// The profile failed: [`LOAD_FAILED`].
    Failed,
    /// The profile is in: the profile card and the sections.
    Loaded,
}

/// What a user does on the deck view, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Fetches this deck's profile and cards, as the screen does when it opens or its deck id changes. What shows stays until the replies replace it, and replies to requests for the deck shown before are dropped.
    Load(Uuid),
    /// Tries a failed load again, from Retry or the connection coming back: the skeletons show while the fetches are out. Does nothing unless the profile or the deck failed.
    Retry,
    /// The profile arrived for the fetch holding `ticket`.
    ProfileLoaded {
        /// The ticket from the [`Effect::FetchProfile`] this answers.
        ticket: Ticket,
        /// The deck's profile.
        profile: Box<DeckProfile>,
    },
    /// The full deck arrived for the fetch holding `ticket`.
    DeckLoaded {
        /// The ticket from the [`Effect::FetchDeck`] this answers.
        ticket: Ticket,
        /// The deck with its entries, warnings and command-zone cards.
        deck: Box<Deck>,
    },
    /// The commander's card arrived for the fetch holding `ticket`.
    CommanderLoaded {
        /// The ticket from the [`Effect::FetchCommander`] this answers.
        ticket: Ticket,
        /// The commander's card.
        card: Box<Card>,
    },
    /// The request holding `ticket` failed. The client has already reported it and raised its error toast.
    Failed {
        /// The ticket from the effect this answers.
        ticket: Ticket,
    },
    /// The request holding `ticket` found no deck: the server no longer has it. The client has already reported it and raised its toast.
    Gone {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// Picks the currency the Budget section's running prices read in.
    PickCurrency(PriceCurrency),
    /// Goes back to the deck list.
    Back,
    /// Opens Edit deck.
    Edit,
    /// Opens the deck's cards.
    Cards,
    /// Presses one of the deck's action buttons.
    More(MoreItem),
    /// Presses a retailer on the Budget section's Buy row. Nothing until the deck loads with a card.
    Buy(Retailer),
    /// Opens Add cards, the below-minimum warning's first fix.
    AddCards,
    /// Sets a card's quantity, a copy-limit warning's fix.
    SetQuantity {
        /// The card's printing id.
        card: Uuid,
        /// The quantity to set.
        quantity: i32,
    },
    /// Clears the commander, an invalid-commander warning's fix.
    ClearCommander,
    /// Takes a card out of the deck, a warning's fix.
    RemoveCard {
        /// The card's printing id.
        card: Uuid,
    },
    /// The quantity set by the request holding `ticket` went through.
    QuantitySet {
        /// The ticket from the [`Effect::SetQuantity`] this answers.
        ticket: Ticket,
    },
    /// The commander cleared by the request holding `ticket` went through.
    CommanderCleared {
        /// The ticket from the [`Effect::ClearCommander`] this answers.
        ticket: Ticket,
    },
    /// The card removal holding `ticket` went through.
    CardRemoved {
        /// The ticket from the [`Effect::RemoveCard`] this answers.
        ticket: Ticket,
    },
    /// Clones the deck under `name`, from the Clone dialog's Save. A name that trims to nothing does nothing.
    Clone {
        /// The name as typed.
        name: String,
    },
    /// The clone holding `ticket` went through as the deck with this id.
    Cloned {
        /// The ticket from the [`Effect::CloneDeck`] this answers.
        ticket: Ticket,
        /// The new deck's id.
        deck_id: Uuid,
    },
    /// Creates a share link, from the Share dialog.
    Share,
    /// The share holding `ticket` went through with this link token.
    Shared {
        /// The ticket from the [`Effect::ShareDeck`] this answers.
        ticket: Ticket,
        /// The new link's token.
        token: Uuid,
    },
    /// Copies the live share link, from the Share dialog.
    CopyLink,
    /// Disables the share link, from the Share dialog.
    StopShare,
    /// The unshare holding `ticket` went through.
    Unshared {
        /// The ticket from the [`Effect::UnshareDeck`] this answers.
        ticket: Ticket,
    },
    /// Clears the deck's skips, from the Clear skips dialog.
    ClearSkips,
    /// The clear holding `ticket` went through.
    SkipsCleared {
        /// The ticket from the [`Effect::ClearSkips`] this answers.
        ticket: Ticket,
    },
    /// Deletes the deck, from the Delete dialog.
    Delete,
    /// The delete holding `ticket` went through.
    Deleted {
        /// The ticket from the [`Effect::DeleteDeck`] this answers.
        ticket: Ticket,
    },
}

/// What an action asks the client to do, in the order it should do it.
///
/// Every request carries its own [`Ticket`] and answers with its success action or [`Action::Failed`] carrying it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Fetches the deck's profile; answers with [`Action::ProfileLoaded`].
    FetchProfile {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck to fetch.
        deck_id: Uuid,
    },
    /// Fetches the full deck; answers with [`Action::DeckLoaded`].
    FetchDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck to fetch.
        deck_id: Uuid,
    },
    /// Fetches the commander's card; answers with [`Action::CommanderLoaded`].
    FetchCommander {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The commander's printing id.
        card: Uuid,
    },
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Sets a card's quantity; answers with [`Action::QuantitySet`].
    SetQuantity {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck holding the card.
        deck_id: Uuid,
        /// The card's printing id.
        card: Uuid,
        /// The quantity to set.
        quantity: i32,
    },
    /// Clears the deck's commander with [`Slot::Commander`](crate::deck::command_zone::Slot::Commander)'s update request; answers with [`Action::CommanderCleared`].
    ClearCommander {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck to update.
        deck_id: Uuid,
    },
    /// Takes a card out of the deck; answers with [`Action::CardRemoved`].
    RemoveCard {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck holding the card.
        deck_id: Uuid,
        /// The card's printing id.
        card: Uuid,
    },
    /// Clones the deck; answers with [`Action::Cloned`].
    CloneDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck to clone.
        deck_id: Uuid,
        /// The copy's name, trimmed.
        name: String,
    },
    /// Creates a share link; answers with [`Action::Shared`].
    ShareDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck to share.
        deck_id: Uuid,
    },
    /// Disables the share link; answers with [`Action::Unshared`].
    UnshareDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck to stop sharing.
        deck_id: Uuid,
    },
    /// Clears the deck's skips; answers with [`Action::SkipsCleared`]. A client that buffers skips sends them first, so they clear too.
    ClearSkips {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck to clear.
        deck_id: Uuid,
    },
    /// Deletes the deck; answers with [`Action::Deleted`].
    DeleteDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck to delete.
        deck_id: Uuid,
    },
    /// Copies `text` to the clipboard, then raises `done`. A client whose copy fails raises its own toast instead.
    Copy {
        /// The text to copy.
        text: String,
        /// The toast once it is copied.
        done: Toast,
    },
    /// Opens this dialog.
    OpenDialog(Dialog),
    /// Opens this address outside the client.
    OpenUrl(String),
    /// Goes to this screen.
    Navigate(Destination),
    /// Goes to this screen in place of the current one, so Back skips a screen with nothing left to show.
    Replace(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// Which of an action's effects raise a toast, for [`keep_one`].
fn toast_line(effect: &Effect) -> Option<Line<'_>> {
    match effect {
        Effect::Toast(toast) => Some(Line::Toast(toast)),
        _ => None,
    }
}

/// The labels and actions of a warning's fix buttons, in order. Empty for a warning with nothing to tap.
#[must_use]
pub fn fix_buttons(warning: &DeckWarning) -> Vec<(String, Action)> {
    let Some(fix) = warning::fix(warning) else {
        return Vec::new();
    };
    let actions = match fix {
        Fix::BelowMinimum => vec![Action::AddCards, Action::More(MoreItem::ImportCards)],
        Fix::Quantity { card, quantity } => vec![Action::SetQuantity { card, quantity }],
        Fix::ClearCommander => vec![Action::ClearCommander],
        Fix::Remove { card } => vec![Action::RemoveCard { card }],
    };
    warning::labels(fix).into_iter().zip(actions).collect()
}

/// The line under the Clone dialog's name, from the same check as Create and Edit ([`deck_form::name_error`]), or `None` when the name is a deck name.
#[must_use]
pub fn clone_name_error(name: &str) -> Option<String> {
    deck_form::name_error(name)
}

/// Whether the Clone dialog's Save is on: the name passes [`clone_name_error`].
#[must_use]
pub fn can_clone(name: &str) -> bool {
    clone_name_error(name).is_none()
}

/// One fetch's last result, and the ticket of the request still out for it.
#[derive(Debug, Clone, PartialEq)]
struct Fetch<T> {
    value: Fetched<T>,
    waiting: Option<Ticket>,
}

impl<T> Default for Fetch<T> {
    fn default() -> Self {
        Self {
            value: Fetched::Loading,
            waiting: None,
        }
    }
}

impl<T> Fetch<T> {
    /// Sends a request holding `ticket`. The last result stays until its reply lands.
    fn start(&mut self, ticket: Ticket) {
        self.waiting = Some(ticket);
    }

    /// Lands `value` when `ticket` is the request still out. Returns whether it landed.
    fn settle(&mut self, ticket: Ticket, value: Fetched<T>) -> bool {
        if self.waiting != Some(ticket) {
            return false;
        }
        self.waiting = None;
        self.value = value;
        true
    }

    fn loaded(&self) -> Option<&T> {
        match &self.value {
            Fetched::Loaded(value) => Some(value),
            Fetched::Loading | Fetched::Failed => None,
        }
    }
}

/// A fetch's result. Loading only until the first reply.
#[derive(Debug, Clone, PartialEq)]
enum Fetched<T> {
    Loading,
    Failed,
    Loaded(T),
}

/// The full deck with the metrics worked out from it once.
#[derive(Debug, Clone, PartialEq)]
struct Loaded {
    deck: Deck,
    /// Present when the deck has a mainboard or command-zone card.
    metrics: Option<DeckMetrics>,
}

/// A request that changes the deck, waiting for its reply.
#[derive(Debug, Clone, PartialEq)]
enum Mutation {
    SetQuantity { card: Uuid, quantity: i32 },
    ClearCommander,
    RemoveCard { card: Uuid },
    Clone(String),
    Share,
    Unshare,
    ClearSkips,
    Delete,
}

impl Mutation {
    /// The request to send for this change to `deck_id`, holding `ticket`.
    fn effect(&self, ticket: Ticket, deck_id: Uuid) -> Effect {
        match self {
            Self::SetQuantity { card, quantity } => Effect::SetQuantity {
                ticket,
                deck_id,
                card: *card,
                quantity: *quantity,
            },
            Self::ClearCommander => Effect::ClearCommander { ticket, deck_id },
            Self::RemoveCard { card } => Effect::RemoveCard {
                ticket,
                deck_id,
                card: *card,
            },
            Self::Clone(name) => Effect::CloneDeck {
                ticket,
                deck_id,
                name: name.clone(),
            },
            Self::Share => Effect::ShareDeck { ticket, deck_id },
            Self::Unshare => Effect::UnshareDeck { ticket, deck_id },
            Self::ClearSkips => Effect::ClearSkips { ticket, deck_id },
            Self::Delete => Effect::DeleteDeck { ticket, deck_id },
        }
    }
}

/// A reply that a [`Mutation`] went through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Landed {
    QuantitySet,
    CommanderCleared,
    CardRemoved,
    Cloned(Uuid),
    Shared(Uuid),
    Unshared,
    SkipsCleared,
    Deleted,
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// The deck view for one visit. Starts loading with no deck; send [`Action::Load`] to start the fetches.
///
/// Every request takes a fresh [`Ticket`] from one [`Generation`], so a ticket names exactly one request. A fetch's reply lands only while its ticket is the one still out for that fetch; a request that changes the deck lands only while no [`Action::Load`] has come since.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    deck_id: Option<Uuid>,
    profile: Fetch<DeckProfile>,
    deck: Fetch<Loaded>,
    commander: Option<Card>,
    commander_waiting: Option<Ticket>,
    currency: PriceCurrency,
    currency_seeded: bool,
    share_token: Option<Uuid>,
    mutations: Vec<(Ticket, Mutation)>,
    hint: HintOffer,
    tickets: Generation,
}

impl State {
    /// The screen as it first draws: loading, no deck, dollars.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// The currency follows the first profile's price target currency, once per visit, and the user's pick after that. The share link follows every profile that lands, and Share and Stop sharing after that. The hint is offered once per visit, on the first [`Action::Load`].
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        let mut effects = self.dispatch(action);
        keep_one(&mut effects, toast_line);
        effects
    }

    fn dispatch(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load(deck_id) => self.load(deck_id),
            Action::Retry => self.retry(),
            Action::ProfileLoaded { ticket, profile } => self.profile_loaded(ticket, *profile),
            Action::DeckLoaded { ticket, deck } => {
                self.deck_loaded(ticket, *deck);
                Vec::new()
            }
            Action::CommanderLoaded { ticket, card } => {
                if self.commander_waiting == Some(ticket) {
                    self.commander_waiting = None;
                    self.commander = Some(*card);
                }
                Vec::new()
            }
            Action::Failed { ticket } => {
                self.failed(ticket);
                Vec::new()
            }
            Action::Gone { ticket } => {
                self.failed(ticket);
                vec![Effect::Replace(Destination::DeckList)]
            }
            Action::PickCurrency(currency) => {
                self.currency = currency;
                Vec::new()
            }
            Action::Back => vec![Effect::Navigate(Destination::Back)],
            Action::Edit => self.navigate(Destination::EditDeck),
            Action::Cards => self.navigate(Destination::DeckCards),
            Action::AddCards => self.navigate(Destination::AddCards),
            Action::More(item) => self.more(item),
            Action::Buy(retailer) => self
                .buy_link(retailer)
                .map(Effect::OpenUrl)
                .into_iter()
                .collect(),
            Action::CopyLink => self.share_token.map(copy_link).into_iter().collect(),
            Action::SetQuantity { card, quantity } => {
                self.mutate(Mutation::SetQuantity { card, quantity })
            }
            Action::ClearCommander => self.mutate(Mutation::ClearCommander),
            Action::RemoveCard { card } => self.mutate(Mutation::RemoveCard { card }),
            Action::Clone { name } => {
                let name = name.trim();
                if name.is_empty() {
                    return Vec::new();
                }
                self.mutate(Mutation::Clone(name.to_owned()))
            }
            Action::Share => self.mutate(Mutation::Share),
            Action::StopShare => self.mutate(Mutation::Unshare),
            Action::ClearSkips => self.mutate(Mutation::ClearSkips),
            Action::Delete => self.mutate(Mutation::Delete),
            Action::QuantitySet { ticket } => self.land(ticket, Landed::QuantitySet),
            Action::CommanderCleared { ticket } => self.land(ticket, Landed::CommanderCleared),
            Action::CardRemoved { ticket } => self.land(ticket, Landed::CardRemoved),
            Action::Cloned { ticket, deck_id } => self.land(ticket, Landed::Cloned(deck_id)),
            Action::Shared { ticket, token } => self.land(ticket, Landed::Shared(token)),
            Action::Unshared { ticket } => self.land(ticket, Landed::Unshared),
            Action::SkipsCleared { ticket } => self.land(ticket, Landed::SkipsCleared),
            Action::Deleted { ticket } => self.land(ticket, Landed::Deleted),
        }
    }

    fn deck_loaded(&mut self, ticket: Ticket, deck: Deck) {
        let metrics =
            (!deck.entries.is_empty() || !deck.command_zone_cards.is_empty()).then(|| {
                DeckMetrics::from_entries_and_command_zone(&deck.entries, &deck.command_zone_cards)
            });
        self.deck
            .settle(ticket, Fetched::Loaded(Loaded { deck, metrics }));
    }

    /// Lands the reply to the request holding `ticket`, when it is still waiting and is the kind `landed` answers.
    fn land(&mut self, ticket: Ticket, landed: Landed) -> Vec<Effect> {
        let Some(mutation) = self.settle(ticket) else {
            return Vec::new();
        };
        match (mutation, landed) {
            (Mutation::SetQuantity { quantity, .. }, Landed::QuantitySet) => {
                let mut effects = vec![Effect::Toast(Done::QuantitySet(quantity).toast())];
                effects.extend(self.fetch_deck());
                effects
            }
            (Mutation::ClearCommander, Landed::CommanderCleared) => {
                let format = self.profile().and_then(|p| p.format);
                self.commander = None;
                let mut effects = vec![Effect::Toast(Done::CommanderCleared(format).toast())];
                effects.extend(self.fetch_profile());
                effects.extend(self.fetch_deck());
                effects
            }
            (Mutation::RemoveCard { .. }, Landed::CardRemoved) => {
                let mut effects = vec![Effect::Toast(Done::CardRemoved.toast())];
                effects.extend(self.fetch_deck());
                effects
            }
            (Mutation::Clone(name), Landed::Cloned(deck_id)) => vec![
                Effect::Toast(Done::Cloned(name).toast()),
                Effect::Navigate(Destination::Deck(deck_id)),
            ],
            (Mutation::Share, Landed::Shared(token)) => {
                self.share_token = Some(token);
                vec![copy_link(token)]
            }
            (Mutation::Unshare, Landed::Unshared) => {
                self.share_token = None;
                vec![Effect::Toast(Done::LinkDisabled.toast())]
            }
            (Mutation::ClearSkips, Landed::SkipsCleared) => {
                vec![Effect::Toast(Done::SkipsCleared.toast())]
            }
            (Mutation::Delete, Landed::Deleted) => vec![Effect::Replace(Destination::DeckList)],
            _ => Vec::new(),
        }
    }

    fn load(&mut self, deck_id: Uuid) -> Vec<Effect> {
        self.deck_id = Some(deck_id);
        self.mutations.clear();
        let mut effects: Vec<Effect> = self
            .fetch_profile()
            .into_iter()
            .chain(self.fetch_deck())
            .collect();
        if self.hint == HintOffer::Waiting {
            self.hint = HintOffer::Offered;
            effects.push(Effect::OfferHint(OneTimeHint::FirstDeck));
        }
        effects
    }

    /// Fetches again whatever failed, back on its skeleton, and leaves a load that landed alone.
    fn retry(&mut self) -> Vec<Effect> {
        let mut effects = Vec::new();
        if matches!(self.profile.value, Fetched::Failed) {
            self.profile.value = Fetched::Loading;
            effects.extend(self.fetch_profile());
        }
        if matches!(self.deck.value, Fetched::Failed) {
            self.deck.value = Fetched::Loading;
            effects.extend(self.fetch_deck());
        }
        effects
    }

    fn profile_loaded(&mut self, ticket: Ticket, profile: DeckProfile) -> Vec<Effect> {
        let (commander, currency, share_token) = (
            profile.commander_id,
            profile.price_target_currency,
            profile.share_token,
        );
        if !self.profile.settle(ticket, Fetched::Loaded(profile)) {
            return Vec::new();
        }
        if !self.currency_seeded {
            self.currency_seeded = true;
            if let Some(currency) = currency {
                self.currency = currency;
            }
        }
        self.share_token = share_token;
        let Some(card) = commander else {
            self.commander = None;
            self.commander_waiting = None;
            return Vec::new();
        };
        let ticket = self.tickets.next_ticket();
        self.commander_waiting = Some(ticket);
        vec![Effect::FetchCommander { ticket, card }]
    }

    fn failed(&mut self, ticket: Ticket) {
        if self.commander_waiting == Some(ticket) {
            self.commander_waiting = None;
            return;
        }
        if self.profile.settle(ticket, Fetched::Failed) || self.deck.settle(ticket, Fetched::Failed)
        {
            return;
        }
        let _ = self.settle(ticket);
    }

    fn fetch_profile(&mut self) -> Option<Effect> {
        let deck_id = self.deck_id?;
        let ticket = self.tickets.next_ticket();
        self.profile.start(ticket);
        Some(Effect::FetchProfile { ticket, deck_id })
    }

    fn fetch_deck(&mut self) -> Option<Effect> {
        let deck_id = self.deck_id?;
        let ticket = self.tickets.next_ticket();
        self.deck.start(ticket);
        Some(Effect::FetchDeck { ticket, deck_id })
    }

    fn navigate(&self, to: fn(Uuid) -> Destination) -> Vec<Effect> {
        self.deck_id
            .map(|id| Effect::Navigate(to(id)))
            .into_iter()
            .collect()
    }

    fn more(&self, item: MoreItem) -> Vec<Effect> {
        if !self.is_enabled(item) {
            return Vec::new();
        }
        match item {
            MoreItem::ImportCards => self.navigate(Destination::ImportDeck),
            MoreItem::ExportCards => self.navigate(Destination::ExportDeck),
            MoreItem::CloneDeck => vec![Effect::OpenDialog(Dialog::Clone)],
            MoreItem::ShareDeck => vec![Effect::OpenDialog(Dialog::Share)],
            MoreItem::ClearSkips => vec![Effect::OpenDialog(Dialog::ClearSkips)],
            MoreItem::DeleteDeck => vec![Effect::OpenDialog(Dialog::Delete)],
        }
    }

    /// Sends a request that changes the deck. Nothing before [`Action::Load`].
    fn mutate(&mut self, mutation: Mutation) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let ticket = self.tickets.next_ticket();
        let effect = mutation.effect(ticket, deck_id);
        self.mutations.push((ticket, mutation));
        vec![effect]
    }

    /// Takes the request holding `ticket` off the waiting list, if it is there.
    fn settle(&mut self, ticket: Ticket) -> Option<Mutation> {
        let at = self.mutations.iter().position(|(t, _)| *t == ticket)?;
        Some(self.mutations.remove(at).1)
    }

    /// Where the profile fetch stands.
    #[must_use]
    pub fn status(&self) -> Status {
        match self.profile.value {
            Fetched::Loading => Status::Loading,
            Fetched::Failed => Status::Failed,
            Fetched::Loaded(_) => Status::Loaded,
        }
    }

    /// The deck's id, once [`Action::Load`] has named it.
    #[must_use]
    pub fn deck_id(&self) -> Option<Uuid> {
        self.deck_id
    }

    /// The deck's profile, once it has loaded.
    #[must_use]
    pub fn profile(&self) -> Option<&DeckProfile> {
        self.profile.loaded()
    }

    /// The commander's card, once it has loaded. The profile row falls back to the profile's commander name without it.
    #[must_use]
    pub fn commander(&self) -> Option<&Card> {
        self.commander.as_ref()
    }

    /// The full deck, once it has loaded.
    #[must_use]
    pub fn deck(&self) -> Option<&Deck> {
        self.deck.loaded().map(|l| &l.deck)
    }

    /// Whether the full deck has yet to arrive for the first time. A reload keeps the deck it replaces, so this stays false through one.
    #[must_use]
    pub fn deck_loading(&self) -> bool {
        matches!(self.deck.value, Fetched::Loading)
    }

    /// The deck's metrics, present once the deck has a mainboard or command-zone card.
    #[must_use]
    pub fn metrics(&self) -> Option<&DeckMetrics> {
        self.deck.loaded().and_then(|l| l.metrics.as_ref())
    }

    /// The metrics when the deck has a mainboard card to chart.
    #[must_use]
    pub fn charted(&self) -> Option<&DeckMetrics> {
        self.metrics()
            .filter(|m| m.land_count > 0 || m.nonland_count > 0)
    }

    /// Whether the deck has a card to export or buy.
    #[must_use]
    pub fn has_cards(&self) -> bool {
        self.metrics().is_some()
    }

    /// The deck's warnings, empty until the deck loads.
    #[must_use]
    pub fn warnings(&self) -> &[DeckWarning] {
        self.deck().map_or(&[], |d| &d.warnings)
    }

    /// The Profile card's card count: the metrics' total, command zone included, else the profile's count.
    #[must_use]
    pub fn card_count(&self) -> usize {
        match (self.metrics(), self.profile()) {
            (Some(metrics), _) => metrics.total_cards,
            (None, Some(profile)) => usize::try_from(profile.card_count).unwrap_or(0),
            (None, None) => 0,
        }
    }

    /// The sections with something to show, in order. Empty until the profile loads.
    ///
    /// Budget shows with a price target or metrics, Tags with any tag, Distributions and Draw odds with a mainboard card, Mana with a mainboard card or a land target, Warnings with any warning.
    #[must_use]
    pub fn sections(&self) -> Vec<Section> {
        let Some(profile) = self.profile() else {
            return Vec::new();
        };
        let charted = self.charted().is_some();
        Section::ALL
            .into_iter()
            .filter(|section| match section {
                Section::Budget => view::has_budget(profile, self.metrics().is_some()),
                Section::Tags => view::tag_count(profile) > 0,
                Section::Distributions | Section::DrawOdds => charted,
                Section::Mana => charted || profile.land_target.is_some(),
                Section::Warnings => !self.warnings().is_empty(),
            })
            .collect()
    }

    /// The count beside a section's title: the tag count on Tags, the warning count on Warnings.
    #[must_use]
    pub fn badge(&self, section: Section) -> Option<String> {
        match section {
            Section::Tags => self.profile().map(|p| view::tag_count(p).to_string()),
            Section::Warnings => Some(self.warnings().len().to_string()),
            Section::Budget | Section::Distributions | Section::Mana | Section::DrawOdds => None,
        }
    }

    /// Whether a skeleton stands in for the stats: the deck is on its first fetch, Mana isn't showing, and the deck could have stats.
    #[must_use]
    pub fn stats_loading(&self) -> bool {
        self.deck_loading()
            && !self.sections().contains(&Section::Mana)
            && self.profile().is_some_and(view::metrics_possible)
    }

    /// The currency the Budget section's running prices read in.
    #[must_use]
    pub fn currency(&self) -> PriceCurrency {
        self.currency
    }

    /// The Budget section's currency chips: shown once the deck has metrics.
    #[must_use]
    pub fn currency_chips_shown(&self) -> bool {
        self.metrics().is_some()
    }

    /// The [`TOTAL_PRICE`] row's value in the picked currency. `None` until the profile loads.
    #[must_use]
    pub fn total_price_line(&self) -> Option<String> {
        let total = self.metrics().and_then(|m| self.totals(m).0);
        self.profile()
            .map(|p| view::total_price_line(p, total, self.currency))
    }

    /// The [`AVERAGE_CARD_PRICE`] row's value in the picked currency, [`NO_PRICE`] when nothing is priced. `None` without metrics, which hides the row.
    #[must_use]
    pub fn average_price_line(&self) -> Option<String> {
        let metrics = self.metrics()?;
        Some(
            self.totals(metrics)
                .1
                .map_or_else(|| NO_PRICE.to_owned(), |v| self.currency.format_amount(v)),
        )
    }

    /// The running total and average in the picked currency.
    fn totals(&self, metrics: &DeckMetrics) -> (Option<f64>, Option<f64>) {
        match self.currency {
            PriceCurrency::Usd => (metrics.total_price_usd, metrics.avg_price_usd),
            PriceCurrency::Eur => (metrics.total_price_eur, metrics.avg_price_eur),
            PriceCurrency::Tix => (metrics.total_price_tix, metrics.avg_price_tix),
        }
    }

    /// The [`AVERAGE_PT`] row's value, `power / toughness` with `-` for a missing side. `None` when neither is known, which hides the row.
    #[must_use]
    pub fn average_pt_line(&self) -> Option<String> {
        let metrics = self.metrics()?;
        if metrics.avg_power.is_none() && metrics.avg_toughness.is_none() {
            return None;
        }
        let side = |v: Option<f64>| v.map_or_else(|| "-".to_owned(), |v| format!("{v:.1}"));
        Some(format!(
            "{} / {}",
            side(metrics.avg_power),
            side(metrics.avg_toughness)
        ))
    }

    /// The [`LANDS`] row's value: the land count, then ` / ` and the land target when the deck has one.
    #[must_use]
    pub fn lands_line(&self) -> String {
        let count = self.metrics().map_or(0, |m| m.land_count);
        match self.profile().and_then(|p| p.land_target) {
            Some(target) => format!("{count} / {target}"),
            None => count.to_string(),
        }
    }

    /// The [`AVERAGE_MANA_VALUE`] row's value, to one decimal. `None` without a mainboard card, which hides the row.
    #[must_use]
    pub fn average_mana_value_line(&self) -> Option<String> {
        self.charted().map(|m| format!("{:.1}", m.avg_cmc))
    }

    /// Whether an action button can be pressed: Export waits for a card.
    #[must_use]
    pub fn is_enabled(&self, item: MoreItem) -> bool {
        match item {
            MoreItem::ExportCards => self.has_cards(),
            MoreItem::ImportCards
            | MoreItem::CloneDeck
            | MoreItem::ShareDeck
            | MoreItem::ClearSkips
            | MoreItem::DeleteDeck => true,
        }
    }

    /// The `TCGplayer` and `Card Kingdom` links: the profile's command zone, then the main and side boards. `None` until the deck loads with a card.
    #[must_use]
    pub fn buy_links(&self) -> Option<(String, String)> {
        let deck = self.deck().filter(|_| self.has_cards())?;
        let names: Vec<&str> = self
            .profile()
            .map(|p| {
                [
                    p.commander_name.as_deref(),
                    p.partner_commander_name.as_deref(),
                    p.background_name.as_deref(),
                    p.signature_spell_name.as_deref(),
                ]
                .into_iter()
                .flatten()
                .collect()
            })
            .unwrap_or_default();
        let entries: Vec<&DeckEntry> = deck
            .entries
            .iter()
            .filter(|e| e.deck_card.board.is_active() || e.deck_card.board.is_sideboard())
            .collect();
        Some((
            buy::tcgplayer_url(&entries, &names),
            buy::cardkingdom_url(&entries, &names),
        ))
    }

    /// The Buy row's link for `retailer`, while [`Self::buy_links`] has them.
    fn buy_link(&self, retailer: Retailer) -> Option<String> {
        let (tcg, ck) = self.buy_links()?;
        Some(match retailer {
            Retailer::TcgPlayer => tcg,
            Retailer::CardKingdom => ck,
        })
    }

    /// The live share link's token: `Some` while the deck has a public link.
    #[must_use]
    pub fn share_token(&self) -> Option<Uuid> {
        self.share_token
    }

    /// The live share link, which the Share dialog shows under its line: `Some` while the deck has
    /// a public link.
    #[must_use]
    pub fn share_url(&self) -> Option<String> {
        self.share_token.map(view::share_link)
    }

    /// The Share dialog's line: [`SHARE_SHARED`] while the deck has a link, else [`SHARE_UNSHARED`].
    #[must_use]
    pub fn share_body(&self) -> &'static str {
        if self.share_token.is_some() {
            SHARE_SHARED
        } else {
            SHARE_UNSHARED
        }
    }

    /// The Clone dialog's prefill, `{name} (clone)`, empty until the profile loads.
    #[must_use]
    pub fn clone_name(&self) -> String {
        self.profile()
            .map(|p| view::clone_name(&p.name))
            .unwrap_or_default()
    }
}

/// Copies the link for `token` and says so.
fn copy_link(token: Uuid) -> Effect {
    Effect::Copy {
        text: view::share_link(token),
        done: Done::LinkCopied.toast(),
    }
}
