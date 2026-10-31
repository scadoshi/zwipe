//! A generation counter for replies that can arrive out of order.
//!
//! A screen that fires a request per keystroke, or refetches while an older
//! fetch is still out, can see the replies land out of order. Each request
//! takes a [`Ticket`] before it goes out; when its reply comes back the
//! screen asks [`Generation::is_current`] and drops the reply if a newer
//! request has taken a ticket since. The app wraps this in a Dioxus hook;
//! other clients keep it in their screen state.

/// One request's place in its [`Generation`]. Compare it after the reply
/// arrives; it says nothing on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ticket(u64);

/// A screen's request generation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Generation(u64);

impl Generation {
    /// Starts a new generation and returns its ticket. Every ticket handed
    /// out before this one stops being current. The counter wraps instead
    /// of overflowing.
    pub fn next_ticket(&mut self) -> Ticket {
        self.0 = self.0.wrapping_add(1);
        Ticket(self.0)
    }

    /// The current generation's ticket, for a request that continues it (a
    /// next page) rather than replacing it (a fresh search). A later
    /// [`next_ticket`](Self::next_ticket) makes it stale like any other.
    #[must_use]
    pub fn current(self) -> Ticket {
        Ticket(self.0)
    }

    /// Whether no newer generation has started since `ticket` was taken, so
    /// its reply may land.
    #[must_use]
    pub fn is_current(self, ticket: Ticket) -> bool {
        self.0 == ticket.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_ticket_is_current() {
        let mut generation = Generation::default();
        let ticket = generation.next_ticket();
        assert!(generation.is_current(ticket));
    }

    #[test]
    fn a_newer_ticket_makes_the_older_one_stale() {
        let mut generation = Generation::default();
        let older = generation.next_ticket();
        let newer = generation.next_ticket();
        assert!(
            !generation.is_current(older),
            "the older ticket still lands"
        );
        assert!(generation.is_current(newer));
    }

    #[test]
    fn current_joins_the_generation_without_ending_it() {
        let mut generation = Generation::default();
        let fresh = generation.next_ticket();
        let page = generation.current();
        assert!(generation.is_current(fresh));
        assert!(generation.is_current(page));
        generation.next_ticket();
        assert!(
            !generation.is_current(page),
            "a fresh search kept the old page"
        );
    }

    #[test]
    fn the_counter_wraps_instead_of_overflowing() {
        let mut generation = Generation(u64::MAX);
        let ticket = generation.next_ticket();
        assert_eq!(ticket, Ticket(0));
        assert!(generation.is_current(ticket));
    }
}
