# Multi-Printing Support + Oracle ID Deck Constraint

Switch Scryfall sync from `oracle_cards` to `default_cards` to import tokens and all printings. Change `deck_cards` unique constraint from `(deck_id, scryfall_data_id)` to `(deck_id, oracle_id)` so a deck can only have one logical card regardless of printing, while still tracking which printing is selected.

---

## Context

- Token cards referenced in `all_parts` don't exist in the database because `oracle_cards` excludes them.
- `default_cards` includes all printings + tokens — search already deduplicates via `ROW_NUMBER() OVER (PARTITION BY oracle_id ORDER BY released_at DESC)`.
- Long-term: users will be able to select preferred printings (art/set). Default is latest printing.

---

## Phase 1: Switch Scryfall Sync to Default Cards

### Step 1: Change sync endpoint

**File:** `zerver/src/bin/zervice.rs` — line 63

```rust
// Before
card_service.scryfall_sync(BulkEndpoint::OracleCards).await?;
// After
card_service.scryfall_sync(BulkEndpoint::DefaultCards).await?;
```

One-line change. `BulkEndpoint::DefaultCards` is already defined in `zerver/src/lib/inbound/external/scryfall/bulk.rs`.

### Step 2: Run sync and verify

```bash
cargo run --bin zervice
```

This will take longer than usual — `default_cards` is ~80k cards vs ~30k for `oracle_cards`. The existing `batch_delta_upsert` handles it (upserts on `scryfall_data.id`). New printings get new rows. Existing rows get updated if data changed.

### Step 3: Verify tokens exist

After sync, check that token cards are now in the database:

```sql
SELECT COUNT(*) FROM scryfall_data WHERE layout = 'token';
SELECT sd.name FROM scryfall_data sd
JOIN card_profiles cp ON sd.id = cp.scryfall_data_id
WHERE cp.is_token = true
LIMIT 10;
```

### Step 4: Verify search dedup still works

Existing search queries already deduplicate by `oracle_id`:
- `search_scryfall_data` in `zerver/src/lib/outbound/sqlx/card/mod.rs:183`
- `find_cards_by_exact_names` in `zerver/src/lib/outbound/sqlx/card/mod.rs:800`

Both use `PARTITION BY COALESCE(scryfall_data.oracle_id, scryfall_data.id) ORDER BY released_at DESC` and take `rn = 1`. No changes needed — users will still see one card per search result (latest printing).

---

## Phase 2: Oracle ID Deck Constraint

### Step 1: Update deck_cards migration

**File:** `zerver/migrations/20250810194459_create_deck_cards.sql`

**Database is not live** — modify the existing migration in place.

```sql
CREATE TABLE deck_cards (
    deck_id UUID NOT NULL,
    scryfall_data_id UUID NOT NULL,
    oracle_id UUID NOT NULL,
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
    CONSTRAINT deck_card_oracle_unique UNIQUE (deck_id, oracle_id),
    CONSTRAINT positive_quantity CHECK (quantity > 0)
);

CREATE INDEX idx_deck_cards_oracle_id ON deck_cards(oracle_id);
```

**Key changes:**
- Added `oracle_id UUID NOT NULL` column
- Changed unique constraint from `(deck_id, scryfall_data_id)` to `(deck_id, oracle_id)`
- Added index on `oracle_id` for lookups
- `scryfall_data_id` remains as FK — it points to the selected printing
- No FK on `oracle_id` to `scryfall_data.oracle_id` — oracle_id is a Scryfall concept, not a PK in our schema

**After modifying:**
```bash
sqlx database drop && sqlx database create && sqlx migrate run
cargo run --bin zervice  # re-sync cards
cargo sqlx prepare --workspace
```

### Step 2: Update DeckCard domain model

**File:** `zwipe-core/src/domain/deck/models/deck_card.rs`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeckCard {
    pub deck_id: Uuid,
    pub scryfall_data_id: Uuid,
    pub oracle_id: Uuid,             // NEW
    pub quantity: Quantity,
    pub maybeboard: bool,
}
```

### Step 3: Update DatabaseDeckCard

**File:** `zerver/src/lib/outbound/sqlx/deck/models.rs`

```rust
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckCard {
    pub deck_id: String,
    pub scryfall_data_id: String,
    pub oracle_id: String,            // NEW
    pub quantity: i32,
    pub maybeboard: bool,
}
```

Update `TryFrom<DatabaseDeckCard> for DeckCard` to parse `oracle_id`.

### Step 4: Update CreateDeckCard request

**File:** `zwipe-core/src/domain/deck/requests/create_deck_card.rs`

Add `oracle_id: Uuid` to the struct. The caller must resolve this before creating the request.

```rust
pub struct CreateDeckCard {
    pub user_id: Uuid,
    pub deck_id: Uuid,
    pub scryfall_data_id: Uuid,
    pub oracle_id: Uuid,             // NEW
    pub quantity: Quantity,
    pub maybeboard: bool,
    pub email_verified: bool,
}
```

### Step 5: Update HTTP contract

**File:** `zwipe-core/src/http/contracts/deck_card.rs`

The frontend sends `scryfall_data_id` — it doesn't know about `oracle_id`. The backend must resolve it.

Option A: Frontend also sends `oracle_id` (requires the Card object to carry it, which it does — `card.scryfall_data.oracle_id`).

Option B: Backend resolves `oracle_id` from `scryfall_data` before inserting.

**Recommendation: Option A.** The frontend already has the full `Card` object with `scryfall_data.oracle_id`. Add it to the HTTP contract:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpCreateDeckCard {
    pub scryfall_data_id: String,
    pub oracle_id: String,           // NEW
    pub quantity: i32,
    pub maybeboard: Option<bool>,
}
```

This avoids a server-side lookup per insert. The backend validates the oracle_id matches the scryfall_data record if paranoid, but for now trust the client.

### Step 6: Update all repository SQL queries

**File:** `zerver/src/lib/outbound/sqlx/deck/mod.rs`

#### create_deck_card (line ~97)
```sql
INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, maybeboard)
VALUES ($1, $2, $3, $4, $5)
RETURNING deck_id, scryfall_data_id, oracle_id, quantity, maybeboard
```

#### get_deck_cards (line ~186)
```sql
SELECT deck_id, scryfall_data_id, oracle_id, quantity, maybeboard
FROM deck_cards WHERE deck_id = $1
```

#### update_deck_card (line ~289)
WHERE clause stays `deck_id = $X AND scryfall_data_id = $Y` — updating by specific printing is fine. RETURNING must include `oracle_id`.

#### delete_deck_card (line ~331)
WHERE clause stays `deck_id = $X AND scryfall_data_id = $Y`. Deleting by specific printing is correct.

#### bulk_create_deck_cards / import (line ~366)
```sql
INSERT INTO deck_cards (deck_id, scryfall_data_id, oracle_id, quantity, maybeboard)
VALUES (...)
ON CONFLICT (deck_id, oracle_id) DO UPDATE
    SET scryfall_data_id = EXCLUDED.scryfall_data_id,
        quantity = EXCLUDED.quantity,
        maybeboard = EXCLUDED.maybeboard
RETURNING deck_id::TEXT, scryfall_data_id::TEXT, oracle_id::TEXT, quantity, maybeboard
```

**Important:** ON CONFLICT now targets `(deck_id, oracle_id)`. If a user re-imports a card that's already in the deck with a different printing, the printing gets updated to the import's resolved printing (latest by default). This is correct behavior.

#### count_cards_in_deck (line ~110)
No change needed — counts by `deck_id` regardless.

#### card_count in get_deck_profile
No change needed — SUM(quantity) doesn't reference oracle_id.

### Step 7: Update import service flow

**File:** `zerver/src/lib/domain/deck/services.rs` — lines 260-351

The import currently resolves card names to `(scryfall_data_id, quantity)`. It needs to also resolve `oracle_id`.

The `find_cards_by_exact_names` query already returns full `Card` objects which include `scryfall_data.oracle_id`. Update the batch building logic:

```rust
// Before: Vec<(Uuid, i32)> — (scryfall_data_id, quantity)
// After:  Vec<(Uuid, Uuid, i32)> — (scryfall_data_id, oracle_id, quantity)
```

Update `bulk_create_deck_cards` to accept and use the oracle_id.

### Step 8: Update frontend add screen

**File:** `zwiper/src/lib/inbound/screens/deck/card/add.rs` — line 215

The `add_card_to_deck` closure creates `HttpCreateDeckCard`. It needs to include `oracle_id`:

```rust
let request = HttpCreateDeckCard::new(
    &card.scryfall_data.id.to_string(),
    &card.scryfall_data.oracle_id.map(|id| id.to_string()).unwrap_or_default(),
    1,
);
```

The `Card` object already has `scryfall_data.oracle_id` available.

### Step 9: Update create_deck_card handler

**File:** `zerver/src/lib/inbound/http/handlers/deck_card/create_deck_card.rs`

Pass `oracle_id` from the HTTP body through to `CreateDeckCard::new()`.

### Step 10: Duplicate detection on add screen

**File:** `zwiper/src/lib/inbound/screens/deck/card/add.rs`

Currently `deck_cards_ids` is a `HashSet<Uuid>` of `scryfall_data_id`s used to filter out cards already in the deck. With multi-printing, the same logical card could have a different `scryfall_data_id` in the deck vs search results.

**Fix:** Change `deck_cards_ids` to track `oracle_id` instead of `scryfall_data_id`:

```rust
let mut deck_cards_oracle_ids = use_signal(HashSet::<Uuid>::new);

// On deck load:
let mut oracle_ids: HashSet<_> = deck
    .entries
    .iter()
    .filter_map(|entry| entry.card.scryfall_data.oracle_id)
    .collect();
deck_cards_oracle_ids.set(oracle_ids);

// In search result filtering:
.filter(|card| {
    card.scryfall_data.oracle_id
        .map(|oid| !deck_oracle_ids.contains(&oid))
        .unwrap_or(true)
})
```

This ensures a card is excluded from search results even if the deck contains a different printing of it.

---

## Phase 3: Printing Selector UI

Allow users to browse all printings of a card and select which one to use in their deck. Affects price accuracy (different printings have different prices) and lets users pick preferred art.

### Backend: Get Printings Endpoint

**New endpoint:** `GET /api/cards/:oracle_id/printings`

Returns all `ScryfallData` rows sharing the given `oracle_id`, ordered by `released_at DESC` (newest first). The search dedup CTE already does `PARTITION BY oracle_id` — this is the inverse: return ALL rows for one oracle_id instead of one row per oracle_id.

**File:** `zerver/src/lib/outbound/sqlx/card/mod.rs` — add query:

```sql
SELECT sd.* FROM scryfall_data sd
JOIN card_profiles cp ON sd.id = cp.scryfall_data_id
WHERE sd.oracle_id = $1
ORDER BY sd.released_at DESC
```

**File:** `zerver/src/lib/inbound/http/handlers/card/` — add handler
**File:** `zerver/src/lib/inbound/http/routes.rs` — add route
**File:** `zwipe-core/src/http/paths.rs` — add path constant

### Frontend Client

**File:** Create `zwiper/src/lib/outbound/client/card/get_printings.rs`

```rust
pub trait ClientGetPrintings {
    fn get_printings(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ApiError>> + Send;
}
```

### Frontend: Printing Bottom Sheet

**Location:** Deck card view screen → expanded CardRow → tap image button

Currently tapping the image button opens an `ImagePreview` modal. Change behavior:

1. **Tap "printing" button** (rename from "image") → opens a **PrintingSheet** bottom sheet
2. Bottom sheet shows:
   - Current printing's image (large, centered)
   - Set name + collector number + release year below the image
   - Price for this printing (USD/EUR/TIX)
   - **Horizontal sliding row** of all other printings (thumbnail-sized card images)
3. User slides through printings — tapping one selects it
4. Selection sends `UpdateDeckCard` request with the new `scryfall_data_id` (oracle_id stays the same on the row)
5. Toast: "printing updated" on success
6. Local state updates optimistically — the card image in the deck list reflects the new printing

### UpdateDeckCard for Printing Change

**File:** `zwipe-core/src/http/contracts/deck_card.rs`

`HttpUpdateDeckCard` currently has `update_quantity` and `maybeboard`. Add optional `scryfall_data_id`:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpUpdateDeckCard {
    pub update_quantity: Option<i32>,
    pub maybeboard: Option<bool>,
    pub scryfall_data_id: Option<String>,  // NEW — change printing
}
```

When `scryfall_data_id` is present, the backend updates the printing:

```sql
UPDATE deck_cards SET scryfall_data_id = $1 WHERE deck_id = $2 AND oracle_id = $3
```

**Important:** The WHERE clause uses `oracle_id` (not old `scryfall_data_id`) since that's the stable identifier. The backend should verify the new `scryfall_data_id` has the same `oracle_id` as the existing row to prevent mismatches.

### PrintingSheet Component

**File:** Create `zwiper/src/lib/inbound/screens/deck/card/components/printing_sheet.rs`

**Props:**
```rust
pub struct PrintingSheetProps {
    pub card: Card,                          // Current card (with current printing)
    pub deck_id: Uuid,
    pub open: Signal<bool>,
    pub on_printing_changed: EventHandler<Card>,  // Callback with new printing
}
```

**Behavior:**
1. On open: fetch printings via `client.get_printings(oracle_id, &session)`
2. Cache results in signal (don't re-fetch on every open for same card)
3. Show current printing highlighted in the sliding row
4. On selection:
   - Call `update_deck_card` with new `scryfall_data_id`
   - On success: fire `on_printing_changed` with the new Card
   - Parent updates local state (swap the Card in deck_cards/quantity_map)
5. On close: dismiss

**Sliding row layout:**
```
┌──────────────────────────────────────────┐
│         [Current Printing Image]          │
│     Innistrad: Midnight Hunt · #123       │
│          USD $1.50 · EUR €1.30            │
├──────────────────────────────────────────┤
│ [thumb] [thumb] [SELECTED] [thumb] [thumb]│  ← horizontal scroll
│  M21     MH2     MID        2XM    SLD    │
└──────────────────────────────────────────┘
```

### Price Impact

Changing printings automatically updates the deck's price stats since `DeckMetrics::from_entries()` reads `scryfall_data.prices` from each entry's Card. When the printing changes, the Card in the DeckEntry has different prices, and the next metrics computation reflects it. No extra work needed — just swap the Card data in local state.

### Verification Checklist (Phase 3)

- [ ] `GET /api/cards/:oracle_id/printings` endpoint returns all printings
- [ ] Frontend client method for fetching printings
- [ ] "printing" button on expanded CardRow (replaces or sits alongside "image")
- [ ] PrintingSheet bottom sheet opens with current printing highlighted
- [ ] All printings load as thumbnails in horizontal sliding row
- [ ] Tapping a printing updates the deck card via `update_deck_card`
- [ ] Local state updates optimistically (image, prices reflect new printing)
- [ ] Toast "printing updated" on success
- [ ] Price stats update after printing change
- [ ] Backend validates new scryfall_data_id shares oracle_id with existing row

### Phase 3 Files

| File | Change |
|------|--------|
| `zerver/.../outbound/sqlx/card/mod.rs` | Add get_printings query |
| `zerver/.../inbound/http/handlers/card/` | Add get_printings handler |
| `zerver/.../inbound/http/routes.rs` | Add printings route |
| `zwipe-core/src/http/paths.rs` | Add path constant |
| `zwipe-core/src/http/contracts/deck_card.rs` | Add `scryfall_data_id` to HttpUpdateDeckCard |
| `zerver/.../outbound/sqlx/deck/mod.rs` | Handle scryfall_data_id update in update_deck_card |
| `zwiper/.../outbound/client/card/get_printings.rs` | **NEW** — client method |
| `zwiper/.../deck/card/components/printing_sheet.rs` | **NEW** — bottom sheet component |
| `zwiper/.../deck/card/components/card_row.rs` | "printing" button triggers sheet |
| `zwiper/.../deck/card/view.rs` | Wire printing sheet, handle on_printing_changed |

---

## Verification Checklist

### Phase 1 (Sync)
- [ ] Zervice uses `BulkEndpoint::DefaultCards`
- [ ] Sync completes (~80k cards)
- [ ] Token cards exist in database
- [ ] Search still returns one result per logical card (dedup works)
- [ ] Card profiles created for new printings

### Phase 2 (Constraint)
- [ ] `deck_cards` has `oracle_id UUID NOT NULL` column
- [ ] Unique constraint is `(deck_id, oracle_id)`
- [ ] Index on `oracle_id`
- [ ] DeckCard domain model has `oracle_id`
- [ ] DatabaseDeckCard has `oracle_id` with TryFrom
- [ ] HttpCreateDeckCard includes `oracle_id`
- [ ] CreateDeckCard request includes `oracle_id`
- [ ] All RETURNING clauses include `oracle_id`
- [ ] Import resolves and passes `oracle_id` through
- [ ] ON CONFLICT targets `(deck_id, oracle_id)`
- [ ] Add screen filters search results by `oracle_id` not `scryfall_data_id`
- [ ] `cargo sqlx prepare --workspace` succeeds
- [ ] All existing tests updated
- [ ] `cargo clippy` clean

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zerver/src/bin/zervice.rs` | `OracleCards` → `DefaultCards` |
| `zerver/migrations/20250810194459_create_deck_cards.sql` | Add `oracle_id`, change unique constraint |
| `zwipe-core/.../deck/models/deck_card.rs` | Add `oracle_id: Uuid` |
| `zwipe-core/.../deck/requests/create_deck_card.rs` | Add `oracle_id: Uuid` |
| `zwipe-core/src/http/contracts/deck_card.rs` | Add `oracle_id` to HttpCreateDeckCard |
| `zerver/.../outbound/sqlx/deck/models.rs` | Add `oracle_id` to DatabaseDeckCard + TryFrom |
| `zerver/.../outbound/sqlx/deck/mod.rs` | Update ~6 SQL queries |
| `zerver/.../domain/deck/services.rs` | Import flow passes oracle_id |
| `zerver/.../http/handlers/deck_card/create_deck_card.rs` | Pass oracle_id |
| `zerver/.../http/handlers/deck_card/import_deck_cards.rs` | Pass oracle_id |
| `zwiper/.../deck/card/add.rs` | Send oracle_id, filter by oracle_id |
| `zwiper/.../outbound/client/deck_card/create_deck_card.rs` | Send oracle_id in request |
