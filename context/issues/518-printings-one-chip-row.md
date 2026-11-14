# 518. Printings stacks its facts, artist and prices on three lines

State: fixing | Owner: thinker | Type: polish | PR: - (pushed to release) | Build seen: 042bd5290

## Report
Scotty (screenshot `target/composer/printings-scotty.jpg`, the app): the info under Printings' cover flow is three stacked rows: the set chip, the number chip and plain text "2 of 6"; then the artist chip; then the price pills, left-aligned. Make it one centered row of chips that wraps (set, number, the place as a chip too, the artist, the prices), with the existing chips, on the app and the web wherever Printings opens.

## Before
Headless Chrome, signed-in fake session, the printings read mocked with six printings, `target/composer/printings-before-390.png` and `-1440.png`: at 390 the narrow sheet shows the set and number chips with "1 of 6" as plain text, the artist chip on its own line, then the prices from the left; at 1440 the wide modal's row has the place as plain text after the prices.

## Cause
`components/src/printing_sheet.rs` drew the narrow sheet's facts (`PrintingFacts`, with the place as text), `ArtistChip` and `CardPrices` as separate blocks, and the app (`zwiper/.../printing_sheet.rs`) did the same with its own price row (`CardInfoDisplay`, left-aligned by `.app-printing-sheet .card-prices`). The wide modal ordered its own spans, the place again as text.

## Fix
`PrintingChips` is the one row every Printings sheet draws: the set (`stat-chip-p3`, cut short first), `#number` and "N of M" (`stat-chip-muted`), `ArtistChip`, then the prices (`stat-chip-money`, as the wide modal had them), centered and wrapping. `PrintingChipsGhost` is its loading shape. The narrow web sheet keeps Save under the row at the right; the wide modal's row holds the chips and Select together. `PrintingFacts`, the narrow `CardPrices`, `CardPricesGhost` and the app's `CardInfoDisplay` are gone.

## Tests
- `components/tests/printing_sheet.rs` `the_narrow_sheet_shows_its_printing_as_one_row_of_chips`: one `printing-chips` row holding set, number, the place as a chip, the artist and the prices, in that order, and no second line (red before).

## After
`target/composer/printings-after-390.png` and `-1440.png`: at 390 one centered row of chips wrapping onto three lines (set, number, place; artist and two prices; TIX); at 1440 one centered row ending in Select.

## Not checked
- The app on a device: it draws the same component.
