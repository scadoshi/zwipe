//! Which app a collection export came from.

use serde::{Deserialize, Serialize};

/// The app whose export an import read, told apart by the header row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionSource {
    /// ManaBox's collection, binder or list export.
    ManaBox,
    /// Archidekt's collection export.
    Archidekt,
    /// A value this build doesn't know, from a newer server. The server never stores or sends
    /// it for a value it knows.
    #[serde(other)]
    Unknown,
}
