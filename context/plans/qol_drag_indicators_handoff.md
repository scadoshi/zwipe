# Handoff — Live Swipe Drag Indicators (PARKED)

Status of QOL bundle **item A** (`context/plans/qol_bundle.md`). The gesture
logic is built and wired; the **visual treatment is deliberately unbuilt and
undecided**. This doc is the decision record so the feature can be finished in
one focused sitting later.

## Initial intent

Give live feedback **while dragging** a card so users stop forgetting which
direction does what (top Reddit-launch complaint: "I kept forgetting which way
is which and had to undo"). Today the only feedback is the post-swipe toast —
nothing happens *during* the drag.

The plan: as the top card is dragged past a small reveal threshold, surface a
directional cue for the active direction that intensifies with drag distance —
Add (right) / Skip (left) / Maybe (up) / Undo (down).

## Where we stand

**Logic: done and shipping (inert).** `SwipeStack`
(`zwiper/src/lib/inbound/components/interactions/swipe/stack.rs`) derives the
cue every render and exposes it on the stack container, but renders **no visual
treatment**:

- `CUE_REVEAL_PX = 12.0` — drag distance before the cue begins, well under the
  commit threshold.
- A `cue: Option<(Direction, f64)>` computed from the existing swipe state —
  active direction from `traversing_axis` + sign of `delta_from_start_point()`,
  gated by `config.allowed_directions`, with
  `intensity = ((along - CUE_REVEAL_PX) / (distance_threshold - CUE_REVEAL_PX))`
  clamped to `0.0..=1.0`.
- Exposed as `data-swipe-cue="left|right|up|down|none"` and a
  `--swipe-cue-intensity: <0.0..=1.0>` CSS custom property on `.swipe-stack`.

Nothing reads those hooks yet, so there is **zero visual change** in the app.
`cargo check -p zwiper` and `cargo clippy -p zwiper --all-targets -- -D warnings`
are clean. No screens were touched; the data layer (`state.rs`) was read only,
not modified.

**Visual: not built.** A working color-flood prototype (a directional gradient
bleeding in from the viewing-area edge, ramped by intensity) was implemented and
rejected as "tacky" — a soft glow clashes with the app's crisp identity
(JetBrains Mono, the 28px background grid, 1px solid outlines, lettered
tracking). It was removed.

## The decision

**Parked 2026-06-28.** Ship the inert logic; do not ship a glow/haze/soft
gradient. Revisit the visual only if launch feedback asks for it. Whatever gets
built must be crisp/geometric and match the terminal/grid aesthetic — no glows.

Crisp directions floated but not chosen:
- **Frame glow** — crisp colored outline on the active card, brightest toward
  the drag side; matches the app's border language. (Safest.)
- **Corner reticle ticks** — mono bracket marks at the drag-side corners that
  sharpen toward commit; most distinctly "terminal UI".
- **Edge grid-charge** — the existing background grid lines near the active edge
  take the direction color; most uniquely on-brand, subtlest, most fiddly.
- **Edge scan bar** — a thin solid bar inside the active edge that fills toward
  commit; doubles as a progress-to-commit meter.

## What's left to do (in detail)

1. **Pick a visual** from the list above (or a new crisp idea). Color
   vocabulary is fixed: `--color-success` right, `--color-error` left,
   `--color-warning` up, `--accent-tertiary` down.
2. **Render it in `stack.rs`** off the already-computed `cue`. Either consume
   the `data-swipe-cue` / `--swipe-cue-intensity` hooks from CSS, or render
   markup inside `.swipe-stack` driven by `cue_dir` / `cue_intensity`. Keep it
   on the top card / viewing area only — the exiting-overlay cards must not get
   it.
3. **Handle release.** Note `onswipeend` calls `self.reset()` (`onswipe.rs`), so
   the gesture state clears the instant you let go — a naively mounted element
   vanishes instead of fading. For a fade-back-on-release, keep the visual
   element(s) always mounted and drive only opacity/intensity via CSS
   transition, suppressing the transition during the active drag (mirror how the
   card transform uses `transition: transform 0s` while `is_swiping`).
4. **Add the CSS** in `assets/main.css`, grouped under the swipe-stack styles
   (single hunk — other branches also append here). No needless comments.
5. **Verify:** `cargo check -p zwiper` + `cargo clippy -p zwiper --all-targets
   -- -D warnings`, then `dx serve` and drag each direction; confirm it fades
   back if released before committing and does not regress commit thresholds or
   the enter/exit animations.

## Deferred (not in this branch)

- **User toggle** to disable the cue (some users may not want it). Out of scope
  here — needs a setting + persistence + a flag plumbed into the stack. Add only
  if feedback warrants.
- **Remove-screen color inversion.** On the Remove screen the horizontal
  semantics invert (right = remove from deck = destructive/red; left = keep =
  green), opposite of Add. The cue currently colors by physical direction, so
  honoring this means giving the stack screen context (a prop from `remove.rs`)
  — beyond the original "don't touch the screens" scope. Decide alongside the
  visual.
