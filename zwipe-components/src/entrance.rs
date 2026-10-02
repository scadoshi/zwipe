//! The entrance: how a page's headline pieces arrive.
//!
//! A number counts up from zero ([`CountUp`]), block-glyph art resolves from
//! noise left to right ([`Decode`]), and both run again whenever the page's
//! [`Replay`] counter is bumped, which the nav logo does on a click. A page
//! that provides no counter gets each piece once, on mount.
//!
//! Everything is prerendered settled, so the page reads right before any
//! script runs, and the motion only ever starts from what is already there.
//! A viewer who asks for less motion gets the settled value at once.

use dioxus::prelude::*;

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
async fn sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(ms).await;
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(std::time::Duration::from_millis(u64::from(ms))).await;
}

/// How long a number takes to reach its value, and the tick between frames.
const COUNT_MS: f64 = 1000.0;
const COUNT_TICK_MS: u32 = 16;

/// A number that counts up from zero, easing out so the last digits settle,
/// printed with thousands separators. A value that changes mid-count, a live
/// figure arriving, becomes the new target.
#[component]
pub fn CountUp(value: u64) -> Element {
    let replay = use_replay();
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
                let eased = 1.0 - (1.0 - t).powi(3);
                shown.set(scaled(*target.peek(), eased));
            }
            shown.set(*target.peek());
        });
    });
    rsx! { "{with_separators(shown())}" }
}

/// `value` at `fraction` of the way up, never past it. Counts are far below
/// 2^53, so the float conversion is exact.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn scaled(value: u64, fraction: f64) -> u64 {
    ((value as f64) * fraction.clamp(0.0, 1.0)).round() as u64
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

/// How long the glyphs take to settle, and the tick between frames.
const DECODE_MS: u64 = 700;
const DECODE_TICK_MS: u32 = 40;
/// Columns the front runs past the end, so the last ones get their wobble.
const OVERRUN: usize = 4;
/// The glyphs a column shows before it resolves.
const NOISE: [char; 5] = ['░', '▒', '▓', '▎', '▍'];

/// Block-glyph art that resolves left to right from noise. Spaces stay
/// spaces throughout, so the shape is there from the first frame and only
/// the texture changes.
#[component]
pub fn Decode(text: &'static str) -> Element {
    let replay = use_replay();
    let mut shown = use_signal(|| text.to_string());
    use_effect(move || {
        let run = replay();
        spawn(async move {
            if reduced_motion().await {
                shown.set(text.to_string());
                return;
            }
            let frames = DECODE_MS.div_ceil(u64::from(DECODE_TICK_MS));
            // The columns resolve on a front that runs across the art.
            let columns = text
                .lines()
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0);
            for frame in 1..=frames {
                sleep_ms(DECODE_TICK_MS).await;
                if replay.peek().ne(&run) {
                    return;
                }
                let front = usize::try_from(frame).unwrap_or(usize::MAX) * (columns + OVERRUN)
                    / usize::try_from(frames).unwrap_or(1);
                shown.set(decoded(text, front, frame));
            }
            shown.set(text.to_string());
        });
    });
    rsx! { "{shown()}" }
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
    fn the_number_rises_to_its_value_and_no_further() {
        assert_eq!(scaled(3_657, 0.0), 0);
        assert_eq!(scaled(3_657, 0.5), 1_829);
        assert_eq!(scaled(3_657, 1.0), 3_657);
        assert_eq!(scaled(3_657, 1.5), 3_657);
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
