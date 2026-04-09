# SwipeStack — Architecture Notes

## What shipped

The card-swipe flow on the add and remove screens was overhauled in three commits on the `swipe-updates` branch, merged to `main`:

- `f4fdf953` — SwipeStack: peeking stack with drag tilt, undo enter animations, container-level transitionend
- `711f5447` — SwipeStack: exiting overlay, down-swipe clamp, lower gesture thresholds
- `567db788` — Merge branch 'swipe-updates'

Before: a single `<img>` inside a `Swipeable` wrapper. On commit, a 75ms CSS exit animation played, then `current_index++` swapped the image source. Two problems:

1. The next card's image wasn't in the DOM until its turn — every swipe showed a blank frame while the next image decoded.
2. The rebound/re-render felt like "one card switching pictures" rather than a deck.

After: a peeking stack of up to 10 cards rendered at once, exit-via-overlay, instant gesture responsiveness, drag tilt, and down-clamp for undo.

## Architecture — the current design in one picture

```
SwipeStack (zwiper/src/lib/inbound/components/interactions/swipe/stack.rs)
│
├── Main stack loop
│   for (i, card) in cards[0..N].enumerate().rev()
│       ├── i == 0: top card — gesture handlers, drag transform, tilt
│       ├── i >  0: peek layer — translateY(i * 6px) scale(1 - i * 0.03)
│       └── key: card.scryfall_data.id (stable across rerenders)
│
├── Exiting overlay loop
│   for (id, dir, card) in exiting_overlay()
│       └── absolute-positioned, z-index 100, CSS keyframe exit, removes
│           itself on animationend
│
└── State
    ├── state: Signal<SwipeState>          — current gesture (top card only)
    ├── exiting_overlay: Signal<Vec<...>>  — in-flight exits (supports concurrent)
    └── entering: Signal<Option<Direction>> — undo enter-keyframe request (prop)
```

## Key design decisions — the "why" behind the code

### 1. Exit-via-overlay, not exit-in-place

**Problem:** Rapid swiping was rate-limited to ~200ms per commit because the stack used a single-slot `exit_target: Option<(Uuid, Direction)>` and gated new gestures on `exit_target.is_some()`. Two cards couldn't exit concurrently without corrupting the state.

**Solution:** Replaced the single slot with `exiting_overlay: Vec<(Uuid, Direction, Card)>`. On commit, the card is pushed onto the overlay AND `current_index` advances synchronously. The main stack immediately renders the new top (interactive on the very next render), while the exiting card lives in an overlay layer above the stack, playing a one-shot CSS keyframe that removes itself on `animationend`.

**Result:** No gesture-block guard. Multiple cards can be mid-flight simultaneously. The user can chain commits as fast as their finger moves.

The overlay approach also eliminated a whole class of bugs from the earlier attempts: stale `exit_target` blocking new gestures, dual-exit corruption on rapid commits, `is_exiting` closure captures going stale across rerenders, race conditions between `transitionend` and parent `current_index` updates. All of those evaporate when the exit animation is a fire-and-forget CSS keyframe on a decoupled overlay element.

### 2. Down-swipe visual clamp — `delta.y.min(0.0)`

**User constraint:** "The card should not move at all on down-swipe." Down-swipe is undo — the card being swiped shouldn't reposition at all, but the previous card should slide back in via the existing enter animation.

**Solution:** A one-line clamp in the top-card style branch:

```rust
let ty = if s.traversing_axis == Some(Axis::Y) {
    delta.y.min(0.0)  // Suppress downward movement, allow upward
} else {
    0.0
};
```

**Why this works without touching `SwipeState` or `SwipeConfig`:** The gesture detection (`set_latest_swipe`) operates on the raw delta from `delta_from_start_point()`, not on the rendered transform. So `Direction::Down` still fires on threshold — the clamp only affects what the user sees during the drag. Undo still works; the card just stays anchored. `SwipeState` and `SwipeConfig` are untouched.

### 3. Gesture thresholds tuned for rapid swiping

Originally 150px distance / 5.0 px/ms speed — conservative. The user wanted sensitivity. Now in both `add.rs` and `remove.rs`:

```rust
let swipe_config = SwipeConfig::new(
    vec![Direction::Left, Direction::Right, Direction::Up, Direction::Down],
    60.0, // distance threshold in px
    1.5,  // speed threshold in px/ms
);
```

**How the thresholds compose:** `set_latest_swipe` commits if **either** `distance > distance_threshold` **OR** `(distance > 10px AND speed > speed_threshold)`. The 10px hard floor on the speed path is baked into `state.rs:157` as `DISTANCE_THRESHOLD_FOR_SPEED_TO_BE_VALID` — it prevents a single tap from registering as an infinitely-fast tiny movement.

So a 60px deliberate drag commits, OR a quick flick of 10+px at >1.5 px/ms commits. Both feel responsive. If they start misfiring on accidental drags, bump the distance back to 80-100px.

### 4. Drag tilt — rotation tied to horizontal delta

```rust
let rot = tx * TILT_PER_PX;  // TILT_PER_PX = 0.06 deg/px
```

Only applies on X-axis drags (vertical drags stay flat). Closes the Android polish item from `context/status/todo.md` where cards used to translate without rotating on Android WebView.

### 5. CSS class owns the transform transition

Earlier iterations had inline `transition: transform {secs}s` per card, which caused choppy peek→top promotion because peek cards had no inline transition and the class transition wasn't picked up when the inline style changed. Current design:

```css
.swipe-stack-card {
    transition: transform 0.2s ease-out;  /* class owns it */
}
```

The only inline override is during an active drag, where the card must follow the finger 1:1:

```rust
let transition_override = if s.is_swiping {
    "transition: transform 0s;"
} else {
    ""
};
```

This means peek→top promotion, sub-threshold return-to-origin, and any transform change all interpolate smoothly through the class transition — zero per-card bookkeeping.

### 6. Keyed rendering with stable card ids

Every card wrapper has `key: "{card_id}"` (using `card.scryfall_data.id`). This is load-bearing — Dioxus uses the key to preserve DOM elements across renders when the slice shifts. When `current_index` advances, the card at the new position-0 (which was at position-1 before) keeps its DOM element; only its inline style changes. The class transition on `transform` interpolates from the peek position to the origin position, giving the smooth "grow into place" animation.

### 7. Enter animation for undo — keyframe, not transition

When undo fires, the parent sets `entering_direction: Signal<Option<Direction>>`. The stack applies a CSS class (`card-stack-enter-{dir}`) to the new top card, which triggers a one-shot keyframe animation. On `animationend`, the stack clears the signal.

**Why keyframe instead of transition:** Keyframes run regardless of initial DOM state (no `from → to` CSS timing race), so the animation fires reliably even on a freshly-mounted element. Transitions require the browser to observe a style change on an existing element, which can be flaky when an element is added mid-undo.

## Data flow: a single rapid swipe sequence

```
User flicks right on card A
│
├── ontouchmove → state.ontouchmove → rerender → card A follows finger
├── ontouchend → state.ontouchend → state.latest_swipe = Some(Right)
│
└── dispatch_latest:
    ├── state.latest_swipe = None
    ├── exiting_overlay.push((A.id, Right, A))          ← overlay gets A
    └── on_swipe_right.call(A)                          ← parent advances index
        │
        └── add.rs advance_after_commit:
            ├── action_history.push(SwipeAction::Do { ... })
            ├── add_card_to_deck(A)                     ← backend POST
            └── current_index += 1                      ← slice shifts

Next render:
├── Main stack: cards[1..] — B is now at i=0, origin transform via CSS transition
├── Overlay: [(A.id, Right, A)] — A flies off via card-stack-exit-right keyframe
└── Gestures: B's ontouchstart passes guards — interactive immediately

180ms later:
├── A's animationend → exiting_overlay.retain(|id| != A.id) → A removed from DOM
└── B's class transition completes (200ms) → B at origin, no visible change
```

The key insight: the exit animation's lifetime is **decoupled** from the main stack's state. The user can commit swipe after swipe without waiting for any previous animation to finish.

## Timings and tunables

All of these are single-point-of-change if you want to adjust feel:

| Parameter | Value | Location |
|---|---|---|
| Stack depth | 10 cards | `stack.rs` `STACK_DEPTH` |
| Peek vertical offset | 6px per layer | `stack.rs` `PEEK_OFFSET_PX` |
| Peek scale step | 0.03 per layer | `stack.rs` `PEEK_SCALE_STEP` |
| Tilt rate | 0.06 deg/px | `stack.rs` `TILT_PER_PX` |
| Class transition (peek→top, return-to-origin) | 0.2s ease-out | `main.css` `.swipe-stack-card` |
| Exit keyframe duration | 0.18s ease-out forwards | `main.css` `@keyframes card-stack-exit-*` |
| Enter keyframe duration | 0.2s ease-out | `main.css` `@keyframes card-stack-enter-*` |
| Distance threshold | 60px | `add.rs` / `remove.rs` `SwipeConfig::new` |
| Speed threshold | 1.5 px/ms | `add.rs` / `remove.rs` `SwipeConfig::new` |
| Speed-path distance floor | 10px | `state.rs:157` `DISTANCE_THRESHOLD_FOR_SPEED_TO_BE_VALID` |

## Files involved

**Core component:**
- `zwiper/src/lib/inbound/components/interactions/swipe/stack.rs` — `SwipeStack` component (the whole thing)
- `zwiper/src/lib/inbound/components/interactions/swipe/mod.rs` — module registration, re-exports

**Gesture infrastructure (unchanged from pre-refit — still used):**
- `zwiper/src/lib/inbound/components/interactions/swipe/state.rs` — `SwipeState`, thresholds, `set_latest_swipe`
- `zwiper/src/lib/inbound/components/interactions/swipe/config.rs` — `SwipeConfig`
- `zwiper/src/lib/inbound/components/interactions/swipe/ontouch.rs` / `onmouse.rs` / `onswipe.rs` — gesture event traits
- `zwiper/src/lib/inbound/components/interactions/swipe/direction.rs` — `Direction` enum
- `zwiper/src/lib/inbound/components/interactions/swipe/axis.rs` — `Axis` enum

**Parent screens:**
- `zwiper/src/lib/inbound/screens/deck/card/add.rs` — uses `SwipeStack`, threads `entering_direction`, handles undo
- `zwiper/src/lib/inbound/screens/deck/card/remove.rs` — same pattern, local `displayed_cards` mutation model

**Undo data model:**
- `zwiper/src/lib/inbound/screens/deck/card/components/action_history.rs` — `SwipeAction` enum variants now carry `exited: Direction` so undo can replay the same direction

**Styles:**
- `zwiper/assets/main.css` — `.swipe-stack`, `.swipe-stack-card`, `.swipe-stack-exiting`, `@keyframes card-stack-{enter,exit}-{left,right,up,down}`

## Edge cases the current design handles

- **Stack shorter than STACK_DEPTH near end of buffer:** `cards.into_iter().take(STACK_DEPTH)` naturally produces 0–10 elements. Stack shrinks honestly.
- **Empty stack:** `!cards().is_empty()` check in `add.rs` / `remove.rs` renders `CardSkeleton` instead.
- **Last card swiped:** Exits via overlay, main stack becomes empty, `CardSkeleton` shows. Overlay removes the card on `animationend`.
- **Undo during a previous exit:** The overlay card is still mid-flight; the parent rolls back `current_index`, `entering_direction` is set, the newly-restored top plays its enter keyframe. Both animations coexist because they're on separate DOM elements.
- **Sub-threshold return:** Gesture released below threshold → `latest_swipe = None` → `state.reset()` clears points → style recomputes to origin → class transition animates the return over 0.2s.
- **Downward sub-threshold drag:** Completely invisible because of the `delta.y.min(0.0)` clamp — never moved, nothing to return from.

## Known trade-offs

- **Gesture-block during exit is removed entirely** in the overlay design. If a card is mid-exit in the overlay and the user immediately starts dragging the new top, gestures work but the visual state briefly has both cards at overlapping positions until the overlay's keyframe progresses. In practice this is imperceptible at 180ms.
- **Exiting card loses drag continuity.** Once a swipe commits, the exiting card's transform is driven by the CSS keyframe — it no longer follows the finger. If the user drags past the commit threshold and then tries to drag back below it, the commit has already fired. This matches the intended "no rebound" behavior from the plan but might surprise users coming from Tinder-style swipers where you can "save" a commit with a quick reverse flick.
- **Stack depth is fixed at 10.** Lowering it would reduce DOM cost but also shrink the visual peek effect. Raising it wastes image decode work since peek layers 5+ are barely visible.

## Where to look next if something feels wrong

- **Choppy grow:** Check `.swipe-stack-card` transition in `main.css` — it should be 0.2s ease-out. If peek cards are missing it, they'll snap.
- **Stuck gestures after commit:** Check `exiting_overlay` clearing logic in `stack.rs` — the `onanimationend` handler should retain only ids that don't match. Check CSS keyframe `animationend` is firing (the `forwards` fill is important).
- **Misfiring swipes / false commits:** Raise the thresholds in `add.rs` / `remove.rs`. Distance to 80px is a safe first bump.
- **Down-swipe visually moves the card:** The `delta.y.min(0.0)` clamp in `stack.rs` is load-bearing — if that's missing, the clamp is gone.
- **Drag tilt not appearing:** Check `rot = tx * TILT_PER_PX` in the top-card style branch. Y-axis drags have `tx = 0` so no rotation, as intended.

## Relation to `clone-deck.md`

Unrelated feature but built on top of the same main branch. The clone-deck plan is in this same `context/plans/` directory and was executed in commits `ff367d86` (backend) and `fdf01a99` (frontend). No interaction with the swipe flow.
