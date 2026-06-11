# Synergy data layer — what zerver needs to build, from the worker's side

**Status: written 2026-06-11, a message from the synergy worker's side of the
aisle.** The worker is built and verified end-to-end against the dev database:
creating a deck got its commander's synergy payload cached **4.4 seconds
later** with zero zerver involvement. What remains is entirely on this side —
one migration and a read path. This plan is the contract.

The worker itself is a private service and stays that way; in this repo it is
only ever "the synergy worker" / "synergy data layer." It runs as a separate
daemon against the **same Postgres** zerver/zervice use, on the same host.

---

## The headline: zerver writes nothing

The worker discovers demand by itself. Every poll cycle (30s default) it diffs
`decks.commander_id` against the cache and fills whatever is missing. Creating
a deck **is** the trigger — there is no enqueue call, no API, no write-side
code in this repo, ever. zerver's entire job is:

1. **One migration** creating the two tables below (so sqlx offline mode and
   the read path own the schema here, where the consumers live).
2. **A read path** from `commander_synergy` into ranking.

## Migration DDL

```sql
-- Fill queue. Written and drained entirely by the synergy worker; zerver
-- never touches it. Exists here because this repo owns the canonical schema.
CREATE TABLE synergy_requests (
    oracle_id       uuid PRIMARY KEY,
    commander_name  text NOT NULL,
    requested_at    timestamptz NOT NULL DEFAULT now(),
    attempts        int NOT NULL DEFAULT 0,
    last_attempt_at timestamptz,
    last_error      text
);

-- Cache: one row per commander, reduced jsonb payload. zerver READS this.
CREATE TABLE commander_synergy (
    oracle_id       uuid PRIMARY KEY,
    commander_name  text NOT NULL,
    payload         jsonb NOT NULL,
    fetched_at      timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_commander_synergy_fetched_at ON commander_synergy (fetched_at);
```

## Contract surface (coordinate migrations!)

Beyond the two tables, the worker has a **read-only** dependence on:

- `decks(commander_id)` — joined to find deck commanders
- `scryfall_data(id, oracle_id, name)` — to resolve commander identity

A migration reshaping those columns breaks the worker's discovery query
(loudly, in its cycle error log — nothing in zerver breaks). Flag any such
migration so both sides move in lockstep.

## Payload shape

`payload` is a reduced, source-agnostic JSON document:

```json
{
  "lists": [
    {
      "tag": "highsynergycards",
      "header": "High Synergy Cards",
      "cards": [
        {
          "name": "Some Card Name",
          "slug": "some-card-name",
          "synergy": 0.21,
          "num_decks": 2014,
          "potential_decks": 7850
        }
      ]
    }
  ]
}
```

- `lists[].tag` — stable machine key. Observed tags: `newcards`,
  `highsynergycards`, `topcards`, plus one per card type (`creatures`,
  `instants`, `sorceries`, `utilityartifacts`, `enchantments`,
  `planeswalkers`, `utilitylands`, `manaartifacts`, `lands`).
  Real pages carry 12–13 lists, ~240–270 cards total, ~10–12KB per row.
- `synergy` — float roughly in −1..1; higher = more synergistic with this
  commander than with the average deck. Can be null (e.g. `newcards` entries).
- `num_decks` / `potential_decks` — how many decks that could run the card do.
  `num_decks / potential_decks` is the inclusion rate.
- **Resolve cards by exact `name`** against `scryfall_data.name` — the `slug`
  is not a Scryfall identifier and card-level ids are deliberately absent.
  `find_cards_by_exact_names` (card/ports.rs, card/services.rs — what the
  Archidekt importer uses) already does this resolution.

## Read path sketch

`decks.commander_id` is a **printing id** (`scryfall_data.id`), not an
oracle_id — the read path must resolve printing → oracle, exactly like the
worker's discovery query does:

```sql
SELECT cs.payload
FROM decks d
JOIN scryfall_data s ON s.id = d.commander_id
JOIN commander_synergy cs ON cs.oracle_id = s.oracle_id
WHERE d.id = $1
```

- **Graceful absence is the design.** A miss means "no signal yet" — rank
  without the boost, never block, never error. The row appears seconds later.
- **Null-oracle printings are silently skipped — a deliberate choice.**
  `scryfall_data.oracle_id` is nullable (reversible/Secret Lair printings,
  the same class the Archidekt importer special-cases). A commander on a
  null-oracle printing never joins, so it never caches and never gets the
  boost — no crash on either side, just no signal. If it ever bites, the
  cheap fix is a name fallback, same as imports.
- Staleness is fine: `fetched_at` is refreshed weekly by the worker; a stale
  payload keeps serving if a refresh fails. The read path should not care.
- **Internal signal only.** Use it to order/boost (smart stack ordering,
  suggestions); do not print the raw numbers in the UI as an editorial stat.

## How to test from this side

1. Run the migration (dev db). **Dev-only gotcha:** the worker's testing
   already bootstrapped both tables ad hoc (from its contract file), so the
   migration's plain `CREATE TABLE` will collide. Drop them first
   (`DROP TABLE synergy_requests, commander_synergy;`) so the canonical
   migration creates them and `_sqlx_migrations` records it. Prod never has
   this problem — the tables won't exist there until the migration runs.
2. Start the synergy worker pointed at the same `DATABASE_URL` (it's a
   `cargo run` daemon; ask the worker's repo for details).
3. Create a deck with any commander through the app/API.
4. Within one poll interval (+ a couple seconds of fetch politeness):
   `SELECT commander_name, fetched_at FROM commander_synergy` shows the row.
   Measured on dev: 4.4s end-to-end with a 10s poll; production default is a
   30s poll, so worst case ~35s.
5. Failures park in `synergy_requests` with `attempts`/`last_error` for
   diagnosis and never retry-storm.

## Nice-to-have follow-up (separate work)

A `deck_opened` kind in `user_events` (only `deck_created`/`deck_completed`
exist today). Generic activity analytics on its own merits; the worker could
later use it to prioritize refreshes toward actively-viewed commanders
instead of pure staleness order.
