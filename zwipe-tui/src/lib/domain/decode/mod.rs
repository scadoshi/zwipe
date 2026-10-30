//! The logo's entrance, ported from orangui's `Decode`: a fast wave of noise draws the shape left to right out of blank, and a slower decode wave follows, turning the noise into the art.

use std::time::Duration;

/// How long the art takes to settle.
pub(crate) const DURATION: Duration = Duration::from_millis(850);

/// The time between frames.
pub(crate) const TICK: Duration = Duration::from_millis(40);

/// Columns the decode front runs past the end, so the last ones get their wobble.
const OVERRUN: usize = 4;

/// The share of the pass, in percent, the noise wave takes to cover the art.
const NOISE_PCT: usize = 45;

/// How far into the pass, in percent, the decode wave starts.
const DECODE_DELAY_PCT: usize = 35;

/// The glyphs a column shows before it resolves.
const NOISE: [char; 5] = ['░', '▒', '▓', '▎', '▍'];

/// `art` as it looks `elapsed` into its entrance. From [`DURATION`] on, it is `art` itself.
pub(crate) fn at(art: &str, elapsed: Duration) -> String {
    let frames = frame_count();
    let frame = usize::try_from(elapsed.as_millis() / TICK.as_millis()).unwrap_or(usize::MAX);
    if frame >= frames {
        return art.to_owned();
    }
    let (noise, decode) = fronts(columns_of(art), frame, frames);
    decoded(art, decode, noise, u64::try_from(frame).unwrap_or(0))
}

fn frame_count() -> usize {
    usize::try_from(DURATION.as_millis() / TICK.as_millis()).unwrap_or(1)
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

#[cfg(test)]
mod tests {
    use zwipe_core::domain::logo;

    use super::*;

    fn glyphs(text: &str) -> impl Iterator<Item = char> + '_ {
        text.chars().filter(|c| *c != '\n')
    }

    #[test]
    fn it_starts_blank_and_ends_on_the_art() {
        assert!(glyphs(&at(logo::ZWIPE, Duration::ZERO)).all(|c| c == ' '));
        assert_eq!(at(logo::ZWIPE, DURATION), logo::ZWIPE);
        assert_eq!(at(logo::ZWIPE, Duration::from_secs(5)), logo::ZWIPE);
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
