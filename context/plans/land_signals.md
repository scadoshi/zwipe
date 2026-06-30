# Land Count Signals

**Status: SHIPPED 2026-06-29 (on `main`, not yet deployed/store-built).** Built
the full version, not just the MVP: a per-deck **land target** (stepper —
explicit override else format heuristic), persisted server-side, shown in the
deck Profile, with crossing toasts on add/remove/qty and a below-target warning.
The broader auto-land-base / hard-cap ideas below remain future work.

## Goal

While building a deck (Add / Remove screens), give the user lightweight feedback
about their land count — e.g. a toast "You've hit your land count" when the deck
reaches a sensible land target, so they stop under/over-running lands without
having to do the math.

This is the **simplest first step** toward the broader land tooling requested in
the launch thread — it sets up, but is smaller than, auto land base (req #4) and
a hard land cap (req #5) in `feature_requests.md`.

## MVP scope

- A **target land count** for the deck.
- Detect lands among the deck's mainboard cards.
- When a swipe/add pushes the land count to (or past) the target, fire a **toast**
  ("You've reached your land target (~X)"). Optionally a gentler nudge when over.
- No new screens — just a signal woven into the existing add/remove flow.

## Key decision — where does the target come from?

1. **Heuristic default (recommended for MVP):** derive a target from the format /
   deck size (e.g. ~36–38 for a 100-card Commander deck), no user input. Pure
   client-side, ships in the QOL client build.
2. **User-set per deck:** add a `land_target` deck field. More flexible and lines
   up with req #5 (land cap), but needs a `zwipe-core` field + migration + server
   deploy.
3. **Both:** heuristic default, user can override later.

**Recommendation:** ship (1) now as a client-only signal; layer (2) on when we
build the land-cap / auto-land-base features so it's all one coherent land model.

## Land detection

A card is a land if its `type_line` contains "Land". Count distinct mainboard
land entries × quantity (basics can be multiples). Watch the edge cases:
- **MDFCs / DFCs** where one face is a land (e.g. "Creature // Land") — decide
  whether a back-face land counts (probably yes, partially).
- Cards that are lands but enter tapped/conditional — out of scope for a raw
  count.

## Trigger logic

- On add (right-swipe of a land) or on the Add/Remove screen mount, recompute the
  mainboard land count.
- If it **crosses** the target (was below, now ≥), toast once. Avoid re-toasting
  every subsequent land (debounce on the crossing, not the count).
- Keep it advisory — never block adding more lands.

## Open questions

- Heuristic target by format: what numbers? (Commander ~37, 60-card ~17, etc.)
- Should non-land-but-ramp/rocks count toward an effective "mana source" target
  rather than strictly lands? (That edges toward the auto-land-base feature —
  keep MVP to literal lands.)
- Toast wording + whether to also signal "below target" when removing lands.

## Relationship to other plans

- Precursor to **auto land base** (req #4) and **land cap** (req #5). When those
  land, fold this signal into the shared land model (user-set target, pip-ratio
  suggestions, hard cap).

## Effort & deploy

MVP (heuristic default) is **client-only** — counts lands from already-loaded deck
data and fires a toast. Ships in the next client build, no server work. The
user-set-target variant is a later server-touching follow-up.
