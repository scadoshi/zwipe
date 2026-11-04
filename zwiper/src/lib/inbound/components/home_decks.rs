//! Home's Decks panel, the shared `components` piece re-exported here, with
//! the deck list the app caches above the router for it.

pub(crate) use components::HomeDecks;
use uuid::Uuid;
use zwipe_ui::screens::home::DeckTile;

/// The last deck list fetched, kept above the router (as
/// `Signal<Option<CachedDecks>>`) so a return to Home lists the decks at
/// once while the refetch is out.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedDecks {
    /// The user the decks belong to; another user's copy is never shown.
    pub user_id: Uuid,
    /// What zerver answered, most recently updated first.
    pub decks: Vec<DeckTile>,
}
