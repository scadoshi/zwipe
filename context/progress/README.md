# Progress — Where We Are

The project's running state, split across three files. Each answers a different
question; keeping them distinct is what stops this area from rotting into one
giant changelog.

| File | Question it answers | Voice |
|------|---------------------|-------|
| [`overview.md`](overview.md) | **Where are we?** What's shipped, what's live, what's in flight right now. | Narrative, past/present tense |
| [`todo.md`](todo.md) | **What's next?** Open, actionable items only. | Imperative, unchecked `[ ]` |
| [`backlog.md`](backlog.md) | **What might we do someday?** Ideas not yet committed to. | Loose, unscheduled |
| [`feature_requests.md`](feature_requests.md) | **What are users asking for?** Raw, sourced intake to weight before promoting. | Tabular, Impact × Effort |

## The boundary

- **overview** is the story of the project — read it to catch up. When something
  ships, its outcome lands here as a line or two (the detail lives in git).
- **todo** is the live worklist. An item leaves `todo.md` when it's done — it does
  **not** get checked off and kept. Completed work is summarized in `overview.md`
  instead, so `todo.md` always reads as "what's left," never as a history log.
- **backlog** is the holding pen. Promote an item to `todo.md` when it becomes
  next-up; until then it stays here.

If you find yourself scrolling past a wall of finished `[x]` items to reach the
live ones, `todo.md` has drifted — flush the done items into `overview.md` and
trim it back.

> Security notes used to live here (`security.md`). They're parked in
> [`../archive/security.md`](../archive/security.md) until a dedicated security
> review brings them back.
