//! The rate limiters' buckets. [`RateLimits::default`] is what the server runs with; tests build
//! the router with their own, so no bucket refills while a test runs.

use std::time::Duration;

/// One limiter's bucket: `burst` requests at once, then one more every `period`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limit {
    /// How long the bucket takes to regain one request.
    pub period: Duration,
    /// How many requests the bucket holds.
    pub burst: u32,
}

/// Every rate limiter's bucket: per address on the public routes, per user on the private ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimits {
    /// 5 req / 30s: tight limit, brute-force target
    pub login: Limit,
    /// 5 req / 1hr: rarely needed legitimately
    pub register: Limit,
    /// 20 req / 1min: clients refresh on cold start
    pub refresh: Limit,
    /// 5 req / 1hr per IP: password reset is rare
    pub forgot_password: Limit,
    /// 10 req / 1hr per IP: verify-email, confirm-email-change, reset-password and its check
    pub verify_reset: Limit,
    /// 60 req / min per IP: card metadata is public and CF-cached, but a tight
    /// limit guards the origin from someone bypassing CF to scrape directly.
    pub public_card: Limit,
    /// A burst of 10, then one every 3s per IP: each example page is a real card search, heavier
    /// than the catalog reads, and Cloudflare caches repeat pages for an hour.
    pub public_examples: Limit,
    /// 30 req / 2s per IP: public marketing aggregates. CF cache absorbs most
    /// traffic; this guards origin if someone bypasses CF or warms many POPs.
    pub public_marketing: Limit,
    /// 30 req / 2s per IP: min-version gate poll. ~30-byte payload hit once a
    /// minute per app; modest cap guards origin without risking lockout noise.
    pub public_client: Limit,
    /// 30 req / 2s per IP: public changelog. Fetched once per app launch and
    /// Cloudflare edge-caches it, so origin traffic is tiny; this guards the
    /// origin if someone bypasses CF or warms many POPs.
    pub public_changelog: Limit,
    /// 10 req / min per IP: pre-auth funnel events. A legitimate session
    /// fires a handful ever (app opened, register viewed/submitted); this is
    /// an unauthenticated write, so keep the row-spam ceiling low.
    pub anonymous_event: Limit,
    /// Burst 2, then 1 req / min per IP: crash reports. A legitimate client
    /// posts at most one per launch; this is an unauthenticated write that
    /// inserts a row per call, so it gets the strictest limiter (the `crash_id`
    /// upsert makes anything past it idempotent).
    pub crash_report: Limit,
    /// 30 req / 2s per IP: public shared-deck reads. A page load fetches once
    /// and CF may cache briefly; this guards origin against token scanning.
    pub public_share: Limit,
    /// 30 req / minute per IP, burst 10: shared decks' preview images. A
    /// render costs ~100ms of CPU plus art fetches, so this sits inside the
    /// share routes' own limit; Cloudflare's edge serves repeat requests, so
    /// origin only sees a link's first unfurls.
    pub share_preview: Limit,
    /// 30 req / 2s per IP: health checks. Generous for frequent polling
    /// (manual curls, uptime monitors) while capping unauthenticated floods,
    /// notably /health/database which pings Postgres.
    pub health: Limit,
    /// 500 req / 5min (~1.67/s avg): generous for swiping, keyed by user ID
    pub private: Limit,
    /// burst 2, then 1 req/30min: account mutations are done once; 2 attempts covers typos
    pub sensitive: Limit,
    /// burst 20, then 1 req/10s: commander autocomplete needs headroom for fast typers
    pub card_search: Limit,
    /// burst 1, then 1 req/60s: resend verification: a fast multi-click sends
    /// one email, the rest get 429. Window matches the client cooldown timer.
    pub resend_verification: Limit,
    /// The same for a pending email change's link, on its own budget so one
    /// resend does not hold up the other.
    pub resend_email_change: Limit,
    /// 12 req/min: clients flush usage every ~30s; this gives ample headroom
    pub metrics_usage: Limit,
    /// burst 5, then 1 req/min: each import is a fetch from Archidekt made
    /// from this origin's address, so one user cannot spend its reputation.
    pub archidekt_import: Limit,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            login: Limit {
                period: Duration::from_secs(6),
                burst: 5,
            },
            register: Limit {
                period: Duration::from_mins(12),
                burst: 5,
            },
            refresh: Limit {
                period: Duration::from_secs(3),
                burst: 20,
            },
            forgot_password: Limit {
                period: Duration::from_mins(12),
                burst: 5,
            },
            verify_reset: Limit {
                period: Duration::from_secs(360),
                burst: 10,
            },
            public_card: Limit {
                period: Duration::from_secs(1),
                burst: 60,
            },
            public_examples: Limit {
                period: Duration::from_secs(3),
                burst: 10,
            },
            public_marketing: Limit {
                period: Duration::from_secs(2),
                burst: 30,
            },
            public_client: Limit {
                period: Duration::from_secs(2),
                burst: 30,
            },
            public_changelog: Limit {
                period: Duration::from_secs(2),
                burst: 30,
            },
            anonymous_event: Limit {
                period: Duration::from_secs(6),
                burst: 10,
            },
            crash_report: Limit {
                period: Duration::from_secs(60),
                burst: 2,
            },
            public_share: Limit {
                period: Duration::from_secs(2),
                burst: 30,
            },
            share_preview: Limit {
                period: Duration::from_secs(2),
                burst: 10,
            },
            health: Limit {
                period: Duration::from_secs(2),
                burst: 30,
            },
            private: Limit {
                period: Duration::from_millis(600),
                burst: 500,
            },
            sensitive: Limit {
                period: Duration::from_mins(30),
                burst: 2,
            },
            card_search: Limit {
                period: Duration::from_secs(10),
                burst: 20,
            },
            resend_verification: Limit {
                period: Duration::from_secs(60),
                burst: 1,
            },
            resend_email_change: Limit {
                period: Duration::from_secs(60),
                burst: 1,
            },
            metrics_usage: Limit {
                period: Duration::from_secs(5),
                burst: 12,
            },
            archidekt_import: Limit {
                period: Duration::from_secs(60),
                burst: 5,
            },
        }
    }
}

impl RateLimits {
    /// Every limiter with its burst kept and its period set to `period`.
    #[must_use]
    pub fn with_every_period(self, period: Duration) -> Self {
        let every = |limit: Limit| Limit { period, ..limit };
        Self {
            login: every(self.login),
            register: every(self.register),
            refresh: every(self.refresh),
            forgot_password: every(self.forgot_password),
            verify_reset: every(self.verify_reset),
            public_card: every(self.public_card),
            public_examples: every(self.public_examples),
            public_marketing: every(self.public_marketing),
            public_client: every(self.public_client),
            public_changelog: every(self.public_changelog),
            anonymous_event: every(self.anonymous_event),
            crash_report: every(self.crash_report),
            public_share: every(self.public_share),
            share_preview: every(self.share_preview),
            health: every(self.health),
            private: every(self.private),
            sensitive: every(self.sensitive),
            card_search: every(self.card_search),
            resend_verification: every(self.resend_verification),
            resend_email_change: every(self.resend_email_change),
            metrics_usage: every(self.metrics_usage),
            archidekt_import: every(self.archidekt_import),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pins every bucket by value, so changing a limit takes an edit here too.
    #[test]
    #[expect(clippy::too_many_lines, reason = "one pinned row per limiter")]
    fn the_defaults_are_the_server_s_limits() {
        let limits = RateLimits::default();
        let pinned = [
            ("login", limits.login, Duration::from_secs(6), 5),
            ("register", limits.register, Duration::from_mins(12), 5),
            ("refresh", limits.refresh, Duration::from_secs(3), 20),
            (
                "forgot_password",
                limits.forgot_password,
                Duration::from_mins(12),
                5,
            ),
            (
                "verify_reset",
                limits.verify_reset,
                Duration::from_secs(360),
                10,
            ),
            (
                "public_card",
                limits.public_card,
                Duration::from_secs(1),
                60,
            ),
            (
                "public_examples",
                limits.public_examples,
                Duration::from_secs(3),
                10,
            ),
            (
                "public_marketing",
                limits.public_marketing,
                Duration::from_secs(2),
                30,
            ),
            (
                "public_client",
                limits.public_client,
                Duration::from_secs(2),
                30,
            ),
            (
                "public_changelog",
                limits.public_changelog,
                Duration::from_secs(2),
                30,
            ),
            (
                "anonymous_event",
                limits.anonymous_event,
                Duration::from_secs(6),
                10,
            ),
            (
                "crash_report",
                limits.crash_report,
                Duration::from_secs(60),
                2,
            ),
            (
                "public_share",
                limits.public_share,
                Duration::from_secs(2),
                30,
            ),
            (
                "share_preview",
                limits.share_preview,
                Duration::from_secs(2),
                10,
            ),
            ("health", limits.health, Duration::from_secs(2), 30),
            ("private", limits.private, Duration::from_millis(600), 500),
            ("sensitive", limits.sensitive, Duration::from_mins(30), 2),
            (
                "card_search",
                limits.card_search,
                Duration::from_secs(10),
                20,
            ),
            (
                "resend_verification",
                limits.resend_verification,
                Duration::from_secs(60),
                1,
            ),
            (
                "resend_email_change",
                limits.resend_email_change,
                Duration::from_secs(60),
                1,
            ),
            (
                "metrics_usage",
                limits.metrics_usage,
                Duration::from_secs(5),
                12,
            ),
            (
                "archidekt_import",
                limits.archidekt_import,
                Duration::from_secs(60),
                5,
            ),
        ];
        for (name, limit, period, burst) in pinned {
            assert_eq!(limit, Limit { period, burst }, "{name}");
        }
    }

    #[test]
    fn with_every_period_keeps_each_burst() {
        let hour = Duration::from_secs(3600);
        let stretched = RateLimits::default().with_every_period(hour);
        assert_eq!(
            stretched.login,
            Limit {
                period: hour,
                burst: 5
            }
        );
        assert_eq!(
            stretched.private,
            Limit {
                period: hour,
                burst: 500
            }
        );
    }
}
