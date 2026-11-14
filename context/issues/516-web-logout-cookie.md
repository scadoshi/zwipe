# 516. A web logout that fails to reach the server leaves the refresh cookie live

State: fixing | Owner: thinker | Type: security | PR: - (pushed to release) | Build seen: 5db0a69bf

zwipe-context issue #44, from security audit run-3 (fingerprint `web-session/logout:cookie-survives-failed-server-logout`).

## Report
zwiper-web's logout cleared the stored session whether `POST /api/auth/cookie/logout` landed or not. The refresh token lives in the `HttpOnly` `__Host-zwipe_refresh` cookie, which page code cannot delete; only a logout the server answers expires it. When the logout failed (offline, past the 5 s timeout, or `ensure_fresh` failing first on a 5xx or 429), Login showed but the cookie stayed good for its 14 days, and the next person on that browser could resume the account deliberately (a console `fetch` to `/api/auth/cookie/refresh`, or a forged `zwiper-web.session` entry).

## Before
`zerver/tests/auth_cookie_session.rs`: a logout with the cookie but no access token (an expired one, or one `ensure_fresh` could not get) answered 401, and the cookie still refreshed afterwards.

## Cause
- `cookie_logout` (`zerver/src/lib/inbound/http/handlers/auth/cookie_session.rs`) required a valid access token, and its route sat in the private router behind a per-user rate limiter that answers 401 without one, so the cookie alone could not end its session.
- `zwiper-web/src/lib/inbound/session/logout.rs` tried once, needed `ensure_fresh` to succeed first, logged a failure and moved on: no retry, nothing left to try again.

## Fix
- **Server:** `/api/auth/cookie/logout` is public, under the refresh route's IP rate limit, and still checks the `Origin`. With an access token it behaves as before. Without one, the cookie alone names the session: the family its token belongs to ends, and a cookie whose token the server does not hold ends nothing, so a forged cookie logs nobody out. The cookie expires either way.
- **Client:** logout tries the token route, then the cookie alone (`CookieLogoutAlone`, no bearer). If neither lands, a `zwiper-web.logout-pending` mark stays in storage and the logout goes again by the cookie at the next start and whenever the browser comes back online, until it lands. A new login replaces the cookie and clears the mark.
- **Wire:** additive. The existing `CookieLogout` (with bearer) is unchanged, and the native app does not use the cookie routes. A web build ahead of the server keeps working: its first try is the token route, and the cookie-only fallback that an old server refuses stays pending until the server is live.

## Tests
- `logout_with_the_cookie_alone_ends_its_session`: red before (401, cookie still refreshes); green after (2xx, `Max-Age=0`, refresh 401).
- `a_forged_cookie_logs_nobody_out`: the victim's id with a token not theirs ends nothing; red with the cookie-only path falling back to "end every session".
- `logout_by_the_cookie_still_needs_an_allowed_origin`: 403.
- zwiper-web: `a_logout_the_token_route_misses_goes_by_the_cookie`, `a_logout_no_route_reaches_waits_for_the_next_start` (mark kept while offline, cleared once heard), `a_logout_the_token_route_ends_sends_nothing_more`; `pending_logout::the_mark_stays_until_cleared`.
- `route_coverage`: the route answers without a token (403 with no `Origin`).

## Not covered
- A logout made fully offline leaves the cookie live until the browser is next online with the page open: a page cannot expire an `HttpOnly` cookie, and the next person can still call refresh by hand before then. A session-only cookie (no `Max-Age`, gone when the browser closes) would close that, at the cost of staying signed in across browser restarts; that is a product call.
- A browser run: the checks are the server's router tests and the client's logic tests.
