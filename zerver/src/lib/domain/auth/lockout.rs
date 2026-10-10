//! Login lockout keyed by account **and** client address.
//!
//! Five wrong passwords for one account from one address lock that pair out
//! for thirty minutes. Other addresses are untouched, so nobody can lock a
//! user out of their own account by guessing at it from elsewhere. Brute
//! force across many addresses is held back by the per-address login rate
//! limit and the password policy.
//!
//! Counters live in memory. zerver is one process, and a restart clearing
//! them hands an attacker five more guesses, nothing more.

use dashmap::DashMap;
use std::{
    net::IpAddr,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
use uuid::Uuid;

/// Wrong passwords from one address before the pair is locked.
pub const MAX_FAILURES: u32 = 5;
/// Failures older than this no longer count.
pub const FAILURE_WINDOW: Duration = Duration::from_secs(30 * 60);
/// How long a locked pair stays locked.
pub const LOCKOUT: Duration = Duration::from_secs(30 * 60);

/// Entries are swept this often, counted in recorded failures.
const SWEEP_EVERY: usize = 1024;

#[derive(Debug, Clone, Copy)]
struct Failures {
    count: u32,
    last_failed_at: Instant,
    locked_until: Option<Instant>,
}

/// In-memory failed-login counters keyed by `(user, address)`.
#[derive(Debug, Default)]
pub struct LoginLockout {
    entries: DashMap<(Uuid, IpAddr), Failures>,
    recorded: AtomicUsize,
}

impl LoginLockout {
    /// Whether this account is currently locked for this address.
    pub fn is_locked(&self, user_id: Uuid, ip: IpAddr) -> bool {
        self.is_locked_at(user_id, ip, Instant::now())
    }

    /// Counts one wrong password. Returns `true` when the pair just locked.
    pub fn record_failure(&self, user_id: Uuid, ip: IpAddr) -> bool {
        self.record_failure_at(user_id, ip, Instant::now())
    }

    /// Forgets the pair's failures after a successful login.
    pub fn clear(&self, user_id: Uuid, ip: IpAddr) {
        self.entries.remove(&(user_id, ip));
    }

    fn is_locked_at(&self, user_id: Uuid, ip: IpAddr, now: Instant) -> bool {
        self.entries
            .get(&(user_id, ip))
            .and_then(|f| f.locked_until)
            .is_some_and(|until| until > now)
    }

    fn record_failure_at(&self, user_id: Uuid, ip: IpAddr, now: Instant) -> bool {
        let mut entry = self.entries.entry((user_id, ip)).or_insert(Failures {
            count: 0,
            last_failed_at: now,
            locked_until: None,
        });
        let fresh = now.duration_since(entry.last_failed_at) <= FAILURE_WINDOW;
        entry.count = if fresh { entry.count + 1 } else { 1 };
        entry.last_failed_at = now;
        let just_locked =
            entry.count >= MAX_FAILURES && entry.locked_until.is_none_or(|u| u <= now);
        if just_locked {
            entry.locked_until = Some(now + LOCKOUT);
        }
        drop(entry);

        if self.recorded.fetch_add(1, Ordering::Relaxed) % SWEEP_EVERY == SWEEP_EVERY - 1 {
            self.sweep(now);
        }
        just_locked
    }

    /// Drops entries whose failures have aged out and whose lock has ended, so
    /// the map does not grow with every address that ever got a password wrong.
    fn sweep(&self, now: Instant) {
        self.entries.retain(|_, f| {
            let counting = now.duration_since(f.last_failed_at) <= FAILURE_WINDOW;
            let locked = f.locked_until.is_some_and(|u| u > now);
            counting || locked
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (Uuid, IpAddr) {
        (Uuid::new_v4(), "203.0.113.7".parse().unwrap())
    }

    #[test]
    fn locks_after_max_failures_and_not_before() {
        let lockout = LoginLockout::default();
        let (user, ip) = pair();
        let now = Instant::now();
        for n in 1..MAX_FAILURES {
            assert!(!lockout.record_failure_at(user, ip, now), "failure {n}");
            assert!(!lockout.is_locked_at(user, ip, now));
        }
        assert!(lockout.record_failure_at(user, ip, now));
        assert!(lockout.is_locked_at(user, ip, now));
    }

    #[test]
    fn another_address_is_unaffected() {
        let lockout = LoginLockout::default();
        let (user, ip) = pair();
        let other: IpAddr = "198.51.100.9".parse().unwrap();
        let now = Instant::now();
        for _ in 0..MAX_FAILURES {
            lockout.record_failure_at(user, ip, now);
        }
        assert!(lockout.is_locked_at(user, ip, now));
        assert!(!lockout.is_locked_at(user, other, now));
    }

    #[test]
    fn lock_and_counter_expire() {
        let lockout = LoginLockout::default();
        let (user, ip) = pair();
        let now = Instant::now();
        for _ in 0..MAX_FAILURES {
            lockout.record_failure_at(user, ip, now);
        }
        let later = now + LOCKOUT + Duration::from_secs(1);
        assert!(!lockout.is_locked_at(user, ip, later));
        // The window has passed too, so the next failure starts a new count.
        assert!(!lockout.record_failure_at(user, ip, later));
    }

    #[test]
    fn success_clears_the_pair() {
        let lockout = LoginLockout::default();
        let (user, ip) = pair();
        let now = Instant::now();
        for _ in 0..MAX_FAILURES - 1 {
            lockout.record_failure_at(user, ip, now);
        }
        lockout.clear(user, ip);
        assert!(!lockout.record_failure_at(user, ip, now));
    }

    #[test]
    fn sweep_keeps_live_entries_only() {
        let lockout = LoginLockout::default();
        let (user, ip) = pair();
        let (stale_user, stale_ip) = pair();
        let now = Instant::now();
        lockout.record_failure_at(stale_user, stale_ip, now);
        let later = now + FAILURE_WINDOW + Duration::from_secs(1);
        lockout.record_failure_at(user, ip, later);
        lockout.sweep(later);
        assert!(lockout.entries.contains_key(&(user, ip)));
        assert!(!lockout.entries.contains_key(&(stale_user, stale_ip)));
    }
}
