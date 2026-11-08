# 301. Edit deck still asks to leave after a picker change was discarded

State: fixing | Owner: laptop | PR: #439 | Build seen: 1d7053e88

## Report
Droid, Android app (debug APK of release at 1d7053e88 on prod, Pixel 9a emulator), 2026-11-08. In Edit deck, opening the Format picker, tapping Pauper, closing with ‹ and choosing Discard puts Format back, but the form still counts as changed: header Back then asks "Leave without saving? Your changes to this deck will be lost." with nothing changed. Opening Edit deck and leaving straight away shows no prompt, so the Discard is what leaves it dirty. Seen after Deck tags and Oracle tags discards too.

## Repro
1. Open any deck, tap the pencil (Edit deck).
2. Tap Format, tap Pauper, tap ‹, tap Discard. Format still reads Commander.
3. Tap ‹ in the header: "Leave without saving?" appears.
