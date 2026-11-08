//! About: the client's version, the links to the website and the privacy policy, the credits, and the release history from the launch's fetch under chip rows that narrow it from `Latest` to a year, a month and a minor version.
//!
//! Where each release files is [`crate::changelog`]; this model adds what the screen does with it: which branch draws while the launch fetch is out or after it failed, the chip rows a filter opens and what each chip selects, the rows over the history and where each one leads, and Back. The history comes in on [`Action::Load`] from the client's [`ChangelogCache`], with no retry and no compiled fallback. The client's version, the web's anchors and every scroll stay in the client.

#[cfg(test)]
mod tests;

use zwipe_core::domain::site::WEB_BASE;
use zwipe_core::http::contracts::changelog::HttpChangelog;

use crate::changelog::{self, ChangelogCache, Filter, Listed};

/// The screen's title.
pub const TITLE: &str = "About";

/// The Website row's button.
pub const WEBSITE_LINK: &str = "zwipe.net \u{2197}";

/// The Privacy policy row's button.
pub const VIEW: &str = "View";

/// The panel title over the Scryfall credit and the Fan Content notice.
pub const CREDITS: &str = "Credits";

/// The panel title over the history.
pub const CHANGELOG: &str = "Changelog";

/// The note in place of the history when the launch fetch failed.
pub const FAILED: &str = "Couldn't load the changelog";

/// The first chip, which goes back to [`Filter::Latest`].
pub const LATEST: &str = "Latest";

/// The Back button, to the screen before.
pub const BACK: &str = "Back";

/// The Version row's value: `v` and the client's version.
#[must_use]
pub fn version_label(version: &str) -> String {
    format!("v{version}")
}

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The screen the user came from, through the client's history.
    Previous,
    /// The privacy policy, from its row.
    PrivacyPolicy,
}

/// A row in the top panel, in the order [`ROWS`] lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Row {
    /// The client's version.
    Version,
    /// The link to the website.
    Website,
    /// The link to the privacy policy.
    PrivacyPolicy,
}

/// The top panel's rows, top to bottom.
pub const ROWS: [Row; 3] = [Row::Version, Row::Website, Row::PrivacyPolicy];

impl Row {
    /// The row's label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Version => "Version",
            Self::Website => "Website",
            Self::PrivacyPolicy => "Privacy policy",
        }
    }
}

/// Which branch the screen draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The launch fetch is still out: the skeleton.
    Loading,
    /// The launch fetch failed: [`FAILED`].
    Failed,
    /// The history: the chip rows over the release cards.
    Loaded,
}

/// One chip: its label, whether it is lit, and the filter pressing it picks. Pressing the deepest lit chip goes up a level; pressing any other selects it, which clears the levels below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chip {
    /// The chip's words: [`LATEST`], a year, a month or a minor version.
    pub label: String,
    /// Whether the chip is lit.
    pub selected: bool,
    /// What [`Action::Pick`] should carry when the chip is pressed.
    pub target: Filter,
}

/// What a user does on About, and each change to the client's cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Opens the screen on `Latest` with the cache as it stands. Send it again when the launch fetch lands while the screen is up.
    Load(ChangelogCache),
    /// Narrows the list to this filter, a chip's [`Chip::target`].
    Pick(Filter),
    /// Opens the website, from the Website row.
    Website,
    /// Opens the privacy policy, from its row.
    PrivacyPolicy,
    /// Goes back to the screen before.
    Back,
}

/// What an action asks the client to do.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Opens this address outside the client.
    OpenUrl(String),
    /// Goes to this screen.
    Navigate(Destination),
}

/// About for one visit. Starts loading on `Latest`; send [`Action::Load`] with the client's cache.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct State {
    cache: ChangelogCache,
    filter: Filter,
}

impl State {
    /// The screen as it first draws: loading, on `Latest`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load(cache) => {
                self.cache = cache;
                self.filter = Filter::Latest;
            }
            Action::Pick(filter) => {
                if self.status() == Status::Loaded {
                    self.filter = filter;
                }
            }
            Action::Website => return vec![Effect::OpenUrl(WEB_BASE.to_owned())],
            Action::PrivacyPolicy => return vec![Effect::Navigate(Destination::PrivacyPolicy)],
            Action::Back => return vec![Effect::Navigate(Destination::Previous)],
        }
        Vec::new()
    }

    /// Which branch draws.
    #[must_use]
    pub fn status(&self) -> Status {
        match self.cache {
            ChangelogCache::Loading => Status::Loading,
            ChangelogCache::Failed => Status::Failed,
            ChangelogCache::Loaded(_) => Status::Loaded,
        }
    }

    /// The history, once loaded.
    #[must_use]
    pub fn changelog(&self) -> Option<&HttpChangelog> {
        match &self.cache {
            ChangelogCache::Loaded(found) => Some(found),
            ChangelogCache::Loading | ChangelogCache::Failed => None,
        }
    }

    /// What the chips have narrowed the list to.
    #[must_use]
    pub fn filter(&self) -> &Filter {
        &self.filter
    }

    /// The chip rows: `Latest` and the years (`Earlier` last, when a release landed there), then the selected year's months, then the selected month's minor versions. Empty until loaded.
    #[must_use]
    pub fn chip_rows(&self) -> Vec<Vec<Chip>> {
        self.changelog().map_or_else(Vec::new, |found| {
            chip_rows(&changelog::listed(found), &self.filter)
        })
    }

    /// The releases the filter shows, in listing order. Empty until loaded.
    #[must_use]
    pub fn shown(&self) -> Vec<Listed<'_>> {
        self.changelog().map_or_else(Vec::new, |found| {
            changelog::visible(&changelog::listed(found), &self.filter)
        })
    }
}

/// The chip rows for `selection` over `listed`.
fn chip_rows(listed: &[Listed<'_>], selection: &Filter) -> Vec<Vec<Chip>> {
    let chip = |label: String, own: Filter, lit: bool| Chip {
        label,
        selected: lit,
        target: if lit && own == *selection {
            own.parent()
        } else {
            own
        },
    };

    let mut first = vec![chip(
        LATEST.to_owned(),
        Filter::Latest,
        *selection == Filter::Latest,
    )];
    first.extend(changelog::years(listed).into_iter().map(|(year, label)| {
        let lit = selection.year() == Some(year.as_str());
        chip(label, Filter::Year(year), lit)
    }));
    let mut rows = vec![first];

    let Some(year) = selection.year() else {
        return rows;
    };
    rows.push(
        changelog::months(listed, year)
            .into_iter()
            .map(|(month, label)| {
                let lit = selection.month() == Some(month.as_str());
                let own = Filter::Month {
                    year: year.to_owned(),
                    month,
                };
                chip(label, own, lit)
            })
            .collect(),
    );

    let Some(month) = selection.month() else {
        return rows;
    };
    rows.push(
        changelog::minors(listed, month)
            .into_iter()
            .map(|minor| {
                let lit = selection.minor() == Some(minor.as_str());
                let own = Filter::Minor {
                    year: year.to_owned(),
                    month: month.to_owned(),
                    minor: minor.clone(),
                };
                chip(minor, own, lit)
            })
            .collect(),
    );
    rows
}
