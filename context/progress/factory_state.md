# Factory state

A handoff for the composer session: where the factory stands, so a fresh or compacted composer can pick it up. The rules are in [`../development/factory.md`](../development/factory.md); the nit list and board are in [`web_nits.md`](web_nits.md). Update this file when the queue changes shape.

## As of 2026-11-06, evening

- `main` is at `b0d713965` (deployed: web, zite, zerver, all green). `release` is 37 commits ahead with everything merged since.
- Scotty's call for this round: when every queued item below has merged into `release`, push `release` to `main` (`git push origin origin/release:main`, a fast-forward; never switch the laptop's shared checkout) and watch the four deploys. Then he tests.

## Workers and their queues

Herdr agents by name. Remote ones need `herdr --machine <m>`. Each queue item is already sent to the worker as a "NEXT PLAN, start after X" prompt, so they run on their own.

| Machine | Working on | Then |
|---|---|---|
| halo | 93 to 95, the docked details panel: load jump, card pinned to center, panel default by space, no lift (`nits/halo-93`) | nothing queued |
| thinker | 98, 99, chip row gaps, the row's host extras at the bottom (`nits/thinker-98`), including Activity's Skipped under | nothing queued |
| scotland | 97, the swipe screens' footer legroom: docks stop above the footer (`nits/scotland-97`) | 100 and 102, the cover-flow's edge fade and arrow buttons (`nits/scotland-100`; reuse laptop's ARROW_* icons from 103) |
| laptop | 103, rounded SVG arrows in the swipe dock (`nits/laptop-103`) | 104, overscroll shows footer/nav color (`nits/laptop-104`; edits the 90 first-paint CSS in zwiper-web/index.html and its drift test) |
| surfer (halo, `wS:p1`) | a read-only QA pass of both sites, if its Chrome extension connects; it has been blocked twice. Low stakes. | report at `~/Developer/surfer-report.md` on halo |

## How the composer runs the loop

- Plans and the shared preamble (`rules2.md`) live in the composer's job folder, `~/.claude/jobs/dfa7e2c1/tmp/plans/`. The composer's own clone of zwipe (for board commits and conflict merges) is `~/.claude/jobs/dfa7e2c1/tmp/composer`.
- Review each PR against scadoshi-rust and the repo rules, then merge into `release` once CI is green (`gh pr merge <n> --merge`). Workers never merge.
- The common conflict is `zwipe-core/src/content/changelog/mod.rs` (two PRs adding 1.18.1 entries). Resolve by merging `origin/release` into the PR branch in the composer clone and keeping both entries: `~/.claude/jobs/dfa7e2c1/tmp/resolve_changelog.py` does it. Then push the branch (a normal merge commit, never a force-push) and merge.
- Board edits (`web_nits.md`) go on `release` from the composer clone.
- Merging cairn or portfolio PRs (they deploy from `main`) needs Scotty's explicit word each time.

## Open questions for Scotty

None pending. Recent decisions: no URL redirects for the renamed routes (92); keep two columns on Decks; dragging modals is fine; guides stay on zwipe.net (todo logged).

## Next Surfer passes (Scotty, 2026-11-06)

Once this round lands, Surfer reruns everything it has checked so far at about half window width, then as narrow as it can go, to cover the phone layout from a laptop. On the first pass, Chrome on halo wouldn't resize below about 1420px, so Scotty is setting up the window, or Surfer uses DevTools device emulation.
