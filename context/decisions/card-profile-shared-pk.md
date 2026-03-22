# Decision: Remove Surrogate PK from card_profiles

## Status

Planned — implement after Remove Card screen is complete.

---

## The Problem

`card_profiles` currently has two UUIDs for the same card:

```
card_profiles
├── id UUID PRIMARY KEY DEFAULT gen_random_uuid()   ← surrogate, internal only
└── scryfall_data_id UUID NOT NULL UNIQUE           ← FK to scryfall_data, used everywhere
```

`id` has never been the actual lookup key anywhere in the application. Every route,
every SQL query, and every API call uses `scryfall_data_id` as the real identifier.
The surrogate `id` exists because it was the default "always add a UUID PK" instinct,
not because anything actually needs it.

This ambiguity already caused a real bug: `ClientDeleteDeckCard` and `ClientUpdateDeckCard`
name their parameter `card_profile_id` — suggesting callers should pass `card_profile.id` —
but the backend route actually takes `scryfall_data_id`. The `remove.rs` screen was written
using `card.card_profile.id` and failed at runtime with "deck card not found" on every swipe.

---

## The Fix: Shared Primary Key Pattern

Make `scryfall_data_id` both the PK and the FK:

```sql
CREATE TABLE card_profiles (
    scryfall_data_id UUID PRIMARY KEY
        REFERENCES scryfall_data (id) ON DELETE RESTRICT,
    is_valid_commander BOOLEAN NOT NULL DEFAULT FALSE,
    is_token           BOOLEAN NOT NULL DEFAULT FALSE,
    created_at         TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP NOT NULL DEFAULT NOW()
);
```

This is the canonical SQL pattern for a strict 1:1 relationship (sometimes called an
"identifying relationship"). The child table's row *is* the parent's row extended with
domain attributes. It is not bad practice — it is the correct practice for this relationship.

After this change there is one UUID per card. `card_profile.id` IS `scryfall_data.id`.
The naming confusion and the class of bug it causes cannot recur.

---

## Scope of Changes

### Migrations (zerver/migrations/)

- New migration: drop `id` column, drop the `gen_random_uuid()` default, promote
  `scryfall_data_id` to `PRIMARY KEY`.
- Drop `idx_card_profiles_scryfall_data_id` (redundant once it becomes the PK).
- No other tables FK into `card_profiles.id` — confirmed by migration search.

### Domain Model (zerver/src/lib/domain/card/models/)

- `CardProfile` struct: remove `id: Uuid` field; the struct's identity is now
  `scryfall_data_id: Uuid` (or rename that field to `id` — decision to make at
  implementation time; keeping the name `scryfall_data_id` is more explicit).
- `get_card_profile.rs`: `GetCardProfile(Uuid)` and `CardProfileIds(Vec<Uuid>)` wrap
  UUIDs used to look up card profiles. After the change, those UUIDs are
  `scryfall_data_id` values — the types themselves may not need to change, but their
  documentation and any callers that previously passed the surrogate `id` need auditing.

### SQLx Adapter (zerver/src/lib/outbound/sqlx/card/)

- `get_card_profile_with_id`: query uses `WHERE id = $1` → becomes
  `WHERE scryfall_data_id = $1` (or just `WHERE id = $1` if field renamed).
- `get_card_profiles_with_ids`: `WHERE id = ANY($1)` → same update.
- Upsert helper in `helpers/upsert_card.rs`: confirm upsert targets `scryfall_data_id`
  (likely already correct since it syncs from Scryfall).
- `SELECT id, scryfall_data_id, ...` projections will simplify — only one UUID column
  to select.

### Frontend Client Traits (zwiper/src/lib/outbound/client/)

Three client trait methods carry the wrong parameter name:

| File | Current param | Correct name |
|---|---|---|
| `deck_card/delete_deck_card.rs` | `card_profile_id` | `scryfall_data_id` |
| `deck_card/update_deck_card.rs` | `card_profile_id` | `scryfall_data_id` |
| `card/get_card.rs` | `card_profile_id` | `scryfall_data_id` |

These are rename-only changes — the UUID being passed was already correct at the call
sites (routes use `scryfall_data_id`). The param name just lied about what it was.

### Frontend Screens (zwiper/src/lib/inbound/screens/)

- `deck/card/add.rs` lines 132 and 138: uses `card.card_profile.id` for in-memory
  de-duplication of the card stack. After the domain model change, this field either
  becomes `card.card_profile.scryfall_data_id` or `card.card_profile.id` if the field
  is renamed. Either way it is a read-only comparison — no API call involved.
- `deck/card/remove.rs`: already patched to use `card.scryfall_data.id` at the call site.
  If `CardProfile` gains an `id` field that equals `scryfall_data_id`, the internal
  de-dup in `remove_current_card` (which uses `card.card_profile.id`) should be
  re-evaluated for clarity.

---

## What This Does NOT Change

- The `scryfall_data` table — untouched.
- All HTTP routes — they already accept `scryfall_data_id` in the path.
- Deck card operations — `deck_cards` FKs into `scryfall_data`, not `card_profiles`.
- The `sleeve()` helper and `Card` assembly logic — structural, not ID-dependent.
- Auth, user, deck domain — nothing outside the card subdomain is affected.

---

## Approach

1. Write a new migration (do not edit the original — existing dev envs may have run it).
2. Update `CardProfile` struct and its SQLx `FromRow` derive.
3. Update `get_card_profile_with_id` and `get_card_profiles_with_ids` SQL queries.
4. Rename client trait parameters from `card_profile_id` → `scryfall_data_id`.
5. Audit `add.rs` and `remove.rs` de-duplication logic for any field name fallout.
6. Run `cargo test --workspace` and `cargo clippy --workspace`.
7. Reset dev database (`./zcripts/denv/mac/setup.sh`) and verify sync still works.
