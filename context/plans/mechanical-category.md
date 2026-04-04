# Mechanical Category — Full Plan

Add a multi-tag mechanical category system to cards so users can filter and group by strategic role (ramp, draw, removal, etc.).

---

## Taxonomy (24 Categories)

```rust
pub enum MechanicalCategory {
    Ramp,
    Draw,
    Removal,
    Wipe,
    Counterspell,
    Protection,
    Evasion,
    Finisher,
    Tokens,
    Lifegain,
    Blink,
    Recursion,
    Mill,
    Burn,
    Drain,
    Pump,
    Anthem,
    Counters,
    Copy,
    Sacrifice,
    Stax,
    Untap,
    Tutor,
    GraveyardHate,
}
```

Cards can have **multiple** categories (Sol Ring = Ramp, Swords to Plowshares = Removal, etc.).

---

## Phase 1: Schema + Domain Type (zwipe-core + zerver)

### Step 1: Define MechanicalCategory enum in zwipe-core

**File:** Create `zwipe-core/src/domain/card/models/mechanical_category.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MechanicalCategory {
    Ramp,
    Draw,
    Removal,
    Wipe,
    Counterspell,
    Protection,
    Evasion,
    Finisher,
    Tokens,
    Lifegain,
    Blink,
    Recursion,
    Mill,
    Burn,
    Drain,
    Pump,
    Anthem,
    Counters,
    Copy,
    Sacrifice,
    Stax,
    Untap,
    Tutor,
    GraveyardHate,
}
```

Add:
- `Display` impl returning the snake_case name (e.g., "graveyard_hate")
- `display_name()` returning human-readable (e.g., "Graveyard Hate")
- `all()` returning `&'static [MechanicalCategory]` (all 24 variants)
- `TryFrom<&str>` for parsing from strings
- Round-trip serde tests

Register the module in `zwipe-core/src/domain/card/models/mod.rs`.

### Step 2: Update database schema

**File:** `zerver/migrations/20250810194451_create_card_profiles.sql`

**The database is not live.** Modify the existing migration to add the column directly:

```sql
CREATE TABLE card_profiles (
    scryfall_data_id UUID PRIMARY KEY,
    is_token BOOLEAN NOT NULL DEFAULT FALSE,
    mechanical_categories JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_scryfall_data
        FOREIGN KEY (scryfall_data_id)
        REFERENCES scryfall_data (id)
        ON DELETE RESTRICT
);

CREATE INDEX idx_card_profiles_is_token ON card_profiles(is_token);
CREATE INDEX idx_card_profiles_categories ON card_profiles USING GIN(mechanical_categories);
```

Using JSONB array (e.g., `["ramp", "draw"]`) with a GIN index for efficient `@>` containment queries.

**Important:** After modifying the migration, reset and re-run:
```bash
sqlx database drop && sqlx database create && sqlx migrate run
```
Then re-sync via zervice and `cargo sqlx prepare --workspace`.

### Step 3: Update CardProfile domain type

**File:** `zwipe-core/src/domain/card/models/card_profile.rs`

Add the field:

```rust
pub struct CardProfile {
    pub scryfall_data_id: Uuid,
    pub is_token: bool,
    pub mechanical_categories: Vec<MechanicalCategory>,  // NEW
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
}
```

Default to empty vec for uncategorized cards.

### Step 4: Update DatabaseCardProfile adapter

**File:** `zerver/src/lib/outbound/sqlx/card/card_profile.rs`

Add the JSONB field:

```rust
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseCardProfile {
    pub scryfall_data_id: Uuid,
    pub is_token: bool,
    pub mechanical_categories: Option<Json<Vec<String>>>,  // NEW
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

Update `TryFrom<DatabaseCardProfile> for CardProfile` to parse the strings into `MechanicalCategory` variants, silently skipping unrecognized strings (forward compatibility).

### Step 5: Update all CardProfile SQL queries

Every query that returns `DatabaseCardProfile` must now SELECT `mechanical_categories`. Search for `card_profiles` in the SQL queries in `zerver/src/lib/outbound/sqlx/card/`.

---

## Phase 2: Filtering + Grouping (zwipe-core + zerver)

### Step 1: Add filter to CardFilter

**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/mod.rs`

```rust
// mechanical category
mechanical_categories_contains_any: Option<Vec<MechanicalCategory>>,
mechanical_categories_contains_all: Option<Vec<MechanicalCategory>>,
```

Add getters, builder field, setters, unsetters — follow the `keywords_contains_any/all` pattern exactly.

### Step 2: Add SQL filter

**File:** `zerver/src/lib/outbound/sqlx/card/mod.rs`

In `search_scryfall_data`, the card_profiles table is already JOINed. Add:

```rust
if let Some(categories) = request.mechanical_categories_contains_any() {
    let cat_strings: Vec<String> = categories.iter().map(|c| c.to_string()).collect();
    sep.push("card_profiles.mechanical_categories ?| ");
    sep.push_bind_unseparated(cat_strings);
}

if let Some(categories) = request.mechanical_categories_contains_all() {
    let cat_strings: Vec<String> = categories.iter().map(|c| c.to_string()).collect();
    sep.push("card_profiles.mechanical_categories @> ");
    sep.push_bind_unseparated(serde_json::to_value(cat_strings).unwrap());
}
```

The `?|` operator checks if the JSONB array contains ANY of the values. The `@>` operator checks if it contains ALL. Both use the GIN index.

### Step 3: Add GroupByOption::Category

**File:** `zwipe-core/src/domain/card/models/search_card/group_cards.rs`

Add variant:

```rust
pub enum GroupByOption {
    CardType,
    Cmc,
    Color,
    Category,  // NEW
}
```

Update `all()`, `Display`, and add `classify_category` function.

**Grouping logic:** Since cards can have multiple categories, a card tagged `["ramp", "draw"]` appears in BOTH the "ramp" and "draw" groups. The `classify` function returns a single bucket index — for multi-category, the card gets cloned into each matching bucket.

This requires changing the grouping implementation. Currently `classify` returns one `usize`. For Category grouping, use a different path:

```rust
GroupByOption::Category => {
    // Multi-bucket: card appears in every category it belongs to
    let labels = MechanicalCategory::all()
        .iter()
        .map(|c| c.display_name())
        .collect();
    // ... iterate cards, for each card check its categories,
    // push into each matching bucket
}
```

Cards with NO categories go into an "uncategorized" bucket.

### Step 4: Add DeckMetrics category breakdown

**File:** `zwipe-core/src/domain/deck/models/deck_metrics.rs`

Add a field:

```rust
pub struct DeckMetrics {
    // ... existing fields
    pub category_counts: Vec<(MechanicalCategory, usize)>,  // NEW
}
```

In `from_entries`, count categories (accounting for quantity and multi-tag):

```rust
let mut cat_map: HashMap<MechanicalCategory, usize> = HashMap::new();
for entry in active_entries {
    let qty = *entry.deck_card.quantity as usize;
    for cat in &entry.card.card_profile.mechanical_categories {
        *cat_map.entry(*cat).or_default() += qty;
    }
}
// Sort by count descending, convert to vec
```

This gives output like: `[("removal", 8), ("ramp", 12), ("draw", 10)]`.

---

## Phase 3: Frontend — Filter + Group + Metrics Display

### Step 1: Add Category filter section to CardFilterSheet

**File:** `zwiper/src/lib/inbound/screens/deck/card/filter/` — Create new `category.rs`

A chip-based multi-select section showing all 24 categories. Selecting chips sets `mechanical_categories_contains_any` on the filter builder.

Register in `card_filter_sheet.rs` as a new accordion section.

### Step 2: Add Category chip to grouping bar

**File:** `zwiper/src/lib/inbound/screens/deck/card/view.rs`

Add "category" to the grouping chip row alongside type/cmc/color.

### Step 3: Display category breakdown in deck stats

**File:** `zwiper/src/lib/inbound/screens/deck/components/deck_stats.rs`

Add a collapsible section showing category counts from `DeckMetrics.category_counts`. Display as a compact list:

```
removal: 8  |  ramp: 12  |  draw: 10  |  ...
```

Or as a small horizontal bar chart — follow existing chart patterns.

---

## Classification Strategy: Three Layers

Categories are populated via a layered approach, each layer correcting and improving the last. The JSONB column is the single source of truth regardless of how the tags were generated.

### Layer 1: Runtime Oracle Text Heuristics (70-80% accuracy)

Built into the app at launch. Pure pattern matching on oracle_text, type_line, keywords, and produced_mana. Runs at query time (or on card sync) with no external dependencies.

**Examples of heuristic rules:**
- **Ramp:** `produced_mana` is non-empty AND (type contains "Creature" or "Artifact") AND cmc <= 3; OR oracle_text matches "search your library for a.*land"
- **Draw:** oracle_text contains "draw" near "card"
- **Removal:** oracle_text contains "destroy target" or "exile target" (but NOT "destroy all")
- **Wipe:** oracle_text contains "destroy all" or "exile all" or "each creature gets -X/-X"
- **Counterspell:** type is Instant AND oracle_text contains "counter target spell"
- **Burn:** oracle_text contains "deals * damage to" AND color identity includes Red
- **Drain:** oracle_text contains both "loses * life" and "gains * life"
- **Tutor:** oracle_text contains "search your library"
- **Tokens:** oracle_text contains "create" near "token"
- **Mill:** oracle_text contains "mills" or "top * cards * into * graveyard"

These heuristics are imperfect — they'll miss cards with unusual phrasing and may false-positive on edge cases. That's fine. Layer 2 corrects them.

**Implementation:** Build as a pure function in zwipe-core: `fn classify_by_heuristics(card: &Card) -> Vec<MechanicalCategory>`. Run during Scryfall sync in zervice to pre-populate the JSONB column for cards that don't already have AI-assigned tags. Cards with existing AI tags are NOT overwritten.

### Layer 2: AI Classification Client (90-95% accuracy)

A separate crate/binary (`zort`) that connects to the database, reads cards in batches, and uses an LLM to classify them. Writes tags back via UPDATE queries.

**Architecture:**
- Standalone Rust binary in its own workspace crate (`zort/`)
- Reads cards from Postgres directly (needs DATABASE_URL)
- Sends batches of (name, type_line, oracle_text) to LLM API
- Receives category tags per card
- UPDATEs card_profiles.mechanical_categories via SQL
- Tracks which cards it has classified (only processes unclassified or changed cards)

**Operations:**
- `zort classify` — classify all untagged cards (initial run)
- `zort reclassify` — re-classify all cards (full re-run after taxonomy changes)
- `zort delta` — classify only cards updated since last run
- `zort audit` — compare heuristic tags vs AI tags, report discrepancies

**LLM prompt:** Send the 24-category taxonomy with definitions + batch of cards. Receive JSON mapping card name → category array. Use Claude Haiku for cost efficiency.

**Cost:** ~35k cards / 100 per batch = ~350 API calls. ~$5-15 for full classification.

### Layer 3: Fine-Tuned Lightweight Model (95-99% accuracy, future)

Long-term goal: train a small, specialized model on MTG card classification data. Input: oracle_text + type_line. Output: category tags.

**Training data:** Use Layer 2's AI-generated tags as training labels, with human review on a sample for quality. The 35k card corpus with AI-assigned labels becomes the training set.

**Advantages over LLM API:**
- Runs locally (no API costs, no latency)
- Can be embedded in zervice sync pipeline
- Consistent results (no prompt sensitivity)
- Fast enough for real-time classification of new cards

**When to build:** After Layer 2 has been running and tags have been reviewed/corrected for a cycle. The corrected tags become high-quality training data.

---

## Verification Checklist

### Phase 1 (Schema)
- [ ] `MechanicalCategory` enum with 24 variants in zwipe-core
- [ ] Serde round-trip, Display, TryFrom, all() tests
- [ ] Existing `create_card_profiles` migration updated with `mechanical_categories JSONB`
- [ ] GIN index on mechanical_categories
- [ ] CardProfile has `mechanical_categories: Vec<MechanicalCategory>`
- [ ] DatabaseCardProfile has JSONB field with TryFrom conversion
- [ ] All card_profiles SQL queries select new column
- [ ] sqlx prepare succeeds

### Phase 2 (Filter + Group)
- [ ] CardFilter has `mechanical_categories_contains_any/all`
- [ ] SQL uses `?|` and `@>` operators with GIN index
- [ ] GroupByOption::Category works (multi-bucket)
- [ ] DeckMetrics has category_counts
- [ ] Tests for grouping, filtering, metrics

### Phase 3 (Frontend)
- [ ] Category filter section in CardFilterSheet
- [ ] "category" grouping chip on deck card view
- [ ] Category breakdown in deck stats

### Layer 1 (Heuristics)
- [ ] `classify_by_heuristics(card) -> Vec<MechanicalCategory>` pure function in zwipe-core
- [ ] Heuristic rules for all 24 categories
- [ ] Integrated into zervice sync (pre-populate JSONB, don't overwrite AI tags)
- [ ] Tests with known cards (Sol Ring → ramp, Swords → removal, etc.)

### Layer 2 (AI Client)
- [ ] Standalone binary with DATABASE_URL access
- [ ] Batch read → LLM classify → batch UPDATE pipeline
- [ ] classify / reclassify / delta / audit subcommands
- [ ] Prompt with 24-category taxonomy
- [ ] Does not overwrite human-reviewed tags (future flag)

### Layer 3 (Fine-Tuned Model — Future)
- [ ] Training data export from Layer 2 tags
- [ ] Model training pipeline
- [ ] Embedded in zervice for real-time classification

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwipe-core/.../card/models/mechanical_category.rs` | **NEW** — enum + Display + TryFrom + all() |
| `zwipe-core/.../card/models/mod.rs` | Register module |
| `zwipe-core/.../card/models/card_profile.rs` | Add `mechanical_categories` field |
| `zwipe-core/.../search_card/card_filter/mod.rs` | Add 2 filter fields |
| `zwipe-core/.../search_card/card_filter/builder/*.rs` | Setters, getters |
| `zwipe-core/.../search_card/card_filter/getters.rs` | Getters |
| `zwipe-core/.../search_card/group_cards.rs` | Add Category variant, multi-bucket logic |
| `zwipe-core/.../deck/models/deck_metrics.rs` | Add category_counts |
| `zerver/migrations/20250810194451_create_card_profiles.sql` | Add column + GIN index |
| `zerver/.../outbound/sqlx/card/card_profile.rs` | Add JSONB field + conversion |
| `zerver/.../outbound/sqlx/card/mod.rs` | Add SQL filter, update SELECT queries |
| `zwiper/.../filter/category.rs` | **NEW** — category filter section |
| `zwiper/.../filter/card_filter_sheet.rs` | Register category section |
| `zwiper/.../deck/card/view.rs` | Add Category grouping chip |
| `zwiper/.../deck/components/deck_stats.rs` | Add category breakdown display |
| `zwipe-core/.../card/models/mechanical_category.rs` | Add `classify_by_heuristics()` function |
| `zerver/src/bin/zervice.rs` | Integrate heuristic classification into sync pipeline |
| `zort/src/main.rs` (separate crate) | **NEW** — AI classification client binary |
