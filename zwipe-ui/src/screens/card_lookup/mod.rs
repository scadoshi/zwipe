//! The card screen: one card opened by its name's slug (`/card/<slug>`) or one printing by its id
//! (`/card/<slug>/<id>`), signed in or out.
//!
//! A name opens the card when one playable card has it, a short list when several do (each to
//! its own printing link), and [`NOT_FOUND`] when none does; a token or art card that shares a
//! playable card's name stays out of the count, and a card whose full name it is wins over cards
//! it only names the front face of. A printing link loads by its id alone, and a
//! stale slug in it is swapped for the card's own. Create account, the dock's signed-out button,
//! goes to Register and back to the card. Send to signed out goes to Login and comes
//! back to the card with the sheet asked for; the screen opens it once and drops the ask from
//! the route, so Back or a reload never opens it again; on a list, the ask goes with the card
//! picked from it. Share copies the card's link: the name's
//! link for a card opened by name, the printing's link for a printing, which is where a name
//! several cards share leads.
//!
//! The Printings sheet picks another printing of the card shown, which takes the page's place
//! until another pick; Send to and Share take it too. A route can name one (`?printing=<id>`):
//! the page waits for it once the card loads, and falls back to the card when it is unknown or
//! another card's.
//!
//! The model holds what loaded and decides what each action does. Drawing the card, the sheets,
//! the share sheet and the clipboard stay in the client.

pub mod stats;
#[cfg(test)]
mod tests;

use std::fmt;
use std::str::FromStr;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::criteria::PLAYABLE_LAYOUTS;
use zwipe_core::domain::card::slug::{self, CardSlug};
use zwipe_core::http::contracts::card::HttpCardStats;

use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// The line when no card has the name.
pub const NOT_FOUND: &str = "No card by that name";

/// The toast once the card's link is copied.
pub const LINK_COPIED: &str = "Share link copied";

/// The screen's title while no card or name names it: loading, no card, or a failed fetch.
pub const SCREEN_TITLE: &str = "Card";

/// What a card route asks the screen to open once the card shows, its `open` query value. A
/// route with no `open` asks for nothing, so the route holds an `Option<Ask>`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Ask {
    /// Send to: `?open=send-to`. The only value, so it is also the default the router asks for
    /// when it parses a query.
    #[default]
    SendTo,
}

impl Ask {
    /// The query value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SendTo => "send-to",
        }
    }
}

impl fmt::Display for Ask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An `open` query value that names no [`Ask`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseAskError(pub String);

impl fmt::Display for ParseAskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown card ask {:?}", self.0)
    }
}

impl std::error::Error for ParseAskError {}

impl FromStr for Ask {
    type Err = ParseAskError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "send-to" => Ok(Self::SendTo),
            other => Err(ParseAskError(other.to_owned())),
        }
    }
}

/// The heading over the cards that share a name, the first one's.
#[must_use]
pub fn several_title(name: &str) -> String {
    format!("Cards named {name}")
}

/// How the screen was opened: the route's parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Open {
    /// `/card/<slug>`: every card with this name, as the route spells it.
    Named {
        /// The slug segment, unchecked.
        slug: String,
    },
    /// `/card/<slug>/<id>`: one printing. The slug is cosmetic.
    Printing {
        /// The slug segment, unchecked.
        slug: String,
        /// The printing's Scryfall id.
        id: Uuid,
    },
}

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// The card a slug names, asking for Send to when `send`.
    Named {
        /// The name's slug.
        slug: CardSlug,
        /// Whether the screen opens Send to once the card shows.
        send: bool,
        /// The Scryfall id of the printing picked over the card, if any.
        printing: Option<Uuid>,
    },
    /// One printing, asking for Send to when `send`.
    Printing {
        /// The slug segment ([`slug::printing_path`]'s).
        slug: String,
        /// The printing's Scryfall id.
        id: Uuid,
        /// Whether the screen opens Send to once the card shows.
        send: bool,
        /// The Scryfall id of the printing picked over the card, if any.
        printing: Option<Uuid>,
    },
    /// Login, for Send to signed out.
    Login,
    /// Create account, from the dock signed out.
    Register,
}

/// What the screen shows, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The fetch is out: the card's skeleton.
    Loading,
    /// One card: [`State::card`].
    One,
    /// Several cards share the name: [`State::cards`] under [`State::title`].
    Several,
    /// No card has the name, or the printing is gone: [`NOT_FOUND`].
    NotFound,
    /// The fetch failed: the load-failed panel with Retry.
    Failed,
}

/// What a user does on the card screen, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Opens the screen from its route. The route the screen is already on, with no ask, does
    /// nothing: that is the screen's own [`Effect::Replace`] dropping an ask.
    Load {
        /// The route's parts.
        open: Open,
        /// Whether the route asks for Send to (`?send`).
        send: bool,
        /// Whether a session exists, so the asked-for Send to can open.
        signed_in: bool,
        /// The Scryfall id of the printing the route picks over the card (`?printing=`).
        printing: Option<Uuid>,
    },
    /// The cards with the name arrived for the fetch holding `ticket`.
    Named {
        /// The ticket from the [`Effect::FetchNamed`] this answers.
        ticket: Ticket,
        /// Every card with the name, in the server's order.
        cards: Vec<Card>,
    },
    /// The printing arrived for the fetch holding `ticket`.
    Printing {
        /// The ticket from the [`Effect::FetchCard`] this answers.
        ticket: Ticket,
        /// The printing.
        card: Box<Card>,
    },
    /// The server has no such printing.
    Missing {
        /// The ticket from the [`Effect::FetchCard`] this answers.
        ticket: Ticket,
    },
    /// The fetch holding `ticket` failed.
    Failed {
        /// The ticket of the fetch that failed.
        ticket: Ticket,
    },
    /// Retry after a failure.
    Retry,
    /// One of several cards that share the name, by its printing id.
    Pick(Uuid),
    /// Send to, for the card on screen.
    SendTo {
        /// Whether a session exists.
        signed_in: bool,
    },
    /// Share, for the card on screen.
    Share,
    /// The dock's Create account, signed out: opens Register, which comes back to the card.
    CreateAccount,
    /// The card's stats arrived for the fetch holding `ticket`.
    Stats {
        /// The ticket from the [`Effect::FetchStats`] this answers.
        ticket: Ticket,
        /// The card's counts.
        stats: HttpCardStats,
    },
    /// The stats fetch holding `ticket` failed.
    StatsFailed {
        /// The ticket of the fetch that failed.
        ticket: Ticket,
    },
    /// Retry the card's stats after a failure.
    RetryStats,
    /// The Printings sheet's pick: another printing of the card shown, or the card itself.
    PickPrinting(Box<Card>),
    /// The route's printing arrived for the fetch holding `ticket`.
    PrintingPicked {
        /// The ticket from the [`Effect::FetchPrinting`] this answers.
        ticket: Ticket,
        /// The printing.
        card: Box<Card>,
    },
    /// The route's printing is gone, or its fetch failed.
    PrintingUnknown {
        /// The ticket from the [`Effect::FetchPrinting`] this answers.
        ticket: Ticket,
    },
}

/// What the client does for an action, in order.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Fetches every card with the name (`fetch_named_cards`).
    FetchNamed {
        /// Handed back on the reply.
        ticket: Ticket,
        /// The name's slug.
        slug: CardSlug,
    },
    /// Fetches the printing (`fetch_card`), answered with [`Action::Printing`], or
    /// [`Action::Missing`] for a 404.
    FetchCard {
        /// Handed back on the reply.
        ticket: Ticket,
        /// The printing's Scryfall id.
        id: Uuid,
    },
    /// Replaces the route in history with this one.
    Replace(Destination),
    /// Moves to this route.
    Navigate(Destination),
    /// Saves this route for Login to return to after a sign-in.
    RememberReturn(Destination),
    /// Opens Send to for `card`.
    OpenSendTo {
        /// The card it sends.
        card: Box<Card>,
        /// Whether `card` is a printing picked over the card, which the sent toast names.
        picked: bool,
    },
    /// Fetches the route's printing (`fetch_card`), answered with [`Action::PrintingPicked`],
    /// or [`Action::PrintingUnknown`] for a 404 or a failure.
    FetchPrinting {
        /// Handed back on the reply.
        ticket: Ticket,
        /// The printing's Scryfall id.
        id: Uuid,
    },
    /// Writes the printing shown into the route in place. A client whose routes carry no
    /// printing does nothing.
    ShowPrinting(Destination),
    /// Fetches the card's stats (`fetch_card_stats`), answered with [`Action::Stats`] or
    /// [`Action::StatsFailed`].
    FetchStats {
        /// Handed back on the reply.
        ticket: Ticket,
        /// The card's oracle id.
        oracle_id: Uuid,
    },
    /// Copies `text`, then raises `done`. A client that hands the link to a share sheet instead
    /// raises nothing.
    Copy {
        /// The card's link.
        text: String,
        /// The toast once it is copied.
        done: Toast,
    },
}

/// What loaded.
#[derive(Debug, Clone, Default, PartialEq)]
enum Shown {
    #[default]
    Loading,
    One(Box<Card>),
    Several(Vec<Card>),
    NotFound,
    Failed,
}

/// The printing that takes the loaded card's place.
#[derive(Debug, Clone, Default, PartialEq)]
enum Pick {
    /// None: the loaded card shows.
    #[default]
    Loaded,
    /// The route's printing, by its Scryfall id, fetched once the card shows. The page waits on
    /// it.
    Wanted(Uuid),
    /// A printing of the loaded card, shown in its place.
    Shown(Box<Card>),
}

/// Where the card's stats stand, for the client to pick what its stats widgets draw.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CardStats {
    /// The fetch is out, or no card shows yet.
    #[default]
    Loading,
    /// The card's counts.
    Loaded(HttpCardStats),
    /// The fetch failed: [`Action::RetryStats`] asks again.
    Failed,
}

/// The card screen's state. [`State::new`] is the screen before [`Action::Load`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct State {
    open: Option<Open>,
    shown: Shown,
    /// Whether Send to opens once the card shows.
    send_pending: bool,
    fetch: Generation,
    stats: CardStats,
    stats_fetch: Generation,
    pick: Pick,
    pick_fetch: Generation,
}

impl State {
    /// The screen before its route loads.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client does next.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load {
                open,
                send,
                signed_in,
                printing,
            } => self.load(open, send, signed_in, printing),
            Action::Named { ticket, cards } => {
                if !self.fetch.is_current(ticket) {
                    return Vec::new();
                }
                let slug = match &self.open {
                    Some(Open::Named { slug }) => slug.clone(),
                    Some(Open::Printing { .. }) | None => String::new(),
                };
                let cards = named_matches(cards, &slug);
                self.shown = match cards.len() {
                    0 => Shown::NotFound,
                    1 => cards
                        .into_iter()
                        .next()
                        .map_or(Shown::NotFound, |card| Shown::One(Box::new(card))),
                    _ => Shown::Several(cards),
                };
                let mut effects = self.fetch_stats();
                effects.extend(self.fetch_wanted());
                effects.extend(self.send_if_asked());
                effects
            }
            Action::Printing { ticket, card } => self.printing_arrived(ticket, card),
            Action::Missing { ticket } => {
                if self.fetch.is_current(ticket) {
                    self.shown = Shown::NotFound;
                    self.send_pending = false;
                }
                Vec::new()
            }
            Action::Failed { ticket } => {
                if self.fetch.is_current(ticket) {
                    self.shown = Shown::Failed;
                }
                Vec::new()
            }
            Action::Retry => self.fetch_open(),
            Action::Pick(id) => {
                let Some(picked) = self.cards().iter().find(|c| c.scryfall_data.id == id) else {
                    return Vec::new();
                };
                let slug = own_printing_slug(picked);
                let send = std::mem::take(&mut self.send_pending);
                vec![Effect::Navigate(Destination::Printing {
                    slug,
                    id,
                    send,
                    printing: None,
                })]
            }
            Action::SendTo { signed_in } => {
                let Some(card) = self.card().cloned() else {
                    return Vec::new();
                };
                if signed_in {
                    return vec![self.open_send_to(card)];
                }
                let Some(back) = self.route(/* send */ true) else {
                    return Vec::new();
                };
                vec![
                    Effect::RememberReturn(back),
                    Effect::Navigate(Destination::Login),
                ]
            }
            Action::CreateAccount => self
                .route(/* send */ false)
                .map(Effect::RememberReturn)
                .into_iter()
                .chain([Effect::Navigate(Destination::Register)])
                .collect(),
            Action::Share => match (&self.open, self.card()) {
                (Some(open), Some(card)) => vec![Effect::Copy {
                    text: share_link(open, card, self.picked()),
                    done: link_copied(),
                }],
                _ => Vec::new(),
            },
            Action::Stats { ticket, stats } => {
                self.stats_answered(ticket, CardStats::Loaded(stats))
            }
            Action::StatsFailed { ticket } => self.stats_answered(ticket, CardStats::Failed),
            Action::RetryStats => {
                if self.stats != CardStats::Failed {
                    return Vec::new();
                }
                self.fetch_stats()
            }
            Action::PickPrinting(card) => self.pick_printing(card),
            Action::PrintingPicked { ticket, card } => self.printing_answered(ticket, Some(card)),
            Action::PrintingUnknown { ticket } => self.printing_answered(ticket, None),
        }
    }

    /// Opens the screen at `open`: drops a Send to ask from the route at once, keeping it for
    /// when the card shows if a session exists, then asks for what the route names.
    fn load(
        &mut self,
        open: Open,
        send: bool,
        signed_in: bool,
        printing: Option<Uuid>,
    ) -> Vec<Effect> {
        // The route the screen is on, without an ask, is the screen's own Replace dropping the
        // ask or writing a pick: the card is already loading or shown, and an asked-for Send to
        // still waits.
        if !send && self.open.as_ref() == Some(&open) {
            return self.want(printing);
        }
        self.open = Some(open);
        self.send_pending = send && signed_in;
        self.pick_fetch.next_ticket();
        self.pick = printing.map_or(Pick::Loaded, Pick::Wanted);
        let mut effects = Vec::new();
        if send && let Some(route) = self.route(/* send */ false) {
            effects.push(Effect::Replace(route));
        }
        effects.extend(self.fetch_open());
        effects
    }

    /// Asks for what the route names, or finds nothing when its slug isn't one.
    fn fetch_open(&mut self) -> Vec<Effect> {
        let ticket = self.fetch.next_ticket();
        match &self.open {
            Some(Open::Named { slug }) => {
                if let Ok(slug) = slug.parse::<CardSlug>() {
                    self.shown = Shown::Loading;
                    vec![Effect::FetchNamed { ticket, slug }]
                } else {
                    self.shown = Shown::NotFound;
                    self.send_pending = false;
                    Vec::new()
                }
            }
            Some(Open::Printing { id, .. }) => {
                self.shown = Shown::Loading;
                vec![Effect::FetchCard { ticket, id: *id }]
            }
            None => Vec::new(),
        }
    }

    /// Shows the printing that arrived for the fetch holding `ticket`, fixing a stale slug in the
    /// route, then asks for its stats and opens an asked-for Send to.
    fn printing_arrived(&mut self, ticket: Ticket, card: Box<Card>) -> Vec<Effect> {
        if !self.fetch.is_current(ticket) {
            return Vec::new();
        }
        let id = card.scryfall_data.id;
        let own = own_printing_slug(&card);
        self.shown = Shown::One(card);
        let mut effects = self.fetch_stats();
        if let Some(Open::Printing { slug, .. }) = &self.open
            && *slug != own
        {
            effects.push(Effect::Replace(Destination::Printing {
                slug: own,
                id,
                send: false,
                printing: self.printing(),
            }));
        }
        effects.extend(self.fetch_wanted());
        effects.extend(self.send_if_asked());
        effects
    }

    /// Takes the printing a route of the same card names: nothing when it is the one shown or
    /// already wanted, the loaded card again when it names none or the card itself, else waits
    /// for it.
    fn want(&mut self, printing: Option<Uuid>) -> Vec<Effect> {
        if printing == self.printing() {
            return Vec::new();
        }
        self.pick_fetch.next_ticket();
        match printing {
            Some(id)
                if self
                    .loaded()
                    .is_none_or(|loaded| loaded.scryfall_data.id != id) =>
            {
                self.pick = Pick::Wanted(id);
                self.fetch_wanted()
            }
            Some(_) | None => {
                self.pick = Pick::Loaded;
                self.send_if_asked()
            }
        }
    }

    /// Asks for the wanted printing once one card shows: none is wanted, or it is the card
    /// itself, and nothing goes out. Several cards, or none, drop it.
    fn fetch_wanted(&mut self) -> Vec<Effect> {
        let Pick::Wanted(id) = self.pick else {
            return Vec::new();
        };
        match self.loaded() {
            Some(loaded) if loaded.scryfall_data.id == id => {
                self.pick = Pick::Loaded;
                Vec::new()
            }
            Some(_) => {
                let ticket = self.pick_fetch.next_ticket();
                vec![Effect::FetchPrinting { ticket, id }]
            }
            None if self.shown == Shown::Loading => Vec::new(),
            None => {
                self.pick = Pick::Loaded;
                Vec::new()
            }
        }
    }

    /// Takes the route's printing fetch's answer, `card` or none, when `ticket` is still the
    /// current fetch's: shows it when it is the wanted printing of the loaded card, else falls
    /// back to the card.
    fn printing_answered(&mut self, ticket: Ticket, card: Option<Box<Card>>) -> Vec<Effect> {
        if !self.pick_fetch.is_current(ticket) {
            return Vec::new();
        }
        let Some(card) = card.filter(|card| {
            matches!(self.pick, Pick::Wanted(id) if id == card.scryfall_data.id)
                && self.loaded().is_some_and(|loaded| same_card(loaded, card))
        }) else {
            return self.drop_wanted();
        };
        self.pick = Pick::Shown(card);
        self.send_if_asked()
    }

    /// Falls back from a wanted printing that is unknown or another card's to the loaded card,
    /// taking it out of the route, then opens an asked-for Send to.
    fn drop_wanted(&mut self) -> Vec<Effect> {
        self.pick = Pick::Loaded;
        let mut effects: Vec<Effect> = self
            .route(/* send */ false)
            .map(Effect::ShowPrinting)
            .into_iter()
            .collect();
        effects.extend(self.send_if_asked());
        effects
    }

    /// Shows the sheet's pick, `card`, in the loaded card's place, the loaded card again when it
    /// is that one, and writes it into the route. A printing of another card does nothing.
    fn pick_printing(&mut self, card: Box<Card>) -> Vec<Effect> {
        let Some(loaded) = self.loaded() else {
            return Vec::new();
        };
        if self.card().is_none() || !same_card(loaded, &card) {
            return Vec::new();
        }
        self.pick = if loaded.scryfall_data.id == card.scryfall_data.id {
            Pick::Loaded
        } else {
            Pick::Shown(card)
        };
        self.pick_fetch.next_ticket();
        self.route(/* send */ false)
            .map(Effect::ShowPrinting)
            .into_iter()
            .collect()
    }

    /// Opens Send to for `card`, the card on screen.
    fn open_send_to(&self, card: Card) -> Effect {
        Effect::OpenSendTo {
            card: Box::new(card),
            picked: self.picked(),
        }
    }

    /// Takes the stats fetch's answer, `answer`, when `ticket` is still the current fetch's.
    fn stats_answered(&mut self, ticket: Ticket, answer: CardStats) -> Vec<Effect> {
        if self.stats_fetch.is_current(ticket) {
            self.stats = answer;
        }
        Vec::new()
    }

    /// Asks for the shown card's stats under a new ticket, so an answer for another card is
    /// dropped. A card with no oracle id has no stats to ask for, so its figures stay blank.
    fn fetch_stats(&mut self) -> Vec<Effect> {
        let ticket = self.stats_fetch.next_ticket();
        let Some(oracle_id) = self.loaded().and_then(|c| c.scryfall_data.oracle_id) else {
            self.stats = CardStats::Loaded(HttpCardStats::default());
            return Vec::new();
        };
        self.stats = CardStats::Loading;
        vec![Effect::FetchStats { ticket, oracle_id }]
    }

    /// Opens the asked-for Send to, once, when one card shows. On a list the ask waits for the
    /// pick ([`Action::Pick`]), which carries it to that card's printing.
    fn send_if_asked(&mut self) -> Vec<Effect> {
        if !self.send_pending
            || matches!(self.shown, Shown::Loading | Shown::Several(_))
            || matches!(self.pick, Pick::Wanted(_))
        {
            return Vec::new();
        }
        self.send_pending = false;
        let Some(card) = self.card().cloned() else {
            return Vec::new();
        };
        vec![self.open_send_to(card)]
    }

    /// The route the screen is on, asking for Send to when `send`. A name that isn't a slug has
    /// no route to come back to.
    fn route(&self, send: bool) -> Option<Destination> {
        let printing = self.printing();
        match self.open.as_ref()? {
            Open::Named { slug } => Some(Destination::Named {
                slug: slug.parse().ok()?,
                send,
                printing,
            }),
            Open::Printing { slug, id } => Some(Destination::Printing {
                slug: slug.clone(),
                id: *id,
                send,
                printing,
            }),
        }
    }

    /// The Scryfall id of the printing picked or wanted over the loaded card, for the route.
    fn printing(&self) -> Option<Uuid> {
        match &self.pick {
            Pick::Loaded => None,
            Pick::Wanted(id) => Some(*id),
            Pick::Shown(card) => Some(card.scryfall_data.id),
        }
    }

    /// Whether a picked printing shows in the loaded card's place.
    fn picked(&self) -> bool {
        matches!(self.pick, Pick::Shown(_))
    }

    /// The card the route loaded, when one did.
    fn loaded(&self) -> Option<&Card> {
        match &self.shown {
            Shown::One(card) => Some(card),
            Shown::Loading | Shown::Several(_) | Shown::NotFound | Shown::Failed => None,
        }
    }

    /// What the screen shows.
    #[must_use]
    pub fn status(&self) -> Status {
        match self.shown {
            Shown::Loading => Status::Loading,
            Shown::One(_) if matches!(self.pick, Pick::Wanted(_)) => Status::Loading,
            Shown::One(_) => Status::One,
            Shown::Several(_) => Status::Several,
            Shown::NotFound => Status::NotFound,
            Shown::Failed => Status::Failed,
        }
    }

    /// Where the card's stats stand.
    #[must_use]
    pub fn stats(&self) -> CardStats {
        self.stats
    }

    /// The card, when one shows: the printing picked, else the card the route loaded.
    #[must_use]
    pub fn card(&self) -> Option<&Card> {
        match &self.pick {
            Pick::Loaded => self.loaded(),
            Pick::Wanted(_) => None,
            Pick::Shown(card) => Some(card),
        }
    }

    /// The cards that share the name, when several do.
    #[must_use]
    pub fn cards(&self) -> &[Card] {
        match &self.shown {
            Shown::Several(cards) => cards,
            Shown::Loading | Shown::One(_) | Shown::NotFound | Shown::Failed => &[],
        }
    }

    /// The heading over several cards.
    #[must_use]
    pub fn title(&self) -> Option<String> {
        self.cards()
            .first()
            .map(|c| several_title(&c.scryfall_data.name))
    }

    /// The screen's title, for the browser tab and the folded nav: the card's name when one
    /// shows, the [`several_title`] heading when several share a name, else [`SCREEN_TITLE`].
    #[must_use]
    pub fn screen_title(&self) -> String {
        match &self.shown {
            Shown::One(card) => card.scryfall_data.name.clone(),
            Shown::Several(_) => self.title().unwrap_or_else(|| SCREEN_TITLE.to_owned()),
            Shown::Loading | Shown::NotFound | Shown::Failed => SCREEN_TITLE.to_owned(),
        }
    }

    /// The line to show in place of a card.
    #[must_use]
    pub fn message(&self) -> Option<&'static str> {
        (self.shown == Shown::NotFound).then_some(NOT_FOUND)
    }
}

/// The cards the name `slug` opens, in the server's order: its playable ones (search's playable
/// layouts), so a token or art card that shares the name never makes a list of one real card,
/// and of those the ones whose full name is `slug`, so `bind` is Bind and not Bind // Liberate.
/// A front face counts only when no full name matches, and a name with no playable card keeps
/// every match.
fn named_matches(cards: Vec<Card>, slug: &str) -> Vec<Card> {
    let (playable, others): (Vec<Card>, Vec<Card>) = cards
        .into_iter()
        .partition(|c| PLAYABLE_LAYOUTS.contains(&c.scryfall_data.layout.as_str()));
    let pool = if playable.is_empty() {
        others
    } else {
        playable
    };
    let (full, fronts): (Vec<Card>, Vec<Card>) = pool.into_iter().partition(|c| {
        slug::card_slug(&c.scryfall_data.name).is_some_and(|own| own.as_str() == slug)
    });
    if full.is_empty() { fronts } else { full }
}

/// Whether `printing` is a printing of `card`: the same Oracle id.
fn same_card(card: &Card, printing: &Card) -> bool {
    card.scryfall_data.oracle_id.is_some()
        && card.scryfall_data.oracle_id == printing.scryfall_data.oracle_id
}

/// The slug segment of `card`'s printing link ([`slug::printing_path`]'s).
fn own_printing_slug(card: &Card) -> String {
    slug::card_slug(&card.scryfall_data.name)
        .map_or_else(|| slug::PRINTING_SLUG_FALLBACK.to_owned(), String::from)
}

/// The toast after a copied link, carried over to the next screen.
fn link_copied() -> Toast {
    Toast::new(ToastKind::Info, LINK_COPIED, crate::toast::QUICK).carried()
}

/// The link Share copies for `card`, opened as `open`: the printing's link when `picked` over
/// the card.
fn share_link(open: &Open, card: &Card, picked: bool) -> String {
    let name = &card.scryfall_data.name;
    match (open, slug::card_slug(name)) {
        (Open::Named { .. }, Some(own)) if !picked => slug::card_url(&own),
        _ => slug::printing_url(name, card.scryfall_data.id),
    }
}
