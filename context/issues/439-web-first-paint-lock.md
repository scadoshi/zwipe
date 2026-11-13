# 439. A fresh web app tab can sit on the List skeleton for about ten seconds

State: fixing | Owner: halo | PR: release | Build seen: 148a4dffd

## Report
From the composer, from Surfer's trace: `ensure_fresh` (zwiper-web `session/ensure_fresh.rs`) waits on the cross-tab `zwipe-refresh` Web Lock (`session/cross_tab.rs`, `HOLD_LIMIT_MS` 20s) before its refresh, and the deck fetch waits on that refresh, so another tab mid-refresh, a slow refresh (`REQUEST_TIMEOUT` 30s) or a leaked handle stalls first paint. Fix: (1) after taking the lock, skip the refresh when another tab already refreshed the shared session; (2) bound the wait for the lock to a couple of seconds, then refresh anyway; (3) shorten `HOLD_LIMIT_MS` and always release the handle (a drop guard, error paths included). The cross-tab guard against a double refresh stays (refresh-token reuse rules). (4) The loading skeleton ignores the saved View: with View set to Stacks it draws List rows (`screens/deck/cards/view.rs:393`); draw a Stacks skeleton then. Tests for the lock wait and the skip. Queued after 433.

From droider, through the composer: a reload with DevTools attached stayed blank for about 10 to 12 seconds, then the whole page appeared at once.

## Fix
halo:
- **The lock wait is bounded.** `CrossTabLock::acquire` asks for the `zwipe-refresh` Web Lock with an abort signal and gives up after `WAIT_LIMIT_MS` (2s), then refreshes without it, as it does where the browser has no Web Locks. The server answers a refresh token presented twice within `REFRESH_REUSE_SECONDS` (30s by default) with the same successor, which covers two tabs refreshing together. Tokens stay in each tab's memory; nothing passes between tabs (the composer's call).
- **The hold is short and always ends.** `HOLD_LIMIT_MS` goes from 20s to 5s, and the lock goes back when `CrossTabLock` drops, on every path out of the refresh task, error paths included.
- **The skeleton follows the saved View.** With View set to Stacks the deck page loads under `DeckStacksGhost`: each stack's group chip over its cards, tucked by zwipe-ui's `stacks::tucked` as a loaded stack lies.

## Checked
- Real Chrome on real time (DevTools protocol), the app's lock script with a stub bridge: behind a holder that never lets go, a tab goes on at 2.06s and the holder keeps its lock; with a holder that lets go at 650ms, the waiting tab gets the lock at 656ms.
- The Stacks ghost in headless Chrome with the web app's sheets: four stacks of 1, 6, 3 and 4 tucked cards under their chips.
- Not checked: a real multi-tab load against the live server (droider's repro), which needs a signed-in session.

## Verification
-
