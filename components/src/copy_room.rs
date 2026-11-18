//! Room for another copy in the deck a page shows, as the page provides it to the quantity
//! steppers under it: the deck's total toward its card limit, and that limit. A page that
//! doesn't provide it caps a card at 99 copies only.

use dioxus::prelude::*;
use zwipe_core::domain::deck::Quantity;
use zwipe_ui::deck::copies;

/// The deck's total toward its card limit, every board plus each filled command zone slot,
/// and the limit, provided by a deck page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CopyRoom {
    total: ReadSignal<i64>,
    limit: ReadSignal<i64>,
}

impl CopyRoom {
    /// The room left in a deck at `total` of `limit`.
    #[must_use]
    pub const fn new(total: ReadSignal<i64>, limit: ReadSignal<i64>) -> Self {
        Self { total, limit }
    }
}

/// Whether a card holding `qty` copies may take one more: under 99, in a deck under its limit
/// when the page provides a [`CopyRoom`].
#[must_use]
pub fn can_add_copy(qty: i32) -> bool {
    try_consume_context::<CopyRoom>().map_or(qty < Quantity::MAX, |room| {
        copies::can_add_copy(qty, *room.total.read(), *room.limit.read())
    })
}
