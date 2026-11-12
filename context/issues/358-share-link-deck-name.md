# 358. The Share dialog's link names every deck "deck"

State: fixing | Owner: thinker | PR: - | Build seen: production, web

## Report
Composer for Scotty, 2026-11-11, web: the Share dialog shows `/<user>/deck/deck/<token>` instead of the deck's own slug.

## Cause
`DeckDialogsHost` builds the shown link from `name.peek()`, a signal its `use_effect` fills with the deck's name after the first render. The dialog renders before the effect runs, reads the empty name (whose slug falls back to `deck`), and `peek` never subscribes, so nothing renders it again.

## Fix
thinker: the dialog names its link from the `deck_name` prop, which rerenders it when the deck loads; the copy handlers, which run after the effect, keep reading the signal. Reproduced headless on the release build (`/layouttest/deck/deck/<token>` for "Elves Go Wide"); fixed, the shown and the copied link both read `/layouttest/deck/elves-go-wide/<token>` at 1280 and 390.
