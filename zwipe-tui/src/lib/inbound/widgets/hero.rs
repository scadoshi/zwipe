//! The app's `HomeHero` on the sign-in screens: a panel with the ZWIPE logo decoding in, everybody's three counters rolling to their values, and an `Everybody` chip, with `Live` beside it once the numbers were fetched this launch.
//!
//! The counts live in one copy for the whole app ([`Counts`] on `Shared`): restored from disk at launch, fetched again whenever a hero mounts over a copy that is missing, restored or older than [`COUNTS_FRESH`], and saved to disk after each fetch. A failed fetch keeps whatever copy there is, with no message. Every mount replays the decode and the roll.

use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Block;
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

use super::count_up::CountUp;
use super::logo as mark;
use super::measure::{BORDERS, height_of};
use crate::domain::count_up::{self, BLANK};
use crate::domain::theme::Palette;
use crate::inbound::app::{Context, Shared};
use crate::inbound::screens::Request;

/// How long a fetched copy of the counts stays fresh, from the app's `COUNTS_FRESH`.
pub(crate) const COUNTS_FRESH: Duration = Duration::from_secs(60);

/// The three counters' labels, in the app's order.
const LABELS: [&str; 3] = ["Decks created", "Searches run", "Cards swiped"];

/// Rows the panel takes beside the logo: the borders, the gap under the logo, the counters and their labels, and the chips.
const CHROME_ROWS: u16 = 6;

/// The app's one copy of everybody's counts.
#[derive(Debug, Clone, Default)]
pub(crate) struct Counts {
    pub(crate) metrics: Option<HttpPublicMetrics>,
    /// When this launch fetched them; `None` for a copy restored from disk, or none at all.
    pub(crate) fetched_at: Option<Instant>,
}

impl Counts {
    /// The copy restored from disk at launch, which is shown without `Live` and fetched again.
    pub(crate) fn restored(metrics: Option<HttpPublicMetrics>) -> Self {
        Self {
            metrics,
            fetched_at: None,
        }
    }

    /// Whether a hero mounting at `now` fetches them again.
    pub(crate) fn stale(&self, now: Instant) -> bool {
        self.metrics.is_none()
            || self
                .fetched_at
                .is_none_or(|at| now.saturating_duration_since(at) >= COUNTS_FRESH)
    }

    /// Whether the counts were fetched during this launch.
    pub(crate) fn live(&self) -> bool {
        self.fetched_at.is_some()
    }

    /// The three counts in the app's order, `None` while there are none.
    fn values(&self) -> [Option<u64>; 3] {
        let value = |n: i64| Some(u64::try_from(n).unwrap_or(0));
        self.metrics.as_ref().map_or([None; 3], |m| {
            [
                value(m.decks_created),
                value(m.searches),
                value(m.cards_swiped),
            ]
        })
    }
}

/// One mount of the hero: when its entrance starts, and its three counters. A hero that never mounted, `start` `None`, draws settled and still.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Hero {
    start: Option<Instant>,
    counters: [CountUp; 3],
}

impl Default for Hero {
    /// A hero that never mounted, for a screen built without the shared state: the logo settled, the counters as the shared copy has them.
    fn default() -> Self {
        let now = Instant::now();
        Self {
            start: None,
            counters: std::array::from_fn(|_| CountUp::new(now, None)),
        }
    }
}

impl Hero {
    /// A hero mounting at `now` over the shared counts, starting once the splash allows.
    pub(crate) fn new(shared: &Shared, now: Instant) -> Self {
        let start = shared.entrance_start(now);
        Self {
            start: Some(start),
            counters: shared.counts.values().map(|v| CountUp::new(start, v)),
        }
    }

    /// Takes counts that arrived at `now`, landing each counter on its new value.
    pub(crate) fn take(&mut self, counts: &Counts, now: Instant) {
        for (counter, value) in self.counters.iter_mut().zip(counts.values()) {
            if let Some(value) = value {
                counter.set(value, now);
            }
        }
    }

    /// Whether the logo is decoding or a counter is rolling at `now`.
    pub(crate) fn animating(&self, now: Instant) -> bool {
        self.start.is_some_and(|start| {
            mark::animating(now.saturating_duration_since(start))
                || self.counters.iter().any(|c| c.animating(now))
        })
    }

    /// The logo that fits `area` with `room` rows for the panel: the full mark, the compact Z, or none.
    fn logo(width: u16, room: u16) -> Option<&'static str> {
        mark::fitting(
            width.saturating_sub(BORDERS).saturating_sub(BORDERS),
            room.saturating_sub(CHROME_ROWS),
        )
    }

    /// How tall the hero draws in an area `width` wide with at most `room` rows: the panel with the biggest logo that fits, the panel alone, or nothing.
    pub(crate) fn height(width: u16, room: u16) -> u16 {
        match Self::logo(width, room) {
            Some(art) => height_of(art.lines().count()).saturating_add(CHROME_ROWS),
            None if room >= CHROME_ROWS.saturating_sub(1) => CHROME_ROWS.saturating_sub(1),
            None => 0,
        }
    }

    /// Draws the hero in `area` as of `now`.
    pub(crate) fn draw(&self, f: &mut Frame<'_>, shared: &Shared, area: Rect, now: Instant) {
        let palette = &shared.palette;
        if area.height < CHROME_ROWS.saturating_sub(1) || area.width < 8 {
            return;
        }
        let block = Block::bordered().border_style(palette.border);
        let inner = block.inner(area);
        f.render_widget(block, area);
        let art = Self::logo(area.width, area.height);
        let logo_height = art.map_or(0, |a| height_of(a.lines().count()));
        let [logo_area, _, figures, labels, chips] = Layout::vertical([
            Constraint::Length(logo_height),
            Constraint::Length(u16::from(logo_height > 0)),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        if let Some(art) = art {
            let elapsed = self.start.map(|start| now.saturating_duration_since(start));
            mark::draw(f, logo_area, art, palette.logo, elapsed);
        }
        let columns = Layout::horizontal([Constraint::Fill(1); 3]).split(figures);
        let label_columns = Layout::horizontal([Constraint::Fill(1); 3]).split(labels);
        let values = shared.counts.values();
        for (((counter, value), label), (figure, under)) in self
            .counters
            .iter()
            .zip(values)
            .zip(LABELS)
            .zip(columns.iter().zip(label_columns.iter()))
        {
            let shown = if self.start.is_some() {
                counter.shown(now)
            } else {
                value.map_or_else(|| BLANK.to_owned(), count_up::compact)
            };
            f.render_widget(
                Line::from(shown).style(palette.accent).bold().centered(),
                *figure,
            );
            f.render_widget(Line::from(label).style(palette.muted).centered(), *under);
        }
        f.render_widget(chips_line(palette, shared.counts.live()).centered(), chips);
    }
}

/// The scope chips: `Everybody`, and `Live` when the counts were fetched this launch.
fn chips_line(palette: &Palette, live: bool) -> Line<'static> {
    let mut spans = vec![Span::styled(
        " Everybody ",
        palette.accent.reversed().bold(),
    )];
    if live {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(" Live ", palette.add.reversed().bold()));
    }
    Line::from(spans)
}

/// A hero mounting now, asking for the counts again when the shared copy is stale.
pub(crate) fn mount(cx: &mut Context<'_>) -> Hero {
    let now = Instant::now();
    if cx.shared.counts.stale(now) {
        cx.request(Request::FetchCounts);
    }
    Hero::new(cx.shared, now)
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::inbound::app::App;

    fn metrics() -> HttpPublicMetrics {
        HttpPublicMetrics {
            cards_swiped: 28_018,
            searches: 2_065,
            decks_created: 12,
        }
    }

    fn text(app: &App, hero: &Hero, now: Instant, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|f| hero.draw(f, &app.shared, f.area(), now))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect()
    }

    #[test]
    fn a_copy_is_stale_when_missing_restored_or_a_minute_old() {
        let now = Instant::now();
        assert!(Counts::default().stale(now));
        assert!(Counts::restored(Some(metrics())).stale(now));
        let fetched = Counts {
            metrics: Some(metrics()),
            fetched_at: Some(now),
        };
        assert!(!fetched.stale(now + Duration::from_secs(59)));
        assert!(fetched.stale(now + COUNTS_FRESH));
        assert!(fetched.live() && !Counts::restored(None).live());
    }

    #[test]
    fn mounting_over_a_stale_copy_fetches_and_the_counters_land_on_arrival() {
        let mut app = App::new(Palette::terminal());
        let mut cx = Context::new(&mut app.shared);
        let mut hero = mount(&mut cx);
        assert_eq!(cx.into_effects().requests, [Request::FetchCounts]);
        let now = Instant::now();
        assert!(hero.animating(now));
        let rolling = text(&app, &hero, now + Duration::from_millis(100), 60, 24);
        assert!(rolling.contains("Decks created") && rolling.contains("Everybody"));
        assert!(!rolling.contains("Live"));

        app.shared.counts = Counts {
            metrics: Some(metrics()),
            fetched_at: Some(now),
        };
        hero.take(&app.shared.counts, now);
        let landed = now + Duration::from_secs(2);
        assert!(!hero.animating(landed));
        let screen = text(&app, &hero, landed, 60, 24);
        assert!(screen.contains("28.0k") && screen.contains("2,065") && screen.contains("12"));
        assert!(screen.contains("Live"));
        assert!(screen.contains('▉'), "the logo has decoded");
    }

    #[test]
    fn a_fresh_copy_is_not_fetched_and_the_hero_shrinks_to_fit() {
        let mut app = App::new(Palette::terminal());
        app.shared.counts = Counts {
            metrics: Some(metrics()),
            fetched_at: Some(Instant::now()),
        };
        let mut cx = Context::new(&mut app.shared);
        let hero = mount(&mut cx);
        assert_eq!(cx.into_effects().requests, []);
        assert_eq!(Hero::height(80, 40), 17, "the full logo");
        assert_eq!(Hero::height(30, 40), 13, "the Z");
        assert_eq!(Hero::height(30, 6), 5, "no logo");
        assert_eq!(Hero::height(30, 3), 0);
        let before = text(
            &app,
            &hero,
            Instant::now()
                .checked_sub(Duration::from_secs(1))
                .expect("the clock is past a second"),
            60,
            24,
        );
        assert!(before.contains(BLANK), "nothing before its start");
        text(&app, &hero, Instant::now(), 8, 3);
        let still = Hero::default();
        assert!(!still.animating(Instant::now()), "never mounted");
        assert!(text(&app, &still, Instant::now(), 60, 24).contains("28.0k"));
    }
}
