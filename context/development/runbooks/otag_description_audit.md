# Runbook: auditing oracle-tag descriptions (second-pass QA)

**Goal:** independently re-check the already-authored `ORACLE_TAG_DESCRIPTIONS`
against the real cards that carry each tag, and surface inaccuracies for a human to
review. This is the QA counterpart to
[`otag_description_authoring.md`](otag_description_authoring.md): authoring *writes*
descriptions, this audit *checks* them. Ships
[`otag_audit_workflow.js`](otag_audit_workflow.js).

> **Findings only, human-in-the-loop.** The audit never edits the const. It reports
> `wrong` / `suspect` flags with a card example and a suggested fix; a person decides
> what to apply. `ORACLE_TAG_DESCRIPTIONS` is only changed by a separate, deliberate
> commit.

> **Opt-in:** uses the `Workflow` tool, which the user must explicitly authorize.
> Don't launch unprompted.

---

## Method

Two stages, both grounded in real card data pulled live from the local Postgres
catalog (`scryfall_data` + `card_oracle_tags`):

1. **Audit** — an opus agent per chunk pulls up to 12 cards carrying each tag and
   adversarially tries to break the current description (over/under-specification,
   sibling confusion like gives-vs-gains, slug-name traps, style violations).
2. **Verify** — every `wrong`/`suspect` flag from stage 1 goes to a *second*,
   independent opus agent that re-pulls the cards and is prompted to **overturn the
   flag by default**: a flag survives only if the card data clearly proves it, and
   the verifier writes its *own* final suggestion rather than trusting the auditor's.

Output: `{ total, wrong, suspect, overturned, audited }`. `wrong`/`suspect` are the
*verified-surviving* flags; `overturned` lists flags the verify stage rejected (so you
can see what got filtered); `audited` is every slug that got an audit verdict, used for
resume tracking (see [`otag_audited_slugs.txt`](otag_audited_slugs.txt)).

Running notes and the full flag list live in
[`otag_audit_progress.md`](otag_audit_progress.md).

---

## Why the script was improved (the false-flag incident)

**What happened.** The first audit passes checked the top ~2,000 tags (~82% clean).
Of the 23 descriptions flagged **wrong**, a later independent Scryfall spot-check found
**3 of the flags were themselves wrong**, plus a couple of mis-cited card examples. The
description was fine; the *auditor* was mistaken.

**Root cause.** The flags didn't fail randomly. Every bad one was a claim about a
card's **mana cost, color, hybrid-ness, rarity, or whether it has `{X}`** — and the
grounding query only fed the agent `name`, `type_line`, and `oracle_text`. It was never
shown cost/colors/rarity, so it **guessed**, and guessed wrong.

The three false flags (keep these as canaries):

| slug | the bad flag | the real fact it couldn't see |
|------|--------------|-------------------------------|
| `x-doesn-t-matter` | "most tagged cards have no {X}" | Prismatic Ending is `{X}{W}` — it has X |
| `cycle-war-hybrid-planeswalker` | "not hybrid; requires both colors" | Kiora is `{2}{G/U}` — `{G/U}` is a hybrid symbol |
| `cycle-fdn-draft-signpost` | "several members are mono-colored" | Good-Fortune Unicorn is `{1}{G}{W}` — gold, not mono |

The reassuring part: the errors *clustered* on a single withheld input. Fix the input
and the whole class disappears.

**The fixes** (all in [`otag_audit_workflow.js`](otag_audit_workflow.js); #1 also
applied to [`otag_authoring_workflow.js`](otag_authoring_workflow.js), which had the
same blind spot):

1. **Card data in grounding.** The pull now includes `cost` (mana cost, e.g.
   `{2}{G/U}`), `colors` (array; `[]` = colorless), `rarity`, and `mv` (mana value).
   The agent can no longer guess at the fields it kept getting wrong.
2. **Verify stage.** No flag (and no suggested change) is trusted on a single agent's
   say-so. A skeptic re-checks it, biased to overturn. A suggested fix is itself a
   change, so it gets verified too.
3. **Population context.** Sample raised 8 → 12, and the prompt uses `pop` (the true
   number of cards with the tag) to forbid "the whole cycle is uniform" claims unless
   the sample actually supports them. Most false flags on `cycle-*` tags were
   over-generalizations from a partial view.
4. **Evidence discipline.** A shared `EVIDENCE_RULES` block: never assert
   cost/color/rarity/mv that isn't in the pulled data (`{X/Y}` = hybrid, `colors []` =
   colorless), and quote the real cost/oracle text verbatim as evidence.

**Design principle.** Fan-out scales the *work*; grounding + verification keep the
*quality*. Give the agent the ground truth it needs so it never guesses, and
independently verify anything that would drive an action. This is why the false flags
were caught in review rather than shipped, and why they shouldn't recur.

---

## How to run

> **➡️ ACTIVE TASK: the full 0-all RE-AUDIT.** Resume at rank ~501+, tracked by
> [`otag_reaudit_slugs.txt`](otag_reaudit_slugs.txt) with findings in
> [`otag_reaudit_progress.md`](otag_reaudit_progress.md). Use that tracker below (not the
> forward `otag_audited_slugs.txt`, which is paused at 2200). The mechanics are identical;
> only the tracker file differs.

```
Workflow({ scriptPath: "context/development/runbooks/otag_audit_workflow.js",
           args: { pairs: [{ slug, description }, ...] } })
```

Pick the next batch by population, excluding already-audited slugs (for the active re-audit,
exclude those in `otag_reaudit_slugs.txt`):

```bash
export DATABASE_URL="$(grep '^DATABASE_URL=' zerver/.env | cut -d= -f2-)"
F=zerver/src/lib/outbound/sqlx/card/helpers/oracle_tag_descriptions.rs
# pull (slug, current description) pairs for the top-N-by-population NOT already in
# otag_audited_slugs.txt, pass them as args.pairs, then append the returned `audited`
# list to otag_audited_slugs.txt so the next run resumes past them.
```

Resume is self-healing: `otag_audited_slugs.txt` lists only confirmed-audited slugs, so
a partial run (session limit mid-batch) re-queues whatever didn't finish, no gaps or
dupes. Shard large batches into ~250-slug workflows run concurrently.

---

## Regression watch (test the auditor, not just the descriptions)

The auditor is an LLM; it can regress. Watch for the same failure returning:

- **Canary re-run.** Periodically audit the three false-flag slugs above
  (`x-doesn-t-matter`, `cycle-war-hybrid-planeswalker`, `cycle-fdn-draft-signpost`).
  With the current script they should come back **accurate** (or, if flagged, the
  verify stage should **overturn** them). If any is flagged *wrong and upheld*, the
  grounding or evidence rules have regressed — check that `cost`/`colors`/`rarity`/`mv`
  are still in the `groundingCmd` SELECT.
- **Flag-shape smell test.** If a batch shows a spike of flags asserting a card's
  cost, color, hybrid-ness, or rarity, suspect the grounding data first (a schema
  change renaming `mana_cost`/`colors`/`cmc`, or the fields getting dropped from the
  query).
- **Overturn rate.** The `overturned` list is a live quality signal. A healthy run
  overturns a modest slice of flags. Near-zero overturns can mean the verifier is
  rubber-stamping (not re-pulling cards); a very high overturn rate means the auditor
  is over-flagging (too adversarial) and the first-pass prompt needs tightening.
- **Never auto-apply.** Even verified flags are suggestions. Apply to
  `ORACLE_TAG_DESCRIPTIONS` deliberately, in a separate `feat(otags):` / `fix(otags):`
  commit, and skip any flag whose suggested rewrite you don't agree with.
