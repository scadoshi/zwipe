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
}
