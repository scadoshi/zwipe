//! Generation counter for async replies.
//!
//! A screen that fires a request per keystroke, or refetches while an older
//! fetch is still out, can see the replies arrive out of order. Each request
//! takes a [`Ticket`] before it goes out; when its reply comes back the screen
//! asks [`Generation::is_current`] and drops the reply if a newer request has
//! taken a ticket since.
//!
//! [`Ticket`]: crate::inbound::components::generation::Ticket
//! [`Generation::is_current`]: crate::inbound::components::generation::Generation::is_current

use dioxus::prelude::*;

/// One request's place in its [`Generation`]. Compare it after the reply
/// arrives; it says nothing on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticket(u64);

/// The counter behind [`Generation`], kept free of Dioxus so it can be
/// tested on its own.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Counter(u64);

impl Counter {
    fn next_ticket(&mut self) -> Ticket {
        self.0 = self.0.wrapping_add(1);
        Ticket(self.0)
    }

    fn current(self) -> Ticket {
        Ticket(self.0)
    }

    fn is_current(self, ticket: Ticket) -> bool {
        self.0 == ticket.0
    }
}

/// A component's request generation, from [`use_generation`]. Copy, like a
/// signal, and not reactive: taking a ticket re-renders nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Generation(CopyValue<Counter>);

impl Generation {
    /// Starts a new generation and returns its ticket. Every ticket handed
    /// out before this one stops being current.
    pub fn next_ticket(&mut self) -> Ticket {
        self.0.write().next_ticket()
    }

    /// The current generation's ticket, for a request that continues it
    /// (a next page) rather than replacing it (a fresh search). A later
    /// [`next_ticket`](Self::next_ticket) makes it stale like any other.
    pub fn current(&self) -> Ticket {
        self.0.peek().current()
    }

    /// Whether no newer generation has started since `ticket` was taken, so
    /// its reply may land.
    pub fn is_current(&self, ticket: Ticket) -> bool {
        self.0.peek().is_current(ticket)
    }
}

/// A request generation owned by the calling component.
pub fn use_generation() -> Generation {
    Generation(use_hook(|| CopyValue::new(Counter::default())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_ticket_is_current() {
        let mut counter = Counter::default();
        let ticket = counter.next_ticket();
        assert!(counter.is_current(ticket));
    }

    #[test]
    fn a_newer_ticket_makes_the_older_one_stale() {
        let mut counter = Counter::default();
        let older = counter.next_ticket();
        let newer = counter.next_ticket();
        assert!(!counter.is_current(older), "the older ticket still lands");
        assert!(counter.is_current(newer));
    }

    #[test]
    fn current_joins_the_generation_without_ending_it() {
        let mut counter = Counter::default();
        let fresh = counter.next_ticket();
        let page = counter.current();
        assert!(counter.is_current(fresh), "joining ended the fresh request");
        assert!(counter.is_current(page));
        counter.next_ticket();
        assert!(
            !counter.is_current(page),
            "a fresh search kept the old page"
        );
    }

    #[test]
    fn the_counter_wraps_instead_of_overflowing() {
        let mut counter = Counter(u64::MAX);
        let ticket = counter.next_ticket();
        assert_eq!(ticket, Ticket(0));
        assert!(counter.is_current(ticket));
    }

    #[test]
    fn the_hook_hands_out_tickets_that_go_stale() {
        thread_local! {
            static GENERATION: std::cell::Cell<Option<Generation>> = const { std::cell::Cell::new(None) };
        }

        #[component]
        fn Root() -> Element {
            let generation = use_generation();
            GENERATION.with(|g| g.set(Some(generation)));
            rsx! {}
        }

        let mut dom = VirtualDom::new(Root);
        dom.rebuild_in_place();
        let mut generation = GENERATION.with(|g| g.get().expect("the root rendered"));
        dom.in_runtime(|| {
            let older = generation.next_ticket();
            let newer = generation.next_ticket();
            assert!(!generation.is_current(older));
            assert!(generation.is_current(newer));
        });
    }
}
