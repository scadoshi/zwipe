# 381. Sheets and drawers blank before they slide closed

State: fixing | Owner: thinker | PR: - | Build seen: 9680dcacb, web (surfer's ease audit, ~/ease-audit/README.md, rows B)

## Report
Surfer's ease audit, approved by Scotty, 2026-11-12:
- The Filter sheet, the Stacks card drawer and the Format picker remove their contents within one frame of closing, then slide an empty panel out for 300ms. The theme menu keeps its contents and closes cleanly.
- A Filter sheet section opens over 17 frames but closes by dropping 517px in one frame.

## Cause
The three panels are the shared `DrawerSheet`, which mounted its children only while open. The theme menu is the kit's `BottomSheet`, which keeps its children through the slide away. A Filter section rendered its rows only while open, so its collapse lost its height at once.

## Fix
thinker: components' `CloseHold` (`use_close_hold`, `settle_after_close`) keeps a panel's contents from its opening until its latest close has run its 300ms (`CLOSE_MS`, the kit's slide and fold). A reopening or a later close cancels an earlier close's timer. `DrawerSheet` draws its children through it, so all three panels close full. The Filter panel holds the section folding shut the same way, so its rows ease out. Off the browser nothing is timed, as before.

Headless at 1422 (the audit's width), sampled every frame from the close:
- **Filter sheet (×):** slides 0 to 400px and keeps its contents until it is off screen (about 305ms).
- **Stacks drawer (×):** the same, until about 313ms.
- **Format picker (×):** the same, until about 306ms.
- **Filter section collapse:** 582px down to 0 in 18 steps over about 300ms.
