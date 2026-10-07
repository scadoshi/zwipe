//! The entrance: how a page's headline pieces arrive.
//!
//! A number counts up from zero ([`CountUp`]), a formatted figure rolls its
//! digits into place ([`Figure`]), block-glyph art resolves from noise left to
//! right ([`Decode`]), and all of them run again whenever the page's
//! [`Replay`] counter is bumped, which the nav logo does on a click. A page
//! that provides no counter gets each piece once, on mount.
//!
//! Everything is prerendered settled, so the page reads right before any
//! script runs, and the motion only ever starts from what is already there.
//! A viewer who asks for less motion gets the settled value at once.

use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// How many times the page has been asked to run its entrance again. The nav
/// logo bumps it; every `CountUp` and `Decode` under it starts over.
#[derive(Clone, Copy)]
pub struct Replay(pub Signal<u32>);

/// The page's replay counter, or a private one that never moves when the page
/// provides none.
fn use_replay() -> Signal<u32> {
    let own = use_signal(|| 0u32);
    try_consume_context::<Replay>().map_or(own, |replay| replay.0)
}

/// Whether the viewer has asked for less motion. Asked of the page, so it
/// holds in the app's WebView too, through the `dioxus.send` channel the app
/// reads its other measurements over; `false` when nothing answers.
async fn reduced_motion() -> bool {
    let mut eval = document::eval(
        "dioxus.send(window.matchMedia('(prefers-reduced-motion: reduce)').matches);",
    );
    eval.recv::<bool>().await.unwrap_or(false)
}

/// Browser `setTimeout` as a future on the web, tokio's timer elsewhere.
pub(crate) async fn sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(ms).await;
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(std::time::Duration::from_millis(u64::from(ms))).await;
}

/// How long a number takes to reach its value, and the tick between frames.
const COUNT_MS: f64 = 1000.0;
const COUNT_TICK_MS: u32 = 16;

/// A number that rolls to its value: random digits inside a window that
/// narrows to nothing, printed with thousands separators. It never shows a
/// blank or a climb from zero.
///
/// `None` is a figure that has not arrived. It rolls on, for as long as that
/// takes, and lands when the value turns up; a value that changes mid-roll
/// becomes the new target the same way.
#[component]
pub fn CountUp(
    value: Option<u64>,
    /// Print `28.0k` and `1.2m` rather than every digit, for a figure in a
    /// box too narrow for all of them. Below ten thousand it prints in full.
    #[props(default)]
    compact: bool,
) -> Element {
    let replay = use_replay();
    // Its own seed, so three figures waiting side by side do not roll the
    // same digits in step.
    let seed = use_hook(|| {
        SEEDS
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_mul(2_654_435_761)
    });
    let mut target = use_signal(|| value);
    let mut shown = use_signal(|| value);
    use_effect(use_reactive!(|value| target.set(value)));
    use_effect(move || {
        let run = replay();
        spawn(async move {
            if reduced_motion().await {
                shown.set(*target.peek());
                return;
            }
            // Nothing to land on yet: roll at a fixed width until there is.
            let mut tick = 0u64;
            while target.peek().is_none() {
                sleep_ms(COUNT_TICK_MS).await;
                if replay.peek().ne(&run) {
                    return;
                }
                tick = tick.wrapping_add(1);
                shown.set(Some(waiting(noise(seed.wrapping_add(tick)))));
            }
            let frames = (COUNT_MS / f64::from(COUNT_TICK_MS)).ceil();
            let mut frame = 0.0;
            while frame < frames {
                sleep_ms(COUNT_TICK_MS).await;
                // A later replay owns the number now.
                if replay.peek().ne(&run) {
                    return;
                }
                frame += 1.0;
                let t = (frame / frames).min(1.0);
                let Some(value) = *target.peek() else {
                    // It went back to waiting; the next effect run picks it up.
                    return;
                };
                shown.set(Some(rolling(value, t, frame as u64)));
            }
            shown.set(*target.peek());
        });
    });
    let print = if compact { short } else { with_separators };
    rsx! {
        match shown() {
            Some(number) => rsx! { "{print(number)}" },
            // Before any frame runs there is nothing honest to print.
            None => rsx! { "\u{2007}\u{2007}\u{2007}\u{2007}" },
        }
    }
}

/// Digits for a figure that has not arrived: four of them, so the width holds
/// still while it waits.
fn waiting(seed: u64) -> u64 {
    1_000 + seed % 9_000
}

/// Hands each `CountUp` its own starting seed.
static SEEDS: AtomicU64 = AtomicU64::new(1);

/// `value` with a random offset that shrinks to nothing as `fraction` reaches
/// 1, held inside the digit count `value` prints at so the row never widens
/// mid-roll. Counts are far below 2^53, so the float conversion is exact.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn rolling(value: u64, fraction: f64, seed: u64) -> u64 {
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction >= 1.0 || value == 0 {
        return value;
    }
    // Squared, so the digits are wild early and barely move at the end.
    let spread = (1.0 - fraction).powi(2);
    // -1.0 to 1.0 from the same xorshift the decode uses.
    let swing = (noise(seed) % 2001) as f64 / 1000.0 - 1.0;
    let rolled = (value as f64) + (value as f64) * spread * swing;
    (rolled.max(0.0).round() as u64).min(digit_ceiling(value))
}

/// The largest number of the same digit count: 72 gives 99, 3,960 gives 9,999.
fn digit_ceiling(value: u64) -> u64 {
    let mut ceiling = 9;
    while ceiling < value {
        ceiling = ceiling.saturating_mul(10).saturating_add(9);
    }
    ceiling
}

/// One xorshift round, enough randomness for digits that flicker.
fn noise(seed: u64) -> u64 {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

/// 12345 -> "12,345"
pub fn with_separators(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

/// `n` to one decimal with a magnitude letter from ten thousand up: `28.0k`,
/// `1.2m`, `3.4b`. Under that, [`with_separators`]. Truncated, not rounded,
/// so a figure never reads past what it is.
pub fn short(n: u64) -> String {
    const STEPS: [(u64, char); 3] = [(1_000_000_000, 'b'), (1_000_000, 'm'), (1_000, 'k')];
    if n < 10_000 {
        return with_separators(n);
    }
    for (unit, letter) in STEPS {
        if n >= unit {
            let tenths = n * 10 / unit;
            return format!("{}.{}{letter}", tenths / 10, tenths % 10);
        }
    }
    with_separators(n)
}

/// A figure the caller has already formatted (`12.3k`, `1,234.6`, `87%`,
/// `+606`) whose digits roll into place: every digit flickers through random
/// ones and settles over `COUNT_MS`, while the separators, the sign, the unit
/// and anything else stay put, so the width never moves. Rolls on mount, on
/// every replay, and again whenever `text` changes, which is what makes a tap
/// on a counter visibly move its figure.
///
/// `start` holds the roll until it reads true: a card below the fold passes
/// the signal its reveal flips, so its numbers roll as it scrolls into view
/// rather than unseen at mount. A viewer who asks for less motion gets the
/// settled text at once.
#[component]
pub fn Figure(text: String, #[props(default)] start: Option<Signal<bool>>) -> Element {
    let replay = use_replay();
    let seed = use_hook(|| {
        SEEDS
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_mul(2_654_435_761)
    });
    let mut target = use_signal(|| text.clone());
    let mut shown = use_signal(|| text.clone());
    use_effect(use_reactive!(|text| target.set(text)));
    use_effect(move || {
        let run = replay();
        let goal = target();
        // Reading it here subscribes the effect, so the flip to true runs it.
        let go = start.is_none_or(|start| start());
        if !go {
            return;
        }
        spawn(async move {
            if reduced_motion().await {
                shown.set(goal);
                return;
            }
            let frames = (COUNT_MS / f64::from(COUNT_TICK_MS)).ceil();
            let mut frame = 0.0;
            while frame < frames {
                sleep_ms(COUNT_TICK_MS).await;
                if replay.peek().ne(&run) || target.peek().ne(&goal) {
                    // A newer roll owns the figure now.
                    return;
                }
                frame += 1.0;
                let t = (frame / frames).min(1.0);
                shown.set(scrambled(&goal, t, seed.wrapping_add(frame as u64)));
            }
            shown.set(goal);
        });
    });
    rsx! { "{shown()}" }
}

/// `text` with each digit replaced by a random one with a probability that
/// falls to zero as `fraction` reaches 1, squared so the figure is wild early
/// and barely moves at the end. Everything that is not a digit is left alone.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn scrambled(text: &str, fraction: f64, seed: u64) -> String {
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction >= 1.0 {
        return text.to_string();
    }
    let spread = (1.0 - fraction).powi(2);
    text.chars()
        .enumerate()
        .map(|(i, c)| {
            if !c.is_ascii_digit() {
                return c;
            }
            let roll = noise(seed.wrapping_add(i as u64 * 7919));
            if (roll % 1000) as f64 / 1000.0 < spread {
                char::from(b'0' + (roll / 1000 % 10) as u8)
            } else {
                c
            }
        })
        .collect()
}

/// How long the glyphs take to settle, and the tick between frames.
const DECODE_MS: u64 = 700;
const DECODE_TICK_MS: u32 = 40;
/// Columns the front runs past the end, so the last ones get their wobble.
const OVERRUN: usize = 4;
/// The shimmer: how many columns the travelling band covers, and how far it
/// moves each frame.
const SHIMMER_SPAN: usize = 10;
const SHIMMER_STEP: usize = 2;
/// The glyphs a column shows before it resolves.
const NOISE: [char; 5] = ['░', '▒', '▓', '▎', '▍'];

/// Block-glyph art that resolves left to right from noise. Spaces stay
/// spaces throughout, so the shape is there from the first frame and only
/// the texture changes.
///
/// `hover` is a signal the caller sets from the element around this one.
/// While it is true the art keeps resolving, pass after pass, and settles on
/// the pass that finishes after it goes false.
#[component]
pub fn Decode(text: &'static str, #[props(default)] hover: Option<Signal<bool>>) -> Element {
    let replay = use_replay();
    let mut shown = use_signal(|| text.to_string());
    use_effect(move || {
        let run = replay();
        spawn(async move {
            if reduced_motion().await {
                shown.set(text.to_string());
                return;
            }
            resolve(text, shown, || replay.peek().ne(&run)).await;
        });
    });
    // Under the pointer a band of noise travels across the art, over and over,
    // so it shimmers rather than dissolving and resolving in turns.
    use_effect(move || {
        if !hover.is_some_and(|hover| hover()) {
            return;
        }
        spawn(async move {
            if reduced_motion().await {
                return;
            }
            let columns = columns_of(text);
            let lap = columns + SHIMMER_SPAN;
            let mut center = 0;
            while hover.is_some_and(|hover| *hover.peek()) {
                sleep_ms(DECODE_TICK_MS).await;
                center = (center + SHIMMER_STEP) % lap.max(1);
                shown.set(shimmered(text, center, u64::try_from(center).unwrap_or(0)));
            }
            shown.set(text.to_string());
        });
    });
    rsx! { "{shown()}" }
}

/// One pass of the front across `text`, writing each frame into `shown`.
/// Stops early when `give_up` says someone else owns the art now.
async fn resolve(text: &'static str, mut shown: Signal<String>, give_up: impl Fn() -> bool) {
    let frames = DECODE_MS.div_ceil(u64::from(DECODE_TICK_MS));
    // The columns resolve on a front that runs across the art.
    let columns = columns_of(text);
    for frame in 1..=frames {
        sleep_ms(DECODE_TICK_MS).await;
        if give_up() {
            return;
        }
        let front = usize::try_from(frame).unwrap_or(usize::MAX) * (columns + OVERRUN)
            / usize::try_from(frames).unwrap_or(1);
        shown.set(decoded(text, front, frame));
    }
    shown.set(text.to_string());
}

/// The widest line in `text`, in characters.
fn columns_of(text: &str) -> usize {
    text.lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

/// `text` with the band of columns ending at `center` swapped for noise, and
/// everything else as it is.
fn shimmered(text: &str, center: usize, seed: u64) -> String {
    let mut out = String::with_capacity(text.len());
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    for line in text.lines() {
        for (column, glyph) in line.chars().enumerate() {
            let inside = column <= center && column + SHIMMER_SPAN >= center;
            if glyph == ' ' || !inside {
                out.push(glyph);
                continue;
            }
            state = noise(state);
            let pick = usize::try_from(state % 5).unwrap_or(0);
            out.push(NOISE.get(pick).copied().unwrap_or('░'));
        }
        out.push('\n');
    }
    out
}

/// `text` with every glyph right of `front` swapped for noise. The noise
/// changes with `seed` so it flickers between frames; the few columns just
/// behind the front wobble too, so the edge is ragged rather than ruled.
fn decoded(text: &str, front: usize, seed: u64) -> String {
    let mut out = String::with_capacity(text.len());
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut next = move || {
        // xorshift, enough randomness for a flicker.
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for line in text.lines() {
        for (column, glyph) in line.chars().enumerate() {
            if glyph == ' ' {
                out.push(' ');
                continue;
            }
            let wobble = usize::try_from(next() % 4).unwrap_or(0);
            if column + wobble < front {
                out.push(glyph);
            } else {
                let pick = usize::try_from(next() % 5).unwrap_or(0);
                out.push(NOISE.get(pick).copied().unwrap_or('░'));
            }
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrambled_keeps_everything_but_the_digits_and_lands_on_the_text() {
        assert_eq!(scrambled("12.3k", 1.0, 7), "12.3k");
        for frame in 1..60u64 {
            let out = scrambled(
                "1,234.6/day",
                f64::from(u32::try_from(frame).unwrap()) / 60.0,
                frame,
            );
            assert_eq!(out.len(), "1,234.6/day".len());
            for (a, b) in out.chars().zip("1,234.6/day".chars()) {
                if b.is_ascii_digit() {
                    assert!(a.is_ascii_digit(), "{out}");
                } else {
                    assert_eq!(a, b, "{out}");
                }
            }
        }
        assert_eq!(scrambled("goal met", 0.1, 3), "goal met");
        // Early on, something moves.
        assert_ne!(scrambled("123,456,789", 0.05, 11), "123,456,789");
    }

    #[test]
    fn short_keeps_small_numbers_whole_and_shortens_the_rest() {
        assert_eq!(short(2_065), "2,065");
        assert_eq!(short(9_999), "9,999");
        assert_eq!(short(10_000), "10.0k");
        assert_eq!(short(28_018), "28.0k");
        assert_eq!(short(123_456), "123.4k");
        assert_eq!(short(244_855), "244.8k");
        assert_eq!(short(999_999), "999.9k");
        assert_eq!(short(1_234_567), "1.2m");
        assert_eq!(short(123_456_789), "123.4m");
        assert_eq!(short(2_500_000_000), "2.5b");
    }

    #[test]
    fn the_roll_lands_on_its_value_and_stays_the_same_width() {
        assert_eq!(rolling(3_657, 1.0, 7), 3_657, "the end is the value");
        assert_eq!(rolling(3_657, 1.5, 7), 3_657);
        assert_eq!(rolling(0, 0.0, 7), 0, "nothing to roll");
        for frame in 1..200u64 {
            let shown = rolling(
                3_657,
                f64::from(u32::try_from(frame).unwrap()) / 200.0,
                frame,
            );
            assert!(shown <= 9_999, "{shown} keeps four digits");
        }
    }

    #[test]
    fn the_ceiling_is_the_largest_number_of_the_same_width() {
        assert_eq!(digit_ceiling(7), 9);
        assert_eq!(digit_ceiling(72), 99);
        assert_eq!(digit_ceiling(3_960), 9_999);
        assert_eq!(digit_ceiling(0), 9);
    }

    #[test]
    fn separators_every_three_digits() {
        assert_eq!(with_separators(0), "0");
        assert_eq!(with_separators(999), "999");
        assert_eq!(with_separators(1_000), "1,000");
        assert_eq!(with_separators(1_234_567), "1,234,567");
    }

    #[test]
    fn a_front_past_the_end_is_the_text_and_at_the_start_is_noise() {
        let art = "▎▉▉\n ▉ \n";
        assert_eq!(decoded(art, 100, 1), art);
        let noisy = decoded(art, 0, 1);
        assert_eq!(
            noisy.chars().count(),
            art.chars().count(),
            "same glyph count"
        );
        assert_eq!(noisy.chars().nth(4), Some(' '), "spaces stay spaces");
        assert!(
            noisy
                .chars()
                .all(|c| c == ' ' || c == '\n' || NOISE.contains(&c))
        );
    }
}
