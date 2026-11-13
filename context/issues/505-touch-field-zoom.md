# 505. Focusing Quick add on a phone zooms the page

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Composer, 2026-11-13 (taken from laptop): Quick add zooms the page on a phone, likely a field under 16px, which iOS zooms into as it takes focus.

## Before
Headless at 390 with touch, every visible text field's font size: a deck's Quick add 14px and the Commander maybeboard's search 14px; Create deck's and Account's fields 16px; Settings, Playstyle, Decks and Activity show none until a sheet opens. Every `.input-compact` (33 uses: Quick add, the maybeboard's search, the pickers' searches, the filter's text fields) is 0.875rem, and the guides' search 0.8rem.

## Cause
`components/assets/filter.css` `.input-compact { font-size: 0.875rem }` and `guides.css` `.guide-search-input { font-size: 0.8rem }`, at every width and pointer. iOS zooms the page into a focused field under 16px.

## Fix
scotland: on a touch screen (`pointer: coarse`) a compact field and the guides' search read at 1rem; a mouse keeps the compact sizes.

## Verification
- `components/tests/input_zoom.rs` (red before).
- Headless at 390 with touch: Quick add and the Commander maybeboard's search read 16px (14px before); at 1440 with a mouse they keep 14px.
