//! Keeps the rate limiters' memory bounded. Each limiter holds one entry per client it has seen,
//! and governor never drops one on its own, so a stream of new addresses would grow it for the
//! life of the process. [`purging`] shares a limiter's config and starts a task that, every
//! [`LIMITER_PURGE_EVERY`], drops the clients whose budget has fully refilled.

use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use tower_governor::governor::GovernorConfig;
use tower_governor::key_extractor::KeyExtractor;

/// How often each limiter drops the clients whose budget has fully refilled. Every limiter here
/// refills within an hour, so a client stays at most that plus a minute after its last request.
pub const LIMITER_PURGE_EVERY: Duration = Duration::from_secs(60);

/// `config`, shared, with a task on the current Tokio runtime that every `every` drops the
/// clients whose budget has fully refilled (`retain_recent`, then `shrink_to_fit` to hand the
/// memory back). A client still waiting keeps its count. The task holds the limiter weakly and
/// ends once the limiter is dropped. Off a runtime nothing purges.
pub fn purging<K>(
    config: GovernorConfig<K, NoOpMiddleware<QuantaInstant>>,
    every: Duration,
) -> Arc<GovernorConfig<K, NoOpMiddleware<QuantaInstant>>>
where
    K: KeyExtractor,
    K::Key: Hash + Eq + Clone + Send + Sync + 'static,
{
    let config = Arc::new(config);
    let limiter = Arc::downgrade(config.limiter());
    if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        runtime.spawn(async move {
            let mut tick = tokio::time::interval(every);
            // The first tick is immediate; the first purge waits a whole interval.
            tick.tick().await;
            loop {
                tick.tick().await;
                let Some(limiter) = limiter.upgrade() else {
                    break;
                };
                limiter.retain_recent();
                limiter.shrink_to_fit();
            }
        });
    }
    config
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv6Addr};
    use std::time::Duration;

    use tower_governor::governor::GovernorConfigBuilder;

    use super::*;
    use crate::inbound::http::middleware::CfConnectingIpKeyExtractor;

    fn address(n: u128) -> IpAddr {
        IpAddr::V6(Ipv6Addr::from(n))
    }

    #[tokio::test]
    async fn a_client_whose_budget_refilled_is_dropped_and_a_limited_one_kept() {
        let quick = purging(
            GovernorConfigBuilder::default()
                .per_millisecond(10)
                .burst_size(1)
                .key_extractor(CfConnectingIpKeyExtractor)
                .finish()
                .unwrap(),
            Duration::from_millis(50),
        );
        for n in 0..100 {
            let _ = quick.limiter().check_key(&address(n));
        }
        assert_eq!(quick.limiter().len(), 100);
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(
            quick.limiter().len(),
            0,
            "every budget refilled in 10 ms, so all went"
        );

        let slow = purging(
            GovernorConfigBuilder::default()
                .period(Duration::from_secs(3600))
                .burst_size(1)
                .key_extractor(CfConnectingIpKeyExtractor)
                .finish()
                .unwrap(),
            Duration::from_millis(50),
        );
        let _ = slow.limiter().check_key(&address(1));
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(
            slow.limiter().len(),
            1,
            "a client still waiting keeps its count"
        );
    }

    /// The process's resident memory in KiB, from `ps`.
    fn rss_kib() -> u64 {
        let out = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().parse().unwrap()
    }

    /// What a million clients cost a limiter and what one purge gives back, printed:
    /// `cargo test -p zerver --lib limiter_purge -- --ignored --nocapture`.
    #[test]
    #[ignore = "a measurement, not a check: prints memory and timings"]
    #[expect(
        clippy::print_stdout,
        reason = "a measurement: printing its numbers is the point"
    )]
    fn measure_a_million_clients() {
        let config = GovernorConfigBuilder::default()
            .period(Duration::from_secs(1))
            .burst_size(60)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .unwrap();
        let limiter = config.limiter();
        let before = rss_kib();
        let started = std::time::Instant::now();
        for n in 0..1_000_000_u128 {
            let _ = limiter.check_key(&address(n << 64));
        }
        let filled = rss_kib();
        println!(
            "1,000,000 clients: {} KiB more ({} bytes each), inserted in {:?}",
            filled - before,
            (filled - before) * 1024 / 1_000_000,
            started.elapsed()
        );
        // A client drops once its whole burst has refilled: 60 at one a second.
        std::thread::sleep(Duration::from_secs(61));
        let started = std::time::Instant::now();
        limiter.retain_recent();
        let retained = started.elapsed();
        let started = std::time::Instant::now();
        limiter.shrink_to_fit();
        println!(
            "one purge: retain_recent {retained:?}, shrink_to_fit {:?}, {} clients left, {} KiB above the start",
            started.elapsed(),
            limiter.len(),
            rss_kib().saturating_sub(before)
        );
        let purged = rss_kib();
        for n in 1_000_000..2_000_000_u128 {
            let _ = limiter.check_key(&address(n << 64));
        }
        println!(
            "another 1,000,000 new clients after the purge: {} KiB more, the freed memory reused",
            rss_kib().saturating_sub(purged)
        );
    }
}
