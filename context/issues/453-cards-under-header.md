# 453. On phone web, scrolling Cards leaves a header-height strip above the first row

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-13 (iOS Safari): in a deck's Cards on phone web, scrolling hides the header (back, title, `?`, menu), and the list doesn't move up into that space; a header-height strip of grid tiles shows above the first row. The phone app is fine; match it.

## Fix
scotland: where the web's header floats and slides away on scroll (below 1110px), the Cards layer starts at the window's top under the header, as every other screen does, with the header's height as room above its content, so its cards scroll up under the header and fill the space it leaves. It sits under the header (z 100, the header 101) and over the Profile; every sheet opened over it is at 200 and up, and the card details screen (449) covers the header.

## Verification
- Phone web (412): open Cards, scroll down: the header slides away and cards fill its space, with no strip of tiles above them. Scroll up: the header comes back over the cards.
