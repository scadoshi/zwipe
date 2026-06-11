//! Import mode for deck card imports.
//!
//! Both importers (plain-text and Archidekt) either add cards on top of what's
//! already on a board, or replace the board's contents with the imported list.

use serde::{Deserialize, Serialize};

/// How an import treats the cards already on the target board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImportMode {
    /// Add imported cards; cards already on the board keep their place, with
    /// quantities of re-imported cards updated to the imported value.
    #[default]
    Add,
    /// Make the board exactly match the imported list; cards on it that
    /// aren't in the list are removed.
    Replace,
}

impl ImportMode {
    /// Whether this import replaces the target board's contents.
    pub fn is_replace(&self) -> bool {
        matches!(self, Self::Replace)
    }
}
