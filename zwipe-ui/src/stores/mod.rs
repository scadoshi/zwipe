//! The in-memory stores the card screens share for the whole run, as the app keeps them: each screen's filter per deck, the Add screen's parked stacks, the per-deck undo log, adds still in flight, and the filter pickers' catalogs.
//!
//! Nothing here is saved to disk; quitting forgets all of it.

pub mod catalog;
pub mod filters;
pub mod in_flight;
pub mod parked;
pub mod undo;

use uuid::Uuid;
use zwipe_core::domain::card::Card;

use self::catalog::Catalog;
use self::filters::FilterStore;
use self::in_flight::InFlightAdds;
use self::parked::AddStackCache;
use self::undo::UndoStore;

/// Every store the card screens share.
#[derive(Debug, Default)]
pub struct CardStores {
    /// Each screen's filter per deck.
    pub filters: FilterStore,
    /// Add's parked stacks.
    pub stacks: AddStackCache,
    /// The per-deck undo log.
    pub undo: UndoStore,
    /// Adds still in flight.
    pub in_flight: InFlightAdds,
    /// The public catalogs.
    pub catalog: Catalog,
    /// The printings fetched last and the card they belong to, by oracle ID, which reopening the printing sheet on that card reuses.
    pub printings: Option<(Uuid, Vec<Card>)>,
}
