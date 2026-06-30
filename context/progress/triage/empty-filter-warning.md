# Rethink the empty-filter warning — maybe just serve all cards

**Source:** self-noted, 2026-06-29.
**Verdict:** undecided — "consider" item.

Today, when the card filter is empty the add-cards screen leaves the swipe stack
empty and nudges the user to set a filter (so a blank screen doesn't read as "no
cards exist"). Reconsider that choice: an empty filter could instead **just serve
all cards** (the full pool / synergy-ordered default), which may be the friendlier
behavior than a warning.

## To weigh tomorrow

- Behavior change is in `zwiper/.../screens/deck/card/add.rs` — the
  `filter_builder.is_empty()` / `is_empty_ignoring_deck_context()` branches
  (~lines 444–454) and the "leave stack empty and nudge to filter" path (~640s).
- Trade-off: serving everything on an empty filter is more discoverable and
  matches the "just inspire me" discovery idea (`feature_requests.md` #2), but a
  full unfiltered pull is a heavier query and a less-targeted stack. Decide
  whether empty-filter = all-cards, or keep a (lighter-touch) prompt.
- If we serve all cards, make sure deck-context exclusion (in-deck cards) and the
  synergy default ordering still apply.
