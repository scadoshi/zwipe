# Refresh reuse window

**Status: BUILT, PR open for Scotty's review (touches auth). Not deployed.**

- [x] 2026-11-07 Plan written (this file).
- [x] 2026-11-07 Server: a just-rotated refresh token presented again inside the window gets the same successor back (`REFRESH_REUSE_SECONDS`, default 30).
- [x] 2026-11-07 Web: cross-tab single flight for the cookie refresh (Web Locks), so tabs take turns.
- [x] 2026-11-07 Tests: concurrent refreshes share one successor, a replay past the window still revokes.
- [ ] Scotty's review, then merge into `release` and deploy zerver.

**One sentence:** two tabs (or one page firing two refreshes) racing on the same refresh token should both stay logged in, without giving a thief a way to hide a stolen token.

## The race

Surfer's #55: a second app.zwipe.net tab logged the test account out of both tabs.

Both tabs hold the same HttpOnly refresh cookie. When their access tokens go stale they each call `POST /api/auth/cookie/refresh` with it. `use_refresh_token` (zerver `outbound/sqlx/auth/mod.rs`) locks the row `FOR UPDATE`. The first request marks it replaced and mints a successor. The second blocks on the lock, wakes after the first commits, and sees `replaced_at` set. Since 2026-10-10 any replaced token presented again is a replay: the whole family is deleted and the call returns `RefreshSessionError::Revoked`. The first tab's new token dies with the family, so both tabs are signed out.

Why tabs race so readily: the web's session store keeps no access token (`SessionStore` writes a placeholder JWT that expired in 1970), so **every tab refreshes as it loads**. Two tabs opened together, a reload while another tab refreshes, or a browser restoring its tabs at startup all send the same cookie at once. The app (one process, bearer refresh) has an in-process single flight and rarely races itself. The web had one only per tab, and a tab can't see another tab's mutex.

## One tab is enough: a lost reply

Surfer was logged out again after about a minute and a half of full page loads in a single tab. Production logged `token_refresh_failure reason="revoked"` for the test account at 07:44:45Z on 2026-11-07 (read-only check of `/var/log/zwipe/zerver.2026-11-07.log`; successful rotations aren't logged). Each load refreshes, so a load that unloads with a refresh in flight lets the server rotate while the browser never applies the reply. The next load sends the old cookie, which was a replay.

The window covers it: an old token presented again within 30 seconds of its rotation gets the family's newest token, following the chain if the successor was itself rotated meanwhile. `a_lost_response_then_the_old_token_again_gets_the_newest_successor` runs that sequence. It only fails if one token's replies keep getting lost for longer than the window, which then logs out as today.

**Open question: a refresh on every load.** The web refreshes on every full page load even with a valid access token, by design: `SessionStore` never writes a token where page script can read it (`localStorage`), so a reload has no access token. Not refreshing on load means either keeping the access token in script-readable storage (undoing that XSS hardening, a 15-minute token), or authenticating API calls with an HttpOnly access cookie (a wire and CSRF change). Both are Scotty's call. With the window and the lock, the per-load refresh is safe, just a rotation per load.

## Other ways a session ends (Scotty's Firefox, Surfer's login tab)

Scotty's own session keeps dropping in Firefox, and Surfer's Chrome had another tab sitting on the login page. Checked whether a signed-out screen ends the live session:

- **No signed-out screen logs out or refreshes on a timer.** Cookie logout runs only from Log out (the menu and Account) and from deleting the account. A tab's upkeep loop refreshes once a minute only while that tab holds a session; a `/login` tab holds none, so it sends nothing. The signed-out gate refreshes once as it mounts, and only when the tab started with a stored session, to send a live session to Home. That refresh is an ordinary one, so it raced like any other tab load; the lock and the window cover it now.
- **Tab restore is the likeliest Firefox cause.** Firefox restores a window's tabs together, and each one refreshes on load, so a restore is the two-tab race several times over. This fix covers it.
- **Logout is every device.** Log out in any tab, browser or device deletes all of the user's refresh tokens, by design. Logging out of a test build or another browser ends the Firefox session too. Unchanged here.
- **The five-session cap.** Each login starts a family, and a user keeps at most five live tokens, so the five most recently created survive (`prune_users_refresh_tokens`, `MAXIMUM_SESSION_COUNT`). Every rotation makes a new row, so the cap keeps the five most recently refreshed sessions. Someone signed in on six or more places (phone, simulator builds, several browsers, two machines) loses whichever has been idle longest, such as a backgrounded Firefox, whenever another one logs in or refreshes. Families idle for under 14 days still count. Unchanged here: raising the cap, or not counting families idle for days, is a policy call for Scotty.

## History: why there is no grace window today

A 10-second grace window existed from 2026-10-09 to 2026-10-10 (`REFRESH_REPLAY_GRACE`, commit 12fd51a50). Inside it, the loser of a race got a plain 401 and the family lived. Commit 5b4fa69c2 removed it: an attacker who rotated a stolen token just before the victim kept the family, while the victim got the 401 and was signed out. The window hid the attacker and punished the victim.

The fix here is a different window, and the difference is the point.

## The fix: same successor, not a second branch

Inside the window, a token presented again does not get a 401 and does not get a fresh rotation. It gets **the same successor** the first rotation minted. Both racers then hold one token, the family has one head, and replay detection still works one step later:

- Two honest tabs: both get successor S and carry on. Whichever rotates S next leaves the other holding a spent S. That tab's next refresh is a replay of S, but the web's cross-tab lock (below) means it re-reads the shared cookie first, so in practice it never presents S again.
- Thief first, victim inside the window: both hold S. The next time either rotates S, the other's later use of S (past the window) is a replay and kills the family, the thief's token included. Detection is delayed by one rotation, not lost.
- Thief or victim more than the window after the rotation: a replay, revoked exactly as today.

A **fresh rotation** of the same family (the plan's other option) would give each racer an independent branch. Two branches never collide again, so a thief who won a race inside the window would keep a working token indefinitely. That's why this builds the same-successor variant.

## Mechanics

The server stores only the SHA-256 of a refresh token, so it can't hand back a successor's value from the database. The rotation remembers its successor in memory instead:

- `Postgres` (the auth repository) carries a `RefreshReuse`: a map from the consumed token's hash to the successor it minted, the successor's platform, and when it was minted. The window comes from config.
- The winner inserts its entry **before** its transaction commits. A loser blocked on the row lock wakes only after that commit, so it always finds the entry.
- A loser that finds `replaced_at` within the window and an entry for its token gets that successor back. Nothing changes in the database. If that successor was itself rotated inside the window, the lookup follows the chain to the newest one.
- No entry (the server restarted inside the window, or the entry aged out) or a `replaced_at` past the window: the replay path as today, deleting the family and returning `Revoked`.
- Entries older than the window are pruned on every insert. The map holds at most one entry per rotation in the last 30 seconds.

`REFRESH_REUSE_SECONDS` (default 30, 0 to 300, 0 turns the window off) sits beside `ACCESS_TOKEN_MINUTES` in `config.rs`.

One process per deployment is assumed: zerver runs as a single process today. With several processes behind a balancer, a loser on another process would find no entry and fall back to revoking, which is today's behavior and still safe. That would need a shared store if it ever matters.

Ownership and expiry checks are unchanged. A token from another user is still `Forbidden` before any window logic. The family's 30-day absolute lifespan still applies to the successor's next rotation.

## Web: a cross-tab single flight

`ensure_fresh` already single-flights inside one tab with a mutex. Inside that, it now takes a Web Lock (`navigator.locks.request("zwipe-refresh", ...)`, `zwiper-web/src/lib/inbound/session/cross_tab.rs`), so tabs take turns. Each tab still refreshes, since none can see another's access token, but each sends the cookie the one before it rotated, so no two present the same token. That's one extra rotation per tab, cheap, and no tab waits more than one refresh.

The JS side releases the lock after 20 seconds whatever happens, so a lost Rust handle can't wedge every tab. Without `navigator.locks` (very old browsers) the tab goes ahead, and the server window covers the race on its own.

No BroadcastChannel: sharing the fresh session between tabs would save a request per tab but means passing the access token around. Taking turns is enough.

## Wire

No change. Same routes, same request and response shapes. A racing refresh now answers 200 with a session instead of 401, and the app's bearer refresh gets the same server behavior with no client change.

## Tests

`zerver/tests/auth_refresh_families.rs` (needs `DATABASE_URL`, runs in CI):

- Concurrent refreshes of one token all succeed with one shared successor, the family keeps exactly one live row, and that successor rotates normally afterwards.
- A replay inside the window gets the same successor and kills nothing.
- A replay past the window (replaced_at aged in SQL) revokes the family, the successor included.
- The existing rows, absolute-lifespan and evidence-retention tests keep passing.

Unit tests cover the `RefreshReuse` map: lookup inside and outside the window, chaining, pruning.

`auth_cookie_session` and `auth_edges` change the same way: a spent token right away gets the shared successor, and past the window it is refused. All of zerver's tests pass against a local Postgres.

By hand on the web, against a server *without* the window: sign in, then open two tabs on Decks at the same moment. Both stay signed in, and a reload of the first tab afterwards does too.
