//! Waiting, the same way on every target: tokio's timers in the native builds, the browser's in
//! the web build, where tokio has no timer driver and its timers panic.

use std::time::Duration;

/// Resolves after `duration`.
pub async fn sleep(duration: Duration) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::sleep(duration).await;
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(duration).await;
}

/// Runs `work` for up to `limit`: its output, or `None` when the time ran out first.
pub async fn timeout<T>(limit: Duration, work: impl Future<Output = T>) -> Option<T> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::time::timeout(limit, work).await.ok()
    }
    #[cfg(target_arch = "wasm32")]
    {
        use std::pin::pin;
        use std::task::Poll;

        let mut work = pin!(work);
        let mut deadline = pin!(sleep(limit));
        std::future::poll_fn(move |cx| {
            if let Poll::Ready(value) = work.as_mut().poll(cx) {
                return Poll::Ready(Some(value));
            }
            if deadline.as_mut().poll(cx).is_ready() {
                return Poll::Ready(None);
            }
            Poll::Pending
        })
        .await
    }
}

/// A tick every `period`, the first at once. A tick that comes late pushes the next ones back
/// rather than firing the missed ones in a burst.
#[derive(Debug)]
pub struct Ticker {
    /// tokio's interval, with [`tokio::time::MissedTickBehavior::Delay`].
    #[cfg(not(target_arch = "wasm32"))]
    interval: tokio::time::Interval,
    /// The wait between ticks.
    #[cfg(target_arch = "wasm32")]
    period: Duration,
    /// Whether the first tick has gone.
    #[cfg(target_arch = "wasm32")]
    started: bool,
}

impl Ticker {
    /// A ticker every `period`.
    #[must_use]
    pub fn new(period: Duration) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut interval = tokio::time::interval(period);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            Self { interval }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                period,
                started: false,
            }
        }
    }

    /// Resolves at the next tick.
    pub async fn tick(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.interval.tick().await;
        }
        #[cfg(target_arch = "wasm32")]
        {
            if self.started {
                sleep(self.period).await;
            }
            self.started = true;
        }
    }
}
