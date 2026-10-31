//! The app's `.spinner`: one turn every half second, drawn as a quarter-filled circle going round.

use std::time::{Duration, Instant};

/// The four quarters of a turn.
const FRAMES: [&str; 4] = ["◐", "◓", "◑", "◒"];

/// One turn, from the app's `0.5s` spin.
const TURN: Duration = Duration::from_millis(500);

/// The spinner's glyph at `now` for one that started at `since`.
pub(crate) fn frame(since: Instant, now: Instant) -> &'static str {
    let quarter = TURN.as_millis() / 4;
    let step = now.saturating_duration_since(since).as_millis() / quarter.max(1);
    let index = usize::try_from(step % 4).unwrap_or(0);
    FRAMES.get(index).copied().unwrap_or("◐")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_turn_takes_half_a_second() {
        let since = Instant::now();
        assert_eq!(frame(since, since), "◐");
        assert_eq!(frame(since, since + Duration::from_millis(125)), "◓");
        assert_eq!(frame(since, since + TURN), "◐");
    }
}
