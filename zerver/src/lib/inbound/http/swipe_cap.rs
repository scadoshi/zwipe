//! Per-account swipe cap on usage batches.
//!
//! A usage batch reports swipes twice: as the four direction counters, which
//! feed the lifetime, daily and weekly counts (and Home's global "cards
//! swiped"), and as decisions inside the card and commander-select signals,
//! which feed the pooled per-commander tables. Each view gets its own budget
//! of the same size, so the counters and the signals trim alike.
//!
//! The budget for one batch is the smaller of
//! [`HttpUsageBatch::MAX_SWIPES_PER_FLUSH`] and what is left of the account's
//! [`HttpUsageBatch::MAX_SWIPES_PER_MINUTE`] in its current one-minute window.
//! Trimming is deterministic and walks the batch in field order:
//!
//! - Direction counters: right, then left, then up, then down. Each keeps what
//!   fits in the budget left after the ones before it.
//! - Signals: card signals in list order, then commander-select signals in
//!   list order. A card signal spends on `added`, then `skipped`, then
//!   `maybed`; a select signal on `selected`, then `skipped`. A trimmed entry's
//!   `shown` drops by the decisions it lost. An entry left with no decisions
//!   and nothing else to report is removed.
//!
//! Everything else in the batch (searches, removals, deck skips, client
//! errors, pile ends) passes through unchanged. The window is charged with the
//! larger of the two views' kept totals.

use std::time::{Duration, Instant};

use dashmap::DashMap;
use uuid::Uuid;
use zwipe_core::http::contracts::metrics::{CardSignalDelta, CommanderSelectDelta, HttpUsageBatch};

/// Length of the fixed window that [`HttpUsageBatch::MAX_SWIPES_PER_MINUTE`]
/// counts over.
pub const WINDOW: Duration = Duration::from_secs(60);

/// Number of tracked accounts above which expired windows are swept out.
const PRUNE_ABOVE: usize = 4_096;

/// Per-account swipe windows, shared across requests in `AppState`.
#[derive(Debug, Default)]
pub struct SwipeCap {
    windows: DashMap<Uuid, Window>,
}

/// One account's current window.
#[derive(Debug, Clone, Copy)]
struct Window {
    /// When the window opened.
    opened_at: Instant,
    /// Swipes counted so far in the window.
    counted: u32,
    /// Whether the window has logged its over-cap warning.
    warning: Warning,
}

impl Window {
    /// A fresh window opening at `now`.
    fn open(now: Instant) -> Self {
        Self {
            opened_at: now,
            counted: 0,
            warning: Warning::Due,
        }
    }

    /// Whether the window has run its full length by `now`.
    fn expired(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.opened_at) >= WINDOW
    }
}

/// Whether a window still owes its one over-cap warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Warning {
    /// No warning logged yet in this window.
    Due,
    /// The warning for this window is logged.
    Logged,
}

/// A batch after the cap, plus the warning to log, if one is due.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admitted {
    /// The batch to record: clamped, then trimmed to the cap.
    pub batch: HttpUsageBatch,
    /// Set when the batch was trimmed and its window has not warned yet.
    pub warning: Option<Overage>,
}

/// How far a trimmed batch went over the cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overage {
    /// Direction swipes the batch reported, after clamping.
    pub sent: u32,
    /// Direction swipes kept.
    pub kept: u32,
    /// Signal decisions the batch reported, after clamping.
    pub decisions_sent: u32,
    /// Signal decisions kept.
    pub decisions_kept: u32,
}

impl SwipeCap {
    /// An empty cap with no tracked accounts.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clamps `batch`, trims its swipes to what `user_id` has left at `now`,
    /// and charges the account's window with what it keeps.
    ///
    /// A window opens on an account's first batch and resets once
    /// [`WINDOW`] has passed since it opened.
    #[must_use]
    pub fn admit(&self, user_id: Uuid, batch: &HttpUsageBatch, now: Instant) -> Admitted {
        let batch = batch.clamped();
        let admitted = {
            let mut window = self
                .windows
                .entry(user_id)
                .or_insert_with(|| Window::open(now));
            if window.expired(now) {
                *window = Window::open(now);
            }
            let allowance = HttpUsageBatch::MAX_SWIPES_PER_MINUTE
                .saturating_sub(window.counted)
                .min(HttpUsageBatch::MAX_SWIPES_PER_FLUSH);
            let trim = trimmed(batch, allowance);
            window.counted = window.counted.saturating_add(trim.charged());
            let warning = if trim.dropped_any() && window.warning == Warning::Due {
                window.warning = Warning::Logged;
                Some(trim.overage)
            } else {
                None
            };
            Admitted {
                batch: trim.batch,
                warning,
            }
        };
        if self.windows.len() > PRUNE_ABOVE {
            self.windows.retain(|_, window| !window.expired(now));
        }
        admitted
    }
}

/// A batch trimmed to an allowance, with the totals before and after.
#[derive(Debug)]
struct Trim {
    /// The trimmed batch.
    batch: HttpUsageBatch,
    /// Totals before and after trimming.
    overage: Overage,
}

impl Trim {
    /// Swipes the window is charged with: the larger kept total of the two
    /// views, so a batch that reports swipes in only one view still counts.
    fn charged(&self) -> u32 {
        self.overage.kept.max(self.overage.decisions_kept)
    }

    /// Whether trimming dropped anything.
    fn dropped_any(&self) -> bool {
        self.overage.kept < self.overage.sent
            || self.overage.decisions_kept < self.overage.decisions_sent
    }
}

/// Trims `batch`'s direction counters and signal decisions to `allowance`
/// each, in the order the module docs give.
fn trimmed(batch: HttpUsageBatch, allowance: u32) -> Trim {
    let sent = [
        batch.swipes_right,
        batch.swipes_left,
        batch.swipes_up,
        batch.swipes_down,
    ]
    .into_iter()
    .fold(0_u32, u32::saturating_add);

    let mut budget = allowance;
    let swipes_right = spend(&mut budget, batch.swipes_right);
    let swipes_left = spend(&mut budget, batch.swipes_left);
    let swipes_up = spend(&mut budget, batch.swipes_up);
    let swipes_down = spend(&mut budget, batch.swipes_down);
    let kept = allowance.saturating_sub(budget);

    let decisions_sent = batch
        .signals
        .iter()
        .map(card_decisions)
        .chain(batch.select_signals.iter().map(select_decisions))
        .fold(0_u32, u32::saturating_add);

    let mut budget = allowance;
    let signals: Vec<CardSignalDelta> = batch
        .signals
        .iter()
        .filter_map(|signal| trimmed_card_signal(signal, &mut budget))
        .collect();
    let select_signals: Vec<CommanderSelectDelta> = batch
        .select_signals
        .iter()
        .filter_map(|signal| trimmed_select_signal(signal, &mut budget))
        .collect();
    let decisions_kept = allowance.saturating_sub(budget);

    Trim {
        batch: HttpUsageBatch {
            swipes_right,
            swipes_left,
            swipes_up,
            swipes_down,
            signals,
            select_signals,
            ..batch
        },
        overage: Overage {
            sent,
            kept,
            decisions_sent,
            decisions_kept,
        },
    }
}

/// Takes up to `want` from `budget` and returns what it took.
fn spend(budget: &mut u32, want: u32) -> u32 {
    let taken = want.min(*budget);
    *budget = budget.saturating_sub(taken);
    taken
}

/// Swipe decisions in a card signal: adds, skips and maybes.
fn card_decisions(signal: &CardSignalDelta) -> u32 {
    signal
        .added
        .saturating_add(signal.skipped)
        .saturating_add(signal.maybed)
}

/// Swipe decisions in a commander-select signal: selections and skips.
fn select_decisions(signal: &CommanderSelectDelta) -> u32 {
    signal.selected.saturating_add(signal.skipped)
}

/// Spends `budget` on a card signal's decisions. Returns `None` when the
/// signal loses every decision and has no removals to report.
fn trimmed_card_signal(signal: &CardSignalDelta, budget: &mut u32) -> Option<CardSignalDelta> {
    let before = card_decisions(signal);
    let trimmed = CardSignalDelta {
        added: spend(budget, signal.added),
        skipped: spend(budget, signal.skipped),
        maybed: spend(budget, signal.maybed),
        ..*signal
    };
    let after = card_decisions(&trimmed);
    if before > 0 && after == 0 && trimmed.removed == 0 {
        return None;
    }
    Some(CardSignalDelta {
        shown: trimmed.shown.saturating_sub(before.saturating_sub(after)),
        ..trimmed
    })
}

/// Spends `budget` on a commander-select signal's decisions. Returns `None`
/// when the signal loses every decision.
fn trimmed_select_signal(
    signal: &CommanderSelectDelta,
    budget: &mut u32,
) -> Option<CommanderSelectDelta> {
    let before = select_decisions(signal);
    let trimmed = CommanderSelectDelta {
        selected: spend(budget, signal.selected),
        skipped: spend(budget, signal.skipped),
        ..*signal
    };
    let after = select_decisions(&trimmed);
    if before > 0 && after == 0 {
        return None;
    }
    Some(CommanderSelectDelta {
        shown: trimmed.shown.saturating_sub(before.saturating_sub(after)),
        ..trimmed
    })
}

#[cfg(test)]
mod tests {
    use zwipe_core::http::contracts::metrics::{ClientErrorReport, DeckSkipDelta};

    use super::*;

    /// A batch with `right` right swipes and some fields the cap leaves alone.
    fn batch(right: u32) -> HttpUsageBatch {
        HttpUsageBatch {
            swipes_right: right,
            searches: 7,
            deck_skips: vec![DeckSkipDelta {
                deck_id: Uuid::new_v4(),
                skipped: vec![Uuid::new_v4()],
                unskipped: vec![],
            }],
            client_errors: vec![ClientErrorReport {
                screen: "deck_edit".to_string(),
                count: 3,
                ..ClientErrorReport::default()
            }],
            ..HttpUsageBatch::default()
        }
    }

    fn card_signal(added: u32, skipped: u32, maybed: u32) -> CardSignalDelta {
        CardSignalDelta {
            card_oracle_id: Uuid::new_v4(),
            deck_id: Some(Uuid::new_v4()),
            shown: added.saturating_add(skipped).saturating_add(maybed),
            added,
            skipped,
            maybed,
            removed: 0,
        }
    }

    #[test]
    fn under_cap_batch_passes_unchanged() {
        let cap = SwipeCap::new();
        let sent = HttpUsageBatch {
            swipes_left: 20,
            swipes_up: 5,
            signals: vec![card_signal(10, 20, 5)],
            select_signals: vec![CommanderSelectDelta {
                commander_oracle_id: Uuid::new_v4(),
                shown: 4,
                selected: 1,
                skipped: 3,
            }],
            ..batch(10)
        };
        let admitted = cap.admit(Uuid::new_v4(), &sent, Instant::now());
        assert_eq!(admitted.batch, sent);
        assert_eq!(admitted.warning, None);
    }

    #[test]
    fn batch_over_flush_cap_is_trimmed_in_field_order() {
        let cap = SwipeCap::new();
        let sent = HttpUsageBatch {
            swipes_left: 50,
            swipes_up: 30,
            swipes_down: 10,
            ..batch(100)
        };
        let admitted = cap.admit(Uuid::new_v4(), &sent, Instant::now());
        let got = &admitted.batch;
        assert_eq!(
            (
                got.swipes_right,
                got.swipes_left,
                got.swipes_up,
                got.swipes_down
            ),
            (100, 20, 0, 0)
        );
        assert_eq!(got.searches, sent.searches);
        assert_eq!(got.deck_skips, sent.deck_skips);
        assert_eq!(got.client_errors, sent.client_errors);
    }

    #[test]
    fn signals_over_flush_cap_are_trimmed_and_emptied_entries_dropped() {
        let cap = SwipeCap::new();
        let removal_only = CardSignalDelta {
            removed: 1,
            ..card_signal(5, 0, 0)
        };
        let sent = HttpUsageBatch {
            signals: vec![
                card_signal(60, 50, 0),
                card_signal(5, 10, 0),
                removal_only,
                card_signal(1, 0, 0),
            ],
            select_signals: vec![CommanderSelectDelta {
                commander_oracle_id: Uuid::new_v4(),
                shown: 2,
                selected: 1,
                skipped: 1,
            }],
            ..batch(0)
        };
        let admitted = cap.admit(Uuid::new_v4(), &sent, Instant::now());
        let signals = &admitted.batch.signals;
        assert_eq!(signals.len(), 3, "{signals:?}");
        assert_eq!(
            (signals[0].added, signals[0].skipped, signals[0].shown),
            (60, 50, 110)
        );
        assert_eq!(
            (signals[1].added, signals[1].skipped, signals[1].shown),
            (5, 5, 10)
        );
        assert_eq!(
            (signals[2].added, signals[2].removed, signals[2].shown),
            (0, 1, 0)
        );
        assert_eq!(admitted.batch.select_signals, Vec::new());
        assert_eq!(
            admitted
                .warning
                .map(|o| (o.decisions_sent, o.decisions_kept)),
            Some((133, 120))
        );
    }

    #[test]
    fn two_batches_in_a_minute_share_the_minute_cap() {
        let cap = SwipeCap::new();
        let user = Uuid::new_v4();
        let start = Instant::now();
        let first = cap.admit(user, &batch(120), start);
        assert_eq!(first.batch.swipes_right, 120);
        let second = cap.admit(user, &batch(100), start + Duration::from_secs(20));
        assert_eq!(second.batch.swipes_right, 100);
        let third = cap.admit(user, &batch(100), start + Duration::from_secs(40));
        assert_eq!(third.batch.swipes_right, 20);
        let fourth = cap.admit(user, &batch(100), start + Duration::from_secs(50));
        assert_eq!(fourth.batch.swipes_right, 0);
        assert_eq!(fourth.batch.searches, 7);
    }

    #[test]
    fn accounts_have_separate_windows() {
        let cap = SwipeCap::new();
        let now = Instant::now();
        let first = Uuid::new_v4();
        assert_eq!(cap.admit(first, &batch(120), now).batch.swipes_right, 120);
        assert_eq!(cap.admit(first, &batch(120), now).batch.swipes_right, 120);
        assert_eq!(
            cap.admit(Uuid::new_v4(), &batch(120), now)
                .batch
                .swipes_right,
            120
        );
    }

    #[test]
    fn a_new_window_resets_the_count() {
        let cap = SwipeCap::new();
        let user = Uuid::new_v4();
        let start = Instant::now();
        let _ = cap.admit(user, &batch(120), start);
        let _ = cap.admit(user, &batch(120), start);
        assert_eq!(
            cap.admit(user, &batch(120), start + Duration::from_secs(59))
                .batch
                .swipes_right,
            0
        );
        let later = start + WINDOW;
        assert_eq!(cap.admit(user, &batch(120), later).batch.swipes_right, 120);
    }

    #[test]
    fn warning_fires_once_per_window() {
        let cap = SwipeCap::new();
        let user = Uuid::new_v4();
        let start = Instant::now();
        let first = cap.admit(user, &batch(200), start);
        assert_eq!(
            first.warning,
            Some(Overage {
                sent: 200,
                kept: 120,
                decisions_sent: 0,
                decisions_kept: 0,
            })
        );
        assert_eq!(cap.admit(user, &batch(200), start).warning, None);
        assert_eq!(
            cap.admit(user, &batch(200), start + Duration::from_secs(30))
                .warning,
            None
        );
        assert!(
            cap.admit(user, &batch(200), start + WINDOW)
                .warning
                .is_some()
        );
    }

    #[test]
    fn huge_counts_are_clamped_before_counting() {
        let cap = SwipeCap::new();
        let sent = HttpUsageBatch {
            swipes_left: u32::MAX,
            ..batch(u32::MAX)
        };
        let admitted = cap.admit(Uuid::new_v4(), &sent, Instant::now());
        assert_eq!(admitted.batch.swipes_right, 120);
        assert_eq!(
            admitted.warning.map(|o| o.sent),
            Some(2 * HttpUsageBatch::MAX_PER_FLUSH)
        );
    }
}
