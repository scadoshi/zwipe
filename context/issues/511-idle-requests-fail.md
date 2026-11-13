# 511. The app says "Couldn't load" after a minute idle, and Retry fixes it

State: fixing | Owner: thinker | Type: bug | PR: - (pushed to release) | Build seen: 1.20.5

## Report
droider, via the composer: the app shows "Couldn't load deck" (and Home "Couldn't load your decks / playstyle") after about 60 s idle, not only at a cold start. The log shows `/api/deck/profile` and `/api/deck` failing with "network error: error sending request for url"; Retry fixes it. Logs: `halo:/private/tmp/claude-501/.../611/rmload/`.

## Before
From droider's logs (Android emulator, app 1.20.5):
- 18:37:42 a GET, then 37 s idle, then the next GET (`/api/deck/{id}/tokens`) fails 0.2 s after it is sent.
- 19:36:15 a cold start; at 19:36:18 about 18 GETs go out at once and `/api/deck/playstyle` and `/api/deck` fail 2 s later.
- 20:10:47 `/api/deck/profile/{id}` and `/api/deck/{id}` fail together.
A device repro with a count (idle 60 to 90 s, then navigate) is droider's to run.

## Cause
- The native client (`zwipe-client/src/lib.rs`, reqwest over HTTP/2 to Cloudflare) keeps an idle connection pooled for reqwest's default 90 s and sends a request on it without knowing it is gone; the failure is reqwest's kind `Request`, hyper `SendRequest` / `IncompleteMessage`, which prints "error sending request". Nothing retried it.
- The server side does not close it: measured from here, Cloudflare's edge in front of zerver keeps an idle client connection for 400 s, on HTTP/2 (GOAWAY at 400.0 s) and HTTP/1.1 (closed at 400.2 s). The drop is on the path between: the emulator's or a phone's network letting an idle socket go. The cold-start burst failed with no pooled socket at all, so a fresh connection can fail too.

## Fix
- A GET whose connection fails under it (reqwest `is_request` or `is_connect`, not a timeout) goes once more on a fresh connection (`zwipe-client/src/call.rs`, `send_once_more_if_dropped`). A write never goes twice. The browser build is unchanged: the browser keeps its own connections.
- The pool lets an idle connection go after 30 s instead of 90 s.

## Tests
- `a_get_on_a_stale_pooled_connection_goes_again_on_a_fresh_one`: a local server answers once, then drops the kept-alive socket under the next GET. Red before with the app's error (`SendRequest`, `IncompleteMessage`); green after, on one fresh connection.
- `a_write_on_a_stale_pooled_connection_is_not_sent_twice`: the same with a POST: it fails and opens no second connection.

## Not checked
- The device: droider's before count and after on a build with this.
- The card searches are POSTs (read-only), so they don't retry; a stale socket under Quick add still fails once.
