# Partner, Background & Signature Spell — Phase 1: Database + Model

**Depends on:** Commander Filter Phases 1-3 complete.

Add three new nullable UUID columns to the `decks` table and thread them through the full stack: DeckProfile, HTTP contracts, request types, database adapter, and repository queries.

---

## Design Decision: No `oathbreaker_id` Column

The oathbreaker planeswalker reuses `commander_id`. The `format` field already tells us whether `commander_id` holds a creature commander or a planeswalker oathbreaker. This avoids two columns that never coexist and eliminates "which one do I check?" branching everywhere.

**New columns (3):**

| Column | Purpose | When used |
|--------|---------|-----------|
| `partner_commander_id` | Second commander for Partner / Friends Forever / Doctor's Companion | Commander, Duel, PreDH |
| `background_id` | Background legendary enchantment for "Choose a Background" | Commander, Duel, PreDH |
| `signature_spell_id` | Instant/sorcery signature spell | Oathbreaker only |

**Mutual exclusivity:** `partner_commander_id` and `background_id` are mutually exclusive — a commander either has a partner OR a background, never both. Enforcement is in application logic (validate_deck), not a database constraint, since the format determines which is valid.

---

## Step 1: Update Existing Migration (Not a New Migration)

**The database is not live yet.** Instead of creating a new ALTER TABLE migration, modify the existing `create_decks` migration in place.

**File:** `zerver/migrations/20250810194454_create_decks.sql`

Add the three new columns directly to the CREATE TABLE statement:

```sql
CREATE TABLE decks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    commander_id UUID,
    partner_commander_id UUID,
    background_id UUID,
    signature_spell_id UUID,
    format TEXT,
    user_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_user
        FOREIGN KEY (user_id)
        REFERENCES users (id)
        ON DELETE CASCADE,
    CONSTRAINT unique_deck_name_per_user
        UNIQUE(user_id, name)
);

CREATE INDEX idx_decks_user_id ON decks(user_id);
```

No foreign key constraints on the new columns — same pattern as `commander_id` (which also has no FK to scryfall_data). The card IDs reference Scryfall data that may be updated independently.

No indexes needed yet — these columns are not used in WHERE clauses for bulk queries.

**Important:** After modifying the migration, you must reset and re-run migrations locally:
```bash
sqlx database drop && sqlx database create && sqlx migrate run
```
Then re-sync card data via zervice and run `cargo sqlx prepare --workspace`.

---

## Step 2: Update DeckProfile (zwipe-core)

**File:** `zwipe-core/src/domain/deck/models/deck_profile.rs`

Add three new fields with their display name counterparts:

```rust
pub struct DeckProfile {
    pub id: Uuid,
    pub name: DeckName,
    pub commander_id: Option<Uuid>,
    pub partner_commander_id: Option<Uuid>,    // NEW
    pub background_id: Option<Uuid>,           // NEW
    pub signature_spell_id: Option<Uuid>,      // NEW
    pub format: Option<Format>,
    pub user_id: Uuid,
    pub card_count: i64,
    pub commander_name: Option<String>,
    pub partner_commander_name: Option<String>, // NEW
    pub background_name: Option<String>,        // NEW
    pub signature_spell_name: Option<String>,   // NEW
}
```

**Impact:** This struct is used widely. All construction sites must be updated. Search for `DeckProfile {` and `DeckProfile::` across the workspace to find all construction sites. Key locations:
- `DatabaseDeckProfile::try_into()` in `zerver/src/lib/outbound/sqlx/deck/models.rs`
- Test helpers in `validate_deck.rs` (`test_profile` function)
- Any other test files constructing `DeckProfile`

---

## Step 3: Update DatabaseDeckProfile (zerver)

**File:** `zerver/src/lib/outbound/sqlx/deck/models.rs`

Add the new columns to the database adapter struct:

```rust
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckProfile {
    pub id: Uuid,
    pub name: String,
    pub commander_id: Option<Uuid>,
    pub partner_commander_id: Option<Uuid>,    // NEW
    pub background_id: Option<Uuid>,           // NEW
    pub signature_spell_id: Option<Uuid>,      // NEW
    pub format: Option<String>,
    pub user_id: Uuid,
    pub card_count: Option<i64>,
    pub commander_name: Option<String>,
    pub partner_commander_name: Option<String>, // NEW
    pub background_name: Option<String>,        // NEW
    pub signature_spell_name: Option<String>,   // NEW
}
```

Update the `TryFrom<DatabaseDeckProfile> for DeckProfile` impl to map the new fields:

```rust
Ok(Self {
    id: value.id,
    name,
    commander_id: value.commander_id,
    partner_commander_id: value.partner_commander_id,
    background_id: value.background_id,
    signature_spell_id: value.signature_spell_id,
    format,
    user_id: value.user_id,
    card_count: value.card_count.unwrap_or(0),
    commander_name: value.commander_name,
    partner_commander_name: value.partner_commander_name,
    background_name: value.background_name,
    signature_spell_name: value.signature_spell_name,
})
```

---

## Step 4: Update All SQL Queries (zerver)

**File:** `zerver/src/lib/outbound/sqlx/deck/mod.rs`

Every query that returns `DatabaseDeckProfile` must select the new columns. There are 4 queries:

### 4a. `create_deck_profile` INSERT + RETURNING

```sql
INSERT INTO decks (name, commander_id, partner_commander_id, background_id, signature_spell_id, format, user_id)
VALUES ($1, $2, $3, $4, $5, $6, $7)
RETURNING id, name, commander_id, partner_commander_id, background_id, signature_spell_id, format, user_id,
          0::bigint as "card_count",
          (SELECT sd.name FROM scryfall_data sd WHERE sd.id = commander_id) as "commander_name?",
          (SELECT sd.name FROM scryfall_data sd WHERE sd.id = partner_commander_id) as "partner_commander_name?",
          (SELECT sd.name FROM scryfall_data sd WHERE sd.id = background_id) as "background_name?",
          (SELECT sd.name FROM scryfall_data sd WHERE sd.id = signature_spell_id) as "signature_spell_name?"
```

Bind the new request fields (which will be `None` initially since the create request doesn't include them yet — see Step 5).

### 4b. `get_deck_profile` SELECT

```sql
SELECT d.id, d.name, d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id,
       d.format, d.user_id,
       COALESCE(SUM(dc.quantity), 0) as "card_count",
       sd.name as "commander_name?",
       sd_partner.name as "partner_commander_name?",
       sd_bg.name as "background_name?",
       sd_sig.name as "signature_spell_name?"
FROM decks d
LEFT JOIN deck_cards dc ON d.id = dc.deck_id
LEFT JOIN scryfall_data sd ON d.commander_id = sd.id
LEFT JOIN scryfall_data sd_partner ON d.partner_commander_id = sd_partner.id
LEFT JOIN scryfall_data sd_bg ON d.background_id = sd_bg.id
LEFT JOIN scryfall_data sd_sig ON d.signature_spell_id = sd_sig.id
WHERE d.id = $1
GROUP BY d.id, d.name, d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id,
         d.format, d.user_id, sd.name, sd_partner.name, sd_bg.name, sd_sig.name
```

### 4c. `get_deck_profiles` (list) — Same pattern as 4b but `WHERE d.user_id = $1`

### 4d. `update_deck_profile` — Dynamic UPDATE

Add conditional SET clauses for each new field, following the existing pattern:

```rust
if let Some(partner_commander_id) = &request.partner_commander_id {
    sep.push("partner_commander_id = ")
        .push_bind_unseparated(partner_commander_id);
}
if let Some(background_id) = &request.background_id {
    sep.push("background_id = ")
        .push_bind_unseparated(background_id);
}
if let Some(signature_spell_id) = &request.signature_spell_id {
    sep.push("signature_spell_id = ")
        .push_bind_unseparated(signature_spell_id);
}
```

Update the RETURNING clause to include the new columns + subquery name lookups (same pattern as commander_name).

---

## Step 5: Update HTTP Contracts (zwipe-core)

**File:** `zwipe-core/src/http/contracts/deck.rs`

### HttpCreateDeckProfile

```rust
pub struct HttpCreateDeckProfile {
    pub name: String,
    pub commander_id: Option<Uuid>,
    pub partner_commander_id: Option<Uuid>,    // NEW
    pub background_id: Option<Uuid>,           // NEW
    pub signature_spell_id: Option<Uuid>,      // NEW
    pub format: Option<String>,
}
```

Update the `new()` constructor to accept the new fields.

### HttpUpdateDeckProfile

```rust
pub struct HttpUpdateDeckProfile {
    pub name: Option<String>,
    pub commander_id: Opdate<Uuid>,
    pub partner_commander_id: Opdate<Uuid>,   // NEW
    pub background_id: Opdate<Uuid>,          // NEW
    pub signature_spell_id: Opdate<Uuid>,     // NEW
    pub format: Opdate<String>,
}
```

Update the `new()` constructor.

---

## Step 6: Update Domain Request Types (zwipe-core)

**File:** `zwipe-core/src/domain/deck/requests/create_deck_profile.rs`

Add the three new `Option<Uuid>` fields to `CreateDeckProfile` struct and `new()` constructor. These are pass-through — no validation needed at this layer (validation is in `validate_deck`).

**File:** `zwipe-core/src/domain/deck/requests/update_deck_profile.rs`

Add three new `Option<Option<Uuid>>` fields to `UpdateDeckProfile` struct and `new()` constructor. Update the `NoUpdates` check to include the new fields.

---

## Step 7: Update HTTP Handlers (zerver)

**File:** `zerver/src/lib/inbound/http/handlers/deck/create_deck_profile.rs`

Pass the new fields from `body` through to `CreateDeckProfile::new()`.

**File:** `zerver/src/lib/inbound/http/handlers/deck/update_deck_profile.rs`

Convert the new `Opdate<Uuid>` fields to `Option<Option<Uuid>>` via `.into_option()` and pass to `UpdateDeckProfile::new()`.

---

## Step 8: Update validate_deck Card Count (zwipe-core)

**File:** `zwipe-core/src/domain/deck/models/validate_deck.rs`

The `check_card_count` function currently adds 1 if `commander_id.is_some()`. It needs to also count the partner/background/signature_spell:

```rust
fn check_card_count(format: &Format, profile: &DeckProfile, warnings: &mut Vec<DeckWarning>) {
    let mut count = profile.card_count as u32;

    if format.has_commander() && profile.commander_id.is_some() {
        count += 1;
    }
    if profile.partner_commander_id.is_some() {
        count += 1;
    }
    if profile.background_id.is_some() {
        count += 1;
    }
    // Signature spell counts toward the 60-card Oathbreaker deck
    if profile.signature_spell_id.is_some() {
        count += 1;
    }
    // ... rest of min/max checks
}
```

---

## Step 9: Run SQLx Prepare + Tests

```bash
cargo sqlx prepare --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

---

## Step 10: Add Format Helper Methods (zwipe-core)

**File:** `zwipe-core/src/domain/deck/models/format.rs`

Add convenience methods:

```rust
/// Whether this format supports partner commanders.
pub fn supports_partner(&self) -> bool {
    matches!(self, Self::Commander | Self::Duel | Self::Predh)
}

/// Whether this format supports "Choose a Background".
pub fn supports_background(&self) -> bool {
    matches!(self, Self::Commander | Self::Duel | Self::Predh)
}

/// Whether this format requires a signature spell (Oathbreaker).
pub fn has_signature_spell(&self) -> bool {
    matches!(self, Self::Oathbreaker)
}
```

---

## Verification Checklist

- [ ] Existing `create_decks` migration updated with 3 new columns (no new migration file)
- [ ] `DeckProfile` has 6 new fields (3 IDs + 3 names)
- [ ] `DatabaseDeckProfile` has matching fields with `FromRow`
- [ ] All 4 SQL queries (create, get, get_list, update) select new columns
- [ ] Name subqueries join scryfall_data for each new ID
- [ ] HTTP contracts updated (create + update)
- [ ] Domain request types updated (create + update)
- [ ] Handlers pass new fields through
- [ ] `check_card_count` includes partner/background/signature_spell
- [ ] `Format` has `supports_partner()`, `supports_background()`, `has_signature_spell()`
- [ ] `cargo sqlx prepare --workspace` succeeds
- [ ] All existing tests pass (update `test_profile` helper)
- [ ] `cargo clippy` clean

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zerver/migrations/20250810194454_create_decks.sql` | Add 3 columns to existing CREATE TABLE |
| `zwipe-core/.../deck/models/deck_profile.rs` | Add 6 fields |
| `zwipe-core/.../deck/models/format.rs` | Add `supports_partner()`, `supports_background()`, `has_signature_spell()` |
| `zwipe-core/.../deck/models/validate_deck.rs` | Update `check_card_count` |
| `zwipe-core/.../deck/requests/create_deck_profile.rs` | Add 3 fields |
| `zwipe-core/.../deck/requests/update_deck_profile.rs` | Add 3 fields |
| `zwipe-core/src/http/contracts/deck.rs` | Add fields to both HTTP structs |
| `zerver/.../outbound/sqlx/deck/models.rs` | Add 6 fields to DatabaseDeckProfile + TryFrom |
| `zerver/.../outbound/sqlx/deck/mod.rs` | Update 4 SQL queries |
| `zerver/.../http/handlers/deck/create_deck_profile.rs` | Pass new fields |
| `zerver/.../http/handlers/deck/update_deck_profile.rs` | Pass new fields |
