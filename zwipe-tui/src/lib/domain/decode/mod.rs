//! The logo's entrance, ported from orangui's `Decode`: a fast wave of noise draws the shape left to right out of blank, and a slower decode wave follows, turning the noise into the art. On the splash the art leaves the same way in reverse.
//!
//! The pass is orangui's `DECODE_MS` of 850 in frames of `DECODE_TICK_MS`, 40: `ceil(850 / 40)`, 22 frames. Frame `k` shows after the `k`th tick, so the art is blank until the first tick and exact from the 22nd.

use std::time::Duration;

/// The time between frames.
pub(crate) const TICK: Duration = Duration::from_millis(40);

/// How long the decode pass runs, from orangui's `DECODE_MS`.
const DECODE_MS: u128 = 850;

/// How long the art takes to leave a splash, from orangui's `EXIT_MS`.
const EXIT_MS: u128 = 450;

/// How long until the art is settled: the last of the decode pass's frames.
pub(crate) const DURATION: Duration = Duration::from_millis(880);

/// How long until the leaving art is blank.
#[cfg(test)]
pub(crate) const EXIT: Duration = Duration::from_millis(480);

/// Columns the decode front runs past the end, so the last ones get their wobble.
const OVERRUN: usize = 4;

/// The share of the pass, in percent, the noise wave takes to cover the art.
const NOISE_PCT: usize = 45;

/// How far into the pass, in percent, the decode wave starts.
const DECODE_DELAY_PCT: usize = 35;

/// The glyphs a column shows before it resolves.
const NOISE: [char; 5] = ['░', '▒', '▓', '▎', '▍'];

/// `art` as it looks `elapsed` into its entrance: blank before the first frame, and `art` itself from [`DURATION`] on.
pub(crate) fn at(art: &str, elapsed: Duration) -> String {
    let frames = frame_count(DECODE_MS);
    let frame = frame_at(elapsed);
    if frame == 0 {
        return blank(art);
    }
    if frame >= frames {
        return art.to_owned();
    }
    let (noise, decode) = fronts(columns_of(art), frame, frames);
    decoded(art, decode, noise, u64::try_from(frame).unwrap_or(0))
}

/// `art` as it looks `elapsed` into leaving a splash: the two waves in reverse, a fast one turning the art to noise and a slower one clearing it, blank from [`EXIT`] on.
pub(crate) fn leaving(art: &str, elapsed: Duration) -> String {
    let frames = frame_count(EXIT_MS);
    let frame = frame_at(elapsed);
    if frame == 0 {
        return art.to_owned();
    }
    if frame >= frames {
        return blank(art);
    }
    let (noise, clear) = fronts(columns_of(art), frame, frames);
    encoded(art, noise, clear, u64::try_from(frame).unwrap_or(0))
}

/// How many frames a pass of `ms` takes: `ceil(ms / 40)`.
fn frame_count(ms: u128) -> usize {
    usize::try_from(ms.div_ceil(TICK.as_millis()))
        .unwrap_or(1)
        .max(1)
}

/// The frame showing `elapsed` into a pass: one more after each tick.
fn frame_at(elapsed: Duration) -> usize {
    usize::try_from(elapsed.as_millis() / TICK.as_millis()).unwrap_or(usize::MAX)
}

/// `art` with every glyph a space, keeping its shape.
fn blank(art: &str) -> String {
    art.chars()
        .map(|c| if c == '\n' { c } else { ' ' })
        .collect()
}

/// Where the two waves stand at `frame` of `frames`: the noise front, which covers the art in the first [`NOISE_PCT`] of the pass, and the decode front, which starts at [`DECODE_DELAY_PCT`] and runs past the end by the finish.
fn fronts(columns: usize, frame: usize, frames: usize) -> (usize, usize) {
    let pct = frame.saturating_mul(100);
    let span = columns.saturating_add(1);
    let noise = span
        .saturating_mul(pct)
        .checked_div(frames.saturating_mul(NOISE_PCT))
        .unwrap_or(span)
        .min(span);
    let decode = columns
        .saturating_add(OVERRUN)
        .saturating_mul(pct.saturating_sub(frames.saturating_mul(DECODE_DELAY_PCT)))
        .checked_div(frames.saturating_mul(100_usize.saturating_sub(DECODE_DELAY_PCT)))
        .unwrap_or(0);
    (noise, decode)
}

fn columns_of(art: &str) -> usize {
    art.lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

/// `art` with the columns behind the decode `front` resolved, noise up to the `noise` front, and blank past it. Each glyph's reach wobbles by up to three columns, so the fronts are ragged.
fn decoded(art: &str, front: usize, noise: usize, seed: u64) -> String {
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut out = String::with_capacity(art.len());
    for line in art.lines() {
        for (column, glyph) in line.chars().enumerate() {
            if glyph == ' ' {
                out.push(' ');
                continue;
            }
            let wobble = usize::try_from(next() % 4).unwrap_or(0);
            let reach = column.saturating_add(wobble);
            if reach < front {
                out.push(glyph);
            } else if reach >= noise {
                out.push(' ');
            } else {
                let pick = usize::try_from(next() % 5).unwrap_or(0);
                out.push(NOISE.get(pick).copied().unwrap_or('░'));
            }
        }
        out.push('\n');
    }
    out
}

/// `art` on its way out: spaces behind the `clear` front, noise from there to the `noise` front, and the art past that, the edges ragged by the same wobble.
fn encoded(art: &str, noise: usize, clear: usize, seed: u64) -> String {
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut out = String::with_capacity(art.len());
    for line in art.lines() {
        for (column, glyph) in line.chars().enumerate() {
            if glyph == ' ' {
                out.push(' ');
                continue;
            }
            let wobble = usize::try_from(next() % 4).unwrap_or(0);
            let reach = column.saturating_add(wobble);
            if reach < clear {
                out.push(' ');
            } else if reach < noise {
                let pick = usize::try_from(next() % 5).unwrap_or(0);
                out.push(NOISE.get(pick).copied().unwrap_or('░'));
            } else {
                out.push(glyph);
            }
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::logo;

    use super::*;

    fn glyphs(text: &str) -> impl Iterator<Item = char> + '_ {
        text.chars().filter(|c| *c != '\n')
    }

    #[test]
    fn it_starts_blank_and_ends_on_the_art_after_twenty_two_frames() {
        assert_eq!(frame_count(DECODE_MS), 22);
        assert_eq!(TICK * 22, DURATION);
        assert!(glyphs(&at(logo::ZWIPE, Duration::ZERO)).all(|c| c == ' '));
        assert!(glyphs(&at(logo::ZWIPE, TICK - Duration::from_millis(1))).all(|c| c == ' '));
        assert_ne!(
            at(logo::ZWIPE, DURATION - TICK),
            logo::ZWIPE,
            "frame 21 still moves"
        );
        assert_eq!(at(logo::ZWIPE, DURATION), logo::ZWIPE);
        assert_eq!(at(logo::ZWIPE, Duration::from_secs(5)), logo::ZWIPE);
    }

    #[test]
    fn leaving_runs_the_art_to_noise_to_blank() {
        assert_eq!(frame_count(EXIT_MS), 12);
        assert_eq!(leaving(logo::ZWIPE, Duration::ZERO), logo::ZWIPE);
        let middle = leaving(logo::ZWIPE, TICK * 6);
        assert!(glyphs(&middle).any(|c| NOISE.contains(&c)), "{middle}");
        assert!(glyphs(&leaving(logo::ZWIPE, EXIT)).all(|c| c == ' '));
    }

    #[test]
    fn every_frame_keeps_the_arts_shape_and_shows_only_art_noise_or_blank() {
        let art: Vec<Vec<char>> = logo::ZWIPE.lines().map(|l| l.chars().collect()).collect();
        let mut tick = Duration::ZERO;
        while tick < DURATION {
            let frame = at(logo::ZWIPE, tick);
            for (line, original) in frame.lines().zip(&art) {
                assert_eq!(
                    line.chars().count(),
                    original.len(),
                    "lines keep their width"
                );
                for (c, o) in line.chars().zip(original) {
                    assert!(
                        c == ' ' || c == *o || NOISE.contains(&c),
                        "{c:?} at {tick:?}"
                    );
                    if *o == ' ' {
                        assert_eq!(c, ' ', "blank columns stay blank");
                    }
                }
            }
            tick += TICK;
        }
    }

    #[test]
    fn noise_comes_before_the_art_resolves() {
        let early = at(logo::ZWIPE, TICK * 6);
        assert!(glyphs(&early).any(|c| NOISE.contains(&c)), "{early}");
        let late = at(logo::ZWIPE, DURATION - TICK);
        let resolved = glyphs(&late)
            .filter(|c| !NOISE.contains(c) && *c != ' ')
            .count();
        assert!(
            resolved
                > glyphs(&early)
                    .filter(|c| !NOISE.contains(c) && *c != ' ')
                    .count()
        );
    }
}
