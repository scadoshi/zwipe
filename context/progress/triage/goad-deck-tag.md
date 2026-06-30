# Add a "Goad" deck tag

**Source:** user feedback (Zwipe 1.1.2, Android), 2026-06-29.
**Verdict:** decided — build it. Easy, low-risk.

> "idk if it's intentional or just not implemented yet but when making a goad
> deck i couldn't find a goad option when trying to add it as a tag so it
> couldn't find anything perfectly accurate for what i was looking for … besides
> that I love the app and it's very useful and cool seeing very different cards"

Goad is a real, well-known EDH archetype (force-attack / "encourage chaos"
strategies) and is currently missing from the curated `DeckTag` set. We have the
adjacent `Group Slug` and `Politics` tags but nothing that names goad directly.

## Where to add it

`zwipe-core/src/domain/deck/models/deck_tag.rs` — add a `Goad` variant and wire
it through the four sites that every tag touches:

1. The enum variant itself.
2. The all-tags list (`all()` / the array around `Self::GroupSlug`, `Self::Politics`).
3. Display name (the `match` returning `"Group Slug"`, `"Politics"`, …) → `"Goad"`.
4. Plain-language description (the `match` of one-line definitions) — e.g.
   "Force opponents' creatures to attack elsewhere and ride the chaos."

Pure enum addition (JSONB-persisted as a string), so no migration. Backward-
compatible: old clients just won't offer the new tag until they update. Ships
client-side; mind the min-version-gate alignment like any other client change.
