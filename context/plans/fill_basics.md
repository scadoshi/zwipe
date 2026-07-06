# Fill basics — pip-ratio auto land base (FR #4, layer 1)

**Status: PLANNED (2026-07-06). Not started.**

**What this builds, in one sentence:** a one-tap "Fill basics" that counts the
colored mana pips in the deck's nonland cards and adds basic lands in that
ratio until the deck hits its land target — the only place auto-insert doesn't
violate the swipe-everything identity, because there's no decision to take
from the user.

**Why now:** FR #4 is the standing P1 (High impact, every deck, every user);
the tedium it kills is swiping 20+ basics one at a time. Both inputs already
exist: per-deck `land_target` (with format defaults + the land auto-stop) and
full Scryfall data per card. Layers 2/3 (land staples, role presets) are
deliberately **out of scope** — separate doc when their turn comes.

---

## Scope

- **In:** basics only (Plains / Island / Swamp / Mountain / Forest / Wastes),
  mainboard only, fill up to the land target, preview-then-confirm.
- **Out (v1):** snow basics (future toggle), nonbasic suggestions, rebalancing
  basics already in the deck (fill only *adds*; a "rebalance" mode can come
  later), sideboard/maybeboard.

## UX

- **Placement (owner call to confirm):** recommend the deck view, near the
  land count / mana curve — the natural "am I done with lands?" vantage. A
  second hook later: the land-target *crossing* toast's inverse (a nudge when
  the stack empties and lands are still short).
- **Flow:** tap **Fill basics** → dialog previews the exact split ("Adds
  8 Mountain, 6 Forest — 14 to your target of 38") → **Add** / **Cancel**.
  Never silent. If already at/over target: toast "Land target already met."
- Copy uses sentence case, no em dashes (user-facing rules apply).

## The math (core, pure, tested)

All computation client-side in `zwipe-core` so the preview is instant and the
server stays untouched.

1. **Slots to fill** = `land_target` (deck override, else
   `format.default_land_target()`) − current mainboard land count (same
   counting the auto-stop uses). If ≤ 0, stop.
2. **Pip counts** over mainboard **nonland** cards, weighted by `quantity`.
   Parse each card's `mana_cost` string (`"{2}{G}{G}"` tokens — same brace
   grammar `oracle_text.rs::symbol_class` already handles for display):
   - `{G}` → 1.0 to G (likewise W/U/B/R)
   - Hybrid `{R/G}` → 0.5 to each side
   - Two-brid `{2/W}` → 0.5 to W
   - Phyrexian `{G/P}` → 1.0 to G (castable without, but it signals the color)
   - `{C}` → 1.0 to C (fills as Wastes)
   - Generic `{2}`, `{X}`, `{S}` → ignored
   - `mana_cost` empty/None (MDFCs, etc.) → sum the `card_faces`' mana costs
3. **Color gate:** intersect pip colors with the deck's color identity when
   the format has one (Commander/PDH); off-identity pips are dropped (guard —
   shouldn't happen). Formats without identity use pip colors directly.
4. **Split** slots proportionally by pip share, **largest-remainder** rounding
   so the total is exact. Any gated color with a nonzero pip share gets at
   least 1 if slots allow.
5. **Degenerate cases:**
   - No colored pips, colorless identity → all Wastes.
   - No colored pips, colored identity (fresh deck) → even split across the
     identity.
   - No nonland cards at all → same as above.

Deliverable: `BasicsFill::compute(cards, land_target, identity) ->
Vec<(BasicLand, u32)>` in core with a thorough test module (hybrid, phyrexian,
two-brid, MDFC faces, Wastes, rounding exactness, identity gate, degenerates).

## Wiring (no server changes)

The existing import endpoint does everything needed:
`POST /api/deck/{id}/card/import` with `text = "8 Mountain\n6 Forest"`,
`board = "deck"`, `mode = "add"` — one call, resolved server-side by exact
name against `latest_cards` (which already prefers real printings). No new
endpoint, no migration, no `.sqlx` change, backward-compatible everywhere.

Client: one button + preview dialog + the core call + the import call +
refresh deck state. The Add screen's `ensure_lands_excluded` picks the new
count up on next serve automatically.

## Edge cases & calls to confirm at review

| Case | Proposed rule |
|---|---|
| Basics already in deck | Count toward land total; fill only the remainder (no rebalance v1) |
| Split/adventure faces | Both faces' costs count (both are castable) |
| MDFC | Both faces count (simplest; refine later if it skews) |
| Snow decks | v1 adds non-snow; snow toggle later |
| Wastes | Only from `{C}` pips or colorless identity |
| Undo | No special path — cards are normal deck rows, removable as usual (removal-suppression only targets nonbasics searches; basics never appear in the swipe pool anyway once at target) |

## Phases

1. **Core math + tests** (`zwipe-core`) — the whole risk surface, pure.
2. **Client button + preview dialog + import call** (deck view).
3. **Polish (optional, later):** empty-stack nudge hook, snow toggle,
   rebalance mode.

Effort: **S/M, client-only.** Pairs naturally with FR #12 (deck stats sheet)
in the same release — both live on the build screens.
