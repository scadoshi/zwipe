# 282. The web app keeps running the old bundle after a deploy

State: merged | Owner: scotland | PR: #454 | Build seen: 1e03520ef

## Report
Scotty, 2026-11-07: after the 1e03520ef deploy, app.zwipe.net behaved like the old build (281) until a hard refresh. Suspect: `index.html` (or the service worker, if any) is cached long enough to keep serving the previous hashed bundle. Check `zwiper-web/deploy/_headers` and Cloudflare's cache rules for `index.html`. Not approved yet.

## Decision
Scotty, 2026-11-08: no heartbeat. A refresh or an in-app page change should pick up the new app. Plan: index.html served no-cache (refresh always gets the new shell); on in-app navigation, check a tiny version file (throttled, e.g. once per 5 min) and, if a newer build is live, turn that navigation into a full page load to the same target. Never interrupts a form: the leave guard runs first.
