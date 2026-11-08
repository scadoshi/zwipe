# Verify: the 02102ab5e deploy

Deployed 2026-11-08. Surfer (web) and Droid (Android app, plus phone-width web) check each line on live and mark it pass or fail with what they saw. Anything new goes into this folder as a `reported` issue first, never straight to a fix.

Hard-refresh the web app and zwipe.net first (see 282). Droid uses a fresh build of release at 02102ab5e and says which.

Surfer: never open, move or resize browser windows. Size checks use the same-origin frame in your one tab.

| Item | What to check | Web | App | Result |
|---|---|---|---|---|
| 265 | Create deck, Tymna: the partner pile and typeahead offer plain Partner cards (Thrasios, Kraum...) and no Friends forever, Character select, Father & son or Survivors. Wernog: only Friends forever. Pick Tymna + Wernog (filter off): it saves with a "cannot be partners" warning | Surfer | Droid | |
| 299 | Open About, then Decks, Create, type a name, Back: "Leave without saving?" shows. Stay keeps you, Leave leaves. Also browser Back, and the Privacy page instead of About | Surfer | Droid (web) | |
| 301 | Edit deck: Format, tap Pauper, close, Discard; then header Back leaves without asking. Same for Deck tags and Oracle tags | Surfer | Droid | |
| 298 | Header is one row at 1110 and wider; below 1110 it's the ☰ menu (check 1100, 960, 900). No wrapped rows, no empty block. Add's details title and sheets aren't hidden under it | Surfer | | |
| 302 | Add at 900 to 1120 wide with details shown: the card sits clear of the details panel (about 1rem gap) | Surfer | | |
| 297 | Web Decks: tags start folded behind ▸ Tags at every width; a picked tag stays beside the toggle; matches the app | Surfer | Droid (web) | |
| 296 | Decks grouped by Tag, a group folded, hover its tag: full outline on all four sides | Surfer | | |
| 295 | Changelog (web app About, zwipe.net, app About): clicking anywhere on a version row toggles it; fold icon at the right end of the Changelog title row toggles all; zwipe.net version number still links to the version | Surfer | Droid | |
| 294 | zwipe.net About's architecture diagram shows zwiper-web; clients join into "imports all four"; zerver points at zwipe-core | Surfer | | |
| 300 | A shared deck's og:image URL has v=2-<timestamp> and loads the new layout | Surfer | | |
| stats | zwipe.net's og-default.png shows 2,298 decks / 32,028 searches / 265,040 swipes | Surfer | | |
