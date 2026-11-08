//! The oracle tag dictionary: every oracle tag and its description, browsed by letter or searched, with Examples and Use on each row. It opens over the deck form's oracle tag picker or the card filter, the screen that wants a tag.
//!
//! The tags are the client's app-wide catalog, which outlives a visit and refreshes on its own, so the model holds no copy: the client passes what its catalog holds into [`State::body`] on each draw. The model holds the search and the letter, decides the rows and the line in place of them, and says where Examples and Use lead. The letter rail's scrolling, the overlay or panel layout and the row drawing stay in the client.

#[cfg(test)]
mod tests;

use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::user::hints::OneTimeHint;

use crate::dictionary;
use crate::toast::{Toast, ToastKind};

/// The screen's title, also the hint's.
pub const TITLE: &str = "Oracle tags";

/// The search box's placeholder.
pub const SEARCH_PLACEHOLDER: &str = "Search tags or descriptions";

/// The line in place of the dictionary when the catalog failed with nothing cached.
pub const LOAD_FAILED: &str = "Couldn't load the dictionary. Close it and try again.";

/// The toast raised when the catalog fails with nothing cached while the dictionary is up.
pub const LOAD_FAILED_TOAST: &str = "Couldn't load the Oracle tags dictionary";

/// A row's stand-in for a tag with no description.
pub const NO_DESCRIPTION: &str = "No description yet";

/// A row's chip that opens the tag's example cards.
pub const EXAMPLES: &str = "Examples";

/// A row's chip that hands the tag to the screen that opened the dictionary.
pub const USE: &str = "Use";

/// The Back button, to the screen that opened the dictionary.
pub const BACK: &str = "Back";

/// The line when a search matches no tag: `No tags match “{query}”.`, the query trimmed and lowercased.
#[must_use]
pub fn no_match(query: &str) -> String {
    format!("No tags match \u{201c}{query}\u{201d}.")
}

/// The line when no tag files under the letter being browsed: `No tags start with R.`
#[must_use]
pub fn no_letter(letter: char) -> String {
    format!("No tags start with {}.", letter.to_ascii_uppercase())
}

/// A rail chip's label: the letter uppercased, or [`dictionary::OTHER`] as it is.
#[must_use]
pub fn letter_label(letter: char) -> String {
    if letter == dictionary::OTHER {
        dictionary::OTHER.to_string()
    } else {
        letter.to_ascii_uppercase().to_string()
    }
}

/// Where an action sends the user. The client maps each to its own route or overlay.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The screen that opened the dictionary, from Back.
    Previous,
    /// The example cards for the tag with this slug, over the dictionary.
    Examples(String),
}

/// What the client's oracle tag catalog holds as it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Catalog<'tags> {
    /// No tags yet; the fetch is out.
    Loading,
    /// The fetch failed and nothing is cached.
    Failed,
    /// The tags, fresh or stale.
    Loaded(&'tags [OracleTag]),
}

/// What the dictionary draws below its header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    /// The catalog is loading: the controls and rows ghost.
    Loading,
    /// The catalog failed: [`LOAD_FAILED`] alone, with no controls.
    Failed(&'static str),
    /// The search box, the letter rail and the list under them.
    Browse {
        /// The rail's chips, `a` to `z` then [`dictionary::OTHER`] when some tag needs it.
        rail: Vec<RailLetter>,
        /// The rows, or the line in place of them.
        list: List,
    },
}

/// One chip on the letter rail.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RailLetter {
    /// The letter it browses, lowercase, or [`dictionary::OTHER`]; what [`Action::PickLetter`] takes.
    pub letter: char,
    /// Its label, from [`letter_label`].
    pub label: String,
    /// Whether it draws picked: it is the letter being browsed and no search is on.
    pub picked: bool,
}

/// The list under the controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum List {
    /// No tag to list: [`no_match`] while searching, else [`no_letter`].
    Empty(String),
    /// The tags to list, never empty: the search's ranked matches or the letter's tags in slug order.
    Rows(Vec<OracleTag>),
}

/// What a user does on the dictionary, and what the client reports about its catalog.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    /// Opens the dictionary: offers its hint once per visit and makes sure the catalog is warm.
    Load,
    /// The client's catalog settled as failed with nothing cached while the dictionary is up. Sent each time it does.
    CatalogFailed,
    /// The search box's text changed to this.
    Search(String),
    /// Clears the search and browses this letter, lowercase, or [`dictionary::OTHER`].
    PickLetter(char),
    /// Opens the example cards for the tag with this slug.
    OpenExamples(String),
    /// Hands the tag with this slug to the screen that opened the dictionary. The dictionary stays up.
    Use(String),
    /// Goes back to the screen that opened the dictionary.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches the oracle tag catalog unless it is fresh or a fetch is out. The client reports a failure with nothing cached as [`Action::CatalogFailed`].
    EnsureCatalog,
    /// Hands the tag with this slug to the screen that opened the dictionary, which adopts it and says so.
    AdoptTag(String),
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// The dictionary for one visit: browsing `a` with an empty search. Send [`Action::Load`] as it opens.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct State {
    query: String,
    letter: char,
    hint: HintOffer,
}

impl Default for State {
    fn default() -> Self {
        Self {
            query: String::new(),
            letter: 'a',
            hint: HintOffer::Waiting,
        }
    }
}

impl State {
    /// The dictionary as it first draws: browsing `a`, nothing searched.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load => {
                let mut effects = Vec::new();
                if self.hint == HintOffer::Waiting {
                    self.hint = HintOffer::Offered;
                    effects.push(Effect::OfferHint(OneTimeHint::OracleTagDictionary));
                }
                effects.push(Effect::EnsureCatalog);
                effects
            }
            Action::CatalogFailed => {
                vec![Effect::Toast(Toast::normal(
                    ToastKind::Error,
                    LOAD_FAILED_TOAST,
                ))]
            }
            Action::Search(text) => {
                self.query = text;
                Vec::new()
            }
            Action::PickLetter(letter) => {
                self.query.clear();
                self.letter = letter;
                Vec::new()
            }
            Action::OpenExamples(slug) => {
                vec![Effect::Navigate(Destination::Examples(slug))]
            }
            Action::Use(slug) => vec![Effect::AdoptTag(slug)],
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
        }
    }

    /// The search box's text, as typed.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// The letter browsed while no search is on, lowercase, or [`dictionary::OTHER`].
    #[must_use]
    pub fn letter(&self) -> char {
        self.letter
    }

    /// Whether a search is on: the text is not blank once trimmed.
    #[must_use]
    pub fn searching(&self) -> bool {
        !self.matched().is_empty()
    }

    /// The search as it matches: trimmed and lowercased.
    fn matched(&self) -> String {
        self.query.trim().to_lowercase()
    }

    /// What draws below the header, given what the client's catalog holds.
    #[must_use]
    pub fn body(&self, catalog: Catalog<'_>) -> Body {
        match catalog {
            Catalog::Loading => Body::Loading,
            Catalog::Failed => Body::Failed(LOAD_FAILED),
            Catalog::Loaded(tags) => Body::Browse {
                rail: self.rail(tags),
                list: self.list(tags),
            },
        }
    }

    /// The letter rail's chips over `tags`.
    #[must_use]
    pub fn rail(&self, tags: &[OracleTag]) -> Vec<RailLetter> {
        let searching = self.searching();
        dictionary::letters(tags)
            .into_iter()
            .map(|letter| RailLetter {
                letter,
                label: letter_label(letter),
                picked: !searching && letter == self.letter,
            })
            .collect()
    }

    /// The rows over `tags`, or the line in place of them.
    #[must_use]
    pub fn list(&self, tags: &[OracleTag]) -> List {
        let matched = self.matched();
        let rows = dictionary::rows_for(tags, &matched, self.letter);
        if !rows.is_empty() {
            List::Rows(rows)
        } else if matched.is_empty() {
            List::Empty(no_letter(self.letter))
        } else {
            List::Empty(no_match(&matched))
        }
    }
}
