# Factory state

The handoff for the composer session: where the factory stands, so a fresh or compacted composer can pick it up. The rules are in [`../development/factory.md`](../development/factory.md). Open issues are in [`../issues/`](../issues/README.md); nits 1 to 277 are archived in [`web_nits.md`](web_nits.md). Update this file whenever the queue changes shape.

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

## Overnight, 2026-11-06 into 11-07 (Scotty asleep: "keep the factory running")

- Deployed `release` to `main` at `60d705459` (web, zerver and portfolio green; zite was finishing). Droid is on pass 2 against that build, and Surfer on Part 7, a fix check at whatever width its window has.
- The composer keeps reviewing and merging worker PRs into `release`, copies each QA report into `context/qa/` with a row in its README, and commits. No more deploys to `main` until Scotty says.
- Scotty's later word (before sleep, 2026-11-07): "anything that gets reported back that you're like 90% confident you can fix, just fix it." So 157 to 160 went out from Surfer's and Droid's findings. Anything less certain waits for his morning triage.
- Overnight queues: halo has 156 (the auth logout, a PR left open for Scotty), then 159. Thinker has 135/136, then 158 (counts), 143, 153. Scotland has 146, then 157 and 160 (portfolio). Laptop has 151 (A: orangui, zwipe, Cairn on a branch), then 145.

## Overnight progress log (2026-11-07)

Merged into `release` after the `60d705459` deploy: #256 (134/139/148/149), #257 (123), #258 (154/155), #260 (146), #261 (151b, with orangui #10), #262 (159), #263 (135/136), #264 (145), #265 (122b), #266 (161, the browser-back crash), #267 (158B, honest Live labels), #268 (157, with orangui #11's per-theme ink colors), #269 (143 Phase 1, Themed), #270 (153, Swipe confirmations: Chip by default). Portfolio #9 (138) is live.
Scotty approved #259 (156, auth) on 2026-11-07 and it merged into release. Next: once laptop's 162 and scotland's 160 land, deploy release to main and start another Droid and Surfer audit pass. The Cairn branch `fix/app-lifecycle` and the portfolio branch `changelog/cairn-day-rollover` wait on his review. Thinker's 158 (A), lifetime totals that never drop, is parked in `context/plans/lifetime_totals.md`.
143 (#269), composer review: Phase 1 matches the `pip_color_wire.md` precedent. The wire still writes `theme`, both spellings are read, and a test proves PATCH, GET, login and refresh stay parseable by 1.15 to 1.18. Phase 4 sits on `server/pip-themed` with no PR. One deploy note: the migration narrows the CHECK to ('themed', 'classic'), so while the old zerver binary overlaps the migration it could write 'theme' and fail. That's the same brief window the `classic` migration had. Deploy at a quiet hour.
For Scotty's triage: the picker × discarding live picks; a toast over the filter's Oracle tags row (720); Droid P2-4 (theme lost after a force-stop); the Synergy selection highlight (p7-11).

## 2026-11-07, daytime (Scotty awake)

- Deployed `release` to `main` at `48e5f81b9` (zerver, web, zite green). Since then merged into `release`: #271 (162), #272 (163, with orangui #12), #273 (167), #274 (164), #275 (169, orangui #13), #276 (173), #277 and #278 (176 hints), #279 (165, orangui #14), #281 (175 Remove/Undo), #282 (177 chip under the header), #283 (172 one toast per action). Portfolio #10 (160) and #11 (166, 168) are live.
- Scotty's calls: login cap 5 to 10 (#280, test fix pending at halo); both QA agents keep sharing `test`; Cairn `fix/app-lifecycle` merged to Cairn main (`f777c35`) with its portfolio changelog; 158A option 1 approved (thinker, after the skeleton follow-up); the WebView crash fallback approved (laptop, spike first). The Dioxus bug is already dioxus#5885 (fix PRs #5886, #5890); a comment draft for Scotty is at `~/Developer/zwipe/target/dioxus-webview-terminate-issue.md`. Nothing filed.
- In flight: halo 280 test fix, 181, then 170, 171; scotland 178 (priority), 179, 174; thinker skeletons (D5 D6 D7 D9), then 158A; laptop the WebView fallback.
- Next deploy after this batch, with Scotty's go.
- Deployed `3e72d3344` (2026-11-07 midday). Since then merged into release: #293 (158A, retired lifetime counters, a migration), #294 #297 #299 #300 #302 #303 (the scadoshi-rust audit, plus orangui #17 and Cairn main), #295 (crash fallback on the native run loop, orangui #16), #296 (182), #298 (183), #301 (184), #304 (175 dock and toast wrap, orangui #18), #305 (175 swipe-down at the end). Cairn main has the crash fallback (#4), backup/import, the Android fixes and the audit fixes. The next deploy waits on Scotty's go.
- In flight: thinker 165 follow-up (card detail chips in light themes); scotland Droid's seven Cairn fixes. Parked for Scotty: the Quick add 'No results' pill.

## PR #259 (156, the auth logout): composer review notes for Scotty

Left open as agreed. The design is sound: a rotated token presented again within 30s gets the same successor (kept in memory per process), so racing tabs and a refresh lost to a page unload stop revoking the family. Past the window, a replay still revokes. The web adds a Web Lock so tabs refresh one at a time. The ownership check (Forbidden) runs before the reuse path. Points to decide or fix before merging:
- `reuse.rs` recovers a poisoned lock with `PoisonError::into_inner`, which the Rust rules forbid ("a poisoned lock is an error the caller sees"). It should return an error, which on the refresh path means falling back to the replay branch.
- The successor is remembered before the transaction commits. If the commit then fails, a racing loser is handed a token the database never stored, and its next refresh fails as NotFound. That's rare and fails safe (a logout), but `remember` after the commit, with losers re-reading, would be tighter.
- The tradeoff: anyone holding a stolen token can redeem it within 30s of a legitimate rotation. That's the standard reuse-interval tradeoff (Auth0 and others do the same). The window is configurable.
- The map is per process, so a restart or a second zerver instance forgets the window, and the race falls back to today's revoke. That's safe.
- Halo flagged without changing it: the web refreshes on every tab load because it stores no access token, and the 5-session cap may evict an idle Firefox session. Either could still explain Scotty's Firefox drops.

## Waiting on Scotty (as of 23:30)

- Go on Surfer's Part 6 follow-ups for halo: 125 (the portfolio sweep, still two), 118 regressed in the phone details drawer (720 and 504), 52 (picker chip list fade), 53 (zwipe.net stats stuck while labeled Live), 54 (the deploy logged Chrome out; diagnose only).
- A deploy of `release` to `main`. Nothing goes to Cairn's `main` and nothing is filed with Dioxus without him.
- Halo, Surfer and Droid are idle and waiting.

## Rules learned tonight

- Surfer's and Droid's findings get reviewed with Scotty before anything is built. Scotty's own nits I log and send straight out.
- A worker stuck on a permission prompt: press Esc, remove the cause (files go in the repo's `target/`), and re-prompt. Never approve a settings change.
- Workers stop and ask the composer before any wire change. The composer plans it per `context/development/api_evolution.md` (template `context/plans/pip_color_wire.md`, tracker `context/plans/wire_changes.md`) and checks it before it ships.
- Clean the laptop's Rust target folders once one passes about 100 GB, only while nothing is building, keeping `target/scotty-shots`.
- Portfolio copy stays short and plain (the human skill).
- Every agent reports back when it finishes or stops (`herdr --machine laptop agent prompt composer ...`). A `/clear` drops a pane's herdr name, so rename it after (`herdr agent rename wV:p1 laptop`).

## Open threads

- Wire migrations waiting on a floor: `natural` to `classic` (pip color) and `/preferences` to `/settings` are in `wire_changes.md`. `theme` to `themed` joins them once 143's Phase 1 merges.
- Android status and nav bars don't follow the theme: a todo (D32).
- Cairn: anything 151 finds for Cairn goes to Scotty, not straight to Cairn's `main`.

## How the composer runs the loop

- Plans and the worker preamble (`rules2.md`) live in `~/.claude/jobs/dfa7e2c1/tmp/plans/`. The composer's zwipe clone, used for board commits and conflict merges, is `~/.claude/jobs/dfa7e2c1/tmp/composer`.
- Review each PR against scadoshi-rust and the repo rules, then merge into `release` once CI is green (`gh pr merge <n> -R scadoshi/zwipe --merge`). Workers never merge.
- Conflicts are usually `zwipe-core/src/content/changelog/mod.rs`. Merge `origin/release` into the branch in the composer clone, run `~/.claude/jobs/dfa7e2c1/tmp/resolve_changelog.py` (it keeps both entries), and push. Never force-push. For a `Cargo.lock` conflict, take release's copy and `cargo update -p` the changed git dependencies.
- After an orangui merge, the zwipe and portfolio PRs repin orangui to `main` before they merge.

### Evening 2026-11-07
- Deployed f699acc37 (Scotty's go). Since then merged to release, not deployed: #323 #324 #326 (196, 198, 199), #329 #335 #337 #342 (197, 208, 203, 209), #330 #331 #333 (202, 204, 205, 206), #332 (207 sideboard), #341 #345 (200, 201, promo), #339 #346 (210), #340 (211 parked), QA reports #322 #325 #327 #328 #334 #338 #343 #344.
- Held: #336 (207 og tags via a zwipe.net move to Cloudflare Pages), waiting on Scotty's OK and his dashboard cutover (steps in deploy_web.md).
- Waiting on Scotty: triage of Droid p7 + p8 and Surfer p15, p16, p17 findings; reqwest gzip/brotli on the app; 186 (web preview 379 vs app 333); Cloudflare zone cache settings (stats_second_source.md, also 211).
- Running: Droid pass 9 (release APK, merged fixes incl. 210), Surfer Part 18 (copy proofread).

### Late 2026-11-07
- Deployed 80d5b43ab (Scotty's go). Merged since, not deployed: 213, 214, 215 (Any/All, additive wire field), 216 to 225, 203 and 206 follow-ups, web Quick add (#370), Config flags as filters (#373), copy pass #369 #372 #374, orangui#21 + pin (#366).
- Only open PR: #336, held for Scotty's zwipe.net cutover (context/plans/share_preview.md).
- Closed: 212 (Arzakon is WUBRG), the app promo printing (didn't reproduce on prod), synergy_applied (correct by contract).
- Waiting on Scotty: deploy go; reqwest gzip/brotli on the app; Surfer p15-17 and p19 findings; Droid p11 (Activity 146 vs 303 swipes, Draw odds 'by turn 0').

## Night of 2026-11-07 into 11-08 (Scotty asleep)

- Live: `main` at 1e03520ef. zwipe.net is on Cloudflare Pages (`zwipe-site`), www redirects to the apex, GitHub Pages is gone (#416). No deploys until Scotty says.
- The factory now runs through `context/issues/` (see `../development/factory.md`): at most two workers busy, every issue logged first, fixes need a failing test and a live verification.
- Scotty's overnight word: Surfer and Droid work through `issues/verify-1e03520ef.md`. New findings get logged as `reported`. Only an obviously easy fix that can be verified next round goes to a worker; anything bigger stays logged.
- In flight: thinker on #398's conflict then 285 (preview image); halo #406 and scotland #408 conflicts. Laptop idle.
- ManaBox plan (`plans/collection.md`): Q1 answered (Add and Replace modes); Q2 to Q8 wait for Scotty one at a time.
- Parked for Scotty: 282 (stale bundle after deploy), 284 (toast vs footer, reproduced).

### Morning summary (2026-11-08)

- Verify 1e03520ef: every line checked. All pass except 265 (Partner Swipe stack, fixed in #419) and an extra "Stack refreshed" toast (288, fixed in #421). 336 (share previews) checked by curl.
- Merged since the deploy (not live): 221 web, 258, 270, 273, 278, 279, 280, 283, 287 (changelog split, workspace 1.18.8), 265, 288, 289, 290 part 1, #416 (GitHub Pages job removed).
- New, waiting on Scotty: 282 (stale bundle after deploy), 284 (toast vs footer, reproduced), 286 (portfolio orangui line anchors), 290 part 2 (what Keep means on the discard prompt), 291 (logout revokes every session).
- 285 (share preview image) merged in #424 and #425; samples sent to Scotty. Deploy note: zerver must deploy before zite (the zite build waits for it). Workers all idle.

## 2026-11-08 limit
Scotty raised the worker limit to 3 for now ("push it a little bit more"). Busy: thinker 285 part 2, halo 291, scotland 284.

## Queue (2026-11-08, Scotty)
1. Cairn clips (droid) -> Scotty review -> portfolio.
2. Guide screenshots, issue 310 (droid) -> contact sheet -> swap.
3. Store builds on halo (the halo worker; Scotty takes them there): IPA for Zwipe and Cairn (iOS), AAB for Zwipe (Play). Ask Scotty for the version at cut (changelog has 1.18.9-1.18.14 upcoming); date = cut day.
4. Then: marketing video factory discussion (narrated shorts).

## Scotty to-dos (when back)
- Cairn 1.4.0 What's New in App Store Connect still has 1.3.0's notes. Replace with cairn context/operations/store-submissions/1.4.0/whats_new.md (474 chars). It's submitted, so: App Review, remove from review, paste, Save, Add for Review.
- Swap the new store screenshots into App Store Connect and Play Console for Zwipe and Cairn (Droid's set, once approved).

## Next deploy
Optional, low priority: whenever a deploy next happens, a stale-tab check of 282 is nice to have. Not blocking; 282 already passed with a simulated deploy.
