# 409. Printings shows two different skeletons while it loads

State: fixing | Owner: laptop | PR: - (pushed branch, hash in the composer's log) | Build seen: 75fc16036

## Report
Scotty, on the web and in the app: Printings shows two skeletons while it loads. First the old one (one plain card, five dots and four text bars), then once art starts loading the cover flow's fan of card ghosts. Use only the fan from the first frame, with ghost chips for the set, number, prices, "x of y" and Select in place of the old text bars, in their colors per the skeleton rules.

## Cause
While the printings were fetched, the sheet drew its own `PrintingSheetSkeleton` (components for the web, a copy in zwiper for the app). Once they arrived it drew `CoverFlow`, whose stage stays hidden over a flat single-card primer until the middle image paints (or 3 seconds pass), so a held image showed a third look before the fan.

## Fix
laptop: `CoverFlow` takes `loading`. While loading it draws the fan of ghosts (the primer, now the stage's places with a ghost in each, the full reach on each side) and ghost step buttons, and the hosts keep the same `CoverFlow` mounted from loading to loaded, so the fan stays in place as the printings and their images arrive. The primer ghosts the printings' own places once they are in. The wide sheet's info row ghosts as p3-tinted set, muted-tinted number, three money-tinted prices, a bar for the count and a chip for Select; the narrow sheet's foot and the app's info row ghost as money-tinted price chips (and the narrow sheet's Save as a chip in its dock). The old skeleton and its CSS are gone.

## Checked (headless Chrome, app.zwipe.net local, deck "Krenko goblins", Beetleback Chief's 11 printings, the printings fetch and then the card images held)
- 1400, before: the old skeleton while fetching; with images held, a fan of ghosts with the real chips once the 3-second limit passed.
- 1400, after: the fan of ghosts, ghost arrows and the tinted ghost chips from the first frame; with images held, the same fan in the printings' places with the real chips and arrows; then the cards.
- 390, before: the old skeleton, then a single flat ghost card under the arrows.
- 390, after: the fan, ghost arrows, three money-tinted price ghosts and the Save ghost from the first frame; the sheet's height is the same loading and loaded (417.9px, foot 50px both).
- The app (zwiper) builds and its tests pass; it does not build for the browser, so it has no headless shot.

## Verification
-
