//! How long a toast stays up.
//!
//! Three bands rather than a number per call site. zwiper had 119 of those
//! across seven values, so tuning meant editing all of them and guessing
//! which were meant to feel quick.
//!
//! Shared so every surface agrees: a confirmation should not linger twice as
//! long in one app as another. Nothing but `std::time::Duration` is involved,
//! so this costs consumers no dependency.
//!
//! **If your stylesheet drives the toast lifecycle, these are load-bearing.**
//! zwiper's does not: its CSS is a fixed 0.2s entry and the library removes
//! the toast, so a band can change here alone. cairn's `toast.css` animates
//! the whole life keyed on `data-type`, so a band changed here has to change
//! there too, or a toast fades out while it is still up, or sits invisible
//! waiting its turn.

use std::time::Duration;

/// Routine confirmations: saved, copied, added.
pub const TOAST_QUICK: Duration = Duration::from_secs(3);

/// Worth actually reading before it leaves.
pub const TOAST_NORMAL: Duration = Duration::from_secs(5);

/// Failures, and anything naming a next step.
pub const TOAST_LONG: Duration = Duration::from_secs(8);
