# 293. An empty Swipe pile stays on a skeleton card

State: fixing | Owner: laptop | PR: #432 | Build seen: 37d09dbf9

## Report
Droid, Android app (debug APK of release at 37d09dbf9 on prod, Pixel 9a emulator), 2026-11-08. Create deck with Pir, Imaginative Rascal as commander: the form fills Partner with Toothy, Imaginary Friend on its own. Opening the Partner Swipe pile then shows a grey skeleton card for over 30 seconds and never deals a card or says the pile is empty. Toothy is the only legal partner and is already picked, so the pile is probably empty; it should say so instead of looking like it's loading.

## Repro
1. Decks, Create deck, Format Commander.
2. Commander: pick Pir, Imaginative Rascal. Partner fills with Toothy.
3. Tap Swipe beside Partner.
4. Swipe select shows only `.skeleton-card` and no images, toast or empty message after 30 s.
