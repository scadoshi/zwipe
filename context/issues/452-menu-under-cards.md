# 452. On phone web, the menu opens under the Cards layer

State: fixing | Owner: thinker | Type: issue | PR: release | Build seen: production

## Report
Composer for Scotty, 2026-11-12: phone web at app.zwipe.net: open a deck, tap Cards (the full-screen layer), tap the hamburger: nothing seems to happen, because the menu opens under the Cards layer. The menu must sit above Cards (and above card details once 449 makes them full screen). The phone app is fine.

## Cause and fix
The phone web nav is an absolutely placed bar at z-index 101 (`web.css`, below 1110px), and its menu drops inside it. The Cards layer took its z from the overlay stack (`--overlay-z`, 200 and up), so it sat over the nav's menu. scotland's 4059d64f8 (453) puts the phone web Cards layer at z-index 100, under the header and over the Profile, with sheets at 200 and up, so the menu drops over the Cards. No change of thinker's needed.

## Verification
- Phone web: a deck's Cards, then the hamburger: the menu drops over the Cards (droider's before-452 steps); Escape or Back closes it, then Back closes the Cards.
