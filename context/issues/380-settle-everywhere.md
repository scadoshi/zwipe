# 380. Lists and groups snap when a control reorders, adds or removes them

State: fixing | Owner: laptop | PR: - | Build seen: audit builds 9680dcacb (app) and bab3fbcd9 (zwipe.net)

## Report
The ease audit (`~/ease-audit/README.md`, 2026-11-12) found moves that happen in one frame. Part A, via the composer: one shared settle, from #578's script, for every reorder, insert and remove that snaps:
- the two-column List fold (later groups jump columns);
- Show Tokens and Show Command zone;
- To maybeboard, Undo, Star and the + / - warning;
- the sort and filter reorders on the Decks list, Playstyle's chips and zwipe.net Guides.

The settle uses the same tokens (`--ease-enter`, 250ms), does nothing under reduced motion, and leaves nothing jumping after it settles.

## Fix
laptop, web and zwipe.net: `components::settle` replaces the deck page's own settle script, installed once at the web and zwipe.net roots.
- **What it moves.** A container marked `data-settle-scope` holds things named by `data-row`, `data-fold` or `data-settle-key`. A click inside `data-settle`, on a button in a scope or a toast, or on a fold's header starts a 900ms watch, as do typing in a `data-settle` box and the deck page's keys.
- **When it measures.** During the watch, every frame and every DOM change compare each thing's layout with where it last was. A DOM change is caught by a mutation observer, which runs in the task that made the change, so moves start before the browser paints it (as in #586).
- **How it moves.** A jump over 40px slides from where the thing showed. Each jump adds its own move (`composite: 'add'`), so a group that jumps columns while a fold eases carries on smoothly instead of restarting. New things fade in, and things that left fade out as a copy.
- **Rows without a box.** A `display: contents` row (Playstyle's and Activity's card rows) is measured and moved by its first child.
- The dock's Undo is a trigger.

## Checked (headless Chrome, 1422, at real speed)
A per-frame probe recorded each keyed thing's on-screen box (transforms included). It counts a snap when a thing that moved over 60px did so in fewer than 3 frames, the audit's rule. Settle on, then off (reduced motion) as the before:

| Audit row | Snaps with settle | Without | Frames in motion |
|---|---|---|---|
| Two-column fold, Creatures shut and open | 0 | 10 | 15 to 16 |
| Show Tokens on and off | 0 | 34 | 12 |
| Show Command zone off and on | 0 | 32 | 11 |
| Star and Unstar (head card) | 0 | 1 | 9 to 11 |
| + and - with a warning | 0 | 1 | 8 to 9 |
| To maybeboard and Undo | 0 | 26 | 9 to 10 |
| Decks list sort, direction and pip | 0 | 2 to 7 | 10 to 13 |
| Playstyle Type chips | 0 | 1 | 11 to 12 |
| zwipe.net Guides tags | 0 | 4 | 12 |

The probe is `target/380-probe.js` on the laptop.

## Verification
-
