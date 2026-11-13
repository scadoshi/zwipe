# 439. A fresh web app tab can sit on the List skeleton for about ten seconds

State: open | Owner: halo | PR: - | Build seen: 148a4dffd

## Report
From the composer, from Surfer's trace: `ensure_fresh` (zwiper-web `session/ensure_fresh.rs`) waits on the cross-tab `zwipe-refresh` Web Lock (`session/cross_tab.rs`, `HOLD_LIMIT_MS` 20s) before its refresh, and the deck fetch waits on that refresh, so another tab mid-refresh, a slow refresh (`REQUEST_TIMEOUT` 30s) or a leaked handle stalls first paint. Fix: (1) after taking the lock, skip the refresh when another tab already refreshed the shared session; (2) bound the wait for the lock to a couple of seconds, then refresh anyway; (3) shorten `HOLD_LIMIT_MS` and always release the handle (a drop guard, error paths included). The cross-tab guard against a double refresh stays (refresh-token reuse rules). (4) The loading skeleton ignores the saved View: with View set to Stacks it draws List rows (`screens/deck/cards/view.rs:393`); draw a Stacks skeleton then. Tests for the lock wait and the skip. Queued after 433.

From droider, through the composer: a reload with DevTools attached stayed blank for about 10 to 12 seconds, then the whole page appeared at once.

## Fix
-

## Verification
-
