# Verify: the 69f38270e deploy

Deployed 2026-11-08. Surfer checks the web lines on live and marks pass or fail with what it saw. Droid is recording marketing clips, so it only takes the app lines after that. Anything new goes into this folder as a `reported` issue first, never straight to a fix.

Hard-refresh the web app and zwipe.net first. Surfer: never open, move or resize windows; size checks use the same-origin frame in your one tab.

| Item | What to check | Web | App | Result |
|---|---|---|---|---|
| 303 | Hard refresh app.zwipe.net Home (and on throttled Fast 3G): no frame of unstyled content (serif text, blue squares for the logo). View source: stylesheets are `<link rel=stylesheet>` in the head | Surfer | | |
| 304 | Web Home: "as of" and "Everybody" chips sit ~16px under the stat counters at 1440, 1100 and 390, like zwipe.net | Surfer | | |
| 305 | Home's Decks panel: "Create deck" button (not "+") beside All decks, opens Create; fits at 390 | Surfer | Droid | Droid: PASS app (debug build of 69f38270e, prod, 411 wide). Home's Decks panel shows "Create deck" (no "+") beside "All decks" in one row (105px + 90px in a 201px group, no wrap); it opens Create deck, and Back returns Home with no prompt. |
| 306 | Signed in: Decks → About → Privacy → Decks keeps one header (no flash or remount); Create's leave guard still works after About | Surfer | | |
| 307 | zwipe.net About: crate cards include zwiper-web; summary says eight | Surfer | | |
| 309 | zwipe.net/robots.txt allows /deck/ for Twitterbot and friends, still disallows it for `*`. app.zwipe.net/robots.txt is a plain allow-all text file | Surfer | | |
| changelog | Web app About and zwipe.net changelog show 1.18.13 and 1.18.14 upcoming with the new entries | Surfer | | |
