//! Activity: how the user picks, read from their swipes. The year's heatmap and Totals, the Card roles and Oracle tags rate charts over one Show pick, and the Most skipped cards.
//!
//! The client keeps a [`State`] while the screen is up and drops it on leaving. The numbers come from [`crate::activity`]; the model decides what shows, which line stands in for an empty part, and which bar is revealed. Which sections are open, the heatmap's picked day and its release after [`crate::timing::HEAT_PICK`], and every sweep stay in the client: they are layout and animation, and the app's accordion and the web's open columns differ there on purpose.

#[cfg(test)]
mod tests;

use zwipe_core::domain::card::Card;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::suggestions::{HttpActivity, HttpHardPass};

use crate::activity::{self, Bar, HeatLayout, RateKind};
use crate::generation::{Generation, Ticket};

/// The screen's title.
pub const TITLE: &str = "Activity";

/// The line in place of the sections when the activity failed to load.
pub const LOAD_FAILED: &str = "Could not load your activity";

/// The line in place of the sections when the account has nothing to show yet.
pub const NOTHING_YET: &str = "Swipe some cards to see your activity";

/// The Totals chart's title, under the heatmap.
pub const TOTALS: &str = "Totals";

/// The label on the chip row that picks the rate both charts show.
pub const SHOW: &str = "Show";

/// The Most skipped group's header.
pub const PASSES_TITLE: &str = "Most skipped";

/// The label over the commanders an open Most skipped row was skipped under.
pub const SKIPPED_UNDER: &str = "Skipped under";

/// The action bar's button back to the screen before.
pub const BACK: &str = "Back";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The screen the user came from, from Back. The app reaches Activity from Home's counters and goes back there through its history.
    Previous,
}

/// One of the screen's three sections, in the order they draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    /// The year's heatmap and Totals.
    Activity,
    /// The share per card role.
    Roles,
    /// The oracle tags whose share stands out from other players'.
    Tags,
}

impl Section {
    /// Every section, top to bottom.
    pub const ALL: [Self; 3] = [Self::Activity, Self::Roles, Self::Tags];

    /// The section's header.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Activity => TITLE,
            Self::Roles => "Card roles",
            Self::Tags => "Oracle tags",
        }
    }

    /// Whether the section opens with the screen. Only Activity does, and only when it shows.
    #[must_use]
    pub fn opens_on_entry(self) -> bool {
        self == Self::Activity
    }
}

/// Where the activity fetch stands, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The fetch is out: the sections ghost and the counters keep rolling.
    Loading,
    /// The fetch failed: [`LOAD_FAILED`].
    Failed,
    /// Nothing swiped or searched yet: [`NOTHING_YET`].
    Empty,
    /// Something to draw: the sections and Most skipped.
    Loaded,
}

/// What a rate section draws at the picked [`RateKind`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rates {
    /// Nothing to rate yet: this line alone, with no Show chips.
    Unrated(&'static str),
    /// The Show chips over this line: the pick charts nothing.
    Uncharted(&'static str),
    /// The Show chips over this chart, never empty, its counts percents.
    Charted(Vec<Bar>),
}

/// What a user does on Activity, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Fetches the activity, as the screen does when it opens. The rate and the revealed bars stay as they are.
    Load,
    /// The activity arrived for the fetch holding `ticket`.
    Loaded {
        /// The ticket from the [`Effect::FetchActivity`] this answers.
        ticket: Ticket,
        /// The user's activity.
        activity: Box<HttpActivity>,
    },
    /// The fetch holding `ticket` failed. The client has already reported it and raised its error toast.
    Failed {
        /// The ticket from the [`Effect::FetchActivity`] this answers.
        ticket: Ticket,
    },
    /// Picks the share both rate charts show.
    PickRate(RateKind),
    /// Reveals the swiped tags under the role with this key, or hides them when they show.
    ToggleRole(String),
    /// Reveals the definition of the tag with this slug, or hides it when it shows.
    ToggleTag(String),
    /// Swaps in a printing picked from a Most skipped row, on the pass whose card shares its oracle id. A card with no oracle id changes nothing.
    PickPrinting(Box<Card>),
    /// Goes back to the screen before.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches the activity and answers with [`Action::Loaded`] or [`Action::Failed`] carrying this ticket.
    FetchActivity(Ticket),
    /// Goes to this screen.
    Navigate(Destination),
}

/// The activity as the last fetch left it.
#[derive(Debug, Default, Clone, PartialEq)]
enum Load {
    #[default]
    Loading,
    Failed,
    Empty,
    /// Always has something to draw: a reply with nothing is [`Load::Empty`].
    Loaded(Box<HttpActivity>),
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// Activity for one visit. Starts loading on [`RateKind::Added`] with nothing revealed; send [`Action::Load`] to start the fetch.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    load: Load,
    rate: RateKind,
    revealed_role: Option<String>,
    revealed_tag: Option<String>,
    hint: HintOffer,
    generation: Generation,
}

impl State {
    /// The screen as it first draws: loading, Added picked, nothing revealed.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// A reply whose ticket a newer [`Action::Load`] has replaced changes nothing. The hint is offered once per visit, with the first fetch, as the screen opens.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load => {
                self.load = Load::Loading;
                let mut effects = Vec::new();
                if self.hint == HintOffer::Waiting {
                    self.hint = HintOffer::Offered;
                    effects.push(Effect::OfferHint(OneTimeHint::Activity));
                }
                effects.push(Effect::FetchActivity(self.generation.next_ticket()));
                return effects;
            }
            Action::Loaded { ticket, activity } => {
                if self.generation.is_current(ticket) {
                    self.load = if activity::has_anything(&activity) {
                        Load::Loaded(activity)
                    } else {
                        Load::Empty
                    };
                }
            }
            Action::Failed { ticket } => {
                if self.generation.is_current(ticket) {
                    self.load = Load::Failed;
                }
            }
            Action::PickRate(rate) => self.rate = rate,
            Action::ToggleRole(key) => toggle(&mut self.revealed_role, key),
            Action::ToggleTag(slug) => toggle(&mut self.revealed_tag, slug),
            Action::PickPrinting(card) => self.swap_printing(*card),
            Action::Back => return vec![Effect::Navigate(Destination::Previous)],
        }
        Vec::new()
    }

    /// Puts `card` on the pass whose card shares its oracle id.
    fn swap_printing(&mut self, card: Card) {
        let Load::Loaded(found) = &mut self.load else {
            return;
        };
        let target = card.scryfall_data.oracle_id;
        if target.is_none() {
            return;
        }
        if let Some(pass) = found
            .passes
            .iter_mut()
            .find(|p| p.card.scryfall_data.oracle_id == target)
        {
            pass.card = card;
        }
    }

    /// Where the fetch stands.
    #[must_use]
    pub fn status(&self) -> Status {
        match self.load {
            Load::Loading => Status::Loading,
            Load::Failed => Status::Failed,
            Load::Empty => Status::Empty,
            Load::Loaded(_) => Status::Loaded,
        }
    }

    /// The activity as the client handed it in, once it has something to draw.
    #[must_use]
    pub fn activity(&self) -> Option<&HttpActivity> {
        match &self.load {
            Load::Loaded(found) => Some(found),
            Load::Loading | Load::Failed | Load::Empty => None,
        }
    }

    /// Whether the user's counters may stop rolling: once the fetch settles either way, so they resolve with the rest of the screen.
    #[must_use]
    pub fn counts_ready(&self) -> bool {
        self.status() != Status::Loading
    }

    /// The line in place of the sections: [`LOAD_FAILED`] or [`NOTHING_YET`]. `None` while loading or once there is something to draw.
    #[must_use]
    pub fn message(&self) -> Option<&'static str> {
        match self.load {
            Load::Failed => Some(LOAD_FAILED),
            Load::Empty => Some(NOTHING_YET),
            Load::Loading | Load::Loaded(_) => None,
        }
    }

    /// The sections that draw, in order. Activity is left out with no day and no outcome; Card roles and Oracle tags stay, with a line saying what fills them. Empty until the activity loads.
    #[must_use]
    pub fn sections(&self) -> Vec<Section> {
        if self.activity().is_none() {
            return Vec::new();
        }
        Section::ALL
            .into_iter()
            .filter(|&section| section != Section::Activity || self.shows_year())
            .collect()
    }

    /// Whether the Activity section draws: there is a day for the heatmap or an outcome for Totals.
    fn shows_year(&self) -> bool {
        self.shows_heatmap() || self.shows_totals()
    }

    /// Whether the heatmap draws: some day had a swipe or a search.
    #[must_use]
    pub fn shows_heatmap(&self) -> bool {
        self.activity().is_some_and(|a| !a.days.is_empty())
    }

    /// Whether Totals draws: some swipe came to something.
    #[must_use]
    pub fn shows_totals(&self) -> bool {
        self.activity()
            .is_some_and(|a| activity::has_outcomes(&a.outcomes))
    }

    /// The year laid out as the heatmap's grid, while [`Self::shows_heatmap`].
    #[must_use]
    pub fn heat(&self) -> Option<HeatLayout> {
        self.activity()
            .filter(|a| !a.days.is_empty())
            .map(|a| activity::heat_layout(a.through, &a.days))
    }

    /// The heatmap's resting tags: the year's swipes, its searches and "Last year".
    #[must_use]
    pub fn year_tags(&self) -> [String; 3] {
        activity::year_tags(self.activity().map_or(&[], |a| &a.days))
    }

    /// The heatmap's year in words, for screen readers.
    #[must_use]
    pub fn year_line(&self) -> String {
        activity::year_line(self.activity().map_or(&[], |a| &a.days))
    }

    /// Totals' four columns, while [`Self::shows_totals`]; empty otherwise.
    #[must_use]
    pub fn totals(&self) -> Vec<Bar> {
        self.activity()
            .filter(|a| activity::has_outcomes(&a.outcomes))
            .map(|a| activity::outcome_bars(&a.outcomes))
            .unwrap_or_default()
    }

    /// The share both rate charts show.
    #[must_use]
    pub fn rate(&self) -> RateKind {
        self.rate
    }

    /// Card roles at the picked rate. With roles to rate, only Removed can chart none: every role sent has a dealt card, but Removed needs an added one.
    #[must_use]
    pub fn role_rates(&self) -> Rates {
        let Some(found) = self.activity().filter(|a| !a.roles.is_empty()) else {
            return Rates::Unrated(activity::ROLES_EMPTY);
        };
        let bars = activity::role_rate_bars(self.rate, &found.roles, &found.tags);
        if bars.is_empty() {
            Rates::Uncharted(activity::NONE_REMOVED)
        } else {
            Rates::Charted(bars)
        }
    }

    /// Oracle tags at the picked rate. A pick that charts no tag says nothing was removed on Removed with no removal, else that no tag stands out.
    #[must_use]
    pub fn tag_rates(&self) -> Rates {
        let Some(found) = self.activity().filter(|a| !a.tags.is_empty()) else {
            return Rates::Unrated(activity::TAGS_EMPTY);
        };
        let bars = activity::tag_rate_bars(self.rate, &found.tags);
        if bars.is_empty() {
            Rates::Uncharted(activity::tag_empty_line(self.rate, &found.tags))
        } else {
            Rates::Charted(bars)
        }
    }

    /// The role whose tags are revealed, by key.
    #[must_use]
    pub fn revealed_role(&self) -> Option<&str> {
        self.revealed_role.as_deref()
    }

    /// The tag whose definition is revealed, by slug.
    #[must_use]
    pub fn revealed_tag(&self) -> Option<&str> {
        self.revealed_tag.as_deref()
    }

    /// The user's swiped tags under `role`, in the server's order: what a revealed role shows.
    #[must_use]
    pub fn tags_under(&self, role: &str) -> Vec<String> {
        self.activity()
            .map(|a| activity::tags_under(role, &a.tags))
            .unwrap_or_default()
    }

    /// The cards the user keeps passing on, most skipped against other players first. Empty until the activity loads.
    #[must_use]
    pub fn passes(&self) -> &[HttpHardPass] {
        self.activity().map_or(&[], |a| &a.passes)
    }

    /// The line under Most skipped's header with no pass to list.
    #[must_use]
    pub fn passes_line(&self) -> Option<&'static str> {
        self.passes().is_empty().then_some(activity::PASSES_EMPTY)
    }
}

/// A pass's tag where a deck list puts copies: "1 skip", "12 skips".
#[must_use]
pub fn skips(pass: &HttpHardPass) -> String {
    activity::counted(pass.skipped, "skip", "skips")
}

/// Sets `slot` to `key`, or clears it when it already holds `key`.
fn toggle(slot: &mut Option<String>, key: String) {
    if slot.as_deref() == Some(key.as_str()) {
        *slot = None;
    } else {
        *slot = Some(key);
    }
}
