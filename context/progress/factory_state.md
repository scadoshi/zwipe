# Factory state

The handoff for the composer session: where the factory stands, so a fresh or compacted composer can pick it up. The rules are in [`../development/factory.md`](../development/factory.md). Every nit, with its scope, is in [`web_nits.md`](web_nits.md). Update this file whenever the queue changes shape.

## As of 2026-11-06, late night

- `main` is at `811c3bbbe`, deployed and green (web, zite, zerver). `release` holds everything merged since. Scotty says when to push `release` to `main` (`git push origin origin/release:main`, a fast-forward; never switch the laptop's shared checkout). Then I watch the deploys and send Surfer to check the fixes.
- scottyfermo.com (the portfolio) and orangui deploy from their own `main`. Scotty has pre-approved merging green worker PRs into both.

## Agents

| Agent | Where | Role |
|---|---|---|
| composer | laptop `wZ:p1` | Triages, plans, reviews and merges. That's me. |
| laptop | laptop `wV:p1`, `~/Developer/zwipe` | Worker. Owns the app, the simulator and the portfolio. |
| halo | `herdr --machine halo`, worktree `~/Developer/zwipe-halo` only | Worker. Scotty builds to his phone from `~/Developer/zwipe` on halo, which stays on `main` and clean. |
| thinker | `herdr --machine thinker` | Worker. |
| scotland | `herdr --machine scotland` | Worker. |
| surfer | halo `wS:p1` | QA for the three websites in Chrome. Its report is `~/Developer/surfer-report.md` on halo, with shots in `~/Developer/surfer-shots`. Parts 1 to 6 so far. |
| droid | laptop `w0:p1` (unnamed in herdr; address it by pane) | QA for the Android app on the Pixel_9a emulator, against production as user `test`. It builds from its own clone, `~/Developer/droid/zwipe` on `origin/release`. Runbook: `~/Developer/droid/RUNBOOK.md`. Report: `~/Developer/droid/report.md`. |

## Queues (each item is already sent to its worker)

| Worker | Now | Then |
|---|---|---|
| laptop | 131 (deck page top gap, finishing PR #248 with 128 to 130) | **150 and 151 first**: the see-through sheet and long-running app degradation, an investigation. Diagnosis goes to me before any fix. Then 144 (the light/dark race; ask before any wire change), then 145 (keyboard, Retry, D3). |
| halo | 132 (toasts bottom left, stopping above the footer, app too), 133 (tag selectors: drawer or full screen), 137 (dock width) | 140 (44px tap targets, app and portfolio too), 142 (zwipe.net one column) |
| scotland | 122 (the unsaved-changes guard) | 134 and 139 (docks stop above the footer; footer at every width), with 148 and 149 on that branch; 138 (portfolio chart labels, Escape); 146 (the app spacing sweep, D38 included) |
| thinker | 123 (the skeleton audit, web and app) | 135 and 136 (fold carets everywhere; the Types chart); 141 (Activity heatmap); 143 (the `Theme` to `Themed` wire rename: Phase 1 in a PR, Phase 4 on `server/pip-themed` with no PR, a plan in `context/plans/pip_themed_wire.md`); 147 (copy and contrast, D28 and D37) |
| surfer | Part 6: checking 115 to 127 on the live deploy, at full width and then narrower as Scotty resizes | |
| droid | Idle after pass 1. It rechecks on Android once 145 to 147 merge. | |

## Rules learned tonight

- Surfer's and Droid's findings get reviewed with Scotty before anything is built. Scotty's own nits I log and send straight out.
- A worker stuck on a permission prompt: press Esc, remove the cause (files go in the repo's `target/`), and re-prompt. Never approve a settings change.
- Workers stop and ask the composer before any wire change. The composer plans it per `context/development/api_evolution.md` (template `context/plans/pip_color_wire.md`, tracker `context/plans/wire_changes.md`) and checks it before it ships.
- Clean the laptop's Rust target folders once one passes about 100 GB, only while nothing is building, keeping `target/scotty-shots`.
- Portfolio copy stays short and plain (the human skill).

## Open threads

- Wire migrations waiting on a floor: `natural` to `classic` (pip color) and `/preferences` to `/settings` are in `wire_changes.md`. `theme` to `themed` joins them once 143's Phase 1 merges.
- Android status and nav bars don't follow the theme: a todo (D32).
- Cairn: anything 151 finds for Cairn goes to Scotty, not straight to Cairn's `main`.

## How the composer runs the loop

- Plans and the worker preamble (`rules2.md`) live in `~/.claude/jobs/dfa7e2c1/tmp/plans/`. The composer's zwipe clone, used for board commits and conflict merges, is `~/.claude/jobs/dfa7e2c1/tmp/composer`.
- Review each PR against scadoshi-rust and the repo rules, then merge into `release` once CI is green (`gh pr merge <n> -R scadoshi/zwipe --merge`). Workers never merge.
- Conflicts are usually `zwipe-core/src/content/changelog/mod.rs`. Merge `origin/release` into the branch in the composer clone, run `~/.claude/jobs/dfa7e2c1/tmp/resolve_changelog.py` (it keeps both entries), and push. Never force-push. For a `Cargo.lock` conflict, take release's copy and `cargo update -p` the changed git dependencies.
- After an orangui merge, the zwipe and portfolio PRs repin orangui to `main` before they merge.
