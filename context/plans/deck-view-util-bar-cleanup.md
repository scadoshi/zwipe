# Plan: Deck View Util Bar Cleanup

## Goal

Reduce the 9-button util bar to 5 by relocating `buy` into the price stats section and moving rare actions into a "more" bottom sheet.

## Changes

### 1. Move `buy` into `DeckStatsSection`

Place a `buy` button inline with the currency chips row, separated by a `|` divider:

```
[ usd ] [ eur ] [ tix ]  |  [ buy ]
```

- `buy` opens the same buy sheet (TCGplayer / CardKingdom) that currently lives in `view.rs`
- `DeckStatsSection` needs access to the buy sheet signal — pass `show_buy_sheet: Signal<bool>` as a prop
- The buy sheet itself stays in `view.rs` (it overlays the whole screen)

### 2. Slim the main util bar

Keep only high-frequency navigation:

```
[ back ] [ edit ] [ add ] [ remove ] [ more ]
```

### 3. Add "more" bottom sheet

`more` opens a bottom sheet containing the less-frequent actions:

- **view** — view deck cards
- **import** — import from decklist
- **export** — export as decklist
- **delete** — delete deck (keep the existing AlertDialog confirmation)

Use the same bottom sheet pattern as the buy sheet (`.modal-backdrop` + `.bottom-sheet` CSS classes already exist).

## Files Modified

| File | Change |
|------|--------|
| `zwiper/.../deck/view.rs` | Remove `buy` and `view` from util bar, remove `import`/`export`/`delete` from util bar, add `more` button + more sheet |
| `zwiper/.../deck/deck_stats_section.rs` | Add `show_buy_sheet: Signal<bool>` prop, render `buy` button in the chip row |

## Notes

- The buy sheet markup (TCGplayer/CardKingdom links) stays in `view.rs` — it needs `tcg_url` and `ck_url` which are computed there
- `DeckStatsSection` just toggles the signal; the sheet renders in the parent
- No CSS changes needed — `.bottom-sheet`, `.modal-backdrop`, `.btn`, `.chip-row` all exist
- Delete keeps its existing `AlertDialog` confirmation flow, just triggered from the more sheet instead of the util bar
