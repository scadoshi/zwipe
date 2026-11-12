# 360. The wide modals pop in with no motion

State: fixing | Owner: laptop | PR: - (orangui#35) | Build seen: production, web

## Report
Composer for Scotty, 2026-11-11, web: the full-screen and wide modals (the Printings picker, the Oracle tags dictionary, the Oracle tag Examples modal and any other large sheet) just pop in. They should get a softer entrance: a fade with a small rise or grow (from 0.98 and 8px down) over about 250 to 300ms on cubic-bezier(0.22, 1, 0.36, 1), the curve the stacks use. It should be one shared curve and duration from orangui tokens, set on the shared modal rather than per screen. The exit can stay quicker, and prefers-reduced-motion must be respected.

## Cause
- The kit's `.sheet-form-modal` eased in with a transition from its closed state. Printings and the dictionary mount already open, so the transition had no starting style and the modal appeared at once.
- The Examples modal and the narrower dictionary (`.screen.web-large-modal`, from 900px) are fixed screens with no entrance at all.

## Fix
laptop:
- orangui#35:
  - `--ease-enter` (cubic-bezier(0.22, 1, 0.36, 1)) and `--enter-duration` (280ms) on `:root`;
  - `@keyframes modal-enter`, from opacity 0, `translate: 0 0.5rem` and `scale: 0.98`, so it composes with the modal's centering transform;
  - `.sheet-form-modal > .bottom-sheet.show` runs it, and its backdrop fades in with `backdrop-enter`;
  - the close keeps the kit's 0.2s ease;
  - reduced motion turns both off.
- zwipe:
  - `.screen.web-large-modal` runs the same `modal-enter` on the same tokens, so Examples and the narrower dictionary enter the same way, off under reduced motion.
  - The stacks' `--card-stack-ease` reads `--ease-enter`.

## Verification
-
