# 292. Web app's About shows an old changelog for hours after a deploy

State: merged | Owner: halo | PR: #431 | Build seen: 37d09dbf9

## Report
Surfer, live web app (wide), 2026-11-08, during the 37d09dbf9 verify. After a reload, About's changelog still showed 1.18.1 Upcoming (35 items) and 1.18.0 Latest (78 items), the split from before 287. zwipe.net showed the new split. The API itself is right: `GET https://api.zwipe.net/api/changelog` returns 1.18.12 first. But it's served with `cache-control: public, max-age=14400`, and the app's fetch got a cached copy (transferSize 0). The same fetch with `cache: 'no-store'` returns 1.18.12. So for up to 4 hours after a deploy, anyone who opened About before the deploy keeps seeing the old changelog, even after a hard refresh.

## Repro
1. Open About in the web app before a changelog change deploys.
2. After the deploy, hard-refresh and open About again.
3. The old changelog shows until the 4-hour max-age runs out.

## Fix
halo, #431: /api/changelog sends `public, no-cache` plus a SHA-256 ETag and answers 304 on a weak If-None-Match match; the web client fetches it with cache 'no-cache' (Endpoint REVALIDATE flag, wasm only). The 14400 came from the zone's Browser Cache TTL (4h) raising zerver's max-age=3600.

Still for Scotty in Cloudflare: Rule 3 edge-holds /api/changelog 2h ignoring origin (switch to respect origin, or purge on deploy); zone Browser Cache TTL to Respect Existing Headers fixes the browser side generally. Same pattern, not changed: /api/card/oracle-tags, keyword-reminders, roles, ub-franchises (Rule 1 24h edge), marketing/stats (211), featured-flavor.
