//! The card screen: one card opened by its name's slug (`/card/<slug>`) or one printing by its id
//! (`/card/<slug>/<id>`), signed in or out.
//!
//! A name opens the card when one playable card has it, a short list when several do (each to
//! its own printing link), and [`NOT_FOUND`] when none does; a token or art card that shares a
//! playable card's name stays out of the count, and a card whose full name it is wins over cards
//! it only names the front face of. A printing link loads by its id alone, and a
//! stale slug in it is swapped for the card's own. Send to signed out goes to Login and comes
//! back to the card with the sheet asked for; the screen opens it once and drops the ask from
//! the route, so Back or a reload never opens it again; on a list, the ask goes with the card
//! picked from it. Share copies the card's link: the name's
//! link for a card opened by name, the printing's link for a printing, which is where a name
//! several cards share leads.
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

use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// The line when no card has the name.
pub const NOT_FOUND: &str = "No card by that name";

/// The toast once the card's link is copied.
pub const LINK_COPIED: &str = "Share link copied";

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
    },
    /// One printing, asking for Send to when `send`.
    Printing {
        /// The slug segment ([`slug::printing_path`]'s).
        slug: String,
        /// The printing's Scryfall id.
        id: Uuid,
        /// Whether the screen opens Send to once the card shows.
        send: bool,
    },
    /// Login, for Send to signed out.
    Login,
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
    /// Opens Send to for this card.
    OpenSendTo(Box<Card>),
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

/// The card screen's state. [`State::new`] is the screen before [`Action::Load`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct State {
    open: Option<Open>,
    shown: Shown,
    /// Whether Send to opens once the card shows.
    send_pending: bool,
    fetch: Generation,
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
            } => self.load(open, send, signed_in),
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
                self.send_if_asked()
            }
            Action::Printing { ticket, card } => {
                if !self.fetch.is_current(ticket) {
                    return Vec::new();
                }
                let id = card.scryfall_data.id;
                let own = own_printing_slug(&card);
                self.shown = Shown::One(card);
                let mut effects = Vec::new();
                if let Some(Open::Printing { slug, .. }) = &self.open
                    && *slug != own
                {
                    effects.push(Effect::Replace(Destination::Printing {
                        slug: own,
                        id,
                        send: false,
                    }));
                }
                effects.extend(self.send_if_asked());
                effects
            }
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
                vec![Effect::Navigate(Destination::Printing { slug, id, send })]
            }
            Action::SendTo { signed_in } => {
                let Some(card) = self.card().cloned() else {
                    return Vec::new();
                };
                if signed_in {
                    return vec![Effect::OpenSendTo(Box::new(card))];
                }
                let Some(back) = self.route(/* send */ true) else {
                    return Vec::new();
                };
                vec![
                    Effect::RememberReturn(back),
                    Effect::Navigate(Destination::Login),
                ]
            }
            Action::Share => match (&self.open, self.card()) {
                (Some(open), Some(card)) => vec![Effect::Copy {
                    text: share_link(open, card),
                    done: link_copied(),
                }],
                _ => Vec::new(),
            },
        }
    }

    /// Opens the screen at `open`: drops a Send to ask from the route at once, keeping it for
    /// when the card shows if a session exists, then asks for what the route names.
    fn load(&mut self, open: Open, send: bool, signed_in: bool) -> Vec<Effect> {
        // The route the screen is on, without an ask, is the screen's own Replace dropping the
        // ask: the card is already loading or shown, and an asked-for Send to still waits.
        if !send && self.open.as_ref() == Some(&open) {
            return Vec::new();
        }
        self.open = Some(open);
        self.send_pending = send && signed_in;
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

    /// Opens the asked-for Send to, once, when one card shows. On a list the ask waits for the
    /// pick ([`Action::Pick`]), which carries it to that card's printing.
    fn send_if_asked(&mut self) -> Vec<Effect> {
        if !self.send_pending || matches!(self.shown, Shown::Loading | Shown::Several(_)) {
            return Vec::new();
        }
        self.send_pending = false;
        let Some(card) = self.card().cloned() else {
            return Vec::new();
        };
        vec![Effect::OpenSendTo(Box::new(card))]
    }

    /// The route the screen is on, asking for Send to when `send`. A name that isn't a slug has
    /// no route to come back to.
    fn route(&self, send: bool) -> Option<Destination> {
        match self.open.as_ref()? {
            Open::Named { slug } => Some(Destination::Named {
                slug: slug.parse().ok()?,
                send,
            }),
            Open::Printing { slug, id } => Some(Destination::Printing {
                slug: slug.clone(),
                id: *id,
                send,
            }),
        }
    }

    /// What the screen shows.
    #[must_use]
    pub fn status(&self) -> Status {
        match self.shown {
            Shown::Loading => Status::Loading,
            Shown::One(_) => Status::One,
            Shown::Several(_) => Status::Several,
            Shown::NotFound => Status::NotFound,
            Shown::Failed => Status::Failed,
        }
    }

    /// The card, when one shows.
    #[must_use]
    pub fn card(&self) -> Option<&Card> {
        match &self.shown {
            Shown::One(card) => Some(card),
            Shown::Loading | Shown::Several(_) | Shown::NotFound | Shown::Failed => None,
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

/// The slug segment of `card`'s printing link ([`slug::printing_path`]'s).
fn own_printing_slug(card: &Card) -> String {
    slug::card_slug(&card.scryfall_data.name)
        .map_or_else(|| slug::PRINTING_SLUG_FALLBACK.to_owned(), String::from)
}

/// The toast after a copied link.
fn link_copied() -> Toast {
    Toast::new(ToastKind::Info, LINK_COPIED, crate::toast::QUICK)
}

/// The link Share copies for `card`, opened as `open`.
fn share_link(open: &Open, card: &Card) -> String {
    let name = &card.scryfall_data.name;
    match (open, slug::card_slug(name)) {
        (Open::Named { .. }, Some(own)) => slug::card_url(&own),
        _ => slug::printing_url(name, card.scryfall_data.id),
    }
}
