# 475. Printings doesn't name the card or its artist

State: fixing | Owner: laptop | Type: feature | PR: - (pushed branch, hash in the composer's log) | Build seen: eec4e11be

## Report
Scotty, via the composer: the Printings sheet names the card in its header and shows the artist at the bottom as a small art tag (an icon plus the artist's name), on the web and in the app. surfer's before (Chrome macOS, signed in, Surfer 399 Isshin, List, Sol Ring > Printings, at 1440 and 390, 1 of 1 each):
- the header reads only `Printings`;
- the artist shows nowhere but the tiny credit printed on the card image;
- at 1440 the info row has set, number, prices, position and Select; at 390, only the prices and Save.

## Cause
- **Header:** `components/src/printing_sheet.rs` titles both forms with a fixed `Printings` (the narrow `BottomSheet`'s `title` and the wide `CoverFlowSheet`'s `sheet-title`). The app's sheet uses `zwipe_ui::screens::printing_sheet::TITLE`.
- **Artist:** neither sheet renders the artist. `zwipe_ui::card::info::artist` exists but no sheet calls it.

## Fix
laptop:
- **Header:** the sheet's title is the card's name, cut short with an ellipsis where the bar can't hold it.
- **Screen-reader name:** the wide modal's `aria-label` is `Printings of <name>` (`zwipe_ui::screens::printing_sheet::label`).
- **Artist chip:** `components::ArtistChip` draws the printing's artist as the card details' `detail-chip detail-chip-artist`, with the kit's Image icon (`IMAGE_ICON`) before the name. A printing with no artist draws nothing. It sits:
  - in the wide modal's info row, after the collector number;
  - in the narrow sheet and the app, on its own line above the prices and Save.
- **Tests:** `components/tests/printing_sheet.rs` checks the title, the chip with the icon and the artist, and no chip without an artist. The first two fail on the old code.

## Checked
- Local web, signed in, a Commander deck's Atraxa > Printings:
  - at 1440: the title is `Atraxa, Praetors' Voice`, the aria-label is `Printings of Atraxa, Praetors' Voice`, and the chip shows the Image icon and `Justin Hernandez & Alexis Hernandez` in the info row;
  - at 390: the same title, and the chip centered above Save.

## Not checked
- The app on a device or simulator (same `ArtistChip` and stylesheet).
- Below the kit's modal width (the narrow web sheet and the app), the dialog's screen-reader name is the card's name alone. orangui's `BottomSheet` takes its `aria-label` from `title`. A `label` prop on it would give `Printings of <name>` there too.

## Follow-up
surfer's after (halo `~/surfer-shots/after-475/`):
- **Title under the ×:** at 390 a long title ran under the × (a 342px title from x 24).
- **Missing facts:** the narrow sheet still had no set, number or position.

Cause: the × is absolutely placed at the header's left (`.sheet-close`, 1rem in, 2.1rem wide), and the centered title had the whole bar.

Fix:
- **Title:** the printing sheets' header pads 3.6rem on each side, so the title stays centered and ends, cut short, before the ×.
- **Facts:** `components::PrintingFacts` puts the set (cut short first), the `#number` and `n of N` on one centered line above the artist, in the narrow web sheet and the app.

Checked at 390 on local web with Asmoranomardicadaistinaculdacar:
- the title spans 58 to 332px with the × at 16 to 50px, ending in an ellipsis;
- the facts line reads `The List  #MH2-186  5 of 6` within the sheet.

## Verification
-
