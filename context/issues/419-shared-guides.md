# 419. Guides live only on zwipe.net

State: fixing | Owner: halo | PR: pushed to release in phases | Build seen: ea76d3002

## Report
From Scotty, through the composer: guide content moves to one shared place that zwipe.net and the web app both render from. The web app gets public guide routes (signed in or out) with `rel=canonical` to zwipe.net. Every guide link inside the web app opens the web app's guide, and the phone app's guide links open app.zwipe.net's guide in the browser. The plan is `context/plans/shared_guides.md`.

## Fix
halo, in phases:
1. **The guides to zwipe-core:** `content::guides` holds `Guide`, `Block`, `GUIDES`, `image_size`, `screenshots` and `find`. zite renders from it with no visible change, and its build script checks the sitemap's slugs against core's file.
2. **The screenshots to one public path at zwipe.net:** the 22 files move from `zite/assets/guides/` to `zite/public/guide-images/` (served as-is at `https://zwipe.net/guide-images/<slug>/<file>`). zwipe-core's `image_url` builds that address from `site::WEB_BASE`, and zite's pages load it. zite's `asset!` registry is gone. Its tests check that every screenshot is in `public/guide-images/` and matches its frame's size. Not checked live: port 8080, where a debug build points, was held by another session's dx. Surfer checks after deploy.
3. **The rendering to components:** pending.
4. **The web app's routes and the link bases:** pending, after 418.

## Verification
-
