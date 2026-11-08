# Verify: the 1d7053e88 deploy

Deployed 2026-11-08. Surfer (web, wide and 390) and Droid (Android app, plus phone-width web) check each line on live and mark it pass or fail with what they saw. Anything new they notice goes into this folder as a `reported` issue first, never straight to a fix.

Hard-refresh the web app and zwipe.net before starting (see 282). Droid uses a fresh build of release at 1d7053e88 and says which.

Signing out is safe again after 291 is checked, but only the 291 line should sign out.

| Item | What to check | Web | App | Result |
|---|---|---|---|---|
| 285 | A shared deck's preview image has the new layout: grid tiles behind one panel, name first, then pips and the deck list's tags (count, format, power, price, every command zone card, deck tags), curve bottom right, "Art" tag with the artists, zwipe.net bottom right | Surfer | | |
| 291 | Sign in as test on the app and on phone-width web. Sign out of the app: web stays signed in. Sign back in on the app, sign out of web: the app stays signed in | | Droid | |
| 290 part 2 | Format picker: tap Pauper, ×. The prompt says "Keep editing" and "Discard". Same on Deck tags and Oracle tags | Surfer | Droid | |
| 284 | Settings on a short page at 390x700 and 1200x600, top and bottom: toggle Universes Beyond. The toast sits bottom left, just above the footer, never over it | Surfer | Droid (390 web) | |
| 292 | The web app's About shows 1.18.9 to 1.18.12 upcoming and 1.18.0 to 1.18.8 released. /api/changelog sends `no-cache` and an ETag; a reload gets a 304 | Surfer | | |
| 287 web | (rides on 292) the web app's About matches zwipe.net's changelog | Surfer | | |
| 265 | Create deck, Tymna: the partner pile has only plain Partner cards (no Friends forever, Character select, Father & son, Survivors). Wernog: only Friends forever. Save Tymna + Wernog: it saves, with a "cannot be partners" warning | Surfer | Droid | |
| 293 | Create deck, Pir: Toothy fills in. The partner Swipe pile says "No partners to show", not a skeleton | Surfer | Droid | |
