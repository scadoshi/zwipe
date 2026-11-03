//! A swiped card flying off and an undone card coming back, after the app's `card-stack-exit-*` and `card-stack-enter-*` keyframes.
//!
//! A terminal cannot rotate, so the tilt reads as a flip: while the card slides it narrows toward an edge-on sliver, its width for Left and Right and its height for Up. A terminal has no opacity either, so for the second half the card fades to a dim outline and the card underneath shows whole. An exit takes the app's 0.18 s and an entrance its 0.2 s, both on CSS `ease-out`. Down is undo and has no exit.
//!
//! The geometry is pure: [`place`] turns a phase, a direction, the time since the start and the card's resting [`Rect`] into a [`Placement`]. Drawing renders the card at rest into a snapshot buffer and copies it into the frame through the placement, narrowing it by dropping blank and border-only columns or rows first and then clipping its text, never thinning it, and clipping at the bounds, so the card underneath draws first at rest and the moving card lands over it.
//!
//! A screen keeps an `Option<CardMotion<T>>`. A swipe stores [`CardMotion::exit`] with what it needs to draw the card that left (the stack has already moved past it), an undo stores [`CardMotion::enter`] and draws the stack's own top card, and the screen reports itself animating while [`CardMotion::running`] holds. A new swipe replaces the motion outright.

use std::time::{Duration, Instant};

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use zwipe_core::domain::card::Card;
use zwipe_ui::stack::Direction;

use crate::domain::theme::Palette;

/// How long an exit takes: the app's `card-stack-exit-*` duration.
pub(crate) const EXIT: Duration = Duration::from_millis(180);

/// How long an entrance takes: the app's `card-stack-enter-*` duration.
pub(crate) const ENTER: Duration = Duration::from_millis(200);

/// CSS `ease-out` as `cubic-bezier(x1, y1, x2, y2)`, the curve both animations run on.
const EASE_OUT: [f64; 4] = [0.0, 0.0, 0.58, 1.0];

/// From this far off rest the card draws dim: the app's opacity falls from 1 to 0 on the same progress, so past halfway it is mostly gone.
const DIM_FROM: f64 = 0.5;

/// Which way a motion runs, and for an exit what it carries.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Phase<T> {
    /// The card that left, flying off; the screen draws it from this snapshot, since its stack has moved on.
    Exit(T),
    /// The stack's top card coming back from the side it left by.
    Enter,
}

/// A card in motion: its phase, its direction and when it started.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CardMotion<T> {
    phase: Phase<T>,
    direction: Direction,
    started: Instant,
}

impl<T> CardMotion<T> {
    /// A card leaving by `direction` from `now`, carrying `card` to draw it from.
    pub(crate) fn exit(direction: Direction, card: T, now: Instant) -> Self {
        Self {
            phase: Phase::Exit(card),
            direction,
            started: now,
        }
    }

    /// The top card coming back from `direction`, the side it left by, from `now`.
    pub(crate) fn enter(direction: Direction, now: Instant) -> Self {
        Self {
            phase: Phase::Enter,
            direction,
            started: now,
        }
    }

    pub(crate) fn phase(&self) -> &Phase<T> {
        &self.phase
    }

    pub(crate) fn direction(&self) -> Direction {
        self.direction
    }

    /// How long this motion runs.
    pub(crate) fn duration(&self) -> Duration {
        match self.phase {
            Phase::Exit(_) => EXIT,
            Phase::Enter => ENTER,
        }
    }

    /// Whether the motion is still going at `now`, so the screen should report itself animating.
    pub(crate) fn running(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.started) < self.duration()
    }

    /// Where the card draws at `now`, resting at `rest` and clipped to `bounds`; `None` once it is over or wholly off the bounds.
    pub(crate) fn placement(&self, now: Instant, rest: Rect, bounds: Rect) -> Option<Placement> {
        let kind = match self.phase {
            Phase::Exit(_) => Kind::Exit,
            Phase::Enter => Kind::Enter,
        };
        place(
            kind,
            self.direction,
            now.saturating_duration_since(self.started),
            rest,
            bounds,
        )
    }
}

/// What an exit on a card screen carries: the card that left and the face it showed.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Snapshot {
    pub(crate) card: Card,
    pub(crate) face: usize,
}

/// The border of a card leaving by, or coming back from, `direction`: add for right, error for left, warning for up.
pub(crate) fn side_style(palette: &Palette, direction: Direction) -> Style {
    match direction {
        Direction::Right => palette.add,
        Direction::Left => palette.error,
        Direction::Up => palette.warning,
    }
}

/// A phase without its payload, for [`place`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Exit,
    Enter,
}

/// One frame of a motion: where the card is and how it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Placement {
    /// Where the card rests; the snapshot is drawn here.
    pub(crate) rest: Rect,
    /// The card's left edge this frame, which may lie off the bounds.
    pub(crate) x: i32,
    /// The card's top edge this frame, which may lie off the bounds.
    pub(crate) y: i32,
    /// The card's width this frame, at least 1.
    pub(crate) width: u16,
    /// The card's height this frame, at least 1.
    pub(crate) height: u16,
    /// The part of the card inside the bounds, never empty.
    pub(crate) visible: Rect,
    /// Whether it draws dim, for the second half of the way off.
    pub(crate) dim: bool,
}

/// The frame `elapsed` into a motion of `kind` by `direction`, for a card resting at `rest`, clipped to `bounds`.
///
/// An exit runs from rest until its leading edge meets the far bound, narrowing as it goes; an entrance runs the same path backward. Returns `None` once the motion is over, for an empty `rest`, or when nothing is left inside `bounds`.
pub(crate) fn place(
    kind: Kind,
    direction: Direction,
    elapsed: Duration,
    rest: Rect,
    bounds: Rect,
) -> Option<Placement> {
    let duration = match kind {
        Kind::Exit => EXIT,
        Kind::Enter => ENTER,
    };
    if elapsed >= duration || rest.is_empty() {
        return None;
    }
    let eased = ease_out(elapsed.as_secs_f64() / duration.as_secs_f64());
    // How far off rest the card is: 0 at rest, 1 gone.
    let off = match kind {
        Kind::Exit => eased,
        Kind::Enter => 1.0 - eased,
    };
    let (left, top) = (i32::from(rest.x), i32::from(rest.y));
    let (right, bottom) = (i32::from(rest.right()), i32::from(rest.bottom()));
    let (x, y, width, height) = match direction {
        Direction::Right => {
            let travel = i32::from(bounds.right()).saturating_sub(left).max(0);
            let x = left.saturating_add(cells(f64::from(travel) * off));
            (x, top, narrow(rest.width, off), rest.height)
        }
        Direction::Left => {
            let travel = right.saturating_sub(i32::from(bounds.x)).max(0);
            let width = narrow(rest.width, off);
            let edge = right.saturating_sub(cells(f64::from(travel) * off));
            (
                edge.saturating_sub(i32::from(width)),
                top,
                width,
                rest.height,
            )
        }
        Direction::Up => {
            let travel = bottom.saturating_sub(i32::from(bounds.y)).max(0);
            let height = narrow(rest.height, off);
            let edge = bottom.saturating_sub(cells(f64::from(travel) * off));
            (
                left,
                edge.saturating_sub(i32::from(height)),
                rest.width,
                height,
            )
        }
    };
    let visible = clip(x, y, width, height, bounds)?;
    Some(Placement {
        rest,
        x,
        y,
        width,
        height,
        visible,
        dim: off >= DIM_FROM,
    })
}

/// Draws one frame: renders the card at rest into a snapshot filled with `base` (the screen's background, so blank cells keep the theme), then copies it into `buf` through `placement`.
pub(crate) fn draw(
    buf: &mut Buffer,
    placement: &Placement,
    base: Style,
    render: impl FnOnce(&mut Buffer),
) {
    let mut snapshot = Buffer::empty(placement.rest);
    snapshot.set_style(placement.rest, base);
    render(&mut snapshot);
    blit(buf, &snapshot, placement);
}

/// Copies `snapshot`, drawn at `placement.rest`, into `buf` at the placement's frame. A narrowed card gives up its blank and border-only columns (rows, for Up) first, then clips its text from the trailing end, so its words never lose letters and both edges stay. Past halfway it draws dim and only its border, never over the text of the card underneath, as the app's card fades, so that card shows whole.
pub(crate) fn blit(buf: &mut Buffer, snapshot: &Buffer, placement: &Placement) {
    let rest = placement.rest;
    let visible = placement.visible;
    let columns = squeeze(snapshot, rest, placement.width, Axis::Columns);
    let rows = squeeze(snapshot, rest, placement.height, Axis::Rows);
    let last = (rest.width.saturating_sub(1), rest.height.saturating_sub(1));
    for row in visible.top()..visible.bottom() {
        let Some(&source_y) = offset(row, placement.y).and_then(|at| rows.get(at)) else {
            continue;
        };
        for column in visible.left()..visible.right() {
            let Some(&source_x) = offset(column, placement.x).and_then(|at| columns.get(at)) else {
                continue;
            };
            let edge = source_x == 0 || source_y == 0 || source_x == last.0 || source_y == last.1;
            if placement.dim && !edge {
                continue;
            }
            let from = Position::new(
                rest.x.saturating_add(source_x),
                rest.y.saturating_add(source_y),
            );
            let Some(cell) = snapshot.cell(from).cloned() else {
                continue;
            };
            if let Some(target) = buf.cell_mut(Position::new(column, row)) {
                // A fading outline gives way to what the card underneath says, and crosses its lines.
                if placement.dim && !open(target.symbol()) {
                    continue;
                }
                *target = cell;
                if placement.dim {
                    target.modifier.insert(Modifier::DIM);
                }
            }
        }
    }
}

/// Whether a fading outline may draw over `symbol`: a blank cell or a box-drawing line, never text.
fn open(symbol: &str) -> bool {
    symbol
        .chars()
        .all(|c| c == ' ' || ('\u{2500}'..='\u{257f}').contains(&c))
}

/// How far `at` lies past `start`, as an index, or `None` before it.
fn offset(at: u16, start: i32) -> Option<usize> {
    usize::try_from(i32::from(at).saturating_sub(start)).ok()
}

/// Which way a card narrows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Columns,
    Rows,
}

/// Whether `symbol` is blank or a border line running along `axis`, which a narrowing card can lose without losing a letter.
fn spare(symbol: &str, axis: Axis) -> bool {
    match axis {
        Axis::Columns => matches!(symbol, " " | "" | "─" | "━" | "═"),
        Axis::Rows => matches!(symbol, " " | "" | "│" | "┃" | "║"),
    }
}

/// The source offsets, in order, of a span `drawn` long that shows the `rest` card's columns or rows. Both ends stay. Between them it gives up the blank and border-only lines first, from the trailing end, then clips the rest from the trailing end, so text is cut, never thinned. A one-cell sliver shows the leading edge.
fn squeeze(snapshot: &Buffer, rest: Rect, drawn: u16, axis: Axis) -> Vec<u16> {
    let (full, across) = match axis {
        Axis::Columns => (rest.width, rest.height),
        Axis::Rows => (rest.height, rest.width),
    };
    if drawn >= full {
        return (0..full).collect();
    }
    if drawn <= 1 {
        return if full == 0 { Vec::new() } else { vec![0] };
    }
    let last = full.saturating_sub(1);
    let blank = |line: u16| {
        (0..across).all(|other| {
            let (x, y) = match axis {
                Axis::Columns => (line, other),
                Axis::Rows => (other, line),
            };
            snapshot
                .cell(Position::new(
                    rest.x.saturating_add(x),
                    rest.y.saturating_add(y),
                ))
                .is_none_or(|cell| spare(cell.symbol(), axis))
        })
    };
    let mut kept: Vec<u16> = (1..last).collect();
    let mut drop = full.saturating_sub(drawn);
    // Blank lines go first, the trailing ones before the leading.
    let mut at = kept.len();
    while drop > 0 && at > 0 {
        at = at.saturating_sub(1);
        if kept.get(at).is_some_and(|&line| blank(line)) {
            kept.remove(at);
            drop = drop.saturating_sub(1);
        }
    }
    kept.truncate(kept.len().saturating_sub(usize::from(drop)));
    let mut lines = Vec::with_capacity(usize::from(drawn));
    lines.push(0);
    lines.extend(kept);
    lines.push(last);
    lines
}

/// A span `len` long narrowed `off` of the way toward 1 cell.
fn narrow(len: u16, off: f64) -> u16 {
    let lost = cells(f64::from(len.saturating_sub(1)) * off);
    let lost = u16::try_from(lost.max(0)).unwrap_or(u16::MAX);
    len.saturating_sub(lost).max(1)
}

/// The part of a `width` by `height` frame at `x`, `y` inside `bounds`, or `None` when nothing is.
fn clip(x: i32, y: i32, width: u16, height: u16, bounds: Rect) -> Option<Rect> {
    let left = x.max(i32::from(bounds.left()));
    let top = y.max(i32::from(bounds.top()));
    let right = x
        .saturating_add(i32::from(width))
        .min(i32::from(bounds.right()));
    let bottom = y
        .saturating_add(i32::from(height))
        .min(i32::from(bounds.bottom()));
    if right <= left || bottom <= top {
        return None;
    }
    Some(Rect::new(
        u16::try_from(left).ok()?,
        u16::try_from(top).ok()?,
        u16::try_from(right.saturating_sub(left)).ok()?,
        u16::try_from(bottom.saturating_sub(top)).ok()?,
    ))
}

/// A distance in cells, rounded to the nearest.
fn cells(distance: f64) -> i32 {
    let clamped = distance
        .round()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX));
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "the value is rounded and clamped to i32's range"
    )]
    let whole = clamped as i32;
    whole
}

/// CSS `ease-out` at `t` from 0 to 1: solves the curve's x for `t` by bisection, then reads its y.
fn ease_out(t: f64) -> f64 {
    let [x1, y1, x2, y2] = EASE_OUT;
    let t = t.clamp(0.0, 1.0);
    let bezier = |s: f64, p1: f64, p2: f64| {
        let inverse = 1.0 - s;
        3.0 * inverse * inverse * s * p1 + 3.0 * inverse * s * s * p2 + s * s * s
    };
    let (mut low, mut high) = (0.0_f64, 1.0_f64);
    for _ in 0..32 {
        let middle = f64::midpoint(low, high);
        if bezier(middle, x1, x2) < t {
            low = middle;
        } else {
            high = middle;
        }
    }
    bezier(f64::midpoint(low, high), y1, y2)
}

#[cfg(test)]
mod tests {
    use ratatui::widgets::{Paragraph, Widget};

    use super::*;

    const BOUNDS: Rect = Rect::new(0, 0, 100, 40);
    const REST: Rect = Rect::new(30, 10, 40, 20);

    fn at(kind: Kind, direction: Direction, millis: u64) -> Option<Placement> {
        place(kind, direction, Duration::from_millis(millis), REST, BOUNDS)
    }

    #[test]
    fn ease_out_starts_fast_and_lands_gently() {
        assert!(ease_out(0.0).abs() < 1e-6);
        assert!((ease_out(1.0) - 1.0).abs() < 1e-6);
        let half = ease_out(0.5);
        assert!(
            half > 0.6 && half < 0.75,
            "ease-out is past halfway at half time: {half}"
        );
        let mut last = 0.0;
        for step in 1..=20 {
            let now = ease_out(f64::from(step) / 20.0);
            assert!(now >= last, "monotonic at step {step}");
            last = now;
        }
    }

    #[test]
    fn every_motion_starts_or_ends_at_rest() {
        for direction in [Direction::Left, Direction::Right, Direction::Up] {
            let start = at(Kind::Exit, direction, 0).unwrap();
            assert_eq!(
                (
                    start.x,
                    start.y,
                    start.width,
                    start.height,
                    start.visible,
                    start.dim
                ),
                (30, 10, 40, 20, REST, false),
                "an exit starts at rest: {direction:?}"
            );
            let last = at(Kind::Enter, direction, 199).unwrap();
            assert_eq!(
                last.visible, REST,
                "an entrance lands at rest: {direction:?}"
            );
            assert!(!last.dim);
            assert_eq!(at(Kind::Exit, direction, 180), None, "{direction:?}");
            assert_eq!(at(Kind::Enter, direction, 200), None, "{direction:?}");
        }
    }

    #[test]
    fn right_slides_right_and_narrows() {
        let middle = at(Kind::Exit, Direction::Right, 90).unwrap();
        assert!(middle.x > 30, "it has moved right: {middle:?}");
        assert!(
            middle.width < 40 && middle.width >= 1,
            "it narrows: {middle:?}"
        );
        assert_eq!((middle.y, middle.height), (10, 20), "the height stays");
        assert!(middle.dim, "past halfway off it is dim");
        let late = at(Kind::Exit, Direction::Right, 179);
        assert!(
            late.is_none_or(|p| p.visible.width <= 2 && p.visible.x >= 98),
            "at the end it is a sliver at the edge or gone: {late:?}"
        );
    }

    #[test]
    fn left_slides_left_and_narrows() {
        let middle = at(Kind::Exit, Direction::Left, 90).unwrap();
        assert!(
            i32::from(middle.visible.right()) < 70,
            "it has moved left: {middle:?}"
        );
        assert!(middle.width < 40);
        assert_eq!((middle.y, middle.height), (10, 20));
        let late = at(Kind::Exit, Direction::Left, 179);
        assert!(late.is_none_or(|p| p.visible.right() <= 2), "{late:?}");
    }

    #[test]
    fn up_slides_up_and_flattens() {
        let middle = at(Kind::Exit, Direction::Up, 90).unwrap();
        assert!(middle.y < 10, "it has moved up: {middle:?}");
        assert!(middle.height < 20 && middle.height >= 1, "it flattens");
        assert_eq!((middle.x, middle.width), (30, 40), "the width stays");
        let late = at(Kind::Exit, Direction::Up, 179);
        assert!(late.is_none_or(|p| p.visible.bottom() <= 1), "{late:?}");
    }

    #[test]
    fn an_entrance_runs_the_exit_backward() {
        for direction in [Direction::Left, Direction::Right, Direction::Up] {
            let early = at(Kind::Enter, direction, 10);
            let later = at(Kind::Enter, direction, 100).unwrap();
            let rest_distance = |p: &Placement| {
                (p.x - 30).abs()
                    + (p.y - 10).abs()
                    + i32::from(40 - p.width)
                    + i32::from(20 - p.height)
            };
            if let Some(early) = early {
                assert!(
                    rest_distance(&early) > rest_distance(&later),
                    "{direction:?}"
                );
            }
            assert!(
                rest_distance(&later) > 0,
                "still on its way at 100 ms: {direction:?}"
            );
        }
        let from_left = at(Kind::Enter, Direction::Left, 60).unwrap();
        assert!(
            from_left.x < 30,
            "it comes back from the left: {from_left:?}"
        );
        let from_up = at(Kind::Enter, Direction::Up, 60).unwrap();
        assert!(from_up.y < 10, "it comes back from above: {from_up:?}");
    }

    #[test]
    fn tiny_and_degenerate_areas_never_panic() {
        let rests = [
            Rect::ZERO,
            Rect::new(0, 0, 1, 1),
            Rect::new(0, 0, 1, 5),
            Rect::new(5, 5, 3, 1),
            Rect::new(u16::MAX - 1, u16::MAX - 1, 1, 1),
            Rect::new(10, 10, 5, 5),
        ];
        let bounds = [
            Rect::ZERO,
            Rect::new(0, 0, 1, 1),
            Rect::new(0, 0, 3, 3),
            BOUNDS,
        ];
        for kind in [Kind::Exit, Kind::Enter] {
            for direction in [Direction::Left, Direction::Right, Direction::Up] {
                for rest in rests {
                    for bound in bounds {
                        for millis in [0, 1, 45, 90, 179, 199, 500] {
                            if let Some(p) =
                                place(kind, direction, Duration::from_millis(millis), rest, bound)
                            {
                                assert!(!p.visible.is_empty());
                                assert!(p.width >= 1 && p.height >= 1);
                                let mut buf = Buffer::empty(bound);
                                draw(&mut buf, &p, Style::new(), |b| {
                                    crate::inbound::widgets::panel::bordered(
                                        &crate::domain::theme::Palette::terminal(),
                                    )
                                    .render(rest, b);
                                });
                            }
                        }
                    }
                }
            }
        }
        let empty = Buffer::empty(Rect::ZERO);
        assert_eq!(
            squeeze(&empty, Rect::ZERO, 0, Axis::Columns),
            Vec::<u16>::new()
        );
        assert_eq!(
            squeeze(&empty, Rect::ZERO, 3, Axis::Rows),
            Vec::<u16>::new()
        );
    }

    #[test]
    fn a_motion_stops_running_at_its_end() {
        let start = Instant::now();
        let exit = CardMotion::exit(Direction::Right, "A", start);
        assert!(exit.running(start) && exit.running(start + Duration::from_millis(179)));
        assert!(!exit.running(start + EXIT));
        assert_eq!(exit.placement(start + EXIT, REST, BOUNDS), None);
        let enter: CardMotion<&str> = CardMotion::enter(Direction::Up, start);
        assert!(
            enter.running(start + EXIT),
            "an entrance runs a little longer"
        );
        assert!(!enter.running(start + ENTER));
        assert_eq!(enter.phase(), &Phase::Enter);
        assert_eq!(enter.direction(), Direction::Up);
    }

    /// A bordered card at `rest` with `text` on its first inner row.
    fn card(rest: Rect, text: &str) -> Buffer {
        let mut buf = Buffer::empty(rest);
        Paragraph::new(text.to_owned())
            .block(crate::inbound::widgets::panel::bordered(
                &crate::domain::theme::Palette::terminal(),
            ))
            .render(rest, &mut buf);
        buf
    }

    #[test]
    fn narrowing_keeps_both_edges_at_every_width() {
        for full in [1, 2, 7, 64] {
            let rest = Rect::new(0, 0, full, 4);
            let snapshot = card(rest, "Protean Hydra");
            for drawn in 1..=full {
                let lines = squeeze(&snapshot, rest, drawn, Axis::Columns);
                assert_eq!(lines.len(), usize::from(drawn), "{drawn} of {full}");
                assert_eq!(lines.first(), Some(&0), "{drawn} of {full}");
                let want = if drawn == 1 { 0 } else { full - 1 };
                assert_eq!(lines.last(), Some(&want), "{drawn} of {full}");
                assert!(lines.is_sorted(), "{drawn} of {full}: {lines:?}");
            }
        }
    }

    #[test]
    fn a_narrowed_card_loses_blank_columns_then_clips_its_text_whole() {
        let rest = Rect::new(0, 0, 30, 4);
        let snapshot = card(rest, "Protean Hydra\nHydra Protean");
        let row = |drawn: u16| -> String {
            squeeze(&snapshot, rest, drawn, Axis::Columns)
                .into_iter()
                .map(|x| snapshot.cell((x, 1)).unwrap().symbol().to_owned())
                .collect()
        };
        assert_eq!(row(15), "│Protean Hydra│", "only blank columns went");
        assert_eq!(
            row(10),
            "│Protean │",
            "then the text is clipped, not thinned"
        );
        let rows = Rect::new(0, 0, 10, 8);
        let tall = card(rows, "Hydra");
        assert_eq!(
            squeeze(&tall, rows, 4, Axis::Rows),
            [0, 1, 2, 7],
            "an Up card loses its blank rows first"
        );
    }

    #[test]
    fn past_halfway_the_leaving_card_is_a_dim_outline_over_the_card_under_it() {
        let rest = Rect::new(0, 0, 12, 3);
        let bounds = Rect::new(0, 0, 40, 3);
        let snapshot = card(rest, "Leaving");
        let placement = Placement {
            rest,
            x: 2,
            y: 0,
            width: 8,
            height: 3,
            visible: Rect::new(2, 0, 8, 3),
            dim: true,
        };
        let mut buf = card(bounds, "Under the leaving card");
        blit(&mut buf, &snapshot, &placement);
        let row: String = (0..40)
            .map(|x| buf.cell((x, 1)).unwrap().symbol().to_owned())
            .collect();
        assert!(
            row.starts_with("│Under the leaving card"),
            "the card under it shows whole: {row:?}"
        );
        assert!(!buf.cell((2, 1)).unwrap().modifier.contains(Modifier::DIM));
        let top: String = (0..40)
            .map(|x| buf.cell((x, 0)).unwrap().symbol().to_owned())
            .collect();
        assert!(
            top.starts_with("┌─┌──────┐─"),
            "the outline crosses the lines under it: {top:?}"
        );
        let mut blank = Buffer::empty(bounds);
        blit(&mut blank, &snapshot, &placement);
        assert_eq!(
            blank.cell((2, 1)).unwrap().symbol(),
            "│",
            "an outline over blank cells"
        );
        assert!(blank.cell((2, 1)).unwrap().modifier.contains(Modifier::DIM));
        assert_eq!(
            blank.cell((3, 1)).unwrap().symbol(),
            " ",
            "and nothing inside"
        );
    }

    #[test]
    fn the_snapshot_lands_offset_narrowed_and_dim() {
        let rest = Rect::new(2, 1, 6, 3);
        let bounds = Rect::new(0, 0, 20, 5);
        let mut buf = Buffer::empty(bounds);
        let placement = place(
            Kind::Exit,
            Direction::Right,
            Duration::from_millis(60),
            rest,
            bounds,
        )
        .unwrap();
        draw(&mut buf, &placement, Style::new(), |b| {
            crate::inbound::widgets::panel::bordered(&crate::domain::theme::Palette::terminal())
                .render(rest, b);
        });
        let row: String = (0..20)
            .map(|x| buf.cell((x, 1)).unwrap().symbol().to_owned())
            .collect();
        assert!(
            row.starts_with("  ") && row.contains('┌') && row.contains('┐'),
            "both corners survive the narrowing: {row:?}"
        );
        let corner = row.find('┌').unwrap();
        assert!(corner > 2, "the card has moved right of its rest: {row:?}");
        let dimmed = buf.cell((u16::try_from(corner).unwrap(), 1)).unwrap();
        assert_eq!(dimmed.modifier.contains(Modifier::DIM), placement.dim);
    }
}
