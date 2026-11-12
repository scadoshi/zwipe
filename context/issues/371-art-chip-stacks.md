# 371. Art stays live while Stacks is on

State: fixing | Owner: laptop | PR: - | Build seen: production, web

## Report
Composer for Scotty, 2026-11-12: on the deck page, and anywhere the List / Stacks switch is, the Show row's Art chip should be grayed out while Stacks is on, since stacks always show art. It should use the existing disabled-chip look (the grayed-out Side board chip), not be hidden, and come back with its state when List is on.

## Fix
laptop:
- components' `DeckControls` takes `art_disabled`, which passes `disabled` to the Art `Chip`: the kit's own disabled chip, the look the empty Maybe and Side board chips have. The chip keeps its `show_art` state.
- The deck page passes `art_disabled` while the view is Stacks.
- Playstyle's Art chip, which sits beside the List / Stacks switch that #574 adds, is grayed out the same way in #574.

## Verification
-
