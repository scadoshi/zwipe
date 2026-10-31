//! Activity: how the user picks, read from their swipes. The user's own counters, then one accordion of the Activity (the year's heatmap and Totals), Card roles and Oracle tags sections, then the Most skipped card rows. Reached from home's `v`.
//!
//! The payload is fetched on every open and never cached; the counters share home's cache and its 60 second rule, and keep rolling until the payload lands so they resolve with the rest of the screen.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::cell::Cell;
use std::time::{Duration, Instant};

use ratatui::text::{Line, Span};
use zwipe_core::domain::user::models::hints::HINT_ACTIVITY;
use zwipe_core::http::contracts::suggestions::{HttpActivity, HttpHardPass};

use super::{Screen, home};
use crate::domain::activity::{self, Bar, HeatLayout, RateKind};
use crate::domain::home::Figures;
use crate::domain::theme::Palette;
use crate::inbound::app::{Context, Shared};
use crate::inbound::nav::Nav;
use crate::inbound::screens::{self, oracle_tags};
use crate::inbound::toward::Toward;
use crate::inbound::widgets::card_rows::{CardRow, CardRows};
use crate::inbound::widgets::collapsible::Accordion;
use crate::inbound::widgets::info::InfoPopup;
use crate::inbound::widgets::otag::RoleTags;
use crate::inbound::widgets::toast::ToastKind;
use crate::inbound::widgets::{entrance, figures, hint};
use crate::outbound::account::Account;

/// The screen's title and its hint's.
pub(crate) const TITLE: &str = "Activity";

/// Said when the payload failed to load.
pub(crate) const LOAD_FAILED: &str = "Could not load your activity";

/// Said when there is nothing to show yet.
pub(crate) const NOTHING_YET: &str = "Swipe some cards to see your activity";

/// The Most skipped group's header.
pub(crate) const PASSES_TITLE: &str = "Most skipped";

/// The label over a pass's commanders.
pub(crate) const SKIPPED_UNDER: &str = "Skipped under";

/// How long a picked day's readout shows before the year's totals come back, as orangui's `HEAT_PICK_MS`.
pub(crate) const HEAT_PICK: Duration = Duration::from_millis(4000);

/// Each heatmap column waits this much longer than the one before it to sweep in.
pub(crate) const SWEEP_STEP: Duration = Duration::from_millis(12);

/// One cell's sweep, as `heat-sweep`.
pub(crate) const SWEEP: Duration = Duration::from_millis(400);

/// One of the accordion's sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    /// The heatmap and Totals.
    Activity,
    Roles,
    Tags,
}

impl Section {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Activity => TITLE,
            Self::Roles => "Card roles",
            Self::Tags => "Oracle tags",
        }
    }

    /// The section's "?" topic.
    pub(crate) fn topic(self) -> InfoPopup {
        match self {
            Self::Activity => InfoPopup::new(
                "Activity",
                &[
                    "Your swipes and searches each day over the last year, a darker square for a busier day",
                    "A lit square is one of your biggest days. Tap a square to see its counts",
                    "Totals is what the cards dealt to your decks came to: Added and Maybeboard you kept, Skipped you passed on, and Removed you later took out",
                ],
            ),
            Self::Roles => InfoPopup::new(
                "Card roles",
                &[
                    "Your share for each card role. Pick one under Show: Added, Skipped and Maybeboard are shares of the cards dealt to you, and Removed is the share of added cards you later took out",
                    "Tap a role to see the tags you swiped under it. A grey role has none yet",
                    "From your decks with a commander",
                ],
            ),
            Self::Tags => InfoPopup::new(
                "Oracle tags",
                &[
                    "The tags where your share stands out most from other players', with your own share",
                    "A tag everyone passes on shows only after a lot of skips. Tap a tag to see what it means",
                    "From your decks with a commander, for tags dealt at least 10 times",
                ],
            ),
        }
    }
}

/// The Most skipped group's "?" topic.
pub(crate) fn passes_topic() -> InfoPopup {
    InfoPopup::new(
        PASSES_TITLE,
        &[
            "Cards you skip and never add or save, when other players usually keep them",
            "A card everyone passes on shows only after a lot of skips. The tag counts yours, and an open row names the commanders you skipped it under",
            "From your decks with a commander",
        ],
    )
}

/// What has the keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    /// A section's header.
    Header(Section),
    /// The day cursor in the heatmap.
    Heat,
    /// A rate section's Show chips.
    Chips(Section),
    /// A row of a rate section's chart.
    Bar(Section, usize),
    /// The Most skipped rows, which move their own focus.
    Passes,
}

/// The payload and the heatmap laid out from it.
#[derive(Debug)]
pub(crate) struct Loaded {
    pub(crate) activity: HttpActivity,
    pub(crate) heat: HeatLayout,
}

/// Where the payload is.
#[derive(Debug)]
pub(crate) enum Load {
    Loading,
    Failed,
    Loaded(Box<Loaded>),
}

/// The Activity screen's state.
#[derive(Debug)]
pub(crate) struct ActivityView {
    pub(crate) load: Load,
    /// When the screen opened, which drives the loading ghosts and the counters' roll.
    pub(crate) opened: Instant,
    /// When the payload's fetch finished, either way; the counters land from then.
    pub(crate) ready_at: Option<Instant>,
    /// When the counters began landing: the later of the fetch finishing and the counters arriving. Set while drawing, the first time both hold.
    pub(crate) figures_arrived: Cell<Option<Instant>>,
    /// The sections present, in order, with which one is open.
    pub(crate) accordion: Accordion,
    pub(crate) sections: Vec<Section>,
    pub(crate) focus: Focus,
    /// The Show chips' pick, shared by both rate charts.
    pub(crate) rate: RateKind,
    /// When the Activity section last opened, which replays the heatmap's sweep.
    pub(crate) heat_opened: Option<Instant>,
    /// The day under the cursor, an index into the heatmap's cells.
    pub(crate) cursor: usize,
    /// When a day was last picked; its readout shows for [`HEAT_PICK`].
    pub(crate) picked_at: Option<Instant>,
    /// The role whose tags are revealed, and those tags.
    pub(crate) open_role: Option<String>,
    pub(crate) role_tags: RoleTags,
    /// The tag whose definition is revealed.
    pub(crate) open_tag: Option<String>,
    pub(crate) rows: CardRows,
    /// A section's "?" topic.
    pub(crate) overlay: Option<InfoPopup>,
}

impl ActivityView {
    /// A view waiting on its payload.
    pub(crate) fn loading() -> Self {
        Self {
            load: Load::Loading,
            opened: Instant::now(),
            ready_at: None,
            figures_arrived: Cell::new(None),
            accordion: Accordion::new(&[]),
            sections: Vec::new(),
            focus: Focus::Header(Section::Activity),
            rate: RateKind::default(),
            heat_opened: None,
            cursor: 0,
            picked_at: None,
            open_role: None,
            role_tags: RoleTags::default(),
            open_tag: None,
            rows: CardRows::default(),
            overlay: None,
        }
    }

    /// The payload, once loaded.
    pub(crate) fn loaded(&self) -> Option<&Loaded> {
        match &self.load {
            Load::Loaded(loaded) => Some(loaded),
            _ => None,
        }
    }

    /// Takes the payload: lays out the heatmap, opens Activity when it has something to show and leaves it out otherwise, and keeps Card roles and Oracle tags either way, as they say what fills them.
    pub(crate) fn land(&mut self, activity: HttpActivity) {
        let heat = activity::heat_layout(activity.through, &activity.days);
        self.cursor = heat.cells.len().saturating_sub(1);
        let mut sections = Vec::new();
        if !activity.days.is_empty() || activity::has_outcomes(&activity.outcomes) {
            sections.push(Section::Activity);
        }
        sections.push(Section::Roles);
        sections.push(Section::Tags);
        let titles: Vec<&str> = sections.iter().map(|s| s.title()).collect();
        self.accordion = Accordion::new(&titles);
        if let Some(first) = self.accordion.sections.first_mut()
            && sections.first() == Some(&Section::Activity)
        {
            first.open = true;
            self.heat_opened = Some(Instant::now());
        }
        self.focus = match sections.first() {
            Some(&first) => Focus::Header(first),
            None => Focus::Passes,
        };
        self.sections = sections;
        self.load = Load::Loaded(Box::new(Loaded { activity, heat }));
    }

    pub(crate) fn is_open(&self, section: Section) -> bool {
        self.sections
            .iter()
            .position(|&s| s == section)
            .and_then(|i| self.accordion.sections.get(i))
            .is_some_and(|s| s.open)
    }

    /// Opens `section` and closes the rest, or closes it when it is open. Opening Activity replays the heatmap's sweep.
    pub(crate) fn toggle(&mut self, section: Section) {
        let Some(i) = self.sections.iter().position(|&s| s == section) else {
            return;
        };
        self.accordion.focus = i;
        self.accordion.toggle_focused();
        if section == Section::Activity && self.is_open(section) {
            self.heat_opened = Some(Instant::now());
            self.picked_at = None;
        }
    }

    /// The bars of a rate section's chart at the picked rate.
    pub(crate) fn bars(&self, section: Section) -> Vec<Bar> {
        let Some(loaded) = self.loaded() else {
            return Vec::new();
        };
        let a = &loaded.activity;
        match section {
            Section::Roles => activity::role_rate_bars(self.rate, &a.roles, &a.tags),
            Section::Tags => activity::tag_rate_bars(self.rate, &a.tags),
            Section::Activity => Vec::new(),
        }
    }

    /// Whether a rate section has any roles or tags to chart at some share; without any it shows only a line saying what fills it.
    pub(crate) fn has_rates(&self, section: Section) -> bool {
        self.loaded().is_some_and(|l| match section {
            Section::Roles => !l.activity.roles.is_empty(),
            Section::Tags => !l.activity.tags.is_empty(),
            Section::Activity => false,
        })
    }

    /// Whether the heatmap shows: the section is open and there are days.
    fn heat_shows(&self) -> bool {
        self.is_open(Section::Activity)
            && self.loaded().is_some_and(|l| !l.activity.days.is_empty())
    }

    /// Everything the focus can land on, top to bottom.
    pub(crate) fn stops(&self) -> Vec<Focus> {
        let mut stops = Vec::new();
        for &section in &self.sections {
            stops.push(Focus::Header(section));
            if !self.is_open(section) {
                continue;
            }
            match section {
                Section::Activity => {
                    if self.heat_shows() {
                        stops.push(Focus::Heat);
                    }
                }
                Section::Roles | Section::Tags => {
                    if !self.has_rates(section) {
                        continue;
                    }
                    stops.push(Focus::Chips(section));
                    let count = self.bars(section).len();
                    stops.extend((0..count).map(|i| Focus::Bar(section, i)));
                }
            }
        }
        if self.loaded().is_some() {
            stops.push(Focus::Passes);
        }
        stops
    }

    /// Moves the focus one stop `toward` the next or the previous. Returns whether it moved.
    pub(crate) fn step(&mut self, toward: Toward) -> bool {
        let stops = self.stops();
        let Some(at) = stops.iter().position(|&s| s == self.focus) else {
            if let Some(&first) = stops.first() {
                self.focus = first;
            }
            return true;
        };
        let Some(&focus) = toward.neighbor(at).and_then(|i| stops.get(i)) else {
            return false;
        };
        if focus == Focus::Passes && toward == Toward::Next {
            self.rows.set_focused(0);
        }
        self.focus = focus;
        true
    }

    /// Keeps the focus on a stop that still exists after the rate or a section changed: the chart's last row, else its chips, else the section's header.
    pub(crate) fn settle_focus(&mut self) {
        let stops = self.stops();
        if stops.contains(&self.focus) {
            return;
        }
        self.focus = match self.focus {
            Focus::Bar(section, _) => stops
                .iter()
                .rev()
                .find(|s| matches!(s, Focus::Bar(b, _) if *b == section))
                .copied()
                .unwrap_or(Focus::Chips(section)),
            Focus::Heat => Focus::Header(Section::Activity),
            other => other,
        };
        if !self.stops().contains(&self.focus) {
            self.focus = stops.first().copied().unwrap_or(Focus::Passes);
        }
    }

    /// The section the focus is in, `None` on the Most skipped rows.
    pub(crate) fn focused_section(&self) -> Option<Section> {
        match self.focus {
            Focus::Header(s) | Focus::Chips(s) | Focus::Bar(s, _) => Some(s),
            Focus::Heat => Some(Section::Activity),
            Focus::Passes => None,
        }
    }

    /// Picks the rate, which both charts show.
    pub(crate) fn pick_rate(&mut self, rate: RateKind) {
        self.rate = rate;
        self.settle_focus();
    }

    /// Reveals the tags under `role`, or hides them when they show.
    pub(crate) fn toggle_role(&mut self, role: &str) {
        if self.open_role.as_deref() == Some(role) {
            self.open_role = None;
            self.role_tags = RoleTags::default();
            return;
        }
        let tags = self
            .loaded()
            .map(|l| activity::tags_under(role, &l.activity.tags))
            .unwrap_or_default();
        self.open_role = Some(role.to_owned());
        self.role_tags = RoleTags::new(tags);
    }

    /// Reveals `slug`'s definition, or hides it when it shows.
    pub(crate) fn toggle_tag(&mut self, slug: &str) {
        if self.open_tag.as_deref() == Some(slug) {
            self.open_tag = None;
        } else {
            self.open_tag = Some(slug.to_owned());
        }
    }

    /// Moves the day cursor by `columns` and `rows`. Returns whether a day is there; the cursor stays put when none is.
    pub(crate) fn move_cursor(&mut self, columns: isize, rows: isize) -> bool {
        let Some(loaded) = self.loaded() else {
            return false;
        };
        let Some(cell) = loaded.heat.cells.get(self.cursor) else {
            return false;
        };
        let (Some(column), Some(row)) = (
            cell.column.checked_add_signed(columns),
            cell.row.checked_add_signed(rows),
        ) else {
            return false;
        };
        if row >= activity::DAYS_PER_WEEK {
            return false;
        }
        let Some(index) = loaded.heat.index_at(column, row) else {
            return false;
        };
        self.cursor = index;
        self.picked_at = Some(Instant::now());
        true
    }

    /// The readout's tags at this moment: the picked day's for [`HEAT_PICK`], else the year's.
    pub(crate) fn readout(&self) -> [String; 3] {
        let Some(loaded) = self.loaded() else {
            return Default::default();
        };
        let picked = self
            .picked_at
            .filter(|at| at.elapsed() < HEAT_PICK)
            .and_then(|_| loaded.heat.cells.get(self.cursor));
        match picked {
            Some(cell) => activity::day_tags(cell),
            None => activity::year_tags(&loaded.activity.days),
        }
    }

    /// How long the readout's current tags have shown, for their `chip-in`.
    pub(crate) fn readout_age(&self) -> Duration {
        match self.picked_at {
            Some(at) if at.elapsed() < HEAT_PICK => at.elapsed(),
            Some(at) => at.elapsed().saturating_sub(HEAT_PICK),
            None => self.heat_opened.map_or(Duration::MAX, |t| t.elapsed()),
        }
    }

    /// How long the heatmap has been sweeping in.
    pub(crate) fn sweep_elapsed(&self) -> Duration {
        self.heat_opened.map_or(Duration::MAX, |t| t.elapsed())
    }

    /// How long the sweep runs for `columns` columns.
    pub(crate) fn sweep_length(columns: usize) -> Duration {
        SWEEP_STEP
            .saturating_mul(u32::try_from(columns).unwrap_or(u32::MAX))
            .saturating_add(SWEEP)
    }

    /// Whether something on the screen is moving: the counters, a loading ghost, the sweep, a readout or an entrance.
    pub(crate) fn animating(&self) -> bool {
        let rolling = self
            .figures_arrived
            .get()
            .is_none_or(|t| t.elapsed() < figures::ROLL);
        let sweeping = self.loaded().is_some_and(|l| {
            self.is_open(Section::Activity)
                && self.sweep_elapsed() < Self::sweep_length(l.heat.columns)
        });
        let reading = self
            .picked_at
            .is_some_and(|t| t.elapsed() < HEAT_PICK.saturating_add(entrance::DURATION));
        rolling
            || matches!(self.load, Load::Loading)
            || sweeping
            || reading
            || self.readout_age() < entrance::DURATION
    }

    /// The counters to show: none until the payload's fetch finishes, then the user's own if cached.
    pub(crate) fn figures(&self, shared: &Shared) -> Option<Figures> {
        self.ready_at?;
        let figures = shared.caches.my_figures(home::user_id(shared))?;
        if self.figures_arrived.get().is_none() {
            self.figures_arrived.set(Some(Instant::now()));
        }
        Some(figures)
    }

    /// Swaps in a printing picked from a pass's row.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        if let Load::Loaded(loaded) = &mut self.load {
            self.rows
                .take_printing(cx, &mut loaded.activity.passes, |p| &mut p.card);
        }
    }
}

/// A pass as its card row: the skips in place of the quantity, and the commanders it was skipped under in the footer.
pub(crate) fn pass_row<'pass>(palette: &Palette, pass: &'pass HttpHardPass) -> CardRow<'pass> {
    let mut footer = Vec::new();
    if !pass.commanders.is_empty() {
        footer.push(Line::styled(SKIPPED_UNDER, palette.muted));
        let mut chips = Vec::new();
        for (i, name) in pass.commanders.iter().enumerate() {
            if i > 0 {
                chips.push(Span::raw(" "));
            }
            chips.push(Span::styled(format!(" {name} "), palette.base.reversed()));
        }
        footer.push(Line::from(chips));
    }
    CardRow::new(&pass.card)
        .with_qty(pass.skipped)
        .with_qty_tags(vec![activity::counted(pass.skipped, "skip", "skips")])
        .with_footer(footer)
}

/// The screen's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// The payload.
    Load,
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(self) -> &'static str {
        match self {
            Self::Load => "load",
        }
    }
}

/// The screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded(Box<HttpActivity>),
    Failed(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Activity(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Activity(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut ActivityView> {
    match screen {
        Screen::Activity(view) => Some(view),
        _ => None,
    }
}

/// Opens Activity: fetches the payload, the counters when stale, and the tag catalog, and shows the hint the first time.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.push(Screen::Activity(Box::new(ActivityView::loading())));
    cx.request(Request::Load.into());
    if let Some(user) = home::user_id(cx.shared)
        && cx.shared.caches.my_counts_stale(user, Instant::now())
    {
        cx.request(home::Request::MyMetrics.into());
    }
    oracle_tags::ensure_catalog(cx);
    hint::once(cx, HINT_ACTIVITY);
}

/// Shows the hint again, for the header's "?".
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_ACTIVITY);
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request::Load = request;
    let client = &account.client;
    match async { client.activity(&account.fresh().await?).await }.await {
        Ok(found) => Some(Update::Loaded(Box::new(found)).into()),
        Err(e) => Some(screens::failure(&e, |message| {
            Update::Failed(message).into()
        })),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    view.ready_at = Some(Instant::now());
    match update {
        Update::Loaded(found) => view.land(*found),
        Update::Failed(message) => {
            view.load = Load::Failed;
            cx.toast(ToastKind::Error, message);
        }
    }
}
