# QOL Bundle — Launch-Feedback Quick Wins

## Goal

Client-only (`zwiper`) quality-of-life fixes requested in the Reddit launch
thread. Source list: `context/progress/feature_requests.md`.

**Shipped already (removed from this plan):** B — always show the card name;
C — per-field validation errors (red outline + message, via `TextInput`'s
`error: Option<String>` prop; wired across register / change-email /
change-password / forgot-password); E — browse-all-tags hint dialog
(`DeckTag::description()` written, tag picker hint live).

**Remaining:** A (parked), D (not built) below — both still client-side, no
server deploy or migration.

---

## A. Live swipe drag indicators  (req #1, Tenellum) — PARKED

**Status:** live-drag cue works on branch `feat/qol-drag-indicators`; visual
style undecided, parked pending user complaints. See
`progress/project_drag_indicators_parked` memory. Do not re-plan here — revive
the branch if it comes back.

**Problem:** "I kept forgetting which way is which and had to undo." Today the only
feedback is the post-swipe toast — nothing *while* dragging.

**Approach:** In the `SwipeStack` (`components/interactions/swipe/stack.rs`, drag
state in `state.rs`), as the top card is dragged past a small reveal threshold,
show a directional cue that intensifies with drag distance:
- a colored edge/overlay on the card, and
- a short text label ("Add" / "Skip" / "Maybe" / "Undo").

Reuse the color vocabulary already established in the swipe hint dialog so it's
consistent: `--color-success` (right/add), `--color-error` (left/skip),
`--color-warning` (up/maybe), `--accent-tertiary` (down/undo). CSS in
`assets/main.css`.

**Files:** `swipe/stack.rs`, `swipe/state.rs`, `assets/main.css`.
**Effort:** M (the meatiest item, but self-contained). **Impact:** High.

## D. Deck stats reachable while building  (req #12, AdditionalLeopard688)

**Problem:** While swiping in the Add (and Remove) screen there's no quick way to
see card count or mana curve — you have to leave the stack.

**Approach:** Add a **util-bar button on both the Add and Remove screens** that
opens a **bottom sheet showing deck information** — **reuse the metrics/stats
panel from the deck profile screen** (count, curve, etc.) rather than rebuilding
it. Extract that panel into a shared component if it isn't already, then mount it
inside a `BottomSheet` opened from each screen's util bar. Opening/closing the
sheet must not disturb the swipe stack position.

**Files:** `screens/deck/card/add.rs`, `screens/deck/card/remove.rs`, and the deck
profile metrics/stats component (`screens/deck/components/deck_profile.rs` or
wherever the count/curve panel lives — extract/share it). The draw-odds /
distribution charts (`deck_charts.rs`) are the reusable surface to mount.
**Effort:** M (mostly extracting + reusing the existing stats panel). **Impact:** Med.

---

## Open questions / decisions

1. **A — parked:** label style (big centered word vs. corner chip) undecided;
   that's why the branch is on hold.
2. **D — RESOLVED:** util-bar button on Add + Remove → bottom sheet reusing the
   deck profile metrics panel.

## Deferred (need core/server/data work — not this client-only bundle)

- **"Typal" tag** (req #13): touches the `DeckTag` enum in `zwipe-core` + server
  validation → needs a server deploy first. Fold into the next server batch.
- **Prefer original/English printing** (req #9): printing-selection ranking,
  server/data work.
- **Price threshold filter** (#10): needs query support.
- **Persist skipped cards** (#11): needs storage.
- **Auto land base / land cap / MV weighting** (#4, #5, #6): logic/recommender.
- **CubeCobra import** (#16): new import source.

## Verification & ship

- `cargo check -p zwiper` + `cargo clippy -p zwiper --all-targets -- -D warnings`.
- Manual pass in `dx serve`: open deck stats mid-swipe on Add and Remove.
- **Deploy:** client-only — rides the next iOS/Android build. No server deploy, no
  migration, no min-version gate.
