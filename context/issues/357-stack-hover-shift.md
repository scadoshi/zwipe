# 357. Hovering a stack shifts its cards a pixel

State: fixing | Owner: thinker | PR: - | Build seen: production, web wide (screen recording)

## Report
Scotty, 2026-11-11, web, with a screen recording: on the Stacks view, click a card in a stack (it highlights), then hover it: the visible images of every card in that stack get a pixel or two narrower and the whole stack shifts sideways at the same moment. Neighboring stacks do not shift, only that one.

## Cause
`.card-stack:hover > .card-stack-slot { will-change: transform }` promoted the hovered stack's slots to compositor layers. A stack whose column sits at a fractional x (the grid's `1fr` columns, browser zoom, a window width that does not divide evenly) is then drawn snapped to whole pixels, its images re-rasterized narrower and shifted. Layout never changes (every rect stays the same), which is why only that stack moved and only on some widths. Hovering an already-open card starts no transform, so the promotion is the whole visible change.

## Fix
thinker: the hover no longer sets `will-change`; the browser still promotes a slot on its own while its transform runs. Failing-first test `nothing_about_a_stack_promotes_its_cards_to_layers` (`components/src/card_stack.rs`). Headless proof: every card's rect and a pixel compare of every stack, mouse away against hovering its last card. Unfixed at 2x: up to 310,521 of 362,464 pixels change by up to 240 levels, rects identical. Fixed: pixel-identical on every stack at 1x and 2x, zoom 0.9 and 1.1, widths 1283, 1301, 1440 and 1920.
