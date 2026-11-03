//! Home: the hero's counters and their scope, the Color map panel, the featured flavor card, the greeting and the first-login hint.
//!
//! Home's data outlives a visit: the client keeps the counters, the playstyle summary, the flavor card and the scope above its screens, so a return to Home draws them at once. The client hands what it holds to [`Action::Load`] as [`Cached`], the model asks for what is missing or old, and the client writes each reply to its own copy as well as handing it back. The figures come from [`crate::home`], the color map test from [`crate::playstyle::color_map_has_data`] and the retailer links from [`crate::buy`].

#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;

use crate::color_map::{self, ColorMapCounts};
use crate::generation::{Generation, Ticket};
use crate::home::{self, Figures, Scope, UserSource};
use crate::toast::Toast;
use crate::{buy, playstyle};

/// The screen's title.
pub const TITLE: &str = "Home";

/// The tag beside the figures once they come from this launch.
pub const LIVE: &str = "Live";

/// The hero's button that opens Activity.
pub const ACTIVITY: &str = "Activity";

/// The Color map panel's title.
pub const COLOR_MAP: &str = "Color map";

/// The Color map panel's button that opens Playstyle.
pub const PLAYSTYLE: &str = "Playstyle";

/// The Color map panel's line before any deck has a colored card.
pub const NO_PLAY_STYLE: &str = "Build a deck to see your playstyle";

/// The Color map panel's line when the summary failed to load and none is held.
pub const PLAY_STYLE_FAILED: &str = "Could not load your playstyle";

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

/// The action bar's button that opens the deck list.
pub const DECKS: &str = "Decks";

/// The action bar's button that opens Account.
pub const ACCOUNT: &str = "Account";

/// The action bar's button that opens Config.
pub const CONFIG: &str = "Config";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The deck list, from the action bar.
    Decks,
    /// Account, from the action bar.
    Account,
    /// Config, from the action bar.
    Config,
    /// Activity, from the hero.
    Activity,
    /// Playstyle, from the Color map panel.
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
    /// Switches the hero between everybody's counters and the user's own.
    ToggleScope {
        /// Whether the client holds the user's own counters from under [`COUNTS_FRESH`](crate::timing::COUNTS_FRESH) ago, so switching to them asks for none.
        mine_fresh: bool,
    },
    /// Opens the deck list.
    Decks,
    /// Opens Account.
    Account,
    /// Opens Config.
    Config,
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
    /// Fetches the preferences and applies the theme and pip style. Nothing answers; a failure only logs.
    FetchPreferences,
    /// Fetches everybody's counters, stores them, and answers with [`Action::CountsLoaded`] carrying this ticket. A failure only logs.
    FetchCounts(Ticket),
    /// Fetches the user's own counters, stores them, and answers with [`Action::MyCountsLoaded`] carrying this ticket. A failure is reported without a toast.
    FetchMyCounts(Ticket),
    /// Fetches the playstyle summary, stores it, and answers with [`Action::PlayStyleLoaded`] or [`Action::PlayStyleFailed`] carrying this ticket. A failure is reported without a toast.
    FetchPlayStyle(Ticket),
    /// Fetches the featured flavor card, stores it, and answers with [`Action::FlavorLoaded`] or [`Action::FlavorFailed`] carrying this ticket. A failure only logs.
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

/// What the Color map panel draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayStyle {
    /// The summary is out and none is held: the panel ghosts.
    Loading,
    /// The fetch failed and none is held: [`PLAY_STYLE_FAILED`] over the Playstyle button.
    Failed,
    /// No deck has a colored or colorless card: [`NO_PLAY_STYLE`] over the Playstyle button.
    Empty,
    /// The map for these counts, with the Playstyle button at the end of its key.
    Map(ColorMapCounts),
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
    flavor_out: bool,
    generations: Generations,
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
            Action::CountsLoaded { ticket, figures } => {
                if self.generations.counts.is_current(ticket) {
                    self.everybody = Some(CachedFigures {
                        figures,
                        age: Age::Fresh,
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
            Action::CheckFlavor { now } => return self.check_flavor(now).into_iter().collect(),
            Action::FlavorLoaded { ticket, flavor } => {
                if self.generations.flavor.is_current(ticket) {
                    self.flavor = Some(*flavor);
                    self.flavor_out = false;
                }
            }
            Action::FlavorFailed { ticket } => {
                if self.generations.flavor.is_current(ticket) {
                    self.flavor_out = false;
                }
            }
            Action::ToggleScope { mine_fresh } => {
                self.scope = self.scope.toggled();
                let mut effects = vec![Effect::SaveScope(self.scope)];
                if !mine_fresh {
                    effects.extend(self.fetch_my_counts());
                }
                return effects;
            }
            Action::Decks => return vec![Effect::Navigate(Destination::Decks)],
            Action::Account => return vec![Effect::Navigate(Destination::Account)],
            Action::Config => return vec![Effect::Navigate(Destination::Config)],
            Action::Activity => return vec![Effect::Navigate(Destination::Activity)],
            Action::Playstyle => return vec![Effect::Navigate(Destination::Playstyle)],
        }
        Vec::new()
    }

    /// Takes in what the client holds and asks for what is missing or old: the user and preferences with a session, everybody's counters unless fresh, the user's own when they show and are not fresh, the summary with a session, and the flavor card when none is held or it expired. Then offers the first-login hint.
    fn load(&mut self, cached: Cached) -> Vec<Effect> {
        let Cached {
            user,
            scope,
            everybody,
            mine,
            play_style,
            flavor,
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
        self.flavor_out = false;

        let mut effects = Vec::new();
        if user.is_some() {
            effects.push(Effect::FetchUser(self.generations.user.next_ticket()));
            effects.push(Effect::FetchPreferences);
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
        if !stale || self.flavor_out {
            return None;
        }
        self.flavor_out = true;
        Some(Effect::FetchFlavor(self.generations.flavor.next_ticket()))
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

    /// Whether the hero tags its figures [`LIVE`]: everybody's once fetched this launch, the user's own once any copy is held.
    #[must_use]
    pub fn live(&self) -> bool {
        match self.scope {
            Scope::Everybody => self.everybody.is_some_and(|e| e.age != Age::Restored),
            Scope::OnlyMe => self.mine.is_some(),
        }
    }

    /// The color map's seed, from the user's id so Home draws the shape Playstyle does. 0 with no user.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.user
            .map_or(0, |id| color_map::seed_from_id(id.as_u128()))
    }

    /// What the Color map panel draws. A summary held shows even after a failed refetch.
    #[must_use]
    pub fn play_style(&self) -> PlayStyle {
        match &self.play_style {
            Some(summary) => summary
                .color_map
                .as_ref()
                .map(ColorMapCounts::from)
                .filter(playstyle::color_map_has_data)
                .filter(|_| summary.deck_count > 0)
                .map_or(PlayStyle::Empty, PlayStyle::Map),
            None if self.play_style_failed => PlayStyle::Failed,
            None => PlayStyle::Loading,
        }
    }

    /// The featured flavor card held, for its image preview.
    #[must_use]
    pub fn flavor_card(&self) -> Option<&Card> {
        self.flavor.as_ref().map(|f| &f.card)
    }

    /// What the featured flavor panel draws. Any flavor text quotes, even a blank one, as in the app.
    #[must_use]
    pub fn flavor(&self) -> Flavor {
        let Some(card) = self.flavor_card() else {
            return Flavor::Loading;
        };
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

    /// The Buy dialog's retailer links for the flavor card held, `TCGplayer` then `Card Kingdom`. Empty with no card.
    #[must_use]
    pub fn buy_links(&self) -> Vec<BuyLink> {
        let Some(card) = self.flavor_card() else {
            return Vec::new();
        };
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
