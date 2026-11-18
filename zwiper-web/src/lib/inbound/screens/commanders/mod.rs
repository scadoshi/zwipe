//! Commanders: the user's Maybeboard and their Suggested commanders on one
//! screen, side by side on a wide window and one after the other below it.

mod list_filter;
mod maybeboard;
mod screen;
mod suggested;

pub use self::screen::{CommanderMaybeboard, TITLE};
