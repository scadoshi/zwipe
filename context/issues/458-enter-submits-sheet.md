# 458. Enter does nothing in an open sheet with a main action

State: fixing | Owner: laptop | Type: feature | PR: - (pushed branch, hash in the composer's log) | Build seen: app.zwipe.net 4059d64f8

## Report
Scotty, via the composer: on the computer layout of the web app, Enter in an open sheet or drawer with one main action runs it, unless focus is on a control that uses Enter itself. When the action is off only because nothing changed, Enter closes the sheet as its × does. surfer's before (Chrome macOS 1009x958, signed in, deck Surfer 399 Isshin): on Add and on the deck page, open the dock's Filter, open Rarity, click a chip, click the sheet's title so focus is on the body, press Enter. Nothing happens, 3 of 3 on each; the keydown reaches the window with `defaultPrevented` false.

## Cause
No listener answers Enter for a sheet. The only page-wide Enter handling is orangui's dialog keys (`DIALOG_KEYS_JS` in `alert_dialog.rs`), which act only while an `.alert-dialog` is open. A sheet's dock buttons answer Enter only while they hold focus, and focus on the body reaches none of them.

## Fix
laptop, in `components/src/keys/sheet_enter.rs`: `SHEET_ENTER_JS`, loaded once in zwiper-web's head, listens for Enter with no modifier, not repeating or composing, and not already taken. When focus is on no control (a field, button, link, chip or ARIA control), no dialog is open, and the top open modal (highest z-index) is a shown sheet holding a `.sheet-enter` button, Enter clicks that button. When the button is off and marked `sheet-enter-closes`, Enter clicks the sheet's × instead. The marks:
- the card filter's Apply and the deck form pickers' Save: `SHEET_ENTER_OR_CLOSE`;
- Exceptions' Save: `SHEET_ENTER_OR_CLOSE`;
- Change username, Change email and Change password's Save: `SHEET_ENTER` (off there for an error or while saving, so Enter does nothing).
Printings and the theme sheet keep their own Enter. Sheets with no main action (card details, shortcuts, the Oracle tag dictionary and Examples, Swipe select, the deck page's drawers) have no mark. The phone app loads no script.

## Checked
- Local web, shared Commander deck (Atraxa Superfriends), wide 1280, headless Chrome, surfer's steps (Filter, Rarity, Mythic, click the title, Enter):
  - with the fix, the filter applies: the sheet closes, 92 rows become 21, the dock's dot shows;
  - with the script taken out and the app rebuilt (negative control), the same steps leave the sheet open and the 92 rows unchanged.
- Nothing changed, then Enter: the sheet closes, as × does.
- Enter with focus on a chip or an accordion header: the sheet stays open and nothing applies.

## Verification
-
