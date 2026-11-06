# The factory

How a batch of small fixes runs across several machines at once. One composer session plans, reviews and merges; worker sessions build. Every machine compiles and tests on its own.

## Machines

The herdr agents are named after their machines: `composer` and `laptop` on the laptop, `halo` (work laptop), `thinker` (home ThinkPad) and `scotland` (home server). Address them by name, with `herdr --machine <machine>` for the remote ones.

## Branches

- `release` collects approved work. Nothing deploys from it; `deploy-*.yml` only runs on `main`.
- Each worker branches from `origin/release` as `nits/<machine>-<items>`, for example `nits/halo-13-6-3`.
- Scotty moves `release` into `main` to put it live. Nobody else touches `main`.

## The loop

1. The composer picks the next items from [`../progress/web_nits.md`](../progress/web_nits.md), writes a plan per worker, and marks them in progress there with the machine and branch.
2. Batches are split by file, so two workers rarely edit the same rules at once. A batch that crosses clients (app, zite, components) goes to one worker alone.
3. The worker builds only what the plan says. A plan that turns out wrong comes back as a question, not an improvisation.
4. Before pushing, the worker runs `cargo +nightly fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, the tests for the crates it touched, and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --lib`.
5. The worker pushes and opens a pull request into `release` (`gh pr create --base release`), one commit per item, then replies with the PR link and anything it was unsure of.
6. CI (`test.yml`) runs on the PR. The composer reviews the diff against [`scadoshi-rust`](../CLAUDE.md#linting) and this repo's rules, asks for changes on the PR or merges it.
7. After a merge the composer marks the items done in the nits file, and any worker still open rebases on the new `release`.

## Rules for workers

- Work in the machine's normal `~/Developer/zwipe` checkout, never a worktree, and leave the tree clean when done.
- Commits follow [`commit_guidelines.md`](commit_guidelines.md) and the commit rules in `CLAUDE.md`: no Co-Authored-By, "Generated with" or session lines.
- Web-only CSS lives in `zwiper-web/assets/web.css`, inside `@media (min-width: 900px)` for the wide layout and `@media (hover: hover)` for anything a mouse triggers. `zwiper/assets/main.css` changes only when the plan says so.
- Visual changes say in the reply what was checked by eye, at what width, or that nothing was.
