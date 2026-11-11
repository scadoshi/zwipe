# 341. Printings that failed on a slow network stay set-name cards

State: fixing | Owner: scotland | PR: #525 | Build seen: live 21ca16a04

## Report
Composer, from droid's phone-width checks, 2026-11-11: in Printings, a printing whose image failed to load on a slow network shows its set-name card, and stays that way after the network returns, until the sheet is closed and opened again.

## Cause
The cover flow (`components/src/cover_flow.rs`) records each image's arrival in the flow's own state. A failed image goes to the set-name card and nothing ever tries it again; only a new flow, from reopening the sheet, starts the record over.

## Fix
scotland: the flow keeps, per image, how often it was retried. A failed image tries again when the connection comes back (the `online` event, through `use_back_online`) and when its printing is brought to the middle. A retry ghosts the card and gives the image a new `src` (a `#retry-N` fragment, which the server never sees), so the browser asks for it again. Failing-first test `a_failed_printing_loads_again_on_retry`.
Checked headless on web at 1260 with every card image request failed while Printings opened: 12 of 13 printings showed their set-name cards; once requests passed again and the browser fired `online`, all 12 loaded with the sheet still open.
