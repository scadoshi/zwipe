//! Home: the hero's counters and their scope, the Decks panel, the Colors panel, the featured flavor card, the greeting and the first-login hint.
//!
//! Home's data outlives a visit: the client keeps the counters, the playstyle summary, the flavor card and the scope above its screens, so a return to Home draws them at once. The client hands what it holds to [`Action::Load`] as [`Cached`], the model asks for what is missing or old, and the client writes each reply to its own copy as well as handing it back. The figures come from [`crate::home`], the Colors panel's bars from [`crate::colors`] and the retailer links from [`crate::buy`].

#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;

use crate::buy;
use crate::colors::{Bar, ColorCounts};
use crate::generation::{Generation, Ticket};
use crate::home::{self, CountsTag, Figures, Scope, UserSource};
use crate::toast::Toast;

/// The screen's title.
pub const TITLE: &str = "Home";

/// The tag beside the figures once they come from this launch.
pub const LIVE: &str = "Live";

/// The hero's button that opens Activity.
pub const ACTIVITY: &str = "Activity";

/// The Decks panel's chip that opens the deck list.
pub const ALL_DECKS: &str = "All decks";

/// The Decks panel's corner chip while the user has no deck; it opens Create deck.
pub const CREATE_DECK: &str = "Create a deck";

/// The Decks panel's line while the user has no deck.
pub const NO_DECKS: &str = "Your decks show here";

/// The Decks panel's line when the decks failed to load and none are held.
pub const DECKS_FAILED: &str = "Could not load your decks";

/// How many decks the Decks panel lists, the most recently updated first.
pub const LATEST_DECKS: usize = 3;

/// The Colors panel's title.
pub const COLORS: &str = "Colors";

/// The Colors panel's chip that opens Playstyle.
pub const PLAYSTYLE: &str = "Playstyle";

/// The Colors panel's line before any deck has a card.
pub const NO_PLAY_STYLE: &str = "Your playstyle shows here";

/// The Colors panel's line when the summary failed to load and none is held.
pub const PLAY_STYLE_FAILED: &str = "Could not load your playstyle";

/// The featured flavor panel's line when the card failed to load and none is held.
pub const FLAVOR_FAILED: &str = "Could not load the featured flavor";

/// The featured flavor panel's title.
pub const FEATURED_FLAVOR: &str = "Featured flavor";

/// The flavor card's button that previews its image.
pub const IMAGE: &str = "Image";

/// The flavor card's button that opens the Buy dialog, shown only for a priced card.
pub const BUY: &str = "Buy";

/// The Buy dialog's title.
pub const BUY_TITLE: &str = "Buy card";

/// The Buy dialog's body.
pub const BUY_BODY: &str = "Buy this card from a retailer.";

/// The Buy dialog's button that closes it, before the retailer links.
pub const CLOSE: &str = "Close";

/// The Buy dialog's link to `TCGplayer`.
pub const TCGPLAYER: &str = "TCGplayer \u{2197}";

/// The Buy dialog's link to `Card Kingdom`.
pub const CARD_KINGDOM: &str = "Card Kingdom \u{2197}";

/// The Decks panel's title.
pub const DECKS: &str = "Decks";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The deck list, from the Decks panel's All decks chip.
    Decks,
    /// This deck, from its tile in the Decks panel.
    Deck(Uuid),
    /// Create deck, from the Decks panel's corner chip while the user has no deck.
    CreateDeck,
    /// Activity, from the hero.
    Activity,
    /// Playstyle, from the Colors panel.
    Playstyle,
}

/// How old a client's copy of some counters is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Age {
    /// Saved by the last launch: shown, not live, and asked for again.
    Restored,
    /// Fetched this launch under [`COUNTS_FRESH`](crate::timing::COUNTS_FRESH) ago: live, and not asked for again.
    Fresh,
    /// Fetched this launch longer ago: live, and asked for again.
    Stale,
}

/// Counters a client holds, with how old they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CachedFigures {
    /// The counters as figures.
    pub figures: Figures,
    /// How old they are.
    pub age: Age,
    /// When the server's copy was made, which an edge cache can put well
    /// before the fetch; `None` when it did not say.
    pub made_at: Option<DateTime<Utc>>,
}

/// The featured flavor card a client holds, with when the server picks the next one.
#[derive(Debug, Clone, PartialEq)]
pub struct FlavorCard {
    /// The card whose flavor text is quoted.
    pub card: Card,
    /// The top of the UTC hour after it was fetched, when it goes stale.
    pub expires_at: DateTime<Utc>,
}

impl FlavorCard {
    /// `card` as fetched at `now`, expiring at the top of the next UTC hour, when the server picks again.
    #[must_use]
    pub fn fetched(card: Card, now: DateTime<Utc>) -> Self {
        Self {
            card,
            expires_at: home::next_hour(now),
        }
    }

    /// Whether the server has picked a new card by `now`.
    #[must_use]
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now >= self.expires_at
    }
}

/// What the client holds as Home opens, from its caches above the screens.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cached {
    /// The signed-in user's id. `None` fetches nothing that needs a session.
    pub user: Option<Uuid>,
    /// The scope the user last picked.
    pub scope: Scope,
    /// Everybody's counters, if any copy is held.
    pub everybody: Option<CachedFigures>,
    /// The signed-in user's own counters, if a copy of theirs is held. Another user's copy is left out.
    pub mine: Option<CachedFigures>,
    /// The signed-in user's playstyle summary, if a copy of theirs is held. Another user's copy is left out.
    pub play_style: Option<HttpPlayStyleSummary>,
    /// The featured flavor card, if one is held, expired or not.
    pub flavor: Option<FlavorCard>,
    /// The signed-in user's decks, if a copy of theirs is held, newest first. Another user's copy is left out.
    pub decks: Option<Vec<DeckTile>>,
    /// The time Home opens, for the flavor card's expiry.
    pub now: DateTime<Utc>,
}

/// What a user does on Home, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Opens Home with what the client holds. Every fetch starts again, and replies to an earlier visit are dropped.
    Load(Box<Cached>),
    /// The user arrived for the fetch holding `ticket`. The client has already stored them in its session.
    UserLoaded {
        /// The ticket from the [`Effect::FetchUser`] this answers.
        ticket: Ticket,
        /// The user as the server has them now.
        user: Box<User>,
        /// Whether the client has said hello this launch.
        greeted: bool,
    },
    /// The user fetch holding `ticket` failed.
    UserFailed {
        /// The ticket from the [`Effect::FetchUser`] this answers.
        ticket: Ticket,
        /// The user the client's session holds, if any, to greet instead.
        cached: Option<Box<User>>,
        /// Whether the client has said hello this launch.
        greeted: bool,
    },
    /// Everybody's counters arrived for the fetch holding `ticket`. A failed fetch keeps the copy on screen, so the client sends nothing.
    CountsLoaded {
        /// The ticket from the [`Effect::FetchCounts`] this answers.
        ticket: Ticket,
        /// Everybody's counters.
        figures: Figures,
        /// When the server's copy was made, `None` when it did not say.
        made_at: Option<DateTime<Utc>>,
    },
    /// The user's own counters arrived for the fetch holding `ticket`. A failed fetch keeps the figures waiting, so the client sends nothing.
    MyCountsLoaded {
        /// The ticket from the [`Effect::FetchMyCounts`] this answers.
        ticket: Ticket,
        /// The user's own counters.
        figures: Figures,
    },
    /// The playstyle summary arrived for the fetch holding `ticket`.
    PlayStyleLoaded {
        /// The ticket from the [`Effect::FetchPlayStyle`] this answers.
        ticket: Ticket,
        /// The summary.
        summary: HttpPlayStyleSummary,
    },
    /// The playstyle fetch holding `ticket` failed. A summary already held stays on screen.
    PlayStyleFailed {
        /// The ticket from the [`Effect::FetchPlayStyle`] this answers.
        ticket: Ticket,
    },
    /// The decks arrived for the fetch holding `ticket`, newest first as the server lists them.
    DecksLoaded {
        /// The ticket from the [`Effect::FetchDecks`] this answers.
        ticket: Ticket,
        /// Every deck of the user's.
        decks: Vec<DeckTile>,
    },
    /// The deck fetch holding `ticket` failed. Decks already held stay on screen.
    DecksFailed {
        /// The ticket from the [`Effect::FetchDecks`] this answers.
        ticket: Ticket,
    },
    /// Fetches the flavor card again if it has expired by `now` and no fetch is out. The app and the web send it each time the session refreshes.
    CheckFlavor {
        /// The time now.
        now: DateTime<Utc>,
    },
    /// The flavor card arrived for the fetch holding `ticket`. The client has already stored it.
    FlavorLoaded {
        /// The ticket from the [`Effect::FetchFlavor`] this answers.
        ticket: Ticket,
        /// The card with its expiry.
        flavor: Box<FlavorCard>,
    },
    /// The flavor fetch holding `ticket` failed. The card already held stays.
    FlavorFailed {
        /// The ticket from the [`Effect::FetchFlavor`] this answers.
        ticket: Ticket,
    },
    /// Fetches the decks again after a failed load, from the panel's Retry.
    RetryDecks,
    /// Fetches the playstyle summary again after a failed load, from the panel's Retry.
    RetryPlayStyle,
    /// Fetches the flavor card again after a failed load, from the panel's Retry.
    RetryFlavor,
    /// Switches the hero between everybody's counters and the user's own.
    ToggleScope {
        /// Whether the client holds the user's own counters from under [`COUNTS_FRESH`](crate::timing::COUNTS_FRESH) ago, so switching to them asks for none.
        mine_fresh: bool,
    },
    /// Opens the deck list, from the Decks panel's All decks chip.
    Decks,
    /// Opens this deck, from its tile in the Decks panel.
    OpenDeck(Uuid),
    /// Opens Create deck, from the Decks panel's corner chip while the user has no deck.
    CreateDeck,
    /// Opens Activity.
    Activity,
    /// Opens Playstyle.
    Playstyle,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Fetches the user, stores them in the session, and answers with [`Action::UserLoaded`] or [`Action::UserFailed`] carrying this ticket. A failure only logs.
    FetchUser(Ticket),
    /// Fetches the settings and applies the theme and pip style. Nothing answers; a failure only logs.
    FetchSettings,
    /// Fetches everybody's counters, stores them, and answers with [`Action::CountsLoaded`] carrying this ticket. A failure only logs.
    FetchCounts(Ticket),
    /// Fetches the user's own counters, stores them, and answers with [`Action::MyCountsLoaded`] carrying this ticket. A failure is reported without a toast.
    FetchMyCounts(Ticket),
    /// Fetches the playstyle summary, stores it, and answers with [`Action::PlayStyleLoaded`] or [`Action::PlayStyleFailed`] carrying this ticket. A failure is reported without a toast.
    FetchPlayStyle(Ticket),
    /// Fetches the user's decks, stores them, and answers with [`Action::DecksLoaded`] or [`Action::DecksFailed`] carrying this ticket. A failure is reported without a toast.
    FetchDecks(Ticket),
    /// Fetches the featured flavor card, stores it, and answers with [`Action::FlavorLoaded`] or [`Action::FlavorFailed`] carrying this ticket; a failure with no card held shows [`FLAVOR_FAILED`] and Retry.
    FetchFlavor(Ticket),
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Saves the scope for the next launch and for the client's copy.
    SaveScope(Scope),
    /// Sets the client's once-per-launch greeted flag.
    MarkGreeted,
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// One deck in the Decks panel: a tile with the art over the name and format chips.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckTile {
    /// The deck's id, for the route its tile opens.
    pub id: Uuid,
    /// The deck's name.
    pub name: String,
    /// The deck's format, the muted chip under the art when set.
    pub format: Option<Format>,
    /// The deck's color identity in WUBRG order, the pips in the squircle without art; empty for a colorless deck.
    pub color_identity: Vec<Color>,
    /// The commander's art crop in the tile's squircle, when the deck has a commander with one; the deck's pips stand in without one.
    pub art_url: Option<String>,
}

impl From<&DeckProfile> for DeckTile {
    /// Reads the profile's color codes in any case and order; a code that is no color is dropped.
    fn from(profile: &DeckProfile) -> Self {
        let colors: Vec<Color> = profile
            .color_identity
            .iter()
            .filter_map(|code| Color::try_from(code.as_str()).ok())
            .collect();
        Self {
            id: profile.id,
            name: profile.name.to_string(),
            format: profile.format,
            color_identity: Color::all()
                .into_iter()
                .filter(|color| colors.contains(color))
                .collect(),
            art_url: profile.commander_art_url.clone(),
        }
    }
}

/// What the Decks panel draws under its header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckPanel {
    /// The deck count in the header's chip: every deck of the user's, the listed ones included.
    pub count: u32,
    /// The [`LATEST_DECKS`] most recently updated decks, newest first. Never empty: no deck is [`Decks::Empty`].
    pub latest: Vec<DeckTile>,
}

/// The chip in the Decks panel's right corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecksAction {
    /// [`ALL_DECKS`], opening the deck list.
    AllDecks,
    /// [`CREATE_DECK`], opening Create deck.
    CreateDeck,
}

/// What the Decks panel draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decks {
    /// The decks are out and none are held: three ghost tiles.
    Loading,
    /// The fetch failed and none are held: [`DECKS_FAILED`] under the header.
    Failed,
    /// The user has no deck: [`NO_DECKS`] under the header, no count chip, and [`CREATE_DECK`] in the corner.
    Empty,
    /// The count and the latest decks.
    Loaded(DeckPanel),
}

impl Decks {
    /// The chip in the header's corner: Create a deck with no deck, All decks otherwise.
    #[must_use]
    pub fn action(&self) -> DecksAction {
        match self {
            Self::Empty => DecksAction::CreateDeck,
            Self::Loading | Self::Failed | Self::Loaded(_) => DecksAction::AllDecks,
        }
    }
}

/// What the Colors panel draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayStyle {
    /// The summary is out and none is held: the panel ghosts.
    Loading,
    /// The fetch failed and none is held: [`PLAY_STYLE_FAILED`] under the header.
    Failed,
    /// No deck has a colored or colorless card: [`NO_PLAY_STYLE`] under the header.
    Empty,
    /// A bar per color then colorless, each with its count, the largest at full width.
    Bars(Vec<Bar>),
}

/// The featured flavor card's quote and the row under it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Quote {
    /// The flavor text.
    pub text: String,
    /// The card's name.
    pub name: String,
    /// The card's USD price, formatted. The Buy button shows only with one.
    pub price: Option<String>,
}

/// What the featured flavor panel draws.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Flavor {
    /// No card is held yet: the panel ghosts.
    Loading,
    /// The card has no flavor text: the panel stays empty.
    Blank,
    /// The quote, then the name, price, Image and Buy row.
    Quote(Quote),
    /// No card is held and the fetch failed: the panel says so and offers Retry.
    Failed,
}

/// A retailer link in the Buy dialog.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuyLink {
    /// The link's text.
    pub label: &'static str,
    /// Where it opens.
    pub url: String,
}

/// One generation per fetch, so a reply only competes with its own kind.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Generations {
    user: Generation,
    counts: Generation,
    my_counts: Generation,
    play_style: Generation,
    flavor: Generation,
    decks: Generation,
}

/// Where the flavor card's fetch stands.
/// A panel with a Retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    /// Your decks.
    Decks,
    /// The playstyle summary.
    PlayStyle,
    /// The featured flavor card.
    Flavor,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum FlavorFetch {
    /// None is out and the last one, if any, did not fail.
    #[default]
    Idle,
    /// One is out.
    Out,
    /// The last one failed.
    Failed,
}

/// Home for one visit. Starts with nothing held and nothing out; send [`Action::Load`] with what the client holds.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    user: Option<Uuid>,
    scope: Scope,
    everybody: Option<CachedFigures>,
    mine: Option<Figures>,
    play_style: Option<HttpPlayStyleSummary>,
    play_style_failed: bool,
    flavor: Option<FlavorCard>,
    flavor_fetch: FlavorFetch,
    decks: Option<Vec<DeckTile>>,
    decks_failed: bool,
    generations: Generations,
}

/// What the featured flavor panel draws for `card`: any flavor text quotes, even a blank one, as in the app; a card without one is `Blank`.
#[must_use]
pub fn quote(card: &Card) -> Flavor {
    let data = &card.scryfall_data;
    let Some(text) = data.flavor_text.clone() else {
        return Flavor::Blank;
    };
    Flavor::Quote(Quote {
        text,
        name: data.name.clone(),
        price: deck_metrics::card_price(data, PriceCurrency::Usd)
            .map(|p| PriceCurrency::Usd.format_amount(p)),
    })
}

/// The Buy dialog's retailer links for `card`, `TCGplayer` then `Card Kingdom`.
#[must_use]
pub fn buy_links(card: &Card) -> Vec<BuyLink> {
    let data = &card.scryfall_data;
    vec![
        BuyLink {
            label: TCGPLAYER,
            url: buy::tcgplayer_card_url(&data.name, data.tcgplayer_id),
        },
        BuyLink {
            label: CARD_KINGDOM,
            url: buy::cardkingdom_card_url(&data.name),
        },
    ]
}

impl State {
    /// Home before [`Action::Load`]: every panel ghosts and the scope is everybody's.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// A reply whose ticket a newer fetch of its kind has replaced changes nothing. One flavor fetch is out at a time, so a [`Action::CheckFlavor`] while one is out asks for nothing.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load(cached) => return self.load(*cached),
            Action::UserLoaded {
                ticket,
                user,
                greeted,
            } => {
                if self.generations.user.is_current(ticket) {
                    return greet(Some(&user), UserSource::Fetched, greeted);
                }
            }
            Action::UserFailed {
                ticket,
                cached,
                greeted,
            } => {
                if self.generations.user.is_current(ticket) {
                    return greet(cached.as_deref(), UserSource::Cached, greeted);
                }
            }
            Action::CountsLoaded {
                ticket,
                figures,
                made_at,
            } => {
                if self.generations.counts.is_current(ticket) {
                    self.everybody = Some(CachedFigures {
                        figures,
                        age: Age::Fresh,
                        made_at,
                    });
                }
            }
            Action::MyCountsLoaded { ticket, figures } => {
                if self.generations.my_counts.is_current(ticket) {
                    self.mine = Some(figures);
                }
            }
            Action::PlayStyleLoaded { ticket, summary } => {
                if self.generations.play_style.is_current(ticket) {
                    self.play_style = Some(summary);
                    self.play_style_failed = false;
                }
            }
            Action::PlayStyleFailed { ticket } => {
                if self.generations.play_style.is_current(ticket) {
                    self.play_style_failed = true;
                }
            }
            Action::DecksLoaded { ticket, decks } => {
                if self.generations.decks.is_current(ticket) {
                    self.decks = Some(decks);
                    self.decks_failed = false;
                }
            }
            Action::DecksFailed { ticket } => {
                if self.generations.decks.is_current(ticket) {
                    self.decks_failed = true;
                }
            }
            Action::CheckFlavor { now } => return self.check_flavor(now).into_iter().collect(),
            Action::FlavorLoaded { ticket, flavor } => {
                if self.generations.flavor.is_current(ticket) {
                    self.flavor = Some(*flavor);
                    self.flavor_fetch = FlavorFetch::Idle;
                }
            }
            Action::FlavorFailed { ticket } => {
                if self.generations.flavor.is_current(ticket) {
                    self.flavor_fetch = FlavorFetch::Failed;
                }
            }
            Action::RetryDecks => return self.retry(Panel::Decks).into_iter().collect(),
            Action::RetryPlayStyle => return self.retry(Panel::PlayStyle).into_iter().collect(),
            Action::RetryFlavor => return self.retry(Panel::Flavor).into_iter().collect(),
            Action::ToggleScope { mine_fresh } => {
                self.scope = self.scope.toggled();
                let mut effects = vec![Effect::SaveScope(self.scope)];
                if !mine_fresh {
                    effects.extend(self.fetch_my_counts());
                }
                return effects;
            }
            Action::Decks => return vec![Effect::Navigate(Destination::Decks)],
            Action::OpenDeck(id) => return vec![Effect::Navigate(Destination::Deck(id))],
            Action::CreateDeck => return vec![Effect::Navigate(Destination::CreateDeck)],
            Action::Activity => return vec![Effect::Navigate(Destination::Activity)],
            Action::Playstyle => return vec![Effect::Navigate(Destination::Playstyle)],
        }
        Vec::new()
    }

    /// Takes in what the client holds and asks for what is missing or old: the user and settings with a session, everybody's counters unless fresh, the user's own when they show and are not fresh, the summary and the decks with a session, and the flavor card when none is held or it expired. Then offers the first-login hint.
    fn load(&mut self, cached: Cached) -> Vec<Effect> {
        let Cached {
            user,
            scope,
            everybody,
            mine,
            play_style,
            flavor,
            decks,
            now,
        } = cached;
        let mine_fresh = mine.is_some_and(|m| m.age == Age::Fresh);
        let everybody_fresh = everybody.is_some_and(|e| e.age == Age::Fresh);
        self.user = user;
        self.scope = scope;
        self.everybody = everybody;
        self.mine = mine.map(|m| m.figures);
        self.play_style = play_style;
        self.play_style_failed = false;
        self.flavor = flavor;
        self.flavor_fetch = FlavorFetch::Idle;
        self.decks = decks;
        self.decks_failed = false;

        let mut effects = Vec::new();
        if user.is_some() {
            effects.push(Effect::FetchUser(self.generations.user.next_ticket()));
            effects.push(Effect::FetchSettings);
        }
        if !everybody_fresh {
            effects.push(Effect::FetchCounts(self.generations.counts.next_ticket()));
        }
        if !mine_fresh {
            effects.extend(self.fetch_my_counts());
        }
        if user.is_some() {
            effects.push(Effect::FetchPlayStyle(
                self.generations.play_style.next_ticket(),
            ));
            effects.push(Effect::FetchDecks(self.generations.decks.next_ticket()));
        }
        effects.extend(self.check_flavor(now));
        effects.push(Effect::OfferHint(OneTimeHint::FirstLogin));
        effects
    }

    /// Asks for the user's own counters when they show and there is a user to ask for.
    fn fetch_my_counts(&mut self) -> Option<Effect> {
        (self.scope == Scope::OnlyMe && self.user.is_some())
            .then(|| Effect::FetchMyCounts(self.generations.my_counts.next_ticket()))
    }

    /// Asks for the flavor card when none is held or it expired by `now`, unless a fetch is out.
    fn check_flavor(&mut self, now: DateTime<Utc>) -> Option<Effect> {
        let stale = self.flavor.as_ref().is_none_or(|f| f.is_expired(now));
        if !stale || self.flavor_fetch == FlavorFetch::Out {
            return None;
        }
        Some(self.fetch_flavor())
    }

    /// Sends the flavor card's fetch.
    fn fetch_flavor(&mut self) -> Effect {
        self.flavor_fetch = FlavorFetch::Out;
        Effect::FetchFlavor(self.generations.flavor.next_ticket())
    }

    /// A panel's Retry: fetches its part again, the decks and the summary only
    /// with a user, the flavor card unless a fetch is out.
    fn retry(&mut self, panel: Panel) -> Option<Effect> {
        match panel {
            Panel::Decks if self.user.is_some() => {
                self.decks_failed = false;
                Some(Effect::FetchDecks(self.generations.decks.next_ticket()))
            }
            Panel::PlayStyle if self.user.is_some() => {
                self.play_style_failed = false;
                Some(Effect::FetchPlayStyle(
                    self.generations.play_style.next_ticket(),
                ))
            }
            Panel::Flavor if self.flavor_fetch != FlavorFetch::Out => Some(self.fetch_flavor()),
            _ => None,
        }
    }

    /// Whose counters the hero shows.
    #[must_use]
    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// The hero's three figures for the scope, each `None` while it loads.
    #[must_use]
    pub fn figures(&self) -> Figures {
        match self.scope {
            Scope::Everybody => self.everybody.map(|e| e.figures),
            Scope::OnlyMe => self.mine,
        }
        .unwrap_or_default()
    }

    /// The tag under the hero's figures at `now`: everybody's, once fetched
    /// this launch, `Live` while the server's copy is new and "as of" its day
    /// after; the user's own, never edge-cached, `Live` once any copy is held.
    /// `None` before then.
    #[must_use]
    pub fn counts_tag(&self, now: DateTime<Utc>) -> Option<CountsTag> {
        match self.scope {
            Scope::Everybody => self
                .everybody
                .filter(|e| e.age != Age::Restored)
                .map(|e| CountsTag::at(e.made_at, now)),
            Scope::OnlyMe => self.mine.map(|_| CountsTag::Live),
        }
    }

    /// Whether the hero tags its figures [`LIVE`]: everybody's once fetched this launch, the user's own once any copy is held.
    #[must_use]
    pub fn live(&self) -> bool {
        match self.scope {
            Scope::Everybody => self.everybody.is_some_and(|e| e.age != Age::Restored),
            Scope::OnlyMe => self.mine.is_some(),
        }
    }

    /// What the Decks panel draws. Decks held show even after a failed refetch. The playstyle summary's deck count is not the one shown: it counts the decks with cards.
    #[must_use]
    pub fn decks(&self) -> Decks {
        match &self.decks {
            Some(decks) if decks.is_empty() => Decks::Empty,
            Some(decks) => Decks::Loaded(DeckPanel {
                count: u32::try_from(decks.len()).unwrap_or(u32::MAX),
                latest: decks.iter().take(LATEST_DECKS).cloned().collect(),
            }),
            None if self.decks_failed => Decks::Failed,
            None => Decks::Loading,
        }
    }

    /// What the Colors panel draws. A summary held shows even after a failed refetch.
    #[must_use]
    pub fn play_style(&self) -> PlayStyle {
        match &self.play_style {
            Some(summary) => summary
                .color_map
                .as_ref()
                .map(ColorCounts::from)
                .filter(ColorCounts::has_cards)
                .filter(|_| summary.deck_count > 0)
                .map_or(PlayStyle::Empty, |counts| PlayStyle::Bars(counts.bars())),
            None if self.play_style_failed => PlayStyle::Failed,
            None => PlayStyle::Loading,
        }
    }

    /// The featured flavor card held, for its image preview.
    #[must_use]
    pub fn flavor_card(&self) -> Option<&Card> {
        self.flavor.as_ref().map(|f| &f.card)
    }

    /// What the featured flavor panel draws: [`quote`] of the card held, else `Failed` after a failed fetch, else `Loading`.
    #[must_use]
    pub fn flavor(&self) -> Flavor {
        match self.flavor_card() {
            Some(card) => quote(card),
            None if self.flavor_fetch == FlavorFetch::Failed => Flavor::Failed,
            None => Flavor::Loading,
        }
    }

    /// The Buy dialog's retailer links for the flavor card held, [`buy_links`] of it. Empty with no card.
    #[must_use]
    pub fn buy_links(&self) -> Vec<BuyLink> {
        self.flavor_card().map_or_else(Vec::new, buy_links)
    }
}

/// The greeting for `user`, if any: [`Effect::MarkGreeted`] first when it is the launch's hello, then its toast.
fn greet(user: Option<&User>, source: UserSource, greeted: bool) -> Vec<Effect> {
    let Some(greeting) = user.and_then(|u| home::greeting(u, source, greeted)) else {
        return Vec::new();
    };
    let mut effects = Vec::new();
    if greeting.uses_greeting() {
        effects.push(Effect::MarkGreeted);
    }
    effects.push(Effect::Toast(greeting.toast()));
    effects
}
