# Context docs wording pass

**Status: PLANNED 2026-09-21. Not scheduled; run it as a focused pass when
there's a quiet stretch, or chip away per-file whenever a doc gets touched
for another reason (that rule starts now).**

**One sentence:** sweep the prose docs for AI-tell wording (the humanizer
rules) so the owner can actually read them, file by file, without changing
any facts.

## Scope

- `context/**/*.md` and `README.md`. Prose only.
- Code comments are OUT. They have their own standing rules (resolve
  confusion, no narration, no em dashes) and get fixed when the code
  around them is touched, not in a docs sweep.
- Facts never change in this pass. If a sentence looks wrong as well as
  robotic, that's a drift item like `structure_md_refresh.md`, not a
  wording edit; flag it, don't silently "fix" it.

## What to kill (the short list from the humanizer rules)

- Em dashes and spaced-hyphen splices in new text; don't mass-convert old
  ones mid-sentence unless the sentence is being rewritten anyway.
- Significance inflation ("key milestone", "comprehensive", "robust",
  "seamless", "leverage") and headers that promise more than the section
  holds.
- `**Label**: explanation` bullet walls where prose would read faster.
- Rule-of-three padding and negative parallelism ("not just X, it's Y").
- Uniform paragraph rhythm: identical-weight 2-3 sentence blocks.

## Order (read-frequency first)

1. `context/README.md`, `README.md`, `context/CLAUDE.md`
2. `context/architecture/` (structure.md lands separately via its own plan)
3. `context/progress/` (overview.md especially; it's the longest and the
   most-read)
4. `context/operations/`, `context/development/`, `context/product/`
5. `context/plans/` active files last; archived plans are historical
   record and stay as written.

## Working rules

- One commit per file or small related batch, so a bad edit is easy to
  revert without losing the rest.
- Owner spot-reads a file or two from each batch; if the voice is off,
  recalibrate before continuing rather than sweeping on.

## Verification

Diff review only: `git diff --word-diff` per file, confirming no factual
token changed (names, numbers, paths, dates, commands). No build impact.
