//! App-side shim: the deck chart components all live in the `components` crate
//! (shared with the zwipe.net shared-deck page), and the chart-ready data
//! comes from `DeckMetrics` methods in core. This module only re-exports so
//! screen imports stay stable.

pub(crate) use components::{DeckCharts, DrawOdds, ManaCurve, ManaFulfillment};
pub(crate) use zwipe_core::domain::deck::deck_metrics::ManaBalanceRow;
