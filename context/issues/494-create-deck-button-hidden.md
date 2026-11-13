# 494. Create deck's empty form shows no Create button

State: fixing | Owner: thinker | Type: bug | PR: - (pushed to release) | Build seen: 8096283c4

## Report
surfer's walkthrough (`halo:~/surfer-shots/walkthrough/README.md`, item 1), triaged by Scotty: the empty Create deck form has no primary action anywhere on screen; Create only appears once a name is typed. Show Create always; clicking it or pressing Enter with no name shows the name's "Deck name is required" line instead of submitting. The web and the app if the app does the same.

## Before
Headless Chrome at 1440x900, signed-in fake session, `/decks/new`, 3 of 3: no Create button in the dock; Enter in the name field shows "Deck name is required" and sends nothing.

## Cause
`zwiper-web/src/lib/inbound/screens/deck/create.rs` drew the dock's Create button only `if valid` (the name passes `deck_form::name_error`). Its `submit` already shows the name's error and stops when the name fails. The app's form (`zwipe_ui::screens::deck_form`, `save_label`) always offers Save on Create and raises the same error, so it was right already.

## Fix
The web's Create button shows from the start; with no name, a click shows the name's error as Enter does.

## After
Same setup, 3 of 3 at 1440 and once at 390: Create shows on the empty form; a click and Enter each show "Deck name is required"; nothing is sent.

## Not checked
- A unit test: the change is the view's condition, which zwiper-web has no harness to render; the headless runs are the check.
- The app on a device (unchanged).
