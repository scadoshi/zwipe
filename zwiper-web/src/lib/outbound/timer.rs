//! Waiting: the browser's timer in the web build, tokio's elsewhere.

use std::future;
use std::pin::{Pin, pin};
use std::task::Poll;
use std::time::Duration;

/// Resolves after `duration`.
pub async fn sleep(duration: Duration) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::sleep(duration).await;
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(duration).await;
}

/// Runs `work` for up to `limit`. `None` when the time ran out first.
pub async fn timeout<T>(limit: Duration, work: impl Future<Output = T>) -> Option<T> {
    let work = pin!(work);
    let deadline = pin!(sleep(limit));
    match first(work, deadline).await {
        Either::Left(value) => Some(value),
        Either::Right(()) => None,
    }
}

/// Which of two futures finished first.
enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Polls `left` then `right` until one is ready; `left` wins a tie.
async fn first<L, R>(
    mut left: Pin<&mut impl Future<Output = L>>,
    mut right: Pin<&mut impl Future<Output = R>>,
) -> Either<L, R> {
    future::poll_fn(move |cx| {
        if let Poll::Ready(value) = left.as_mut().poll(cx) {
            return Poll::Ready(Either::Left(value));
        }
        if let Poll::Ready(value) = right.as_mut().poll(cx) {
            return Poll::Ready(Either::Right(value));
        }
        Poll::Pending
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn work_that_finishes_in_time_returns_its_value() {
        assert_eq!(timeout(Duration::from_secs(5), async { 7 }).await, Some(7));
    }

    #[tokio::test(start_paused = true)]
    async fn work_that_never_finishes_times_out() {
        let out = timeout(Duration::from_secs(5), future::pending::<()>()).await;
        assert_eq!(out, None);
    }
}
