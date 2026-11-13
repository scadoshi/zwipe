# 439. A fresh web app tab can sit on the List skeleton for about ten seconds

State: fixing | Owner: halo | PR: release | Build seen: 148a4dffd

## Report
From the composer, from Surfer's trace: `ensure_fresh` (zwiper-web `session/ensure_fresh.rs`) waits on the cross-tab `zwipe-refresh` Web Lock (`session/cross_tab.rs`, `HOLD_LIMIT_MS` 20s) before its refresh, and the deck fetch waits on that refresh, so another tab mid-refresh, a slow refresh (`REQUEST_TIMEOUT` 30s) or a leaked handle stalls first paint. Fix: (1) after taking the lock, skip the refresh when another tab already refreshed the shared session; (2) bound the wait for the lock to a couple of seconds, then refresh anyway; (3) shorten `HOLD_LIMIT_MS` and always release the handle (a drop guard, error paths included). The cross-tab guard against a double refresh stays (refresh-token reuse rules). (4) The loading skeleton ignores the saved View: with View set to Stacks it draws List rows (`screens/deck/cards/view.rs:393`); draw a Stacks skeleton then. Tests for the lock wait and the skip. Queued after 433.

From droider, through the composer: a reload with DevTools attached stayed blank for about 10 to 12 seconds, then the whole page appeared at once.

## Fix
halo:
- **The lock wait is bounded.** `CrossTabLock::acquire` asks for the `zwipe-refresh` Web Lock with an abort signal and gives up after `WAIT_LIMIT_MS` (2s), then refreshes without it, as it does where the browser has no Web Locks. The server answers a refresh token presented twice within `REFRESH_REUSE_SECONDS` (30s by default) with the same successor, which covers two tabs refreshing together. Tokens stay in each tab's memory; nothing passes between tabs (the composer's call).
- **The hold is short and always ends.** `HOLD_LIMIT_MS` goes from 20s to 5s, and the lock goes back when `CrossTabLock` drops, on every path out of the refresh task, error paths included.
- **The skeleton follows the saved View, on the web and in the app.** components' `DeckStacksGhost` draws each stack's group chip over its cards, tucked by zwipe-ui's `stacks::tucked` as a loaded stack lies, in two columns with `touch`. The web deck page and the app's Cards layer show it in place of their List ghosts when View is Stacks.
- **Not fixed: the app's Cards layer loads the deck again on every open.** Since 430 the layer mounts the whole Cards screen over the Profile, and each screen keeps its own model and fetch (`fetch_deck` in both), so the layer's ghost shows for the fetch each time. The Profile also loads only when the deck id changes, so edits made in the layer leave it stale until the deck is opened again. Proposed to the composer: one loaded deck owned by the deck route and shared by both, which the layer opens on and its writes update.

- **Follow-ups from Surfer and droider on the ghost.** The Stacks ghost lays out in the loaded stacks' column count: two on touch, else the count the measured width fits (the same `use_stacks_width` and `columns::column_count` `DeckStacks` uses), the grid before the first measure as the loaded stacks start. The web's Cards layer (`?view=cards` at phone width) shows the saved view's ghost while the deck loads; it showed none. The app's featured strip holds its place with one ghost card while the deck loads (unless the profile has landed with no command zone card), and a strip that follows its ghost opens in place with its cards over ghost image boxes, so the controls under it never move.

## Checked
- Real Chrome on real time (DevTools protocol), the app's lock script with a stub bridge: behind a holder that never lets go, a tab goes on at 2.06s and the holder keeps its lock; with a holder that lets go at 650ms, the waiting tab gets the lock at 656ms.
- The Stacks ghost in headless Chrome with the web app's sheets: four stacks of 1, 6, 3 and 4 tucked cards under their chips.
- Not checked: a real multi-tab load against the live server (droider's repro), which needs a signed-in session.

## Verification
-
