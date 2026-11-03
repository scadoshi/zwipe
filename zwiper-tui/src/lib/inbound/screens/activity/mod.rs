//! Activity: how the user picks, read from their swipes. The user's own counters, then one accordion of the Activity (the year's heatmap and Totals), Card roles and Oracle tags sections, then the Most skipped card rows. Reached from home's `v`.
//!
//! What the screen knows and decides is zwipe-ui's activity model (imported `as model`, since `zwipe_ui::activity` is the math): where the fetch stands, which sections draw, the picked rate and each chart, the revealed role and tag, and the passes. This view adds the accordion, the focus, the day cursor and the sweeps. The payload is fetched on every open and never cached; the counters share home's cache and its 60 second rule, and keep rolling until the fetch settles so they resolve with the rest of the screen.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::cell::Cell;
use std::time::{Duration, Instant};

use ratatui::text::{Line, Span};
use zwipe_core::http::contracts::suggestions::{HttpActivity, HttpHardPass};
use zwipe_ui::activity::{Bar, DAYS_PER_WEEK, HeatLayout};
use zwipe_ui::generation::Ticket;
use zwipe_ui::home::Figures;
use zwipe_ui::screens::activity::{
    self as model, Action, Destination, Effect, Rates, Section, State,
};
use zwipe_ui::timing;

use super::{Screen, home};
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

/// Each heatmap column waits this much longer than the one before it to sweep in.
pub(crate) const SWEEP_STEP: Duration = Duration::from_millis(12);

/// One cell's sweep, as `heat-sweep`.
pub(crate) const SWEEP: Duration = Duration::from_millis(400);

/// A section's "?" topic.
pub(crate) fn topic(section: Section) -> InfoPopup {
    match section {
        Section::Activity => InfoPopup::new(
            "Activity",
            &[
                "Your swipes and searches each day over the last year, a darker square for a busier day",
                "A lit square is one of your biggest days. Tap a square to see its counts",
                "Totals is what the cards dealt to your decks came to: Added and Maybeboard you kept, Skipped you passed on, and Removed you later took out",
            ],
        ),
        Section::Roles => InfoPopup::new(
            "Card roles",
            &[
                "Your share for each card role. Pick one under Show: Added, Skipped and Maybeboard are shares of the cards dealt to you, and Removed is the share of added cards you later took out",
                "Tap a role to see the tags you swiped under it. A grey role has none yet",
                "From your decks with a commander",
            ],
        ),
        Section::Tags => InfoPopup::new(
            "Oracle tags",
            &[
                "The tags where your share stands out most from other players', with your own share",
                "A tag everyone passes on shows only after a lot of skips. Tap a tag to see what it means",
                "From your decks with a commander, for tags dealt at least 10 times",
            ],
        ),
    }
}

/// The Most skipped group's "?" topic.
pub(crate) fn passes_topic() -> InfoPopup {
    InfoPopup::new(
        model::PASSES_TITLE,
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

/// The payload as the model holds it, with the heatmap laid out from it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Loaded<'view> {
    pub(crate) activity: &'view HttpActivity,
    pub(crate) heat: &'view HeatLayout,
}

/// The Activity screen's state: the model plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct ActivityView {
    pub(crate) model: State,
    /// The model's heatmap, laid out once as the payload lands.
    pub(crate) heat: Option<HeatLayout>,
    /// When the screen opened, which drives the loading ghosts and the counters' roll.
    pub(crate) opened: Instant,
    /// When the counters began landing: the later of the fetch settling and the counters arriving. Set while drawing, the first time both hold.
    pub(crate) figures_arrived: Cell<Option<Instant>>,
    /// The model's sections, in order, with which one is open.
    pub(crate) accordion: Accordion,
    pub(crate) sections: Vec<Section>,
    pub(crate) focus: Focus,
    /// When the Activity section last opened, which replays the heatmap's sweep.
    pub(crate) heat_opened: Option<Instant>,
    /// The day under the cursor, an index into the heatmap's cells.
    pub(crate) cursor: usize,
    /// When a day was last picked; its readout shows for [`timing::HEAT_PICK`].
    pub(crate) picked_at: Option<Instant>,
    /// The revealed role's tags, with the one whose definition is open.
    pub(crate) role_tags: RoleTags,
    pub(crate) rows: CardRows,
    /// A section's "?" topic.
    pub(crate) overlay: Option<InfoPopup>,
}

impl ActivityView {
    /// A view waiting on its payload, before [`Action::Load`].
    pub(crate) fn loading() -> Self {
        Self {
            model: State::new(),
            heat: None,
            opened: Instant::now(),
            figures_arrived: Cell::new(None),
            accordion: Accordion::new(&[]),
            sections: Vec::new(),
            focus: Focus::Header(Section::Activity),
            heat_opened: None,
            cursor: 0,
            picked_at: None,
            role_tags: RoleTags::default(),
            rows: CardRows::default(),
            overlay: None,
        }
    }

    /// The payload, once it has something to draw.
    pub(crate) fn loaded(&self) -> Option<Loaded<'_>> {
        Some(Loaded {
            activity: self.model.activity()?,
            heat: self.heat.as_ref()?,
        })
    }

    /// Lays out the payload the model just took: the heatmap with the cursor on its last day, and the model's sections with the one that opens on entry open and focused first.
    fn land(&mut self) {
        let Some(heat) = self
            .model
            .activity()
            .map(|a| zwipe_ui::activity::heat_layout(a.through, &a.days))
        else {
            return;
        };
        self.cursor = heat.cells.len().saturating_sub(1);
        self.heat = Some(heat);
        let sections = self.model.sections();
        let titles: Vec<&str> = sections.iter().map(|s| s.title()).collect();
        self.accordion = Accordion::new(&titles);
        if let Some(first) = self.accordion.sections.first_mut()
            && sections.first().is_some_and(|s| s.opens_on_entry())
        {
            first.open = true;
            self.heat_opened = Some(Instant::now());
        }
        self.focus = match sections.first() {
            Some(&first) => Focus::Header(first),
            None => Focus::Passes,
        };
        self.sections = sections;
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

    /// What a rate section draws at the picked rate. The Activity section is no rate section and charts nothing.
    pub(crate) fn rates(&self, section: Section) -> Rates {
        match section {
            Section::Roles => self.model.role_rates(),
            Section::Tags => self.model.tag_rates(),
            Section::Activity => Rates::Charted(Vec::new()),
        }
    }

    /// The bars of a rate section's chart at the picked rate.
    pub(crate) fn bars(&self, section: Section) -> Vec<Bar> {
        match self.rates(section) {
            Rates::Charted(bars) => bars,
            Rates::Unrated(_) | Rates::Uncharted(_) => Vec::new(),
        }
    }

    /// Whether a rate section has roles or tags to chart at some share, so it shows its Show chips; without any it shows only a line saying what fills it.
    pub(crate) fn has_rates(&self, section: Section) -> bool {
        section != Section::Activity && !matches!(self.rates(section), Rates::Unrated(_))
    }

    /// Whether the heatmap shows: the section is open and the model has a heatmap.
    fn heat_shows(&self) -> bool {
        self.is_open(Section::Activity) && self.model.shows_heatmap()
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

    /// Moves the day cursor by `columns` and `rows`. Returns whether a day is there; the cursor stays put when none is.
    pub(crate) fn move_cursor(&mut self, columns: isize, rows: isize) -> bool {
        let Some(heat) = &self.heat else {
            return false;
        };
        let Some(cell) = heat.cells.get(self.cursor) else {
            return false;
        };
        let (Some(column), Some(row)) = (
            cell.column.checked_add_signed(columns),
            cell.row.checked_add_signed(rows),
        ) else {
            return false;
        };
        if row >= DAYS_PER_WEEK {
            return false;
        }
        let Some(index) = heat.index_at(column, row) else {
            return false;
        };
        self.cursor = index;
        self.picked_at = Some(Instant::now());
        true
    }

    /// The readout's tags at this moment: the picked day's for [`timing::HEAT_PICK`], else the model's year.
    pub(crate) fn readout(&self) -> [String; 3] {
        let picked = self
            .picked_at
            .filter(|at| at.elapsed() < timing::HEAT_PICK)
            .and_then(|_| self.heat.as_ref()?.cells.get(self.cursor));
        match picked {
            Some(cell) => zwipe_ui::activity::day_tags(cell),
            None => self.model.year_tags(),
        }
    }

    /// How long the readout's current tags have shown, for their `chip-in`.
    pub(crate) fn readout_age(&self) -> Duration {
        match self.picked_at {
            Some(at) if at.elapsed() < timing::HEAT_PICK => at.elapsed(),
            Some(at) => at.elapsed().saturating_sub(timing::HEAT_PICK),
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
        let sweeping = self.heat.as_ref().is_some_and(|heat| {
            self.is_open(Section::Activity)
                && self.sweep_elapsed() < Self::sweep_length(heat.columns)
        });
        let reading = self
            .picked_at
            .is_some_and(|t| t.elapsed() < timing::HEAT_PICK.saturating_add(entrance::DURATION));
        rolling
            || self.model.status() == model::Status::Loading
            || sweeping
            || reading
            || self.readout_age() < entrance::DURATION
    }

    /// The counters to show: none until the model says they may stop rolling, then the user's own if cached.
    pub(crate) fn figures(&self, shared: &Shared) -> Option<Figures> {
        if !self.model.counts_ready() {
            return None;
        }
        let figures = shared.caches.my_figures(home::user_id(shared))?;
        if self.figures_arrived.get().is_none() {
            self.figures_arrived.set(Some(Instant::now()));
        }
        Some(figures)
    }

    /// Swaps in a printing picked from a pass's row, and keeps that row open.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        let Some(printing) = cx.shared.caches.handoff.printing.take() else {
            return;
        };
        let id = printing.scryfall_data.id;
        let oracle = printing.scryfall_data.oracle_id;
        let held = oracle.is_some()
            && self
                .model
                .passes()
                .iter()
                .any(|p| p.card.scryfall_data.oracle_id == oracle);
        dispatch(self, Action::PickPrinting(Box::new(printing)), cx);
        if held {
            self.rows.expand(id);
        }
    }
}

/// Applies `action` to the model, brings the view in line with it, and performs the effects it returns, in order.
pub(crate) fn dispatch(view: &mut ActivityView, action: Action, cx: &mut Context<'_>) {
    let landing = matches!(action, Action::Loaded { .. });
    let role = view.model.revealed_role().map(str::to_owned);
    let effects = view.model.update(action);
    if landing && view.heat.is_none() {
        view.land();
    }
    let revealed = view.model.revealed_role().map(str::to_owned);
    if revealed != role {
        view.role_tags = RoleTags::new(
            revealed
                .map(|r| view.model.tags_under(&r))
                .unwrap_or_default(),
        );
    }
    view.settle_focus();
    for effect in effects {
        match effect {
            Effect::OfferHint(offer) => {
                hint::once(cx, offer.key());
            }
            Effect::FetchActivity(ticket) => cx.request(Request::Load { ticket }.into()),
            Effect::Navigate(Destination::Previous) => cx.back(),
        }
    }
}

/// A pass as its card row: the skips in place of the quantity, and the commanders it was skipped under in the footer.
pub(crate) fn pass_row<'pass>(palette: &Palette, pass: &'pass HttpHardPass) -> CardRow<'pass> {
    let mut footer = Vec::new();
    if !pass.commanders.is_empty() {
        footer.push(Line::styled(model::SKIPPED_UNDER, palette.muted));
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
        .with_qty_tags(vec![model::skips(pass)])
        .with_footer(footer)
}

/// The screen's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// The payload.
    Load { ticket: Ticket },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(self) -> &'static str {
        match self {
            Self::Load { .. } => "load",
        }
    }
}

/// The screen's results, each carrying its request's ticket back.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded {
        ticket: Ticket,
        activity: Box<HttpActivity>,
    },
    Failed {
        ticket: Ticket,
        message: String,
    },
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

/// Opens Activity: the model fetches the payload and offers the hint, and the counters when stale and the tag catalog load beside it.
pub(crate) fn open(cx: &mut Context<'_>) {
    let mut view = ActivityView::loading();
    dispatch(&mut view, Action::Load, cx);
    if let Some(user) = home::user_id(cx.shared)
        && cx.shared.caches.my_counts_stale(user, Instant::now())
    {
        cx.request(home::Request::MyMetrics.into());
    }
    oracle_tags::ensure_catalog(cx);
    cx.push(Screen::Activity(Box::new(view)));
}

/// Shows the hint again, for the key bar's `H hint`.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(
        cx,
        zwipe_core::domain::user::hints::OneTimeHint::Activity.key(),
    );
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request::Load { ticket } = request;
    let client = &account.client;
    match async { client.activity(&account.fresh().await?).await }.await {
        Ok(found) => Some(
            Update::Loaded {
                ticket,
                activity: Box::new(found),
            }
            .into(),
        ),
        Err(e) => Some(screens::failure(&e, |message| {
            Update::Failed { ticket, message }.into()
        })),
    }
}

/// Hands a reply to the open screen's model, which drops one whose ticket it did not issue. A failure raises its toast even when the screen has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::Loaded { ticket, activity } => Action::Loaded { ticket, activity },
        Update::Failed { ticket, message } => {
            cx.toast(ToastKind::Error, message);
            Action::Failed { ticket }
        }
    };
    if let Some(view) = nav.find_mut(pick) {
        dispatch(view, action, cx);
    }
}
