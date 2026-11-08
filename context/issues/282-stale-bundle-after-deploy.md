# 282. The web app keeps running the old bundle after a deploy

State: reported | Owner: - | PR: - | Build seen: 1e03520ef

## Report
Scotty, 2026-11-07: after the 1e03520ef deploy, app.zwipe.net behaved like the old build (281) until a hard refresh. Suspect: `index.html` (or the service worker, if any) is cached long enough to keep serving the previous hashed bundle. Check `zwiper-web/deploy/_headers` and Cloudflare's cache rules for `index.html`. Not approved yet.
