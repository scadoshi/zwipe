# 312. Three guides have no screenshots

State: merged | Owner: scotland | PR: #461 | Build seen: 1f354453d

## Report
Surfer, live zwipe.net, 2026-11-08, while verifying 310. Every guide has new screenshots except three, which show none at all: Playstyle (/guides/playstyle/), Activity (/guides/activity/) and "Deck tags, card roles, and Oracle tags: how they fit" (/guides/tags-roles-and-oracle-tags/). None of the three has an entry in `zite/src/pages/guides/content.rs`'s `guide_image` table. Playstyle and Activity are two of the four new demo clips on the home page, so they're the screens most worth showing.

## Repro
1. Open https://zwipe.net/guides/playstyle/ (or /activity/, /tags-roles-and-oracle-tags/).
2. No screenshot carousel; the other 19 guides have one.

## Verify 00ed27f59
Surfer, live zwipe.net and app.zwipe.net after a hard refresh, 2026-11-08.

1. **PASS.** Each guide now has a screenshot carousel. Playstyle has 3 (01-colors, 02-oracle-tag-impact, 03-why-this-commander), Activity has 3 (01-heatmap-and-totals, 02-card-roles, 03-most-skipped), and "Deck tags, card roles, and Oracle tags" has 2 (01-deck-and-oracle-tags, 02-role-to-oracle-tags). All are 860x1862 webp with `object-fit: contain`, aspect within 0.4% (rounding), no stretch or crop. Activity's first shot reads "The Activity screen: the counters, a year of days as a heatmap, and Totals for Added, Skipped, Maybeboard and Removed" (1 / 3), Gruvbox dark with grid tiles.
2. **PASS.** No black border on any of the 4 clips. Drew each playing frame (540x1110) to a canvas and sampled 40 points along every edge, at the outermost pixel row and 2 and 4 px in. Edge luminance is 22-40 on all four sides of all four clips, which is the theme's dark background (#282828 is about 40); black would be near 0. The video box matches the clip's aspect exactly (258x531, no letterbox), transparent background, no CSS border. Zoomed top and bottom edges: ~/Developer/surfer-shots/clips-edge-top.png, clips-edge-bottom.png on halo. Side note: Next started the new clip playing for real clicks, but a scripted `.click()` left it paused at 0:00; that's down to how browsers treat synthetic clicks, not something users hit.
3. **Couldn't test with a real stale tab.** My one tab had moved to zwipe.net before this deploy, and going Back to app.zwipe.net loaded fresh (navigation type reload, bundle zwiper-web-dxh95fcc3bcfed7accb.js, which is this build: version.json says 00ed27f59), so no page from the old build was left. The simulated check from verify-1f354453d (a version.json naming another build makes the next screen change after the 5-minute window reload in full) still stands. To test for real I need to keep an app page open across the next deploy; I'll leave the tab on app.zwipe.net now.
