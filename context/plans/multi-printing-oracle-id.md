# Multi-Printing — Remaining Work

Phases 1-2 shipped (`2f52adde`). Sync uses default_cards (~110k cards), oracle_id constraint on deck_cards. Phase 3 (printing selector UI) is built but has a layout bug.

---

## Phase 3 Bug: Printing Sheet Layout

- [ ] **"select printing" button clips over content** — needs layout fix in `zwiper/src/lib/inbound/screens/deck/card/components/printing_sheet.rs`
- The PrintingSheet bottom sheet renders but thumbnails or the button overflow outside expected bounds
- May need z-index, overflow, or positioning adjustments

## Phase 3 Verification (once bug fixed)

- [ ] "printing" button on expanded CardRow opens PrintingSheet
- [ ] PrintingSheet shows current printing highlighted
- [ ] All printings load as thumbnails with set name + release year
- [ ] Tapping a printing updates deck card via update_deck_card
- [ ] Local state updates optimistically (image + prices reflect new printing)
- [ ] Toast "printing updated" on success
- [ ] Price stats recalculate after printing change
