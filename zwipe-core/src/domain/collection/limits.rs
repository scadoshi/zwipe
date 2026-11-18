//! What one collection import may hold. Past any of these the whole import is turned down and
//! nothing changes.

/// Largest import body, in bytes. A 50,000-row export runs to about 8 MB with every column, so
/// the cap is the request's, and [`MAX_ROWS`] is reached first only by short rows.
pub const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

/// Most card rows one import may hold.
pub const MAX_ROWS: usize = 50_000;

/// Longest cell, in characters. The longest real value is a card name or a tag list.
pub const MAX_CELL_CHARS: usize = 512;

/// Most unmatched rows a reply lists; the rest are counted.
pub const MAX_UNMATCHED_LISTED: usize = 200;
