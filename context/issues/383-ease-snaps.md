# 383. Three small snaps: the nav, a label's clear and the Draw odds bars

State: fixing | Owner: thinker | PR: #607 | Build seen: production, web and app

## Report
Composer for Scotty, 2026-11-12, from surfer's ease audit (`~/ease-audit/README.md`, rows 8 to 10):
- the nav bar shifts in one frame arriving at or leaving Home;
- picking a chip in Create deck (Power level, Land target, Other tags) or in a Filter section brings a Clear ×, and the centered label jumps (19px, 38px in the Filter);
- Draw odds turn - and + jump every bar to its new length in one frame.

Shared tokens, reduced motion off.

## Cause
- The nav's right group, which sets where the links sit, gained or lost the `?` hint button between screens, 42px at 1422, and the back box came and went on the left.
- A label row centers its whole group, and the × joined it, so the label moved half the ×'s width.
- Each step remounted the Draw odds rows (their key held the percent), so the bars appeared at their new length.

## Fix
thinker:
- Screens without a back or a hint hold those buttons' slots, hidden (`nav-back-held`, `nav-hint-held`), so the bar keeps one layout.
- In a `.label-row` the × has a fixed width and a negative right margin that gives back its width, margin and the row's gap, so it hangs past the group without moving it. The rule is in components' `filter.css`, so the app, app.zwipe.net and the filter sheet get it.
- The Draw odds bars restyle in place after a step, and `.chart-fill-step` eases their width over `--enter-duration` with `--ease-enter`, with no transition under reduced motion.

## Verification
-
