# 383. Three small snaps: the nav, a label's clear and the Draw odds bars

State: fixing | Owner: thinker | PR: #607, follow-up #TBD | Build seen: production, web and app

## Report
Composer for Scotty, 2026-11-12, from surfer's ease audit (`~/ease-audit/README.md`, rows 8 to 10):
- the nav bar shifts in one frame arriving at or leaving Home;
- picking a chip in Create deck (Power level, Land target, Other tags) or in a Filter section brings a Clear ×, and the centered label jumps (19px, 38px in the Filter);
- Draw odds turn - and + jump every bar to its new length in one frame.

Shared tokens, reduced motion off.

Follow-up, composer from surfer's live sweep, 2026-11-12: in the web deck page's Filter > Types, picking a type brings an Any chip with the ×, and "Basic types include" jumps 21px in one frame.

## Cause
- The nav's right group, which sets where the links sit, gained or lost the `?` hint button between screens, 42px at 1422, and the back box came and went on the left.
- A label row centers its whole group, and the × joined it, so the label moved half the ×'s width.
- A label whose mode chip comes with a pick (Basic types include, Oracle tags include and the other `label_when_set` rows) added the chip to its centered group, so the label moved half the chip's width and gap. A × clear also remounted the row, so it jumped back.
- Each step remounted the Draw odds rows (their key held the percent), so the bars appeared at their new length.

## Fix
thinker:
- Screens without a back or a hint hold those buttons' slots, hidden (`nav-back-held`, `nav-hint-held`), so the bar keeps one layout.
- In a `.label-row` the × has a fixed width and a negative right margin that gives back its width, margin and the row's gap, so it hangs past the group without moving it. The rule is in components' `filter.css`, so the app, app.zwipe.net and the filter sheet get it.
- Follow-up: zwipe-ui's `Row::Label` carries `pick_chip`, the chip that comes with a pick, and components keeps it in the row folded (`label-chip-folded`: no width, padding or side borders, a margin that gives back the gap) while nothing is picked. `.label-chip` eases its opening and folding over `--enter-duration` with `--ease-enter`, off under reduced motion. Label rows keep their key through a clear so the fold eases there too. Headless Chrome at 1422: the label moves 21.5px over 12 to 13 frames on pick, unpick and ×, and sits where it did before.
- The Draw odds bars restyle in place after a step, and `.chart-fill-step` eases their width over `--enter-duration` with `--ease-enter`, with no transition under reduced motion.

## Verification
-
