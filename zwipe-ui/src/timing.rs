//! How long the screens wait, and how much they ask for, as the app does
//! it. Animation timings stay in each client; these change what a screen
//! does, not how it moves.

use std::time::Duration;

/// The wait after the last keystroke before a command-zone search runs on
/// Create and Edit.
pub const COMMAND_ZONE_SEARCH_DEBOUNCE: Duration = Duration::from_millis(800);

/// The wait after the last keystroke before a quick add search runs, on the
/// deck cards view and the commander maybeboard.
pub const QUICK_ADD_DEBOUNCE: Duration = Duration::from_millis(300);

/// Characters a quick add search needs before it runs.
pub const QUICK_ADD_MIN_CHARS: usize = 2;

/// Cards the deck cards view's quick add offers at most.
pub const QUICK_ADD_LIMIT: u32 = 5;

/// Commanders the commander maybeboard's quick add offers at most.
pub const MAYBEBOARD_QUICK_ADD_LIMIT: u32 = 8;

/// How long quantity taps on one card gather before they are sent as one
/// change.
pub const QUANTITY_FLUSH: Duration = Duration::from_millis(300);

/// How long Swipe select's maybeboard toast stays up, shorter than any band
/// so a run of up swipes doesn't stack them.
pub const MAYBEBOARD_TOAST: Duration = Duration::from_millis(1500);

/// The wait before the verification email can be sent again.
pub const RESEND_COOLDOWN: Duration = Duration::from_secs(60);

/// The countdown before Delete account goes through.
pub const DELETE_COUNTDOWN: Duration = Duration::from_secs(5);

/// How often the session upkeep runs: token refresh, the version gate and
/// the flavor card.
pub const UPKEEP_INTERVAL: Duration = Duration::from_secs(60);

/// How long home's counters stay fresh before a return to home fetches
/// them again.
pub const COUNTS_FRESH: Duration = Duration::from_secs(60);

/// How long Activity's heatmap keeps a picked day's readout before letting
/// it go, orangui's heat pick (`orangui_core::heat::PICK_MS`).
pub const HEAT_PICK: Duration = Duration::from_secs(4);

/// Cards one page of a card search or a swipe deck asks for.
pub const PAGE_SIZE: u32 = 25;

/// Cards left in a stack when the next page is fetched.
pub const LOAD_MORE_AT: usize = 5;

/// Most oracle tags the dictionary's search lists.
pub const DICTIONARY_RESULT_CAP: usize = 100;

/// Most oracle tags the deck form's oracle tag picker lists for a search.
pub const ORACLE_TAG_PICKER_LIMIT: usize = 40;

/// Most example cards Oracle tag examples holds.
pub const EXAMPLES_MAX_CARDS: usize = 500;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_heat_pick_is_orangui_s() {
        assert_eq!(
            HEAT_PICK,
            Duration::from_millis(u64::from(orangui_core::heat::PICK_MS))
        );
    }
}
