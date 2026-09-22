# Context docs wording pass

**Status: LARGELY DONE 2026-09-22, and the scope turned out much smaller
than this plan assumed. The per-file rule below still stands for anything
touched from here.**

What the measurement actually showed, so nobody re-runs it:

- The em-dash count is noise. Most are column separators in ASCII trees and
  tables, not prose punctuation, and the standing rule is not to convert old
  ones anyway.
- The `**Label**: explanation` count overstated the problem badly. Of
  `context/README.md`'s 73, nearly all are dated progress-log entries or
  status tables, which is the tabular case the humanizer rules allow.
  Rewriting them would churn a large historical record for no reading gain.
- Openers and closers, which is where the tell usually lives, were already
  specific and dated almost everywhere.

The real finds were two always-loaded files whose problem was accuracy, not
voice:

- `development/newtypes.md` was a 367-line generic hexagonal-architecture
  tutorial that taught `UserId`/`DeckId` wrappers the codebase deliberately
  rejects, and named ports that were never built. Rewritten to 124 lines
  around the newtypes that exist.
- `CLAUDE.md` claimed 35k cards (prod has 118,680), was missing two crates,
  and still showed `outbound/client/` in zwiper after the extraction moved
  it out.

Also done: the status tables in `context/README.md` (the web table listed 9
pages against zite's 14 routes), `progress/backlog.md` (five content-free
stubs separated from the real entries), and `progress/todo.md`.

Left, and probably fine as they are: `cloudflare.md`, `services.md` and
`feature_requests.md`. Their labeled bullets are genuine key-value config
and numbered request IDs.

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

1. `context/README.md` (orientation plus the progress log, so the longest and
   the most-read), root `README.md`, `context/CLAUDE.md`
2. `context/architecture/` (structure.md lands separately via its own plan)
3. `context/README.md` (it absorbed the progress log 2026-09-21: longest
   and most-read), then the rest of `context/progress/` (todo.md,
   backlog.md, feature_requests.md)
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
