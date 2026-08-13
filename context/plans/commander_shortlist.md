# Commander maybeboard (was: commander shortlist)

**Status: FEATURE REQUEST (2026-07-11). Owner direction captured 2026-08-12 —
not yet specced into a build plan.**

## Owner direction (2026-08-12)

- **Name: "commander maybeboard"** — sticks with existing app vocabulary
  instead of introducing "shortlist".
- Saving to it is **only offered during commander search** (the Zwipe-select
  commander flow) — an up-swipe there adds to the maybeboard instead of the
  ambiguous nothing it does today.
- Its own **UI list page**: card entries with pictures, filterable, cards
  expandable (the established card-row grammar).
- Each entry offers **"Create deck with this commander"** → jumps straight to
  the new-deck profile screen with the commander pre-filled, gated on deck
  capacity (20-deck cap / unverified 1-deck cap).
- Confirmed bigger-build shape: a new database table (per-user commander
  maybeboard rows) + a new screen; needs real UI testing time, so it waits
  for a dedicated train rather than riding a polish batch.

## The need

Real and repeatedly felt: while swiping commanders you find one you want to
**consider later** without committing it to a deck. Owner has wanted this in
personal use; users have asked. Today the only commander-swipe flow lives inside
deck creation (start deck → swipe the command zone → pick one), where an
up-swipe / "save" is ambiguous — a deck has **one** commander, so "where did the
commander go?" has no good answer.

## Reframe

"Collecting commanders I'm interested in" and "picking THE commander for this
deck" are **two different intents**. The confusion comes from forcing the first
into a flow built for the second. A per-deck maybeboard of commanders is muddy; a
**per-user shortlist** is clean.

## Direction (recommended, not locked)

Decouple browsing from deck-building with a dedicated **Commanders** space:
- Swipe through commanders (popularity-led like today, filterable); right =
  **shortlist**, left = skip.
- A **Saved Commanders** list to revisit.
- Closer: from a saved commander, **"Start a deck with this commander"** → new
  deck pre-set. This is the actual loop and the payoff.
- Keep the deck-creation command picker pure ("pick one, done").

## Open decisions (why it's parked, not specced)

1. **Storage:** server-side `saved_commanders(user_id, oracle_id / scryfall_data_id,
   saved_at)` — syncs across devices, matches the app's "your stuff syncs" ethos,
   but adds a table + endpoints — vs client-only.
2. **Placement:** a new top-level area + nav entry vs a mode inside the existing
   Zwipe command-zone picker.
3. **Swipe semantics** in the browse area (right = save, left = skip, up/down = ?
   — fewer gestures than the 4-way deck-building swipe).
4. **Deck-creation picker:** does it also expose "save for later" (routing to the
   shared shortlist), or stay pure?
5. **Scope:** legendary creatures only, or all command-zone leaders (partners,
   backgrounds, Oathbreaker planeswalkers, signature spells)?
6. **Sort/filter reuse:** commander search + popularity ordering already exist —
   how much to reuse.

## Next step

Owner decides #1 (storage) and #2 (placement) first — those gate the rest. Then
this becomes a real plan.
