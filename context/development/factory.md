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
