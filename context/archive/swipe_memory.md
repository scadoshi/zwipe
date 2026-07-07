# Swipe memory — deck suppressions + per-user signal + weekly aggregates

**Status: EXECUTED 2026-07-02; server LIVE in prod, client submitted (1.2.3,
iOS build 56 / Android vc17, in store review).** Signal tables and removal
suppression are collecting from existing clients now; skip suppression
activates as 1.2.3 clients roll out. Kept as the design reference — see
"As-built deltas" below for where execution diverged from the original plan.
Absorbs and replaces the earlier `deck_skips.md`.

## As-built deltas (owner-directed during execution)

- **`deck_skips` became `deck_card_suppressions`** — a *suppression set*, not
  an action log: `(deck_id, oracle_id)` PK with a **`source` column as
  provenance, not identity** (`'skip' | 'removal'`, CHECK-enforced, no
  default — every insert states its source; a new source needs a migration).
- **Removals suppress too**: `delete_deck_card` inserts `source = 'removal'`
  server-side (no wire/client change — works for clients already in the wild).
  Single-card path only; bulk replace-mode import deletes do NOT suppress.
  **Re-adding a card deletes its suppression** (covers undo-of-remove).
- **Unskips only delete `source = 'skip'` rows** so an undo can't erase a
  removal suppression.
- **No name-search bypass** (owner call): suppressed cards are excluded from
  the deck-aware search unconditionally; the Clear button is the only way back.
- Endpoint is `DELETE /api/deck/{deck_id}/suppressions` →
  `200 {"cleared": n}` (`HttpClearedSuppressions`); UI label stays
  **"Clear skips"**.
- **Button lives in the deck view's More sheet** (`more_buttons.rs`), not the
  Add screen's filter sheet as originally planned — it's a rare action (owner
  call), so it's tucked with Clone/Export/Delete. It flushes pending skips
  before clearing so the buffer window can't re-suppress.

**What this builds, in one sentence:** left-swipes become durable per deck (the
server stops re-serving them, with a "Clear skips" escape hatch), and the same
flush ingest starts accruing per-user and per-week signal tables that nothing
consumes yet — personalization and weekly badges land later on data that starts
accruing now.

**Why now:** FR #11 (persist skips, High impact) is the loudest core-loop gap;
every day without per-user collection permanently loses signal from the most
engaged early users; all three stores share one ingest surface, so build the
pipe once.

---

## Locked decisions

| Decision | Value |
|---|---|
| Skip scope | Per **deck** (`deck_id`), Add screen **Search source** left-swipes only |
| Skip identity | `oracle_id` (applies across printings, matches in-deck exclusion semantics) |
| Skip cap | **5,000 per deck, evict oldest** (`skipped_at`) — enforced at ingest |
| Filtering | **Server-side**, `NOT EXISTS` subquery in the deck-aware search (not the bind array — skips can be thousands) |
| Clear skips UI | Button in the Add screen's **filter sheet**, next to the existing Clear control |
| Per-user signal | `user_card_signal` — user × commander × card counters, mirrors `commander_card_signal` + `user_id` FK CASCADE |
| Weekly aggregates | **Include now**: `user_week_signal` (scalar counters) + `user_week_facet_signal` (normalized category/color facets). Consumed later by weekly badges (see backlog) |
| Wire | Piggyback `HttpUsageBatch` (`#[serde(default)]` — old clients unaffected, **no min-version gate**) |
| Deploy order | **Server first** (Phase 1), client rides the next release (Phase 2). Signal tables start filling from existing 1.2.0+ clients the moment Phase 1 deploys |
| Privacy/labels | Policy + store-label + email pass handled separately later (owner call); deletion is covered by FK cascades in this plan |

**Separation rule:** clearing skips deletes `deck_skips` rows ONLY — never
signal rows. Skips are a user feature; signals are analytics.

---

## Existing code map (read these first)

| Concern | Where |
|---|---|
| Flush contract | `zwipe-core/src/http/contracts/metrics.rs` — `HttpUsageBatch`, `CardSignalDelta`, `MAX_PER_FLUSH = 10_000`, `MAX_SIGNALS_PER_FLUSH = 1_000`, `clamped()` |
| Client buffer | `zwiper/src/lib/inbound/components/telemetry/usage_buffer.rs` — `UsageBuffer` (atomics + `Mutex<HashMap>` signals), `record_signal`, `record_removal`, `snapshot_and_zero` |
| Client flush loop | `zwiper/src/lib/inbound/components/telemetry/flush_loop.rs` — 30s interval + background (`visibilitychange`/`pagehide`) both call `flush_once` |
| Server ingest | `zerver/src/lib/inbound/http/handlers/metrics/record_usage.rs` → `metrics_service.apply_usage(user.id, &batch)` → `zerver/src/lib/outbound/sqlx/metrics/mod.rs` (one tx: lifetime counters, `user_daily_activity`, `commander_card_signal` upsert loop) |
| Signal migration to mirror | `zerver/migrations/20260629020000_create_commander_card_signal.sql` |
| Deck-aware search exclusion | `zerver/src/lib/domain/deck/services.rs:278-331` builds `exclude_oracle_ids` (in-deck cards) → `search_cards_deck_aware` → SQL in `zerver/src/lib/outbound/sqlx/card/mod.rs` (`search_scryfall_data_deck_aware`) |
| Deck route + handler patterns | `zerver/src/lib/inbound/http/routes.rs:406-420`; ownership-checked handlers in `zerver/src/lib/inbound/http/handlers/deck/` (mirror `delete_deck.rs`) |
| Add screen swipe callbacks | `zwiper/src/lib/inbound/screens/deck/card/add.rs` — `on_swipe_left` (search + maybeboard modes; only **search** records skips), undo via `on_swipe_down` → `undo_last_action` / `SwipeAction` history |
| Filter sheet (Clear button home) | `zwiper/src/lib/inbound/screens/deck/card/filter/card_filter_sheet.rs` (`on_clear` handler ~line 568) |

Card facet lookup (for weekly facets): `latest_cards` (materialized view; has
`oracle_id`, `color_identity`) JOIN `card_profiles ON card_profiles.scryfall_data_id
= latest_cards.id` (has `mechanical_categories`).

---

## Phase 1 — server (deploy first; collection starts immediately)

### 1.1 Migration (one file, e.g. `2026____create_swipe_memory.sql`)

```sql
-- Deck skips: user feature. Cascade: deck delete / account delete (via decks).
CREATE TABLE deck_skips (
    deck_id    UUID        NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    oracle_id  UUID        NOT NULL,
    skipped_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (deck_id, oracle_id)
);

-- Per-user suggestion signal: analytics. Mirrors commander_card_signal + user_id.
CREATE TABLE user_card_signal (
    user_id             UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    commander_oracle_id UUID        NOT NULL,
    card_oracle_id      UUID        NOT NULL,
    shown               BIGINT      NOT NULL DEFAULT 0,
    added               BIGINT      NOT NULL DEFAULT 0,
    skipped             BIGINT      NOT NULL DEFAULT 0,
    maybed              BIGINT      NOT NULL DEFAULT 0,
    removed             BIGINT      NOT NULL DEFAULT 0,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, commander_oracle_id, card_oracle_id)
);

-- Weekly scalar counters (ISO week, Monday UTC). Badge substrate; no consumer yet.
CREATE TABLE user_week_signal (
    user_id      UUID   NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    week_start   DATE   NOT NULL,
    swipes_right INTEGER NOT NULL DEFAULT 0,
    swipes_left  INTEGER NOT NULL DEFAULT 0,
    swipes_up    INTEGER NOT NULL DEFAULT 0,
    swipes_down  INTEGER NOT NULL DEFAULT 0,
    searches     INTEGER NOT NULL DEFAULT 0,
    added        INTEGER NOT NULL DEFAULT 0,
    skipped      INTEGER NOT NULL DEFAULT 0,
    maybed       INTEGER NOT NULL DEFAULT 0,
    removed      INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, week_start)
);

-- Weekly facet counters (accepts by mechanical category / color identity).
-- Normalized rows, not jsonb: counter addition stays a plain upsert.
CREATE TABLE user_week_facet_signal (
    user_id    UUID    NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    week_start DATE    NOT NULL,
    facet      TEXT    NOT NULL,  -- 'category' | 'color'
    key        TEXT    NOT NULL,  -- e.g. 'Removal' / 'W'
    added      INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, week_start, facet, key)
);
```

`week_start` at ingest: `(date_trunc('week', now() AT TIME ZONE 'utc'))::date`.

### 1.2 Wire contract (`zwipe-core/src/http/contracts/metrics.rs`)

Add to `HttpUsageBatch` (both `#[serde(default)]`, so old clients and the
round-trip stay compatible):

```rust
/// Per-deck skip/unskip oracle-id deltas (Add-screen left swipes).
#[serde(default)]
pub deck_skips: Vec<DeckSkipDelta>,

pub struct DeckSkipDelta {
    pub deck_id: Uuid,
    /// Oracle ids left-swiped since the last flush.
    pub skipped: Vec<Uuid>,
    /// Oracle ids whose skip was undone after it had already flushed.
    pub unskipped: Vec<Uuid>,
}
```

Clamps (extend `clamped()` + constants, with unit tests mirroring the existing
ones): `MAX_SKIP_DECKS_PER_FLUSH: usize = 50` (a flush window touches one deck
realistically), and truncate each `skipped`/`unskipped` list to
`MAX_SIGNALS_PER_FLUSH` (1,000).

### 1.3 Ingest (`zerver/.../outbound/sqlx/metrics/mod.rs`, same tx as today)

Order within the existing `apply_usage` transaction, after the
`commander_card_signal` loop:

1. **`user_card_signal`** — same upsert loop as `commander_card_signal` but with
   `user_id` as the leading key. Same clamped inputs; add `user_id` to the
   `INSERT`/`ON CONFLICT` columns.
2. **`user_week_signal`** — one upsert: directional swipe counts + searches from
   the batch scalars; `added/skipped/maybed/removed` summed across
   `batch.signals`. `ON CONFLICT (user_id, week_start) DO UPDATE SET col = col + EXCLUDED.col`.
3. **`user_week_facet_signal`** — for signal deltas with `added > 0`: one query
   fetches facets for the added oracle_ids
   (`SELECT lc.oracle_id, lc.color_identity, cp.mechanical_categories FROM
   latest_cards lc JOIN card_profiles cp ON cp.scryfall_data_id = lc.id WHERE
   lc.oracle_id = ANY($1)`), accumulate `(facet, key) → added` in a HashMap in
   Rust, then upsert each row. Facet keys: each mechanical category string;
   each color letter of `color_identity` (colorless → key `'C'`).
4. **`deck_skips`** — for each `DeckSkipDelta`:
   - **Ownership**: resolve the deck's owner; if not the authenticated user,
     skip the delta silently (don't fail the whole batch — mirrors the
     "vanity data isn't worth erroring" posture).
   - Insert skips: `INSERT ... ON CONFLICT (deck_id, oracle_id) DO UPDATE SET
     skipped_at = now()`.
   - Delete unskips: `DELETE FROM deck_skips WHERE deck_id = $1 AND oracle_id = ANY($2)`.
   - **Cap**: after inserting, evict beyond 5,000:
     `DELETE FROM deck_skips WHERE deck_id = $1 AND oracle_id IN
     (SELECT oracle_id FROM deck_skips WHERE deck_id = $1
      ORDER BY skipped_at DESC OFFSET 5000)`.

Keep each new statement `query!`-checked; **run `cargo sqlx prepare --workspace`
after any query change and commit `.sqlx/`** (CI builds offline).

### 1.4 Serve-side filtering (skip exclusion)

In `search_scryfall_data_deck_aware` (`zerver/.../outbound/sqlx/card/mod.rs`):
the function needs the `deck_id` (today it receives `exclude_oracle_ids` etc. —
thread `deck_id: Option<Uuid>` through from `search_deck_cards` service
(`deck/services.rs` already has it) and push:

```sql
AND NOT EXISTS (
    SELECT 1 FROM deck_skips ds
    WHERE ds.deck_id = $N AND ds.oracle_id = latest_cards.oracle_id
)
```

only when `deck_id` is `Some`. The plain (non-deck) search passes `None` and is
untouched. The client needs no change for filtering to work.

### 1.5 Clear-skips endpoint

- Handler `zerver/.../handlers/deck/clear_deck_skips.rs`, mirroring
  `delete_deck.rs` (auth + ownership → Forbidden on mismatch).
- `DELETE FROM deck_skips WHERE deck_id = $1` returning the count; respond
  `200 { "cleared": n }`.
- Route: `.route("/{deck_id}/skips", delete(clear_deck_skips))` alongside the
  deck routes (`routes.rs` ~line 413).

### 1.6 Phase 1 verification & deploy

- Unit: contract clamp tests; ingest delta math (mirror existing metrics tests).
- Manual (local DB): POST a batch with `deck_skips` + `signals` → all four
  tables move; unskip deletes; cap evicts oldest at 5,001; foreign deck delta
  ignored; clear endpoint 403s a non-owner, deletes for the owner; deck-aware
  search excludes a skipped oracle and serves it again after clear.
- `cargo test --workspace`, clippy `-D warnings`, `cargo sqlx prepare`.
- **Deploy = push to main** (auto-deploys; get explicit owner approval to push).
  From this moment `user_card_signal` / `user_week_*` accrue from existing
  clients; `deck_skips` stays empty until Phase 2 ships.

## Phase 2 — client (rides the next release)

### 2.1 Record skips (`usage_buffer.rs`)

- Add to `UsageBufferInner`: `deck_skips: Mutex<HashMap<Uuid, DeckSkipSet>>`
  where `DeckSkipSet { skipped: HashSet<Uuid>, unskipped: HashSet<Uuid> }`.
- `record_deck_skip(deck_id, oracle_id)` — inserts into `skipped`, removes from
  `unskipped` (re-skip after undo cancels out).
- `retract_deck_skip(deck_id, oracle_id)` — if pending in `skipped`, remove it
  (pre-flush undo); **else** insert into `unskipped` (post-flush undo).
- Drain both in `snapshot_and_zero` into `Vec<DeckSkipDelta>`; batch is `Some`
  if any set is non-empty.

### 2.2 Wire into the Add screen (`add.rs`)

- `on_swipe_left` (**Search source only** — not maybeboard mode):
  `usage_buffer().record_deck_skip(deck_id, card.scryfall_data.oracle_id)`
  (no-op if `oracle_id` is `None`). The existing `record_signal` call stays.
- Undo path (`undo_last_action`, the `SwipeAction::Skip` arm): call
  `retract_deck_skip` with the same ids.
- Out of scope (do NOT record): maybeboard-mode left swipes, Remove screen
  (left = keep), Zwipe-select.

### 2.3 Flush-before-refresh (`add.rs` + `flush_loop.rs`)

Expose the existing `flush_once` (or a thin wrapper) so the Add screen can call
it imperatively; invoke it **before** the Refresh button's re-search and before
a filter-sheet Apply triggers a refetch. This closes the "skip, skip, Refresh
re-serves" gap. (Filtering itself is server-side; this only covers the ≤30s
buffer window.)

### 2.4 Clear-skips button

- Client fn `clear_deck_skips(deck_id)` in `zwiper/.../outbound/client/deck/`
  (mirror an existing DELETE-style client module).
- Button in the filter sheet (`card_filter_sheet.rs`, next to the existing
  Clear control; the sheet needs the `deck_id` — pass it as an optional prop
  from the Add screen, hidden when absent): label **"Clear skips"** (sentence
  case). On success: toast "Skips cleared" + bump `filter_reset_counter` to
  refetch the stack.
- Optional (skip if fiddly): empty-stack hint when exhausted — "You've skipped
  cards. Clear skips to see them again."

### 2.5 Phase 2 verification

- Unit: buffer skip/retract/drain (mirror existing signal buffer tests).
- Sim e2e (`dx serve --platform ios` against a local or prod backend):
  1. Skip cards → background the app (forces flush) → Refresh → skipped cards
     do not return.
  2. Kill + relaunch → still gone.
  3. Skip → undo immediately → card not recorded (check `deck_skips`).
  4. Skip → wait >30s (flushed) → undo → `unskipped` clears the row.
  5. Clear skips → toast → stack refetches → cards return.
- `cargo test --workspace` + clippy `-D warnings`.

---

## Explicitly out of scope (tracked elsewhere)

- Ranking/personalization consumers of `user_card_signal` (extends
  `plans/suggestion_signal.md` Phase 3 thinking).
- Weekly badges / stats page / share cards (backlog: "Weekly Badges + Stats").
- Privacy policy text, store data-safety labels, notification email (owner
  handles in a separate pass; cascades in 1.1 already make deletion correct).
- Skip TTL/decay (schema supports via `skipped_at`; decide on real data).

## Done when

Phase 1: all four tables live in prod, filling from existing clients; skipped
search verification passes. Phase 2: a left swipe on the Add screen survives an
app restart (card stays gone), undo stays consistent pre- and post-flush, and
Clear skips brings everything back. FR #11 marked shipped.
