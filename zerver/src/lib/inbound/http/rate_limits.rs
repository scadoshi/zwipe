//! The rate limiters' buckets. [`RateLimits::default`] is what the server runs with; tests build
//! the router with their own, so no bucket refills while a test runs. An address-keyed limiter
//! has a bucket per address family ([`PerFamily`]).

use std::time::Duration;

/// One limiter's bucket: `burst` requests at once, then one more every `period`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limit {
    /// How long the bucket takes to regain one request.
    pub period: Duration,
    /// How many requests the bucket holds.
    pub burst: u32,
}

/// An address-keyed limiter's two buckets: one for an IPv4 client, one for an IPv6 client's
/// /64, which a household or an office shares, so it gets more room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerFamily {
    /// An IPv4 client's bucket, an IPv4 address written as IPv6 included.
    pub ipv4: Limit,
    /// An IPv6 /64's bucket.
    pub ipv6: Limit,
}

/// Every rate limiter's bucket: per address on the public routes, one pair per address family,
/// and per user on the private ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimits {
    /// 5 req / 30s: tight limit, brute-force target
    ///
    /// IPv6 /64: a burst of 10, the same refill; a brute-force rate stays per network.
    pub login: PerFamily,
    /// 5 req / 1hr: rarely needed legitimately
    ///
    /// IPv6 /64: a burst of 10, the same refill, for a household or office signing up together.
    pub register: PerFamily,
    /// 20 req / 1min: clients refresh on cold start
    ///
    /// IPv6 /64: double the burst and the rate, since every app on the network refreshes on cold
    /// start.
    pub refresh: PerFamily,
    /// 5 req / 1hr per IP: password reset is rare
    ///
    /// IPv6 /64: a burst of 10, the same refill.
    pub forgot_password: PerFamily,
    /// 10 req / 1hr per IP: verify-email, confirm-email-change, reset-password and its check
    ///
    /// IPv6 /64: a burst of 20, the same refill.
    pub verify_reset: PerFamily,
    /// 60 req / min per IP: card metadata is public and CF-cached, but a tight
    /// limit guards the origin from someone bypassing CF to scrape directly.
    ///
    /// IPv6 /64: double the burst and the rate.
    pub public_card: PerFamily,
    /// A burst of 10, then one every 3s per IP: each example page is a real card search, heavier
    /// than the catalog reads, and Cloudflare caches repeat pages for an hour.
    ///
    /// IPv6 /64: double the burst and the rate.
    pub public_examples: PerFamily,
    /// 30 req / 2s per IP: public marketing aggregates. CF cache absorbs most
    /// traffic; this guards origin if someone bypasses CF or warms many POPs.
    ///
    /// IPv6 /64: double the burst and the rate.
    pub public_marketing: PerFamily,
    /// 30 req / 2s per IP: min-version gate poll. ~30-byte payload hit once a
    /// minute per app; modest cap guards origin without risking lockout noise.
    ///
    /// IPv6 /64: double the burst and the rate.
    pub public_client: PerFamily,
    /// 30 req / 2s per IP: public changelog. Fetched once per app launch and
    /// Cloudflare edge-caches it, so origin traffic is tiny; this guards the
    /// origin if someone bypasses CF or warms many POPs.
    ///
    /// IPv6 /64: double the burst and the rate.
    pub public_changelog: PerFamily,
    /// 10 req / min per IP: pre-auth funnel events. A legitimate session
    /// fires a handful ever (app opened, register viewed/submitted); this is
    /// an unauthenticated write, so keep the row-spam ceiling low.
    ///
    /// IPv6 /64: double the burst and the rate, one session per device.
    pub anonymous_event: PerFamily,
    /// Burst 2, then 1 req / min per IP: crash reports. A legitimate client
    /// posts at most one per launch; this is an unauthenticated write that
    /// inserts a row per call, so it gets the strictest limiter (the `crash_id`
    /// upsert makes anything past it idempotent).
    ///
    /// IPv6 /64: double the burst and the rate, one report per device.
    pub crash_report: PerFamily,
    /// 30 req / 2s per IP: public shared-deck reads. A page load fetches once
    /// and CF may cache briefly; this guards origin against token scanning.
    ///
    /// IPv6 /64: double the burst and the rate.
    pub public_share: PerFamily,
    /// 30 req / minute per IP, burst 10: shared decks' preview images. A
    /// render costs ~100ms of CPU plus art fetches, so this sits inside the
    /// share routes' own limit; Cloudflare's edge serves repeat requests, so
    /// origin only sees a link's first unfurls.
    ///
    /// IPv6 /64: double the burst and the rate.
    pub share_preview: PerFamily,
    /// 30 req / 2s per IP: health checks. Generous for frequent polling
    /// (manual curls, uptime monitors) while capping unauthenticated floods,
    /// notably /health/database which pings Postgres.
    ///
    /// IPv6 /64: double the burst and the rate.
    pub health: PerFamily,
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

/// `burst` requests at once, then one more each `period`.
const fn limit(period: Duration, burst: u32) -> Limit {
    Limit { period, burst }
}

/// An address-keyed limiter's IPv4 and IPv6 buckets.
const fn pair(ipv4: Limit, ipv6: Limit) -> PerFamily {
    PerFamily { ipv4, ipv6 }
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            login: pair(
                limit(Duration::from_secs(6), 5),
                limit(Duration::from_secs(6), 10),
            ),
            register: pair(
                limit(Duration::from_mins(12), 5),
                limit(Duration::from_mins(12), 10),
            ),
            refresh: pair(
                limit(Duration::from_secs(3), 20),
                limit(Duration::from_millis(1500), 40),
            ),
            forgot_password: pair(
                limit(Duration::from_mins(12), 5),
                limit(Duration::from_mins(12), 10),
            ),
            verify_reset: pair(
                limit(Duration::from_secs(360), 10),
                limit(Duration::from_secs(360), 20),
            ),
            public_card: pair(
                limit(Duration::from_secs(1), 60),
                limit(Duration::from_millis(500), 120),
            ),
            public_examples: pair(
                limit(Duration::from_secs(3), 10),
                limit(Duration::from_millis(1500), 20),
            ),
            public_marketing: pair(
                limit(Duration::from_secs(2), 30),
                limit(Duration::from_secs(1), 60),
            ),
            public_client: pair(
                limit(Duration::from_secs(2), 30),
                limit(Duration::from_secs(1), 60),
            ),
            public_changelog: pair(
                limit(Duration::from_secs(2), 30),
                limit(Duration::from_secs(1), 60),
            ),
            anonymous_event: pair(
                limit(Duration::from_secs(6), 10),
                limit(Duration::from_secs(3), 20),
            ),
            crash_report: pair(
                limit(Duration::from_secs(60), 2),
                limit(Duration::from_secs(30), 4),
            ),
            public_share: pair(
                limit(Duration::from_secs(2), 30),
                limit(Duration::from_secs(1), 60),
            ),
            share_preview: pair(
                limit(Duration::from_secs(2), 10),
                limit(Duration::from_secs(1), 20),
            ),
            health: pair(
                limit(Duration::from_secs(2), 30),
                limit(Duration::from_secs(1), 60),
            ),
            private: limit(Duration::from_millis(600), 500),
            sensitive: limit(Duration::from_mins(30), 2),
            card_search: limit(Duration::from_secs(10), 20),
            resend_verification: limit(Duration::from_secs(60), 1),
            resend_email_change: limit(Duration::from_secs(60), 1),
            metrics_usage: limit(Duration::from_secs(5), 12),
            archidekt_import: limit(Duration::from_secs(60), 5),
        }
    }
}

impl RateLimits {
    /// Every limiter with its burst kept and its period set to `period`.
    #[must_use]
    pub fn with_every_period(self, period: Duration) -> Self {
        let every = |limit: Limit| Limit { period, ..limit };
        Self {
            login: PerFamily {
                ipv4: every(self.login.ipv4),
                ipv6: every(self.login.ipv6),
            },
            register: PerFamily {
                ipv4: every(self.register.ipv4),
                ipv6: every(self.register.ipv6),
            },
            refresh: PerFamily {
                ipv4: every(self.refresh.ipv4),
                ipv6: every(self.refresh.ipv6),
            },
            forgot_password: PerFamily {
                ipv4: every(self.forgot_password.ipv4),
                ipv6: every(self.forgot_password.ipv6),
            },
            verify_reset: PerFamily {
                ipv4: every(self.verify_reset.ipv4),
                ipv6: every(self.verify_reset.ipv6),
            },
            public_card: PerFamily {
                ipv4: every(self.public_card.ipv4),
                ipv6: every(self.public_card.ipv6),
            },
            public_examples: PerFamily {
                ipv4: every(self.public_examples.ipv4),
                ipv6: every(self.public_examples.ipv6),
            },
            public_marketing: PerFamily {
                ipv4: every(self.public_marketing.ipv4),
                ipv6: every(self.public_marketing.ipv6),
            },
            public_client: PerFamily {
                ipv4: every(self.public_client.ipv4),
                ipv6: every(self.public_client.ipv6),
            },
            public_changelog: PerFamily {
                ipv4: every(self.public_changelog.ipv4),
                ipv6: every(self.public_changelog.ipv6),
            },
            anonymous_event: PerFamily {
                ipv4: every(self.anonymous_event.ipv4),
                ipv6: every(self.anonymous_event.ipv6),
            },
            crash_report: PerFamily {
                ipv4: every(self.crash_report.ipv4),
                ipv6: every(self.crash_report.ipv6),
            },
            public_share: PerFamily {
                ipv4: every(self.public_share.ipv4),
                ipv6: every(self.public_share.ipv6),
            },
            share_preview: PerFamily {
                ipv4: every(self.share_preview.ipv4),
                ipv6: every(self.share_preview.ipv6),
            },
            health: PerFamily {
                ipv4: every(self.health.ipv4),
                ipv6: every(self.health.ipv6),
            },
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
            ("login ipv4", limits.login.ipv4, Duration::from_secs(6), 5),
            ("login ipv6", limits.login.ipv6, Duration::from_secs(6), 10),
            (
                "register ipv4",
                limits.register.ipv4,
                Duration::from_mins(12),
                5,
            ),
            (
                "register ipv6",
                limits.register.ipv6,
                Duration::from_mins(12),
                10,
            ),
            (
                "refresh ipv4",
                limits.refresh.ipv4,
                Duration::from_secs(3),
                20,
            ),
            (
                "refresh ipv6",
                limits.refresh.ipv6,
                Duration::from_millis(1500),
                40,
            ),
            (
                "forgot_password ipv4",
                limits.forgot_password.ipv4,
                Duration::from_mins(12),
                5,
            ),
            (
                "forgot_password ipv6",
                limits.forgot_password.ipv6,
                Duration::from_mins(12),
                10,
            ),
            (
                "verify_reset ipv4",
                limits.verify_reset.ipv4,
                Duration::from_secs(360),
                10,
            ),
            (
                "verify_reset ipv6",
                limits.verify_reset.ipv6,
                Duration::from_secs(360),
                20,
            ),
            (
                "public_card ipv4",
                limits.public_card.ipv4,
                Duration::from_secs(1),
                60,
            ),
            (
                "public_card ipv6",
                limits.public_card.ipv6,
                Duration::from_millis(500),
                120,
            ),
            (
                "public_examples ipv4",
                limits.public_examples.ipv4,
                Duration::from_secs(3),
                10,
            ),
            (
                "public_examples ipv6",
                limits.public_examples.ipv6,
                Duration::from_millis(1500),
                20,
            ),
            (
                "public_marketing ipv4",
                limits.public_marketing.ipv4,
                Duration::from_secs(2),
                30,
            ),
            (
                "public_marketing ipv6",
                limits.public_marketing.ipv6,
                Duration::from_secs(1),
                60,
            ),
            (
                "public_client ipv4",
                limits.public_client.ipv4,
                Duration::from_secs(2),
                30,
            ),
            (
                "public_client ipv6",
                limits.public_client.ipv6,
                Duration::from_secs(1),
                60,
            ),
            (
                "public_changelog ipv4",
                limits.public_changelog.ipv4,
                Duration::from_secs(2),
                30,
            ),
            (
                "public_changelog ipv6",
                limits.public_changelog.ipv6,
                Duration::from_secs(1),
                60,
            ),
            (
                "anonymous_event ipv4",
                limits.anonymous_event.ipv4,
                Duration::from_secs(6),
                10,
            ),
            (
                "anonymous_event ipv6",
                limits.anonymous_event.ipv6,
                Duration::from_secs(3),
                20,
            ),
            (
                "crash_report ipv4",
                limits.crash_report.ipv4,
                Duration::from_secs(60),
                2,
            ),
            (
                "crash_report ipv6",
                limits.crash_report.ipv6,
                Duration::from_secs(30),
                4,
            ),
            (
                "public_share ipv4",
                limits.public_share.ipv4,
                Duration::from_secs(2),
                30,
            ),
            (
                "public_share ipv6",
                limits.public_share.ipv6,
                Duration::from_secs(1),
                60,
            ),
            (
                "share_preview ipv4",
                limits.share_preview.ipv4,
                Duration::from_secs(2),
                10,
            ),
            (
                "share_preview ipv6",
                limits.share_preview.ipv6,
                Duration::from_secs(1),
                20,
            ),
            (
                "health ipv4",
                limits.health.ipv4,
                Duration::from_secs(2),
                30,
            ),
            (
                "health ipv6",
                limits.health.ipv6,
                Duration::from_secs(1),
                60,
            ),
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
            PerFamily {
                ipv4: Limit {
                    period: hour,
                    burst: 5
                },
                ipv6: Limit {
                    period: hour,
                    burst: 10
                },
            },
            "both families"
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
