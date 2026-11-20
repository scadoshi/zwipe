//! A list that shows its rows ten at a time, as the Commanders screen's Maybeboard does: the first
//! ten, then ten more on each Show more until every row shows. Its rows are already loaded, so
//! Show more asks nothing of the server; it still answers with [`MoreGhosts`], the ghost rows
//! easing in at the foot of the list and the next ten easing in over them, as Suggested's does.
//!
//! The client resets it when the list's sort or filter changes, which shows the first ten again.

#[cfg(test)]
mod tests;

use uuid::Uuid;

use crate::commander::more_ghosts::MoreGhosts;

/// The chip under a list that shows more of it.
pub const SHOW_MORE: &str = "Show more";

/// How many rows the list shows at first, and how many more each Show more adds.
pub const MORE_PAGE: usize = 10;

/// How many rows show, and Show more's ghost rows. Starts at the first [`MORE_PAGE`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MorePage {
    shown: usize,
    ghosts: MoreGhosts,
}

impl Default for MorePage {
    fn default() -> Self {
        Self {
            shown: MORE_PAGE,
            ghosts: MoreGhosts::new(),
        }
    }
}

impl MorePage {
    /// The first [`MORE_PAGE`] rows, no ghosts.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Show more's ghost rows.
    #[must_use]
    pub fn ghosts(&self) -> &MoreGhosts {
        &self.ghosts
    }

    /// Whether Show more shows under a list of `total` rows: some are still hidden.
    #[must_use]
    pub fn offers_more(&self, total: usize) -> bool {
        total > self.shown
    }

    /// Whether the next Show more shows every row of a list of `total`, so Show more goes once
    /// it is pressed.
    #[must_use]
    pub fn next_shows_the_rest(&self, total: usize) -> bool {
        total <= self.shown.saturating_add(MORE_PAGE)
    }

    /// Shows the next [`MORE_PAGE`] of `rows`, the ghosts easing in and those rows waiting on
    /// them. `oracle_id` reads a row's oracle id. With nothing more to show it does nothing.
    pub fn show_more<T>(&mut self, rows: &[T], oracle_id: impl Fn(&T) -> Option<Uuid>) {
        if !self.offers_more(rows.len()) {
            return;
        }
        let next = self.shown.saturating_add(MORE_PAGE);
        let revealed = rows
            .get(self.shown..next.min(rows.len()))
            .unwrap_or_default()
            .iter()
            .filter_map(oracle_id)
            .collect();
        self.shown = next;
        self.ghosts.ask();
        self.ghosts.landed(revealed);
    }

    /// An animation named `animation` ended on the ghosts.
    pub fn animation_ended(&mut self, animation: &str) {
        self.ghosts.animation_ended(animation);
    }

    /// Back to the first [`MORE_PAGE`] rows with no ghosts, for a new sort or filter.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// The rows that show, as the run held before the last Show more and the run it revealed
    /// ([`MoreGhosts::runs`]). `oracle_id` reads a row's oracle id.
    #[must_use]
    pub fn runs<'a, T>(
        &self,
        rows: &'a [T],
        oracle_id: impl Fn(&T) -> Option<Uuid>,
    ) -> (&'a [T], &'a [T]) {
        let visible = rows.get(..self.shown.min(rows.len())).unwrap_or_default();
        self.ghosts.runs(visible, oracle_id)
    }
}
