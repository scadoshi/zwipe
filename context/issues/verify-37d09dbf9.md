# Verify: the 37d09dbf9 deploy

Deployed 2026-11-08. Surfer (web, wide and 390) and Droid (Android app, plus phone-width web) check each line on live and mark it pass or fail with what they saw. Anything new they notice goes into this folder as a `reported` issue first, never straight to a fix.

Hard-refresh the web app and zwipe.net before starting (see 282). The Android app needs a fresh build for its lines; Droid says which build it tested.

Not in this deploy: 291 (in progress) and 285 part 2 (the new layout, in progress). Check 285 for the current image only.

| Item | What to check | Web | App | Result |
|---|---|---|---|---|
| 221 web | Open a deck from Home's tile, header Back returns to Home; from Decks, back to Decks | Surfer | | |
| 258 | Skip a card on Add, undo; it shows on Add again (search by exact name, Synergy off). Same for a swipe to maybeboard, undone | Surfer | Droid | |
| 270 | Run a long import: the link box stays at the top, the Imported list scrolls inside a fixed window, the screen doesn't grow | Surfer | Droid | |
| 273 | Delete deck's dialog says "Delete <deck name>? This can't be undone." | Surfer | Droid | |
| 278 | Every label tag (Add's "Add" too) has the tinted fill and a matching outline | Surfer | Droid | |
| 279 | Home's Decks panel has Create deck beside All decks; no wrapping at 390 | Surfer, Droid (390) | Droid | |
| 280 | Playstyle panel titles are all the same size (Summary, Colors, Card roles, Oracle tags, Suggested commanders, Staples, Danger zone) | Surfer | Droid | |
| 283 | Changelog versions collapse with a caret, plus Collapse all, on the web app, zwipe.net and the app's About; all start expanded | Surfer | Droid | |
| 285 | A shared deck link's preview image is generated (deck name, format, colors, price, curve, tags, art when there's a command zone) in the owner's theme. Check the share page's og:image URL loads a PNG | Surfer | | |
| 287 | Changelog shows 1.18.0 to 1.18.8 released and 1.18.9 to 1.18.12 upcoming; zwipe.net's banner names the latest release | Surfer | Droid | |
| 265 | Create deck with a partner commander: the partner Swipe pile only offers legal partners for that commander; changing the commander deals a fresh pile | Surfer | Droid | |
| 288 | Add with the network down, Refresh: only "Connection error", no "Stack refreshed". Online, Refresh says "Stack refreshed" once | Surfer | Droid | |
| 289 | Playstyle Summary's Top color pip reads "Red" to a screen reader (sr-only text), Top identity still "Mono-red" | Surfer | | |
| 290 part 1 | Format picker: type "pau", close, reopen: the search is empty. Same for Deck tags and Oracle tags | Surfer | Droid | |
| 416 | zwipe.net and www.zwipe.net load from Cloudflare Pages; www redirects to the apex keeping the query string | Surfer | | |
