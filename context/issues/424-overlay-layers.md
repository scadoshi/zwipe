# 424. Themes opens under the card details drawer

State: fixing | Owner: laptop | PR: - (pushed branches, hashes in the composer's log) | Build seen: 516d020a1 | Type: issue

## Report
Scotty's recording (web deck page, Stacks, 1450): with a card's details drawer open on the right, opening Themes from the nav does not go over it; the two fight for the same right edge and the drawer stays on top. Rule: the sheet opened last sits over every other; live-preview sheets (Themes, the card filter) do not dim on a wide window, so the page shows the change, and a click outside closes them; decision sheets dim and block; on a phone every sheet and drawer dims, Themes included; menus never dim; Esc and Back close the topmost first.

## Cause
Every sheet drew at z-index 200 (backdrop) and 201 (sheet), so two open sheets tied and DOM order decided, with hand-picked numbers (204/205, 210, 220, 230, 240) covering only the pairs someone had hit. Themes lost outright: it renders inside `.nav-wrapper`, which is sticky at z-index 101 and so a stacking context of its own.

## Fix
laptop, per `context/plans/overlay_layers.md`. orangui (`overlay-layers`): the overlay back stack, which already holds the open overlays in the order they opened and closes the last on Esc and Back, hands each its z-index from its place (200 plus 2 per overlay under it); `BottomSheet` draws at it (`--overlay-z`) and holds it through its slide out; `SheetDim::Phone` leaves the backdrop transparent from 60rem while it still takes the outside click; a sheet inside the nav lifts the nav to its layer (`OverlayLift`); Themes passes `Phone`. zwipe: orangui repinned; `DrawerSheet` takes `layer` and `dim` (the card filter passes `Phone`; card details and the pickers their layer); the custom overlays (Printings' wide modal, examples, dictionary, image preview, swipe select and the app's pickers) write their layer; the hand-picked z-indexes read `--overlay-z` with their old number as the fallback for a host with no stack (zwipe.net).

## Checked (headless Chrome, app.zwipe.net local, deck "Krenko goblins")
- 1450: card drawer (200/201) then Themes: Themes at 202/203 with the nav lifted to 202, on top, page undimmed; a click on the page closes Themes and leaves the drawer.
- 1450: drawer then Printings: Printings at 202/203 dimming the drawer; Esc closes Printings, Esc again closes the drawer.
- 1450: the card filter undimmed; a click outside closes it.
- 390: Themes from the menu dims (0.6).
- The app (zwiper) builds and its tests pass; it shares the code.
- orangui tests: open A then B puts B over A; Back closes B; reopening A puts it over B.

## Verification
-
