# 372. Stacks appear all at once

State: fixing | Owner: thinker | PR: - | Build seen: production, web

## Report
Scotty, 2026-11-12, web: when the deck page switches to Stacks, deal the cards in. In each stack the top card enters first, then each card below it, every card rising in from below and settling into its slot, so the stack fills top to bottom like cards being dealt. All stacks deal at the same time. Keep it brisk: a short per-card stagger, capped, so even a 30-card stack deals well under a second. Use the shared `--ease-enter` token, skip it under prefers-reduced-motion, change nothing about layout afterward (the 357 rule), and deal only on switching into Stacks or on a first load in Stacks, never on a rerender or a filter change.

## Fix
thinker:
- zwipe-ui's `DealCue` says when to deal: entering Stacks arms it, and it fires once the cards are loaded. Staying in Stacks, rerendering, filtering or regrouping never fire it. The timing lives beside it: `DEAL_STAGGER` 18ms, `DEAL_STAGGER_CAP` 16 cards and `DEAL_DURATION` 380ms, so the longest deal takes 668ms.
- The deck page keeps a `dealing` flag for that long, and `CardStack` takes it as `dealing`. While it's on, the stack carries the `dealing` class and each slot its place as `--deal-i`.
- `magic.css` animates only the card button: it rises from 35% below its slot and fades in, on `var(--ease-enter)`, with `calc(min(var(--deal-i), 16) * 18ms)` of delay, filling backwards. The slot, whose transform reveals, is untouched; nothing stays on the card after the deal; no `will-change`. Reduced motion turns it off.
- Headless at real speed, on a 31-card stack: delays 0, 18, 36, ... up to 288ms, the class gone at about 670ms, and no animation left anywhere. A filter change afterward deals nothing. Reduced motion: no animation.
