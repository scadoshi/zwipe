# structure.md refresh

**Status: DONE 2026-09-21 (`dc43200e`).**

Every drift item below was fixed and verified against the tree: the Family
table, the missing `zwipe-components` section, the `metrics/` and `upkeep/`
domains, the zwiper and zite trees, and the theme count (31 everywhere,
including the stale "14 themes" doc comment in zwipe-components). CLAUDE.md's
graph matched at the same time, and was corrected again on 2026-09-22 once the
teardown deleted the `zwiper -> zerver` edge entirely.

**One sentence:** make structure.md true again, stamp every section with
whether it describes something real, removed, or hypothetical, and align
CLAUDE.md's dependency graph while in there.

## The drift (verified 2026-09-21)

- Family table says zwiper depends on zerver "feature-gated for ApiError
  only." Reality: 48 route imports, 6 ApiError, 2 Password. Understating
  this is what let the drift grow unaudited; the table should state the
  real edge until the teardown plan deletes it.
- zwipe-components ships in two app stores and has no section; zort, which
  is explicitly nothing, has one.
- The zerver tree omits the `metrics/` and `upkeep/` domains. metrics is
  the domain decisions.md credits with the erased-service pattern.
- The zwiper tree lists `domain/theme.rs` (gone; themes moved to
  zwipe-components), omits `config.rs`, and never shows zwipe-components
  as a dependency.
- "9-theme system." ALLOWED_THEMES has 31. (README already says 31; the
  zwipe-components lib.rs doc comment says 14 and gets fixed in the same
  pass, it's a one-word edit.)
- Tense: only the zort section says it's hypothetical. Every section gets
  a one-line stamp so a reader knows current vs removed vs sketch without
  cross-checking the code.
- zite: structure.md says static site, decisions.md 2026-04-06 commits it
  to the full deck builder. structure.md is right about today; its zite
  section gets a pointer to the decision so the two docs stop looking
  contradictory.
- CLAUDE.md's crate graph says "(ApiError only)" while its own prose below
  says ApiError and Password. Align both with reality; they shrink again
  as teardown phases land.

## Wording

While rewriting, apply the humanizer pass to the touched sections: this doc
is the hand-a-stranger orientation file and reads half-generated in places.
Untouched sections keep their text; the repo-wide wording sweep is its own
plan (`context_docs_wording_pass.md`).

## Verification

Read-through against `ls` and Cargo.toml, nothing else: this is a docs-only
change, no build impact. Grep the doc for the old numbers (9-theme, ApiError
only) to confirm nothing stale survives.
