# 292. Web app's About shows an old changelog for hours after a deploy

State: approved | Owner: halo | PR: - | Build seen: 37d09dbf9

## Report
Surfer, live web app (wide), 2026-11-08, during the 37d09dbf9 verify. After a reload, About's changelog still showed 1.18.1 Upcoming (35 items) and 1.18.0 Latest (78 items), the split from before 287. zwipe.net showed the new split. The API itself is right: `GET https://api.zwipe.net/api/changelog` returns 1.18.12 first. But it's served with `cache-control: public, max-age=14400`, and the app's fetch got a cached copy (transferSize 0). The same fetch with `cache: 'no-store'` returns 1.18.12. So for up to 4 hours after a deploy, anyone who opened About before the deploy keeps seeing the old changelog, even after a hard refresh.

## Repro
1. Open About in the web app before a changelog change deploys.
2. After the deploy, hard-refresh and open About again.
3. The old changelog shows until the 4-hour max-age runs out.
