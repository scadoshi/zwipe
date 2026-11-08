# 312. Three guides have no screenshots

State: merged | Owner: scotland | PR: #461 | Build seen: 1f354453d

## Report
Surfer, live zwipe.net, 2026-11-08, while verifying 310. Every guide has new screenshots except three, which show none at all: Playstyle (/guides/playstyle/), Activity (/guides/activity/) and "Deck tags, card roles, and Oracle tags: how they fit" (/guides/tags-roles-and-oracle-tags/). None of the three has an entry in `zite/src/pages/guides/content.rs`'s `guide_image` table. Playstyle and Activity are two of the four new demo clips on the home page, so they're the screens most worth showing.

## Repro
1. Open https://zwipe.net/guides/playstyle/ (or /activity/, /tags-roles-and-oracle-tags/).
2. No screenshot carousel; the other 19 guides have one.
