# The factory

How issues get fixed across several machines: logged first, reproduced, fixed with a failing test, merged, deployed on Scotty's word, then verified live before they close. One composer session runs it; worker sessions build; Surfer and Droid check.

## Machines

The herdr agents are named after their machines: `composer` and `laptop` on the laptop, `halo` (work laptop), `thinker` (home ThinkPad) and `scotland` (home server). Address them by name, with `herdr --machine <machine>` for the remote ones.

## Branches

- `release` collects approved work. Nothing deploys from it; `deploy-*.yml` only runs on `main`.
- Each worker branches from `origin/release` as `nits/<machine>-<items>`, for example `nits/halo-13-6-3`.
- Scotty moves `release` into `main` to put it live. Nobody else touches `main`.

## Issues

Every bug, nit and small feature is a file in [`../issues/`](../issues/README.md) and moves through fixed states: reported, approved, reproduced, fixing, merged, verified, closed. Nothing is fixed before it's logged, and nothing counts as done until QA has seen it work on the live site.

## Limits

- At most two workers have an issue at a time. The rest stand by; idle costs nothing.
- One issue per branch and PR, unless two touch the same lines.
- Polish (accessibility beyond Tab reachability, labels and alt text; tiny visual tweaks) gets a "skip?" from the composer before anyone builds it.
- Scotty decides one question at a time, in plain words.

## The loop

1. **Report.** Scotty, Surfer, Droid or a worker sees something. The composer writes `NNN-slug.md` in `reported` before anything else happens, with what, where and which build.
2. **Approve.** The composer asks Scotty, or approves a clear bug inside scope he already gave. Skipped issues close with his reason.
3. **Reproduce.** A worker or QA writes exact steps against a named build. No repro, no fix: the issue waits for more detail.
4. **Fix.** The worker branches from `origin/release` as `nits/<machine>-<NNN>`, writes a test that fails first, fixes, runs the gates (below) and opens a PR into `release`. Visual changes come with before and after screenshots.
5. **Merge.** The composer reviews against [`scadoshi-rust`](../CLAUDE.md#linting) and this repo's rules, merges on green CI, and moves the issue to `merged`.
6. **Deploy.** Only on Scotty's word: `git push origin origin/release:main`. Before asking, the composer writes `issues/verify-<sha>.md` listing every merged issue with who checks it where.
7. **Verify.** After the deploy, Surfer and Droid hard-refresh and work through the verify file. Pass moves an issue to `verified`, then `closed`. Fail sends it back to `reproduced` with what they saw. Anything new they find goes in as `reported`.

## The control: before, cause, after

"Looks good" is not a test. An agent that glanced at a change and a second agent that agreed have proved nothing. Every bug and every visible change carries a control: proof it was broken, a traced cause, and proof it now works by the same steps. Agents forget this, so every brief points here and every report follows it.

### 1. Before (QA or the reporter)

- Exact steps from a clean start: build hash, app or browser, width or device, account, deck, then each tap or key in order.
- Expected and actual, in words, with any error text or status code.
- How often: run it from a clean start a few times and give the count ("3 of 3", "1 of 5") and anything that changes it (first load only, phone web only, after Back).
- Save a clip or still as `before-NNN`. If the steps don't show the problem, say so. That is a finding, and the issue goes back for detail. No repro, no fix.

### 2. Cause (the worker)

- Follow the repro into the code until you can name the file and line or rule that produces it, with evidence: a failing assertion, a log line, the DOM or state you saw.
- The cause has to explain the conditions: why phone web and not the app, why 1 of 5, why only after a reload. If it doesn't, it isn't the cause yet.
- If you can't run the thing that fails (device, browser, data), say so. A guess is a proposal, not a fix.
- If the same kind of bug keeps coming back, stop patching and tell the composer. The design may be wrong.

### 3. Fix (the worker)

- Write the test that fails first and watch it fail for the stated reason. Then fix. Never edit or weaken that test to get green.
- Negative control: put the old code back and see the test go red again.
- Change the cause, not the symptom. No swallowed errors, no retries or timeouts added to make it pass, no deleted checks, unless the report says why.
- Smallest diff that fixes it. No unrelated edits.

### 4. Report (the worker)

End every report with:

- **Cause:** file, line, why.
- **Changed:** files.
- **Checked:** exact commands and their exit status (run with `set -o pipefail`; never judge a gate through `| tail` or by grepping for "failed").
- **Not checked:** what you couldn't run (a real phone, iOS Safari, signed in). Mark it unknown, don't imply it passed.
- **Tester steps:** the before steps, and what they should show now.

Never write "fixed" for something you didn't rerun.

### 5. After (QA)

- Check the deployed build is the one that holds the fix (version.json, the commit, the app's build) before testing.
- Run exactly the before steps, from a clean start, the same number of times. Save `after-NNN`. Same steps, opposite result, or it isn't fixed.
- Look at the screen, not only the DOM: it animates, lines up, isn't covered or clipped. Pair a screenshot with what you checked in the DOM or network when it matters.
- Check both places it lives: the app and phone web, wide and 390, owner and shared.
- QA observes and reports. It doesn't fix, merge or deploy.

### 6. Break it (QA)

Then try to break the fix your own way and report everything you find, asked for or not: other widths and devices, Back and gestures, rotate, reload, background and return, fast or repeated input, focus in a field, a sheet or drawer open, slow network, odd orders, long lists, empty states.

### 7. Close the loop (the composer)

- Nothing closes on a worker's say-so. It closes when QA's after passes, or when Scotty says so.
- Every bug QA or Scotty catches after a "pass" becomes a regression test, or a line in QA's standing list, so the factory catches it next time.
- After two failed rounds on the same issue, stop and rewrite the brief instead of sending another fix.
- Design, scope and data questions go to Scotty, not to a guess.

## Gates

Before pushing, the worker runs `cargo +nightly fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, the tests for the crates it touched, and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` (not `--lib`: CI documents the zite and zwiper binaries too). Wire changes follow [`api_evolution.md`](../architecture/api_evolution.md) and go through the composer first.

## Roles

- **composer:** logs, asks Scotty, assigns, reviews, merges, writes the verify file, never deploys without his word.
- **workers** (laptop, halo, thinker, scotland): reproduce, fix, test, report back to the composer on done, blocked or question.
- **Surfer:** the three websites in Chrome, wide and 390.
- **Droid:** the Android app on the emulator, and phone-width web in emulator Chrome.

## Rules for workers

- Work in the machine's normal `~/Developer/zwipe` checkout, never a worktree, and leave the tree clean when done. Create the plan's branch before the first edit and check `git branch --show-current` before every commit: never commit on `main` or `release`. On the laptop the checkout is shared with Scotty, so a stray commit on `main` blocks his push.
- Commits follow [`commit_guidelines.md`](commit_guidelines.md) and the commit rules in `CLAUDE.md`: no Co-Authored-By, "Generated with" or session lines.
- Web-only CSS lives in `zwiper-web/assets/web.css`, inside `@media (min-width: 900px)` for the wide layout and `@media (hover: hover)` for anything a mouse triggers. `zwiper/assets/main.css` changes only when the plan says so.
- Visual changes say in the reply what was checked by eye, at what width, or that nothing was.
