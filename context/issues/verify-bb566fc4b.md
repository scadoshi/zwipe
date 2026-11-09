# Verify: the bb566fc4b deploy (web 1.18.16 changes)

Deployed 2026-11-09. Droid checks on live app.zwipe.net in emulator Chrome at phone width (test login) and marks pass or fail with what it saw. Anything new goes into this folder as a `reported` issue first.

| Item | What to check | Web | App | Result |
|---|---|---|---|---|
| 322 | Decks list: each row's name and tags sit together in the column right of the art, name on top, tags wrapping beneath; nothing drops under the art; skeleton rows while loading match; long names and many tags still read cleanly | Droid | | |
| 321 | Account: every refresh button (Pending email, Verification) has an accessible name (Chrome accessibility tree or TalkBack): "Check the new email", "Check verification" | Droid | | |
| version | About shows v1.18.15; app.zwipe.net/version.json gives bb566fc4b… | Droid | | |
