# Maybeboard — Phase 1: Core + Backend (Data Model + API)

Add `maybeboard: bool` to the deck card pipeline — migration, domain model, HTTP contracts, repository queries, metrics exclusion, validation exclusion.

---

## Design Decisions (Settled)

- **Unique constraint unchanged:** A card appears once per deck — either active or maybeboard, not both. Toggle the flag to move between states.
- **card_count excludes maybeboard:** DeckProfile.card_count only counts active deck cards.
- **DeckMetrics excludes maybeboard:** Metrics computed from active entries only.
- **validate_deck excludes maybeboard:** Validation only applies to active deck cards.
- **Buy links exclude maybeboard by default** (frontend toggle added later).

---

## Step 1: Update Existing Migration (Not a New Migration)

**The database is not live yet.** Instead of creating a new ALTER TABLE migration, modify the existing `create_deck_cards` migration in place.

**File:** `zerver/migrations/20250810194459_create_deck_cards.sql`

Add the `maybeboard` column directly to the CREATE TABLE statement:

```sql
CREATE TABLE deck_cards (
    deck_id UUID NOT NULL,
    scryfall_data_id UUID NOT NULL,
    quantity INT NOT NULL,
    maybeboard BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_deck
        FOREIGN KEY (deck_id)
        REFERENCES decks (id) ON DELETE CASCADE,
    CONSTRAINT fk_scryfall_data_id
        FOREIGN KEY (scryfall_data_id)
        REFERENCES scryfall_data (id) ON DELETE CASCADE,
    CONSTRAINT deck_card_unique UNIQUE (deck_id, scryfall_data_id),
    CONSTRAINT positive_quantity CHECK (quantity > 0)
);
```

**Important:** After modifying the migration, you must reset and re-run migrations locally:
```bash
sqlx database drop && sqlx database create && sqlx migrate run
```
Then re-sync card data via zervice and run `cargo sqlx prepare --workspace`.

---

## Step 2: Update DeckCard (zwipe-core)

**File:** `zwipe-core/src/domain/deck/models/deck_card.rs`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeckCard {
    pub deck_id: Uuid,
    pub scryfall_data_id: Uuid,
    pub quantity: Quantity,
    pub maybeboard: bool,           // NEW
}
```

**Impact:** Every place that constructs a `DeckCard` must now provide `maybeboard`. Search for `DeckCard {` across the workspace.

---

## Step 3: Update DatabaseDeckCard (zerver)

**File:** `zerver/src/lib/outbound/sqlx/deck/models.rs`

```rust
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckCard {
    pub deck_id: String,
    pub scryfall_data_id: String,
    pub quantity: i32,
    pub maybeboard: bool,           // NEW
}
```

Update the `TryFrom<DatabaseDeckCard> for DeckCard` impl:

```rust
Ok(Self {
    deck_id,
    scryfall_data_id,
    quantity,
    maybeboard: value.maybeboard,
})
```

---

## Step 4: Update All Repository SQL Queries (zerver)

**File:** `zerver/src/lib/outbound/sqlx/deck/mod.rs`

### 4a. create_deck_card — INSERT now includes maybeboard

```sql
INSERT INTO deck_cards (deck_id, scryfall_data_id, quantity, maybeboard)
VALUES ($1, $2, $3, $4)
RETURNING deck_id, scryfall_data_id, quantity, maybeboard
```

Bind `request.maybeboard` (new field on CreateDeckCard).

### 4b. get_deck_cards — SELECT now includes maybeboard

```sql
SELECT deck_id, scryfall_data_id, quantity, maybeboard
FROM deck_cards WHERE deck_id = $1
```

Returns ALL cards (both active and maybeboard). The frontend filters locally.

### 4c. update_deck_card — RETURNING now includes maybeboard

The quantity delta update:
```sql
UPDATE deck_cards SET quantity = quantity + $1
WHERE deck_id = $2 AND scryfall_data_id = $3
RETURNING deck_id, scryfall_data_id, quantity, maybeboard
```

Also add support for updating the maybeboard flag. The `UpdateDeckCard` request will have an optional `maybeboard` field. If present, include it in the SET clause:

```rust
// In the update logic:
if let Some(maybeboard) = request.maybeboard {
    // UPDATE deck_cards SET maybeboard = $1 WHERE ...
}
```

Since the current update_deck_card uses a simple `query_as!` (not a dynamic query builder), you may need to either:
- Add a separate method `toggle_maybeboard` on the repository, OR
- Convert to a dynamic QueryBuilder (like update_deck_profile uses)

**Recommendation:** Add a separate `toggle_maybeboard` method. It's cleaner than mixing quantity deltas with boolean flag updates.

```rust
async fn toggle_maybeboard(
    &self,
    request: &ToggleMaybeboard,
) -> Result<DeckCard, ToggleMaybeboardError> {
    // 1. Verify ownership
    // 2. UPDATE deck_cards SET maybeboard = $1 WHERE deck_id = $2 AND scryfall_data_id = $3
    //    RETURNING deck_id, scryfall_data_id, quantity, maybeboard
}
```

### 4d. delete_deck_card — No change needed (deletes regardless of maybeboard status)

### 4e. bulk_create_deck_cards (import) — Include maybeboard in INSERT

```sql
INSERT INTO deck_cards (deck_id, scryfall_data_id, quantity, maybeboard)
VALUES (...)
ON CONFLICT (deck_id, scryfall_data_id)
DO UPDATE SET quantity = EXCLUDED.quantity, maybeboard = EXCLUDED.maybeboard
RETURNING deck_id::TEXT, scryfall_data_id::TEXT, quantity, maybeboard
```

### 4f. card_count queries — Exclude maybeboard

**get_deck_profile** and **get_deck_profiles** currently compute:
```sql
COALESCE(SUM(dc.quantity), 0) as "card_count"
```

Change to:
```sql
COALESCE(SUM(dc.quantity) FILTER (WHERE dc.maybeboard = false), 0) as "card_count"
```

This ensures card_count only reflects the active deck.

### 4g. count_cards_in_deck — Exclude maybeboard

```sql
SELECT COALESCE(SUM(quantity), 0) FROM deck_cards WHERE deck_id = $1 AND maybeboard = false
```

This is used for limit checks when adding cards.

---

## Step 5: Update HTTP Contracts (zwipe-core)

**File:** `zwipe-core/src/http/contracts/deck_card.rs`

### HttpCreateDeckCard

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpCreateDeckCard {
    pub scryfall_data_id: String,
    pub quantity: i32,
    pub maybeboard: Option<bool>,    // NEW — defaults to false if absent
}
```

Update the `new()` constructor to accept maybeboard. For backwards compatibility, `None` means `false`.

### HttpUpdateDeckCard — No change needed

Quantity delta stays as-is. Maybeboard toggling uses a separate endpoint.

### NEW: HttpToggleMaybeboard

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpToggleMaybeboard {
    pub maybeboard: bool,
}
```

### HttpImportDeckCards — No change needed (maybeboard handling is in the parser)

---

## Step 6: Update Domain Request Types (zwipe-core)

### CreateDeckCard

**File:** `zwipe-core/src/domain/deck/requests/create_deck_card.rs`

Add `maybeboard: bool` field:

```rust
pub struct CreateDeckCard {
    pub user_id: Uuid,
    pub deck_id: Uuid,
    pub scryfall_data_id: Uuid,
    pub quantity: Quantity,
    pub maybeboard: bool,            // NEW
    pub email_verified: bool,
}
```

Update `new()` to accept `maybeboard: Option<bool>`, defaulting to `false`.

### NEW: ToggleMaybeboard

**File:** Create `zwipe-core/src/domain/deck/requests/toggle_maybeboard.rs`

```rust
pub struct ToggleMaybeboard {
    pub user_id: Uuid,
    pub deck_id: Uuid,
    pub scryfall_data_id: Uuid,
    pub maybeboard: bool,
}

pub enum InvalidToggleMaybeboard {
    DeckId(uuid::Error),
    ScryfallDataId(uuid::Error),
}

impl ToggleMaybeboard {
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        scryfall_data_id: &str,
        maybeboard: bool,
    ) -> Result<Self, InvalidToggleMaybeboard>
}
```

Register in `zwipe-core/src/domain/deck/requests/mod.rs`.

### ImportDeckCards

**File:** `zwipe-core/src/domain/deck/requests/import_deck_cards.rs`

Update `ImportLine` to include maybeboard:

```rust
pub struct ImportLine {
    pub quantity: i32,
    pub card_name: String,
    pub maybeboard: bool,           // NEW
}
```

Update the parser to detect `// Maybeboard` header. Lines after the header get `maybeboard: true`:

```rust
impl ImportDeckCards {
    pub fn parse(user_id: Uuid, deck_id: Uuid, text: &str, email_verified: bool) -> Self {
        let mut in_maybeboard = false;
        let mut lines = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();

            // Check for maybeboard section header
            if trimmed.eq_ignore_ascii_case("// maybeboard")
                || trimmed.eq_ignore_ascii_case("//maybeboard")
            {
                in_maybeboard = true;
                continue;
            }

            // Parse "{qty} {name}" as before, but tag with maybeboard flag
            if let Some(import_line) = parse_line(trimmed) {
                lines.push(ImportLine {
                    quantity: import_line.quantity,
                    card_name: import_line.card_name,
                    maybeboard: in_maybeboard,
                });
            }
        }
        // ...
    }
}
```

Update `bulk_create_deck_cards` call to pass the maybeboard flag per card.

---

## Step 7: Add Toggle Maybeboard Endpoint (zerver)

### Handler

**File:** Create `zerver/src/lib/inbound/http/handlers/deck_card/toggle_maybeboard.rs`

```rust
pub async fn toggle_maybeboard<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path((deck_id, scryfall_data_id)): Path<(String, String)>,
    Json(body): Json<HttpToggleMaybeboard>,
) -> Result<(StatusCode, Json<DeckCard>), ApiError>
```

### Route

**File:** `zerver/src/lib/inbound/http/routes.rs`

Add route: `PUT /decks/:deck_id/cards/:scryfall_data_id/maybeboard`

### Service Port

Add `toggle_maybeboard` to the `DeckService` trait and `DeckRepository` trait.

### Error Type

```rust
pub enum ToggleMaybeboardError {
    NotFound,
    Forbidden,
    Database(anyhow::Error),
    DeckCardFromDb(anyhow::Error),
}
```

---

## Step 8: Update DeckMetrics (zwipe-core)

**File:** `zwipe-core/src/domain/deck/models/deck_metrics.rs`

`DeckMetrics::from_entries()` currently processes ALL entries. Add a filter at the top:

```rust
impl DeckMetrics {
    pub fn from_entries(entries: &[DeckEntry]) -> Self {
        // Filter out maybeboard cards
        let active_entries: Vec<&DeckEntry> = entries
            .iter()
            .filter(|e| !e.deck_card.maybeboard)
            .collect();

        // Use active_entries for all computations...
    }
}
```

Alternatively, change the iteration to skip maybeboard inline. The filter-first approach is cleaner.

**Update all tests** that construct DeckEntry/DeckCard to include `maybeboard: false`.

---

## Step 9: Update validate_deck (zwipe-core)

**File:** `zwipe-core/src/domain/deck/models/validate_deck.rs`

Filter entries at the top of `validate_deck`:

```rust
pub fn validate_deck(
    deck_profile: &DeckProfile,
    entries: &[DeckEntry],
    commander_card: Option<&Card>,
) -> Vec<DeckWarning> {
    let Some(format) = &deck_profile.format else {
        return vec![];
    };

    // Only validate active deck cards, not maybeboard
    let active_entries: Vec<&DeckEntry> = entries
        .iter()
        .filter(|e| !e.deck_card.maybeboard)
        .collect();

    let mut warnings = Vec::new();
    // Use &active_entries in all check functions...
}
```

This requires updating the check functions to accept `&[&DeckEntry]` instead of `&[DeckEntry]`. Alternatively, collect into `Vec<DeckEntry>` via cloning, but references are more efficient.

**Simpler approach:** Filter once and pass a slice of references. Update internal functions accordingly.

---

## Step 10: Update Deck Assembly (zerver service layer)

**File:** `zerver/src/lib/domain/deck/services.rs` (or wherever get_deck assembles the Deck aggregate)

The `get_deck` service currently:
1. Gets DeckProfile
2. Gets all DeckCards
3. Fetches Card data for each DeckCard
4. Builds DeckEntry for each
5. Calls validate_deck with all entries
6. Returns Deck { profile, entries, warnings }

No change needed here — it returns ALL entries (active + maybeboard). The DeckCard within each DeckEntry now has the `maybeboard` field, so the frontend can filter. validate_deck already filters internally (Step 9).

---

## Step 11: Run SQLx Prepare + Tests

```bash
cargo sqlx prepare --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

---

## Verification Checklist

- [ ] Existing `create_deck_cards` migration updated with `maybeboard` column (no new migration file)
- [ ] DeckCard has `maybeboard: bool` field
- [ ] DatabaseDeckCard has `maybeboard: bool` with FromRow
- [ ] All RETURNING clauses include `maybeboard`
- [ ] `card_count` SQL uses `FILTER (WHERE maybeboard = false)`
- [ ] `count_cards_in_deck` excludes maybeboard
- [ ] HttpCreateDeckCard accepts optional `maybeboard`
- [ ] CreateDeckCard request includes `maybeboard`
- [ ] ToggleMaybeboard request type exists
- [ ] Toggle endpoint: `PUT /decks/:id/cards/:card_id/maybeboard`
- [ ] Import parser detects `// Maybeboard` header
- [ ] DeckMetrics::from_entries filters out maybeboard
- [ ] validate_deck filters out maybeboard
- [ ] All existing tests updated with `maybeboard: false`
- [ ] `cargo sqlx prepare --workspace` succeeds
- [ ] `cargo clippy` clean

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zerver/migrations/20250810194459_create_deck_cards.sql` | Add `maybeboard` column to existing CREATE TABLE |
| `zwipe-core/.../deck/models/deck_card.rs` | Add `maybeboard: bool` |
| `zwipe-core/.../deck/models/deck_metrics.rs` | Filter out maybeboard in from_entries |
| `zwipe-core/.../deck/models/validate_deck.rs` | Filter out maybeboard at top |
| `zwipe-core/.../deck/requests/create_deck_card.rs` | Add `maybeboard` field |
| `zwipe-core/.../deck/requests/toggle_maybeboard.rs` | **NEW** — toggle request type |
| `zwipe-core/.../deck/requests/import_deck_cards.rs` | Add maybeboard to ImportLine, parse header |
| `zwipe-core/.../deck/requests/mod.rs` | Register toggle_maybeboard |
| `zwipe-core/src/http/contracts/deck_card.rs` | Add maybeboard to create, new toggle contract |
| `zerver/.../outbound/sqlx/deck/models.rs` | Add maybeboard to DatabaseDeckCard |
| `zerver/.../outbound/sqlx/deck/mod.rs` | Update ~7 SQL queries |
| `zerver/.../domain/deck/ports.rs` | Add toggle_maybeboard to traits |
| `zerver/.../domain/deck/services.rs` | Add toggle_maybeboard service method |
| `zerver/.../http/handlers/deck_card/toggle_maybeboard.rs` | **NEW** — handler |
| `zerver/.../http/handlers/deck_card/mod.rs` | Register toggle_maybeboard |
| `zerver/.../http/routes.rs` | Add toggle route |
| `zerver/.../http/handlers/deck_card/create_deck_card.rs` | Pass maybeboard to service |
| `zerver/.../http/handlers/deck_card/import_deck_cards.rs` | Pass maybeboard per card |
