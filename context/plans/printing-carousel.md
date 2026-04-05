# Printing Carousel — Plan

Replace the thumbnail-selector printing sheet with a horizontal swipe carousel.

## Current State

`PrintingSheet` (zwiper `screens/deck/card/components/printing_sheet.rs`) shows:
- One large card image (selected printing)
- Set name · collector # · year
- USD · EUR prices
- Horizontal scrollable row of small thumbnails at the bottom (the "jank")
- "select printing" button that fires `update_deck_card`

## Target UX

A full-width card image carousel inside the bottom sheet:
- All printings laid out in a horizontal strip, one card visible at a time
- User's current printing is the **starting position** (not necessarily index 0)
- Swipe left → next printing (newer), swipe right → previous (older)
- Smooth slide transition: outgoing card slides out, incoming slides in from opposite side
- Bounce/resistance at edges (first and last printing)
- iOS-style page dots below the image showing current position
- Below dots: `set name | #collector_number | $usd | €eur | tix` (if applicable)
- Header shows **close** by default; **save** button appears beside close only after swiping to a different printing
- Close (when dirty) = discard + toast "printing selection dismissed"
- Save = `update_deck_card` with new scryfall_data_id, toast "printing updated", close sheet

## Why Not Reuse `Swipeable`

The existing `Swipeable` component is designed for **fling-away** gestures (card flies off screen, callback fires). The carousel needs **snap-to-page** behavior:
- Images stay in a strip; the viewport slides between them
- Release snaps to the nearest page, not "detected direction fires callback"
- Edge bounce requires dampened drag, not hard stop
- The strip position is `-(index * page_width) + drag_offset` during drag

These are fundamentally different motion models. A new `Carousel` component is the right call, but it can reuse `SwipeState`'s point-tracking and velocity math — we just interpret the results differently (snap vs fling).

## Implementation

### Step 1: Carousel State

New file: `zwiper/src/lib/inbound/components/interactions/carousel/state.rs`

```
CarouselState {
    current_index: usize,       // which page is "home"
    page_count: usize,          // total pages
    drag_offset_px: f64,        // current drag displacement
    is_dragging: bool,
    start_x: Option<f64>,       // touch/mouse start X
    velocity: f64,              // px/ms at release, for momentum
    snap_animation_ms: u64,     // transition duration for snap (e.g. 300ms)
}
```

Key methods:
- `on_drag_start(x)` — record start position
- `on_drag_move(x)` — update `drag_offset_px` with resistance at edges
- `on_drag_end(x)` — calculate target index from offset + velocity, snap
- `target_translate_px(page_width)` → `f64` — returns the CSS translateX value:
  - During drag: `-(current_index * page_width) + drag_offset_px`
  - At rest: `-(current_index * page_width)`
- `can_go_left() / can_go_right()` — boundary checks
- Edge resistance: when dragging past first/last, `drag_offset_px` is divided by a dampening factor (e.g. 3.0) to create rubber-band feel

### Step 2: Carousel Component

New file: `zwiper/src/lib/inbound/components/interactions/carousel/mod.rs`

```rust
#[component]
fn Carousel(
    state: Signal<CarouselState>,
    children: Element,  // caller renders the pages
) -> Element
```

Renders:
```
div.carousel-viewport (overflow: hidden, width: 100%)
  div.carousel-strip (display: flex, transform: translateX(...), transition: ...)
    { children }  — each child is one page, width: 100%
```

Touch/mouse handlers on the viewport:
- `ontouchstart` / `onmousedown` → `state.on_drag_start(x)`
- `ontouchmove` / `onmousemove` → `state.on_drag_move(x)`, update transform
- `ontouchend` / `onmouseup` → `state.on_drag_end(x)`, snap to nearest page

During drag: `transition: none` (immediate follow). On release: `transition: transform 300ms ease-out`.

Snap logic on release:
- If `|drag_offset_px| > page_width * 0.3` OR velocity > threshold → advance one page in drag direction
- Otherwise → snap back to current page
- Clamp to `[0, page_count - 1]`

### Step 3: Page Dots Component

New file: `zwiper/src/lib/inbound/components/interactions/carousel/dots.rs`

```rust
#[component]
fn CarouselDots(current: usize, total: usize) -> Element
```

Simple row of small circles, active dot highlighted. CSS:
- `width: 0.5rem; height: 0.5rem; border-radius: 50%`
- Active: `background: var(--color-text)`
- Inactive: `background: var(--color-text); opacity: 0.3`
- Flex row with `gap: 0.375rem`, centered

### Step 4: Rewrite PrintingSheet

Replace the current content of `PrintingSheet` with:

**Header:**
```
div.modal-header
  button "close" (always visible)
  if has_changed:
    button "save" (calls update_deck_card)
```

**Body:**
```
if is_loading:
  spinner
else:
  Carousel { state: carousel_state,
    for printing in printings:
      div { width: 100%,
        img { src: large_image_url, class: "card-image" }
      }
  }

  CarouselDots { current: carousel_state().current_index, total: printings.len() }

  // Info row for the currently visible printing
  div.text-muted (centered)
    "{set_name} | #{collector_number} | $usd | €eur | tix"
```

**State tracking:**
- `initial_index: usize` — set when printings load (position of current scryfall_data_id)
- `has_changed` = `carousel_state().current_index != initial_index`
- Close when dirty → toast "printing selection dismissed"
- Save → `update_deck_card(deck_id, current_scryfall_id, HttpUpdateDeckCard::with_printing(new_id))` → toast "printing updated" → close

**Remove entirely:**
- The horizontal thumbnail row (lines 140-179 of current file)
- The "select printing" button (lines 182-218)

### Step 5: CSS

Add to the app stylesheet:

```css
.carousel-viewport {
  overflow: hidden;
  width: 100%;
  touch-action: pan-y;  /* allow vertical scroll, capture horizontal */
}

.carousel-strip {
  display: flex;
  will-change: transform;
}

.carousel-strip.snapping {
  transition: transform 300ms ease-out;
}

.carousel-page {
  flex: 0 0 100%;
  width: 100%;
  display: flex;
  justify-content: center;
}

.carousel-dots {
  display: flex;
  justify-content: center;
  gap: 0.375rem;
  padding: 0.5rem 0;
}

.carousel-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
  background: var(--color-text);
  opacity: 0.3;
}

.carousel-dot.active {
  opacity: 1;
}
```

## File Changes Summary

| File | Action |
|------|--------|
| `zwiper/.../components/interactions/carousel/mod.rs` | **New** — Carousel component |
| `zwiper/.../components/interactions/carousel/state.rs` | **New** — CarouselState + snap logic |
| `zwiper/.../components/interactions/carousel/dots.rs` | **New** — CarouselDots component |
| `zwiper/.../components/interactions/mod.rs` | **Edit** — add `pub mod carousel;` |
| `zwiper/.../deck/card/components/printing_sheet.rs` | **Rewrite** — carousel + save/close header |
| `zwiper/assets/main.css` (or equivalent) | **Edit** — add carousel CSS |

## Edge Cases

- **Single printing:** No dots, no carousel drag, just show the image. Save button never appears.
- **Loading state:** Spinner fills the carousel viewport area. Carousel initializes after fetch completes.
- **Missing images:** Some printings lack `image_uris` (e.g. tokens with art variants). Fall back to `normal` or `small` size, or show placeholder.
- **Many printings:** Some cards have 50+ printings (e.g. Lightning Bolt). Dots become tiny — may need to cap dot display at ~15 and use a scrolling dot indicator, or just let them wrap. Decide during implementation.
- **Sheet re-open:** Cache still works via `cached_oracle_id`. On re-open, reset carousel to current printing's index (which may have changed if they saved a different one last time).

## Not Changing

- Backend endpoint (`GET /api/card/{oracle_id}/printings`) — works as-is, returns `Vec<Card>` ordered by release date
- `HttpUpdateDeckCard::with_printing()` — works as-is
- `on_printing_changed` callback — still fires on save, caller updates local card state
