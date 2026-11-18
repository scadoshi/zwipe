//! Room for another copy in the deck a page shows, as the page provides it to the quantity
//! steppers under it: the deck's total toward its card limit, and that limit. A page that
//! doesn't provide it caps a card at 99 copies, or at the [`CopyCap`] it provides instead.

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

/// The most copies of one card a page's steppers take, for a page whose rows aren't deck cards:
/// the collection holds up to 9,999 of a printing. Deck pages leave it out and keep 99.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyCap(pub i32);

/// Whether a card holding `qty` copies may take one more: in a deck under its limit when the
/// page provides a [`CopyRoom`]; otherwise under the page's [`CopyCap`], or 99 without one.
#[must_use]
pub fn can_add_copy(qty: i32) -> bool {
    if let Some(room) = try_consume_context::<CopyRoom>() {
        return copies::can_add_copy(qty, *room.total.read(), *room.limit.read());
    }
    let cap = try_consume_context::<CopyCap>().map_or(Quantity::MAX, |CopyCap(cap)| cap);
    qty < cap
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    thread_local! {
        static ANSWERS: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    }

    /// Asks [`can_add_copy`] for each of 98, 99, 9,998 and 9,999 copies, under `cap` when set.
    fn answers(cap: Option<i32>) -> Vec<bool> {
        #[component]
        fn Page(cap: Option<i32>) -> Element {
            if let Some(cap) = cap {
                use_context_provider(|| CopyCap(cap));
            }
            let answers = [98, 99, 9_998, 9_999].map(can_add_copy).to_vec();
            ANSWERS.with(|a| *a.borrow_mut() = answers);
            rsx! {}
        }
        let mut dom = VirtualDom::new_with_props(Page, PageProps { cap });
        dom.rebuild_in_place();
        ANSWERS.with(|a| a.borrow().clone())
    }

    #[test]
    fn a_page_without_a_cap_keeps_a_card_at_99() {
        assert_eq!(answers(None), [true, false, false, false]);
    }

    #[test]
    fn a_page_s_cap_lets_a_card_go_to_it() {
        assert_eq!(answers(Some(9_999)), [true, true, true, false]);
    }
}
