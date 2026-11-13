# 429. On the web at phone width, Printings ignores the arrow keys and Enter until clicked, and the filter can't be pulled down

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-13: on the web app at phone width, the Printings bottom sheet's left and right arrows and Enter do nothing, while the wide modal form takes them. Narrowed: the keys work once the bottom sheet has focus; it never takes focus as it opens, where the modal form does. Fix it with focus on open and focus handed back on close, and check the other bottom sheets for the same gap. Added: the filter's bottom sheet, on the web app and the shared deck page at phone width, can't be pulled down by its title bar as Printings can; reuse the shared bottom sheet's drag and check the other bottom sheets. Added (droider): Account's Change username sheet leaves focus on its Change button and has no dialog role; cover it with the same fix and make the sheets carry `role=dialog` and `aria-modal` where they're modal.

## Fix
scotland:
- `components::keys::use_sheet_focus` focuses an open sheet's body as it opens, remembering the element that had focus, and focuses that element again as the sheet closes or unmounts open. Printings uses it in both forms: the bottom sheet's body and the modal's (which focused itself before and handed nothing back). The bottom sheet's arrows keep the page from scrolling, and Enter on its Save button presses the button only.
- Account's Change username, Change email and Change password sheets focus their first field as they open, through the same hook, and hand focus back to the button that opened them.
- The other sheets: the image preview focuses itself on mount already, and the filter, card details, Examples and the deck form's pickers take their keys through document listeners, not focus.
- orangui 4566a7e moves the bottom sheet's header pull into `SheetDrag` (`use_sheet_drag`), with `BottomSheet` on it unchanged. `DrawerSheet` (the filter, card details, the deck form's pickers) pulls through it while it is a bottom sheet: below the kit's modal width and not a drawer at every width, which leaves the filter; card details and the pickers float in from the right at every width. The Oracle tag dictionary is a full screen below that width.
- orangui fbfcc73 makes `BottomSheet` a `dialog` named by its title, `aria-modal` while it dims the page (`SheetDim::Always`). `DrawerSheet` does the same, `aria-modal` while it dims at the window's width, as do the wide Printings modal and the Oracle tags modal.

## Verification
- At phone width, open a card's Printings: right arrow, left arrow, `l` and `h` step at once with no click, Enter saves a different printing, and focus goes back to the Printings button after it closes. The same on a shared deck, where Enter does nothing.
- At phone width, open the filter and drag its title bar down past a quarter of its height: it closes; a short drag springs it back. The same on a shared deck.
- At phone width, open Account > Change username: the new username field has focus with no tap, and closing the sheet puts focus back on the Change button. The same for Change email and Change password.
- A screen reader names an open sheet by its title, as a dialog.
