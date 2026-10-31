//! The logo's entrance in time, over orangui-core's `decode` frames: a fast wave of noise draws the shape left to right out of blank, and a slower decode wave follows, turning the noise into the art. On the splash the art leaves the same way in reverse.
//!
//! The pass is orangui's `DURATION_MS` of 850 in frames of `TICK_MS`, 40: `ceil(850 / 40)`, 22 frames. Frame `k` shows after the `k`th tick, so the art is blank until the first tick and exact from the 22nd. The terminal rounds both passes up to whole ticks on purpose.

use std::time::Duration;

use orangui_core::decode;

/// The time between frames, orangui-core's [`decode::TICK_MS`].
pub(crate) const TICK: Duration = Duration::from_millis(40);

/// How long until the art is settled: the last of the decode pass's frames.
pub(crate) const DURATION: Duration = Duration::from_millis(880);

/// How long until the leaving art is blank.
#[cfg(test)]
pub(crate) const EXIT: Duration = Duration::from_millis(480);

/// `art` as it looks `elapsed` into its entrance: blank before the first frame, and `art` itself from [`DURATION`] on.
pub(crate) fn at(art: &str, elapsed: Duration) -> String {
    let frames = decode::frames(u64::from(decode::DURATION_MS));
    let frame = frame_at(elapsed);
    if frame == 0 {
        return decode::blank(art);
    }
    if frame >= frames {
        return art.to_owned();
    }
    decode::entry_frame(art, frame, frames)
}

/// `art` as it looks `elapsed` into leaving a splash: the two waves in reverse, a fast one turning the art to noise and a slower one clearing it, blank from `EXIT` (480 ms) on.
pub(crate) fn leaving(art: &str, elapsed: Duration) -> String {
    let frames = decode::frames(u64::from(decode::EXIT_MS));
    let frame = frame_at(elapsed);
    if frame == 0 {
        return art.to_owned();
    }
    if frame >= frames {
        return decode::blank(art);
    }
    decode::exit_frame(art, frame, frames)
}

/// The frame showing `elapsed` into a pass: one more after each tick.
fn frame_at(elapsed: Duration) -> usize {
    usize::try_from(
        elapsed
            .as_millis()
            .checked_div(TICK.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(usize::MAX)
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
        assert_eq!(decode::frames(u64::from(decode::DURATION_MS)), 22);
        assert_eq!(TICK, Duration::from_millis(u64::from(decode::TICK_MS)));
        assert_eq!(TICK * 22, DURATION);
        assert!(glyphs(&at(logo::ZWIPE, Duration::ZERO)).all(|c| c == ' '));
        assert!(
            glyphs(&at(
                logo::ZWIPE,
                TICK.saturating_sub(Duration::from_millis(1))
            ))
            .all(|c| c == ' ')
        );
        assert_ne!(
            at(logo::ZWIPE, DURATION.saturating_sub(TICK)),
            logo::ZWIPE,
            "frame 21 still moves"
        );
        assert_eq!(at(logo::ZWIPE, DURATION), logo::ZWIPE);
        assert_eq!(at(logo::ZWIPE, Duration::from_secs(5)), logo::ZWIPE);
    }

    #[test]
    fn leaving_runs_the_art_to_noise_to_blank() {
        assert_eq!(decode::frames(u64::from(decode::EXIT_MS)), 12);
        assert_eq!(leaving(logo::ZWIPE, Duration::ZERO), logo::ZWIPE);
        let middle = leaving(logo::ZWIPE, TICK * 6);
        assert!(
            glyphs(&middle).any(|c| decode::NOISE_GLYPHS.contains(&c)),
            "{middle}"
        );
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
                        c == ' ' || c == *o || decode::NOISE_GLYPHS.contains(&c),
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
        assert!(
            glyphs(&early).any(|c| decode::NOISE_GLYPHS.contains(&c)),
            "{early}"
        );
        let late = at(logo::ZWIPE, DURATION.saturating_sub(TICK));
        let resolved = glyphs(&late)
            .filter(|c| !decode::NOISE_GLYPHS.contains(c) && *c != ' ')
            .count();
        assert!(
            resolved
                > glyphs(&early)
                    .filter(|c| !decode::NOISE_GLYPHS.contains(c) && *c != ' ')
                    .count()
        );
    }
}
