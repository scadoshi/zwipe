//! Per-stack swipe action models for undo.
//!
//! `AddAction` is the Add screen's search swipe as its parked stacks keep
//! it. The Add and Remove screens' own stacks live in
//! `zwipe_ui::screens::add_cards` and `zwipe_ui::screens::remove_cards`.

/// Add screen, search stack (linear). Field-less: the swiped card is still
/// in the stack at the position undo rewinds to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AddAction {
    /// Left swipe: durable skip.
    Skip,
    /// Right swipe: added to the deck.
    Add,
    /// Up swipe: sent to the maybeboard.
    Maybe,
}
