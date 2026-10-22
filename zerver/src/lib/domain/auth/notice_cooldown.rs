//! One "you already have an account" email per address per hour.
//!
//! Registering with a taken email mails that address's owner. Without a cap,
//! anyone could fill a stranger's inbox five emails an hour from every
//! address they register from. Kept in memory like the login lockout: a
//! restart forgets it, which costs the owner one extra email at most.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;

/// How long after one notice the next one to the same address is skipped.
pub const NOTICE_COOLDOWN: Duration = Duration::from_secs(60 * 60);

/// Entries are swept this often, counted in notices allowed.
const SWEEP_EVERY: usize = 1024;

/// Last notice time by email address.
#[derive(Debug, Default)]
pub struct NoticeCooldown {
    last_sent: DashMap<String, Instant>,
    allowed: AtomicUsize,
}

impl NoticeCooldown {
    /// Whether a notice to this address may go out now. Returns `true` and
    /// starts the cooldown, or `false` while one is running.
    pub fn try_send(&self, email: &str) -> bool {
        self.try_send_at(email, Instant::now())
    }

    fn try_send_at(&self, email: &str, now: Instant) -> bool {
        match self.last_sent.entry(email.to_string()) {
            Entry::Occupied(mut sent) => {
                if now.duration_since(*sent.get()) < NOTICE_COOLDOWN {
                    return false;
                }
                sent.insert(now);
            }
            Entry::Vacant(slot) => {
                slot.insert(now);
            }
        }

        if self.allowed.fetch_add(1, Ordering::Relaxed) % SWEEP_EVERY == SWEEP_EVERY - 1 {
            self.last_sent
                .retain(|_, sent| now.duration_since(*sent) < NOTICE_COOLDOWN);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_notice_within_the_hour_is_skipped() {
        let cooldown = NoticeCooldown::default();
        let now = Instant::now();
        assert!(cooldown.try_send_at("a@b.co", now));
        assert!(!cooldown.try_send_at("a@b.co", now + Duration::from_secs(1)));
        assert!(!cooldown.try_send_at("a@b.co", now + NOTICE_COOLDOWN - Duration::from_secs(1)));
    }

    #[test]
    fn notice_goes_out_again_after_the_hour() {
        let cooldown = NoticeCooldown::default();
        let now = Instant::now();
        assert!(cooldown.try_send_at("a@b.co", now));
        assert!(cooldown.try_send_at("a@b.co", now + NOTICE_COOLDOWN));
    }

    #[test]
    fn addresses_are_independent() {
        let cooldown = NoticeCooldown::default();
        let now = Instant::now();
        assert!(cooldown.try_send_at("a@b.co", now));
        assert!(cooldown.try_send_at("c@d.co", now));
    }
}
