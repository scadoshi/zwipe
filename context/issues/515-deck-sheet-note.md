# 515. The web's deck shortcuts sheet carries a note the groups make redundant

State: fixing | Owner: laptop | Type: issue | PR: - (pushed branch, hash in the composer's log) | Build seen: release 7b335fb52

## Report
Scotty, via the composer: drop "Move to a card, or hover it, then press its key" from the web's shortcuts sheet now that its keys are grouped, and list any other screen's similar intro note.

## Cause
`zwipe_ui::shortcuts::DECK_CARDS_NOTE`, set as the sheet's note by `zwiper-web`'s `sheet_note` for the deck cards page and the card rows, and by the shared deck page.

## Fix
- The constant and `sheet_note` are gone; the web's screens and the shared deck offer their keys with no note. `ShortcutsList::note` stays for a screen that needs one.
- No other screen sets a note: that was the only one.

## Checked
- The `sheet_note` assertions in `zwiper-web`'s `the_deck_cards_sheet_lists_only_that_screen_s_keys` are removed with it; the gate passes.

## Verification
-
