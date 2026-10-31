//! The in-memory stores the card screens share for the whole run, as the app keeps them: each screen's filter per deck, the Add screen's parked stacks, the per-deck undo log, adds still in flight, and the filter pickers' catalogs.
//!
//! Nothing here is saved to disk; quitting forgets all of it.

pub(crate) mod catalog;
pub(crate) mod filters;
pub(crate) mod in_flight;
pub(crate) mod parked;
pub(crate) mod undo;

use uuid::Uuid;
use zwipe_core::domain::card::Card;

use self::catalog::Catalog;
use self::filters::FilterStore;
use self::in_flight::InFlightAdds;
use self::parked::AddStackCache;
use self::undo::UndoStore;

/// Every store the card screens share.
#[derive(Debug, Default)]
pub(crate) struct CardStores {
    pub(crate) filters: FilterStore,
    pub(crate) stacks: AddStackCache,
    pub(crate) undo: UndoStore,
    pub(crate) in_flight: InFlightAdds,
    pub(crate) catalog: Catalog,
    /// The printings fetched last and the card they belong to, by oracle ID, which reopening the printing sheet on that card reuses.
    pub(crate) printings: Option<(Uuid, Vec<Card>)>,
}
