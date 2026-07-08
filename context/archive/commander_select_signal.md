# Commander-select signal — first-party impression + selection counts

**Status: §1-3 SHIPPED TO MAIN 2026-07-07 (`aa10a5be` feat + `fb30e371` docs;
archived 2026-07-07). §4 Consumer B (first-party popularity term) remains
deliberately unbuilt: a data-gated retune, weight 0 until select coverage is
real — this doc is its spec. Server-first: the migration/ingest ride the next
server deploy;
the client instrumentation (select-screen tallies in `UsageBuffer`) rides the
1.4.0 store build alongside the select-serve client leg, so collection starts
the day the new serve reaches users. Consumer A ships dormant (empty table
COALESCEs to the shuffle-only baseline). §4 Consumer B remains deliberately
unbuilt — a later retune gated on coverage. Fast-follow to
[`commander_select_ordering.md`](../archive/commander_select_ordering.md): the select
branch must be live first (its §3 client change shipped) so commanders
actually get served through it and impressions accrue. Entirely zwipe-side —
client instrumentation + a zerver store + a zerver read. Unlike
`commander_popularity`, the synergy worker is not involved.**

**What this builds, in one sentence:** a pooled, PII-free count of which
commanders Zwipe users are *shown* and which they *select*, so the select
serve can (1) weight its wildcard deep-slice toward least-shown commanders
and (2) carry a mild first-party popularity term on top of the EDHREC base.

**Why now:** the select ordering ([`commander_select_ordering.md`](../archive/commander_select_ordering.md))
and its wildcard both work today, but the wildcard deep-slice falls back to
the (deck, day) shuffle alone — there is no select-impression signal to
weight by. Every existing signal table (`commander_card_signal`,
`user_card_signal`, `card_signal_rollup`) keys on the **99-serve's**
commander+card pairs; none of them records commander *selection*. This is
the missing measurement, and it was noted as the "Later" item in the
ordering plan.

## Framing — a refinement layer, never the base

The decks-helmed EDHREC popularity (`commander_popularity`) stays the
dominant ordering base. First-party select counts are a mild term on top,
for the same two reasons that plan gave for not deriving the base from our
own data:

- **Cold-start.** The table begins empty and stays sparse for a while.
  It cannot rank the pool on its own.
- **Circularity.** Users pick from the ordering we served them, so a
  selection signal reinforces whatever we already showed. Left unchecked it
  becomes a feedback loop that just amplifies band 1.

Two structural guards, both already in the serve: EDHREC popularity stays
the base (the first-party term is small and centered, like `W_SIGNAL` for
synergy), and the **wildcard slot is what breaks the loop** — it forces
deep, rarely-shown commanders into view so they can earn impressions the
band ordering would never give them. That is exactly the exposure the
least-shown weighting below consumes.

## 1. Data — `commander_select_signal` table (owned here)

Migration in `zerver/migrations/`. Pooled aggregate, one row per commander,
**no `user_id`, no per-swipe rows** — same privacy posture as
`commander_card_signal` and `user_lifetime_counters`.

```sql
-- First-party commander-select signal: which commanders Zwipe users are
-- shown vs. select. Pure aggregate keyed by commander oracle_id — NO user_id.
-- `shown` is the impression denominator (client-derived as selected + skipped);
-- `selected` is a right-swipe pick as the deck's commander; `skipped` is a
-- left-swipe pass. Read by the select serve (wildcard least-shown weighting +
-- optional popularity term). See context/archive/commander_select_signal.md.
CREATE TABLE commander_select_signal (
    commander_oracle_id UUID        NOT NULL PRIMARY KEY,
    shown               BIGINT      NOT NULL DEFAULT 0,
    selected            BIGINT      NOT NULL DEFAULT 0,
    skipped             BIGINT      NOT NULL DEFAULT 0,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Already aggregated per commander (PK), so unlike `commander_card_signal` it
needs **no rollup materialized view** — the read joins it directly. Confirm
the exact gesture vocabulary against the select screen
(`zwiper/.../swipe_select.rs`); if select ever grows a "maybe" gesture, add
the column then, not speculatively.

## 2. Ingest — client instrumentation + zerver upsert

Mirror the 99-serve suggestion-signal flow (batch, not per-swipe): the
select screen accumulates shown/selected/skipped locally and flushes a
batch to a zerver endpoint, which upserts additively:

```sql
INSERT INTO commander_select_signal (commander_oracle_id, shown, selected, skipped, updated_at)
VALUES ($1, $2, $3, $4, now())
ON CONFLICT (commander_oracle_id) DO UPDATE
    SET shown      = commander_select_signal.shown    + EXCLUDED.shown,
        selected   = commander_select_signal.selected + EXCLUDED.selected,
        skipped    = commander_select_signal.skipped  + EXCLUDED.skipped,
        updated_at = now();
```

Additive (`+=`), not last-writer-wins — this is an accumulating counter, not
a snapshot. Reuse the existing signal-batch plumbing if the shapes line up;
otherwise a thin dedicated endpoint mirroring the 99-serve one.

## 3. Consumer A — wildcard least-shown weighting (primary payoff)

`outbound/sqlx/card/mod.rs`, the commander-select wildcard CTE. Today the
deep-slice exposure is a constant:

```rust
qb.push(") AS shuffle, 0 AS pool_shown ...");   // no select signal yet
```

Replace the constant with the real count, joined the **aliased** way (the
plain-join column-ambiguity trap from the ordering build — `commander_popularity`'s
bare `name`/`oracle_id` colliding with shared filters — applies identically
here; alias to a non-colliding name):

```sql
LEFT JOIN (SELECT commander_oracle_id AS sel_oid, shown AS sel_shown
           FROM commander_select_signal) sel ON sel.sel_oid = latest_cards.oracle_id
-- ... COALESCE(sel.sel_shown, 0) AS pool_shown
```

The deep-slice `ORDER BY pool_shown ASC, shuffle` then serves genuinely
least-shown deep commanders first, shuffle as the tiebreak. **Ships safely
before data exists:** with an empty table every `pool_shown` is `COALESCE(NULL,0)
= 0`, so the ORDER BY collapses to the shuffle — byte-identical to today's
behavior. The weighting simply switches on as counts accrue.

## 4. Consumer B — first-party popularity term (optional, later, gated)

A small term on the ordering base, parallel to synergy's `W_SIGNAL`: shift a
commander by its shrunk, globally-centered select-rate (`selected / shown`),
so a commander our users pick more than their EDHREC rank predicts drifts up.
This is where circularity bites hardest, so:

- **Default weight 0** (pure EDHREC base) — the revert lever, same as
  `W_SIGNAL = 0` reproduces the pre-signal synergy order.
- Shrink toward and center on the global select-rate (a commander with no
  impressions contributes exactly zero), same shrinkage math as the synergy
  signal (`SHRINK_K`).
- Turn it on only once the table has broad coverage and the wildcard has
  been feeding the deep pool long enough that the rate is not purely a
  reflection of what band 1 already showed.

Build Consumer A first; treat B as a deliberate retune, not v1.

## 5. Privacy

Pooled counts, no `user_id`, no per-swipe rows — nothing here identifies a
person or is PII. This is deliberately the lighter posture of
`commander_card_signal`, not the per-user `user_card_signal` (which exists as
an analytics substrate with no consumer). If personalized select ordering is
ever wanted, add a per-user sibling then and weigh the privacy cost at that
point; v1 does not need it.

## 6. Ownership + rollout

Entirely this repo: migration, ingest endpoint, client instrumentation, and
both read-side consumers all live in zwipe. The synergy worker is not
involved — contrast `commander_popularity`, where zynergy owns population.

1. **Migration** — table exists empty, nothing reads or writes it.
2. **Ingest** (client batch + zerver upsert) — counts start accruing. Needs
   the select branch to be live (ordering plan §3 shipped) so commanders are
   actually served and shown.
3. **Consumer A** (wildcard least-shown) — safe to deploy any time after the
   migration; dormant (shuffle-only) until data accrues.
4. **Consumer B** (popularity term) — later, weight 0 until coverage is real.

## 7. Verification

- Ingest: a select session increments `shown`/`selected`/`skipped` additively;
  rerun accumulates rather than overwrites.
- Consumer A: with the table empty, the wildcard slice is byte-identical to
  the shuffle-only baseline (COALESCE floor). Seed a few high-`shown`
  commanders → they sink in the deep-slice ordering, least-shown float up;
  band determinism per (deck, day) unchanged.
- Privacy: schema carries no `user_id`; no per-swipe table exists.

## Later / open questions

- Confirm the select screen's gesture set before pinning columns (does select
  have a "maybe"?).
- Recency: raw lifetime counts drift stale as the meta moves; a windowed or
  decayed variant could come later (schema gains a column, math unchanged).
- Surprise scoring for commanders — the wildcard_slot follow-on ("center the
  signal on a rank-bucket expectation") applies here too once select
  impressions exist: a deep commander that overperforms its EDHREC rank when
  shown is a promotion candidate. See [`wildcard_slot/overview.md`](../archive/wildcard_slot/overview.md).
