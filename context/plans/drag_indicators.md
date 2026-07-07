# Live swipe drag indicators

**Status: PARKED (2026-06). The live-drag cue works on branch
`feat/qol-drag-indicators`; visual style undecided. Revive the branch if
users complain — do not rebuild from scratch.** Feature request #1
(Tenellum, Reddit launch thread); extracted from the retired QOL bundle
(`../archive/qol_bundle.md`) when the rest of it shipped.

## Problem

"I kept forgetting which way is which and had to undo." Today the only
feedback is the post-swipe toast — nothing *while* dragging.

## Approach

In the `SwipeStack` (`zwiper/src/lib/inbound/components/interactions/swipe/stack.rs`,
drag state in `state.rs`), as the top card is dragged past a small reveal
threshold, show a directional cue that intensifies with drag distance:

- a colored edge/overlay on the card, and
- a short text label ("Add" / "Skip" / "Maybe" / "Undo").

Reuse the color vocabulary already established in the swipe hint dialog so
it's consistent: `--color-success` (right/add), `--color-error` (left/skip),
`--color-warning` (up/maybe), `--accent-tertiary` (down/undo). CSS in
`zwiper/assets/main.css`.

## Open question (why it's parked)

Label style — big centered word vs. corner chip — is the unresolved call.
The mechanics on the branch are done; this is purely a visual-taste decision.

## Files

`swipe/stack.rs`, `swipe/state.rs`, `assets/main.css` (all changes live on
the branch already).

## Ship

Client-only: `cargo check -p zwiper` + clippy, manual `dx serve` pass, rides
the next store build. No server deploy, no migration, no min-version gate.
