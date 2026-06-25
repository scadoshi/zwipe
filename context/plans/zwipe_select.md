# Zwipe select — extend to partner / background / signature spell

Swipe-to-pick for command-zone cards. Phase 1 (commander) is **built** on
`feat/zwipe-for-commander`; this plan covers extending it to the other
command-zone fields and finishing the "Zwipe" branding.

## What's already built (phase 1)

- **`OrderByOption::EdhrecRank`** (zwipe-core enum + both `filter_cards.rs` sort
  sites + zerver SQL `ORDER BY edhrec_rank ASC NULLS LAST, name ASC`). Additive
  and backward-compatible — **server must deploy before any client build ships**
  that sends it (per `api_evolution.md`).
- **`CommanderSwipe`** (`zwiper/.../deck/components/commander_swipe.rs`) — a
  full-screen swipe over commander-eligible cards in EDHREC order, using the
  shared `CardFilterSheet`. Always-mounted sibling toggled by `open`, so filter +
  position persist across open/close. Left = skip, right = choose, up = nothing
  (card doesn't move — `stack.rs` clamps upward movement to `Up`-allowed configs),
  down = undo. `?` button + one-time `HintDialog` (`HINT_SWIPE_SELECT`).
- Launched from a primary-outlined **"Zwipe"** chip in the commander label-row
  (`deck_fields.rs`), wired through `create.rs` / `edit.rs` as a sibling overlay.
- Deck-form explainer (`DeckFieldsHint`) + `?` buttons on create/edit
  (`HINT_CREATE_DECK` / `HINT_EDIT_DECK`).
- Hint keys renamed to match screens (`add_deck_cards`, `remove_deck_cards`,
  `swipe_select`, `create_deck`, `edit_deck`).

## Branding (mostly done)

Button = **"Zwipe"**; page title + hint title = **"Zwipe select"**. Keep the
literal **gesture** verb lowercase "swipe" in instructions ("swipe right to
choose") — brand the feature, not the motion. No global swipe→zwipe replace.

## Phase 2 — the other command-zone fields

**Oathbreaker is already covered** — it's the commander field with
`is_commander_in_format(Oathbreaker)`. Net-new targets: **Partner**,
**Background**, **Signature spell**.

### Step 1 — generalize the component

Rename `CommanderSwipe` → **`SwipeSelect`** and replace the hard-coded
`commander_filter()` with a mode that seeds the always-on constraints. Mirror the
filters `deck_fields.rs` already uses for each typeahead:

```rust
enum SwipeMode {
    Commander(Format),       // set_is_commander_in_format(format)
    Partner,                 // set_is_partner(true)
    Background,              // set_is_background(true)
    SignatureSpell(Colors),  // set_is_signature_spell(true) + set_color_identity_within(colors)
}
```

`fn base_filter(mode, builder, offset)` applies the mode's constraint + the
EDHREC-order default + `is_token(false)` + limit/offset (today's
`commander_filter` becomes the `Commander` arm). Everything else
(persistence, hint, swipe semantics, `CardFilterSheet`) stays identical.

The hint title/key stays `swipe_select` (generic) for all modes.

### Step 2 — per-field chips + instances

In `deck_fields.rs`, add a "Zwipe" chip to each field's label-row, each behind a
`show_*_swipe` signal (the chips already only render when their field shows, via
the existing `show_partner` / `show_background` / `show_signature_spell` memos).

In `create.rs` / `edit.rs`, render one `SwipeSelect` sibling per field, passing
the mode and an `on_select` that sets that field's `Signal<Option<Card>>` +
display string:

- Partner → `SwipeMode::Partner`, sets `partner_commander` / `_display`.
- Background → `SwipeMode::Background`, sets `background` / `_display`.
- Signature spell → `SwipeMode::SignatureSpell(commander color identity)`, sets
  `signature_spell` / `_display`. Read the identity from `commander()`'s
  `scryfall_data.color_identity` (matches `deck_fields.rs` signature search).

Each instance keeps its own persisted state (separate `use_signal`s), so all
four zwipe surfaces remember their place independently.

### Gotchas

- **Color identity for signature spell** depends on the chosen commander — if the
  commander changes, the persisted signature-spell stack is stale. Either reset
  that instance's `last_searched` on commander change, or accept it until the
  filter is touched (same minor edge already noted for format changes).
- Partner/background only matter once a commander with that ability is chosen —
  already gated by the field-visibility memos, so no extra guarding.
- Four always-mounted `SwipeSelect` siblings is fine (each renders nothing when
  closed), but confirm the `filter_builder` context providers don't collide —
  each instance provides its **own** local `filter_builder` / `filter_reset_counter`,
  so they're independent. Verify nesting doesn't leak the wrong context.

## Estimate

Low-to-moderate. The component is already generic; the work is the mode enum +
wiring three more chips/instances. ~1 focused session. Lower risk than phase 1.
