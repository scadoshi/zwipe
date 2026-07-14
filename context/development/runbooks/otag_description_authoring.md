# Runbook: authoring oracle-tag descriptions (AI-orchestrated)

**Goal:** grow `ORACLE_TAG_DESCRIPTIONS` (our own plain-English text for Scryfall
oracle tags) in batches, **every description checked against the real cards that
carry the tag**, until coverage is satisfactory. Part 1 of
[`../../plans/otags/tag_descriptions_and_dictionary.md`](../../plans/otags/tag_descriptions_and_dictionary.md).

This is a **repeatable loop** a fresh AI can run cold. It fans out subagents to
draft, then adversarially verify against oracle text, then a human-in-the-loop
(you) validates and splices. It exists because there are ~4,500 tags and reading
each by hand doesn't scale; the verify stage is what keeps accuracy high at scale.

> **Opt-in required:** the fan-out uses the `Workflow` tool, which the user must
> explicitly authorize ("use a workflow" / "fan out agents" / "same fanout method").
> Don't launch it unprompted.

---

## Are these compared against real cards? Yes — two layers

1. **Verify stage (every tag):** each verifier agent pulls the actual `oracle_text`
   of up to 6 real cards that carry the tag and judges the drafted description
   against what those cards literally do. That produces the `accurate` / `minor` /
   `wrong` verdict and any correction. Nothing ships un-grounded.
2. **Human spot-check (sample):** after the workflow, hand-verify ~10 of the most
   obscure/mis-nameable tags per batch against oracle text (query in Step 3) before
   splicing. This is where slug-name traps get caught (e.g. `tapper-creature` is not
   a creature; `group-slug` is damage/drain, not a slowdown).

---

## Prerequisites

- **Local Postgres** with the synced catalog. Connection string:
  `export DATABASE_URL="$(grep '^DATABASE_URL=' zerver/.env | cut -d= -f2-)"`
- **Tables:** `oracle_tags(slug, label, description, parent_ids)`,
  `card_oracle_tags(oracle_id, oracle_tag, source)`,
  `scryfall_data(oracle_id, name, type_line, oracle_text, mana_cost, ...)`.
- **The const file:**
  `zerver/src/lib/outbound/sqlx/card/helpers/oracle_tag_descriptions.rs`
  (`ORACLE_TAG_DESCRIPTIONS: &[(&str, &str)]`). `zervice` overlays it into
  `oracle_tags.description` every sync (ours always wins). No DB write from this
  runbook, no migration, no `MIN_CLIENT_VERSION` bump — additive.
- **The workflow script:** [`otag_authoring_workflow.js`](otag_authoring_workflow.js)
  (sibling file). Edit its `ENV` constant to your absolute path to `zerver/.env`.
- A scratch dir for intermediate JSON (use the session scratchpad, not `/tmp`).

---

## The loop

### 1. Pick the next N unauthored slugs, highest card population first

Population = number of distinct cards carrying the tag. Highest-traffic tags first.

```bash
export DATABASE_URL="$(grep '^DATABASE_URL=' zerver/.env | cut -d= -f2-)"
F=zerver/src/lib/outbound/sqlx/card/helpers/oracle_tag_descriptions.rs
# authored slugs = quoted kebab-case strings in the const (descriptions have spaces/caps, excluded)
AUTHORED=$(grep -oE '"[^"]+"' "$F" | sed 's/"//g' | grep -E '^[a-z0-9-]+$' | sort -u | sed "s/.*/'&'/" | paste -sd, -)
psql "$DATABASE_URL" -t -A -o "$SCRATCH/next.json" -c "
SELECT json_agg(slug) FROM (
  SELECT ot.slug
  FROM oracle_tags ot
  JOIN (SELECT oracle_tag, COUNT(DISTINCT oracle_id) n FROM card_oracle_tags GROUP BY oracle_tag) cnt
    ON cnt.oracle_tag = ot.slug
  WHERE ot.slug NOT IN ($AUTHORED)
  ORDER BY cnt.n DESC
  LIMIT ${N}
) t;"
```

Read `next.json` to get the slug array. (Don't `echo` it through inline python with
`$AUTHORED` unquoted — the shell splits it and breaks the script. Read the file.)

### 2. Run the draft -> verify workflow

Launch [`otag_authoring_workflow.js`](otag_authoring_workflow.js) with the slugs as
`args`. It chunks the slugs (7/chunk) and pipelines each chunk: **sonnet drafts**
(reading oracle text), then **opus verifies** (re-reading oracle text, correcting).

```
Workflow({ scriptPath: "<path>/otag_authoring_workflow.js", args: { slugs: [...] } })
```

Returns `{ total, items: [{ slug, description, verdict, note }] }`. The `description`
is already the **final, verifier-corrected** text.

### 3. Parse, validate, spot-check

Parse `result.items`, then run the **style gate** and a **DB spot-check** of the
obscure ones. Style rules any description must pass:

- no `"` (double quote), no `\` (backslash) — would break the Rust string literal
- no em dash (`—`/`–`), no `[label](link)` syntax, no URL, no `&` (write "and")
- non-blank, unique slug, not already in the const

Spot-check obscure tags against oracle text (pick ~10 keyword-mechanic / archetype /
errata tags from the batch):

```bash
psql "$DATABASE_URL" -F $'\t' --no-align -P footer=off -c "
SELECT t.slug, x.name, LEFT(x.oracle_text,140) AS text
FROM (SELECT unnest(ARRAY['slug-a','slug-b',...]::text[]) AS slug) t
JOIN LATERAL (SELECT DISTINCT ON (sd.oracle_id) sd.name, sd.oracle_text
  FROM card_oracle_tags c JOIN scryfall_data sd ON sd.oracle_id=c.oracle_id
  WHERE c.oracle_tag=t.slug ORDER BY sd.oracle_id LIMIT 2) x ON true
ORDER BY t.slug;"
```

Fix any that don't match the cards (edit the item's `description` before Step 4).

### 4. Splice into the const

Append the batch before the const's closing `];`. Format each entry to
**rustfmt-canonical**: one line if `len(slug) + len(desc) + 13 <= 100`, else the
wrapped 3-line form. Example splicer (Python):

```python
lines = []
for i in items:
    s, d = i['slug'], i['description']
    lines.append(f'    ("{s}", "{d}"),' if len(s)+len(d)+13 <= 100
                 else f'    (\n        "{s}",\n        "{d}",\n    ),')
c = open(F).read()
start = c.index('= &['); close = c.index('\n];', start)   # the const's own closing ];
open(F, 'w').write(c[:close+1] + "\n".join(lines) + "\n" + c[close+1:])
```

Then re-validate the whole const: extract every `("slug", "desc")` (regex tolerant of
one-line and wrapped forms), assert slugs are unique and no description has a
blank / em dash / link / `&`.

### 5. Format, test, report, commit

```bash
cargo +nightly fmt -p zerver            # scope to -p zerver so you don't touch other in-flight work
set -a; source zerver/.env; set +a
cargo test -p zerver --lib oracle_tag_descriptions   # slugs_are_unique + descriptions_are_non_blank
cargo test -p zerver --test repo_oracle_tags         # sync/overlay integration
```

Hand the user a short **report**: counts (`accurate`/`minor`/`wrong`), the `wrong`
ones with card examples, and the full batch table as a file (they can't read 200
rows inline). **Commit only when the user asks** (project rule): the const + any test
fixture change, one-line message, no AI signatures, e.g.
`feat(otags): author N oracle-tag descriptions`.

Ship path from there: user pushes -> next `zervice` overlays all authored text.

---

## Description style rules (what the agents are told, keep in sync with the script)

- ONE short sentence, ideally under ~90 characters. Plain English, present tense,
  address the player as "you" where natural.
- Describe what a card with the tag DOES, functionally. If the slug NAME is
  misleading, **trust the cards, not the name.**
- Start with a verb ("Deals...", "Grants...", "Removal that...") or "A <noun> that...".
- No em dashes, no `[label](slug)` cross-links, no URLs, no `&` (write "and").
- Sibling precision: `gives-X` grants to OTHERS; `gains-X` / `-self` is about ITSELF;
  `-to-all` hits your whole team; `repeatable-X` can be done again and again;
  `typal-X` cares about creatures of type X; `synergy-X` / `hate-X` reward / punish X.
- Many tags are keyword MECHANICS (convoke, threshold, phasing, heroic, bushido,
  ninjutsu, imprint, strive) — define the mechanic plainly in one sentence.

---

## Gotchas / lessons learned

- **`Workflow` args arrive as a JSON string, not an object.** The script guards with
  `const A = typeof args === 'string' ? JSON.parse(args) : (args || {})`. Without it,
  `args.slugs` is undefined -> 0 chunks -> 0 agents (a 40ms no-op run).
- **Test fixture collision.** `repo_oracle_tags.rs` used real slugs (`ramp`) as
  throwaway "no-description" fixtures and asserted `NULL`. Once you author that slug,
  the sync overlay fills it and the test fails. Fixtures that assert a NULL
  description must use a **synthetic slug** (e.g. `test-null-desc`) guaranteed to stay
  out of `ORACLE_TAG_DESCRIPTIONS`.
- **`&` in descriptions.** A literal `&` compiles fine in Rust but can bite if the
  text ever renders in an HTML context (the dictionary page). Write "and".
- **Concurrent AI safety.** Scope formatting to `cargo +nightly fmt -p zerver`;
  never run tree-wide git ops; `git add` only your files by explicit path.
- **No SQLx prepare needed.** The overlay uses runtime `sqlx::query`, not a
  `query!` macro, so `.sqlx/` offline data is untouched.
- **Cost.** ~7 slugs/chunk, sonnet draft + opus verify. A 200-tag batch is ~58 agents
  and ~1.4M output tokens, ~4-5 min wall clock. Scale batch size to appetite.
- **Priority is population, not the catalog order.** Always author highest-card-count
  blanks first so the most-seen tags get covered soonest.

## Progress markers (update as you go)

Coverage is `len(ORACLE_TAG_DESCRIPTIONS)` / ~4,500. Milestones so far: 7 (starter) ->
82 (hand) -> 257 -> 500 -> 700. Log the latest in
[`../../progress/overview.md`](../../progress/overview.md) when you cross a round number.
