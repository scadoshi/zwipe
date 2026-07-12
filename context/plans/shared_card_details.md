# Shared card-details component

**Status: PLAN (2026-07-12).** One shared card-details element (type line, oracle
text, stats, keywords, card roles, with elegant MDFC handling) used by every
consumer so they never drift as the design changes.

## Goal

A single `CardDetails` body rendered identically by:
1. **`CardRow`** (zwipe-components) — the expanded deck-card row. Already shared:
   zwiper's deck view AND zite's shared-deck screen both render
   `zwipe_components::CardRow` (zite wraps it as `SharedCardRow`).
2. **`CardDetailsDialog`** (zwiper `card_info.rs`, renamed from `CardRulesDialog`)
   — the swipe-screen eyeball dialog.

So all three surfaces (deck view, zite shared deck, swipe eyeball) show the exact
same card details, MDFCs included, differing only in the bottom action bar.

## Current state (the divergence)

- **`CardRow`** builds its detail from **top-level** `scryfall_data` fields
  (`type_line`, `oracle_text`). For modal DFCs (e.g. Valki // Tibalt) the top-level
  `oracle_text` is empty (text lives per-face), so **the row shows no oracle text**.
  Old block order (keywords/roles before text), no flip.
- **`CardRulesDialog`** has its own renderer with per-face `build_rules` (handles
  MDFCs), the new **reorder** (type -> text -> stats, then keywords + card roles),
  and **one-face-at-a-time + Flip** (in progress, uncommitted).

Two renderers, only one of which handles MDFCs and the new layout.

## Design

The detail body is identical on every surface; only the **bottom action bar**
differs (read-only view / read-only + image·printing-view / full read-write). So
`CardDetails` is the engine that owns *both* the detail and the action bar.

**`CardDetails`** (new, in `zwipe-components`) renders:
- **Detail** (reordered): current face's type line, oracle text, stats; then the
  whole-card keywords + `CardRoleChips` cluster at the bottom.
- **MDFC handling:** move `build_rules` / `FaceRules` / `stats_line` from zwiper
  `card_info.rs` into zwipe-components; own a `face` signal internally; render a
  flip toggle only when the card has >1 face.
- **A bottom action bar** with:
  - **Default always-on actions it owns:** **Flip** (internal, MDFC-only),
    **Image** (when an `on_image` handler + art are present).
  - **Additional consumer actions**, passed in as an `actions: Element` slot,
    appended to the bar (built with the shared `card-action-btn` styling so they
    stay consistent).

**Consumers** wrap `CardDetails` with their own chrome and configure the bar:
- **`CardRow`** (deck view + zite via `SharedCardRow`): the compact row + expand,
  wrapping `CardDetails`. Passes the full write action set as the `actions` slot:
  qty stepper, printing (**change** mode), star/MVP, move-to-maybe/side. (zite
  passes a read-only subset.)
- **`CardDetailsDialog`** (renamed from `CardRulesDialog`, zwiper): the alert
  dialog + title (name + current-face cost) wrapping `CardDetails`. Passes a
  read-only `actions` slot — printing in **view** mode — plus the dialog's own
  Close. (Add/remove swipe screens: view-only, image + view-printing.)

### Printing has a mode
Some surfaces **change** the printing (deck CardRow — write), others only **view**
it (eyeball dialog, zite — read-only). Model this as a printing action mode
(`View` vs `Change`) so the same button renders the right behavior per surface.

## Flip control placement (decided 2026-07-12): inside `CardDetails`

The shared component renders its own flip toggle for multi-face cards and owns its
`face` signal internally, so the flip is identical on all three surfaces (deck
view, zite, eyeball) with zero per-consumer wiring. Consumers just drop
`CardDetails { card }`.

Placement within the detail: a compact flip control near the type-line row (a
"Flip" / face toggle), only rendered when the card has >1 face. Keep the terminal
aesthetic (a small util-styled control, no glow).

## Files

- **New** `zwipe-components/src/card_details.rs` — `CardDetails` (detail + action
  bar + internal face/flip) + the moved `build_rules`/`FaceRules`/`stats_line` +
  the printing-mode enum. Export from `zwipe-components/src/lib.rs`.
- `zwipe-components/src/card_row.rs` — becomes a thin wrapper: compact row +
  expand-collapse around `CardDetails`, passing its write actions as the `actions`
  slot. Drops the inline top-level detail (type/oracle/stats/keywords/roles) and
  its own action-row markup (moves into `CardDetails`).
- zwiper `card_info.rs` — **rename `CardRulesDialog` -> `CardDetailsDialog`**; it
  becomes the alert-dialog wrapper around `CardDetails` (title + Close + a
  view-only printing action). Drop the local `build_rules`/`FaceRules`/`stats_line`
  and the one-off flip/reorder (folded into `CardDetails`). Update the call sites +
  `RulesButton`/imports referencing the old name.
- No zite change (it uses `CardRow`).

### Naming
- Shared engine: **`CardDetails`**.
- Eyeball dialog: **`CardDetailsDialog`** (was `CardRulesDialog`) — a configured
  wrapper (view-only printing) around `CardDetails`.

## Sequencing

**Gate: after the Phase M `DeckTag` rename settles** (the tree is mid-broken by it;
`DeckTag` -> `String`). Then:
1. Build `CardDetails` in zwipe-components (move `build_rules`, add face + flip).
2. Point `CardRow` and `CardRulesDialog` at it.
3. Verify: `clippy -p zwiper -p zwipe-components --all-targets -D warnings`, nightly
   fmt, and confirm the portfolio (another `zwipe-components` consumer) still builds.
4. Commit.

## Note

This **supersedes** the in-progress one-off reorder + flip in `card_info.rs`
(uncommitted) — those get folded into the shared `CardDetails`. Keep them as a
working reference until the shared component lands, then remove.
