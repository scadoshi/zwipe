# Verify: the next deploy

Not deployed yet. Rename to `verify-<sha>.md` with the deploy date when it ships. Droid checks on live app.zwipe.net in emulator Chrome at phone width (test login) and, for app items, on the Android emulator, and marks pass or fail with what it saw. Anything new goes into this folder as a `reported` issue first.

| Item | What to check | Web | App | Result |
|---|---|---|---|---|
| 315 | Add: applying a filter, toggling the Synergy bolt, Refresh and first load each show the loading ghost (card and details ghosted) until the new pile's top card appears, then that card fades in; no other card shows first, even briefly. A screen recording, stepped frame by frame, catches a flash best | Droid | Droid | |
| 326 | Printings on a card with many printings (Sol Ring, Island): each card shows its own pulsing card-shaped ghost until its image arrives, cards fill in one by one without waiting for the rest, and none is ever an empty box | Droid | Droid | |
