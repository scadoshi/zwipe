# Verify: the 1f354453d deploy

Deployed 2026-11-08. Surfer checks on live and marks pass or fail with what it saw. Anything new goes into this folder as a `reported` issue first.

Surfer: never open, move or resize windows; size checks use the same-origin frame in your one tab. Don't sign out.

| Item | What to check | Web | App | Result |
|---|---|---|---|---|
| 282 | app.zwipe.net serves `cache-control: no-cache` and /version.json gives this build (composer saw both). In a tab opened before a later deploy, the next screen change loads the new build (check after the next deploy, or note how it behaves now) | Surfer | | |
| clips | zwipe.net: 4 demo clips (Swipe to build, Deck stats, Playstyle, Activity) with posters and captions, old 12 gone; pressing next plays the new clip (or note if it shows paused) | Surfer | | |
| 310 | zwipe.net guides: new screenshots (grid tiles, Gruvbox dark) in every guide; nothing stretched or cropped wrong | Surfer | | |
| 311 | Guides filter: "Filter" is a label chip; typing shows the red × sliding in inside the field; × clears | Surfer | | |
