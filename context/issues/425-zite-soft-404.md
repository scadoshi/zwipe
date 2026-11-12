# 425. zwipe.net answers an unknown path with a 200, and Esc leaves the featured card's art open

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-13: (1) zwipe.net's Page not found is served with HTTP 200, a soft 404 that search engines index. An unknown path should answer 404 with the same page, and the shared deck 301 Functions must still win. (2) Esc should close the featured flavor card's art overlay on the home page; a backdrop click already does.

## Fix
scotland:
- The SSG prerenders the not-found page at `/404/`, and the build moves it to a top-level `404.html`, which Cloudflare Pages serves with a 404 for any path it has no file for. Pages stops treating the site as a single-page app once `404.html` exists, so:
  - the per-token pages the SSG can't prerender (`/verify/:token`, `/confirm-email/:token`, `/reset/:token`) get Pages Functions that serve the shell with a 200, as Cloudflare's fallback did;
  - the router's client-side redirects (`/discord`, `/guides/suggested-commanders`) move to `_redirects` as 301s, or they would answer 404 first;
  - the deck Functions still answer before any static file or `404.html`.
- The home page registers the flavor overlay with the overlay back stack, so Esc runs the same 200ms dismissal the backdrop click does.

## Verification
- After deploy: `curl -sI https://zwipe.net/no-such-page` is `404`; `curl -sI https://zwipe.net/verify/x` is `200`; `curl -sI https://zwipe.net/discord` is `301` to `/community/`; `curl -sI https://zwipe.net/<owner>/deck/<slug>/<token>` is `301` to app.zwipe.net.
- On the home page, open the featured card's art and press Esc: it fades out.
