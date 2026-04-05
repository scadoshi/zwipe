# Sideboard Support

Migrate from `maybeboard: bool` to a `board` enum column supporting three states: deck, maybeboard, sideboard. Add sideboard UI to deck card view, remove screen, export/import. No sideboard on swipe add screen — manage from deck view only.

---

## Design Decisions

- **Enum column, not second boolean.** `board TEXT NOT NULL DEFAULT 'deck'` replaces `maybeboard BOOLEAN`. Prevents invalid states (both true). One column, three states.
- **No sideboard on add screen.** Sideboard is a deliberate, late-stage decision. Manage via move buttons in deck card view.
- **Sideboard excluded from metrics, validation, card_count** — same treatment as maybeboard.
- **Format-specific rules:** 15-card sideboard limit for 60-card formats (Standard, Modern, Legacy, Vintage, Pioneer, Pauper). No sideboard for Commander/Brawl/Oathbreaker (singleton formats).
- **Export/import:** `// Sideboard` header — standard format recognized by Moxfield, Archidekt, MTGO.

---

## Phase 1: Data Model Migration (zwipe-core + zerver)

### Step 1: Define Board enum in zwipe-core

**File:** Create `zwipe-core/src/domain/deck/models/board.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Board {
    #[default]
    Deck,
    Maybeboard,
    Sideboard,
}

impl Board {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Deck)
    }

    pub fn is_maybeboard(&self) -> bool {
        matches!(self, Self::Maybeboard)
    }

    pub fn is_sideboard(&self) -> bool {
        matches!(self, Self::Sideboard)
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Deck => "deck",
            Self::Maybeboard => "maybeboard",
            Self::Sideboard => "sideboard",
        }
    }
}
```

Add `TryFrom<&str>` for parsing from database TEXT column. Register in `zwipe-core/src/domain/deck/models/mod.rs`.

### Step 2: Replace `maybeboard: bool` with `board: Board` on DeckCard

**File:** `zwipe-core/src/domain/deck/models/deck_card.rs`

```rust
pub struct DeckCard {
    pub deck_id: Uuid,
    pub scryfall_data_id: Uuid,
    pub oracle_id: Uuid,
    pub quantity: Quantity,
    pub board: Board,       // REPLACES maybeboard: bool
}
```

### Step 3: Update migration

**File:** `zerver/migrations/20250810194459_create_deck_cards.sql`

**Database is not live** — modify existing migration:

```sql
CREATE TABLE deck_cards (
    deck_id UUID NOT NULL,
    scryfall_data_id UUID NOT NULL,
    oracle_id UUID NOT NULL,
    quantity INT NOT NULL,
    board TEXT NOT NULL DEFAULT 'deck',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_deck
        FOREIGN KEY (deck_id)
        REFERENCES decks (id) ON DELETE CASCADE,
    CONSTRAINT fk_scryfall_data_id
        FOREIGN KEY (scryfall_data_id)
        REFERENCES scryfall_data (id) ON DELETE CASCADE,
    CONSTRAINT deck_card_oracle_unique UNIQUE (deck_id, oracle_id),
    CONSTRAINT positive_quantity CHECK (quantity > 0),
    CONSTRAINT valid_board CHECK (board IN ('deck', 'maybeboard', 'sideboard'))
);

CREATE INDEX idx_deck_cards_oracle_id ON deck_cards(oracle_id);
```

After modifying: `sqlx database drop && sqlx database create && sqlx migrate run`, re-sync, `cargo sqlx prepare --workspace`.

### Step 4: Update DatabaseDeckCard

**File:** `zerver/src/lib/outbound/sqlx/deck/models.rs`

```rust
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseDeckCard {
    pub deck_id: String,
    pub scryfall_data_id: String,
    pub oracle_id: String,
    pub quantity: i32,
    pub board: String,      // REPLACES maybeboard: bool
}
```

Update `TryFrom<DatabaseDeckCard> for DeckCard` to parse `board` via `Board::try_from(value.board.as_str())`.

### Step 5: Update all SQL queries

**File:** `zerver/src/lib/outbound/sqlx/deck/mod.rs`

Every query touching `maybeboard` changes to `board`:

| Query | Before | After |
|-------|--------|-------|
| `create_deck_card` INSERT | `maybeboard` column, bind bool | `board` column, bind string (`request.board.display_name()`) |
| `get_deck_cards` SELECT | `maybeboard` | `board` |
| `update_deck_card` SET | `maybeboard = $N` (bool) | `board = $N` (text) |
| `bulk_create_deck_cards` INSERT | `.push_bind(false)` | `.push_bind("deck")` |
| `bulk_create_deck_cards` ON CONFLICT | `maybeboard = EXCLUDED.maybeboard` | `board = EXCLUDED.board` |
| `count_cards_in_deck` WHERE | `maybeboard = false` | `board = 'deck'` |
| `get_deck_profile` FILTER | `WHERE dc.maybeboard = false` | `WHERE dc.board = 'deck'` |
| `get_deck_profiles` FILTER | Same | Same |
| All RETURNING clauses | `maybeboard` | `board` |

### Step 6: Update HTTP contracts

**File:** `zwipe-core/src/http/contracts/deck_card.rs`

**HttpCreateDeckCard:**
```rust
pub struct HttpCreateDeckCard {
    pub scryfall_data_id: String,
    pub oracle_id: String,
    pub quantity: i32,
    pub board: Option<String>,  // REPLACES maybeboard: Option<bool>
                                 // None = "deck", "maybeboard", "sideboard"
}
```

**HttpUpdateDeckCard:**
```rust
pub struct HttpUpdateDeckCard {
    pub update_quantity: Option<i32>,
    pub board: Option<String>,  // REPLACES maybeboard: Option<bool>
}
```

### Step 7: Update domain request types

**File:** `zwipe-core/src/domain/deck/requests/create_deck_card.rs`

Replace `maybeboard: bool` with `board: Board`. Constructor defaults to `Board::Deck`.

**File:** `zwipe-core/src/domain/deck/requests/update_deck_card.rs`

Replace `maybeboard: Option<bool>` with `board: Option<Board>`.

### Step 8: Update validate_deck

**File:** `zwipe-core/src/domain/deck/models/validate_deck.rs`

Replace:
```rust
let active_entries: Vec<DeckEntry> = entries
    .iter()
    .filter(|e| !e.deck_card.maybeboard)
    .cloned()
    .collect();
```
With:
```rust
let active_entries: Vec<DeckEntry> = entries
    .iter()
    .filter(|e| e.deck_card.board.is_active())
    .cloned()
    .collect();
```

**Add sideboard validation:**

```rust
fn check_sideboard_limits(format: &Format, entries: &[DeckEntry], warnings: &mut Vec<DeckWarning>) {
    let sideboard_count: u32 = entries
        .iter()
        .filter(|e| e.deck_card.board.is_sideboard())
        .map(|e| *e.deck_card.quantity as u32)
        .sum();

    if let Some(max) = format.sideboard_max() {
        if sideboard_count > max {
            warnings.push(DeckWarning::new(format!(
                "sideboard has {} cards, {} allows at most {}",
                sideboard_count,
                format.display_name().to_lowercase(),
                max
            )));
        }
    }

    if !format.has_sideboard() {
        if sideboard_count > 0 {
            warnings.push(DeckWarning::new(format!(
                "{} does not use sideboards",
                format.display_name().to_lowercase()
            )));
        }
    }
}
```

### Step 9: Add Format sideboard methods

**File:** `zwipe-core/src/domain/deck/models/format.rs`

```rust
/// Whether this format supports sideboards.
pub fn has_sideboard(&self) -> bool {
    !self.has_commander() // Commander/Brawl/Oathbreaker don't have sideboards
}

/// Maximum sideboard size, if any.
pub fn sideboard_max(&self) -> Option<u32> {
    if self.has_sideboard() { Some(15) } else { None }
}
```

### Step 10: Update DeckMetrics

**File:** `zwipe-core/src/domain/deck/models/deck_metrics.rs`

Replace `!e.deck_card.maybeboard` with `e.deck_card.board.is_active()`.

### Step 11: Update import parser

**File:** `zwipe-core/src/domain/deck/requests/import_deck_cards.rs`

Add `board: Board` to `ImportLine`. Parse `// Sideboard` and `// Maybeboard` headers:

```rust
pub struct ImportLine {
    pub quantity: i32,
    pub card_name: String,
    pub board: Board,
}
```

In the parser, track current section:
```rust
let mut current_board = Board::Deck;

for line in text.lines() {
    let trimmed = line.trim();
    if trimmed.eq_ignore_ascii_case("// maybeboard") || trimmed.eq_ignore_ascii_case("//maybeboard") {
        current_board = Board::Maybeboard;
        continue;
    }
    if trimmed.eq_ignore_ascii_case("// sideboard") || trimmed.eq_ignore_ascii_case("//sideboard") {
        current_board = Board::Sideboard;
        continue;
    }
    // ... parse line, tag with current_board
}
```

Update `bulk_create_deck_cards` to pass `board` per card instead of hardcoding `"deck"`.

---

## Phase 2: Frontend — Deck Card View

### Step 1: Replace all `maybeboard` references with `board`

Global find-replace across zwiper:
- `e.deck_card.maybeboard` → `e.deck_card.board`
- `entry.deck_card.maybeboard = true` → `entry.deck_card.board = Board::Maybeboard`
- `entry.deck_card.maybeboard = false` → `entry.deck_card.board = Board::Deck`
- `!e.deck_card.maybeboard` → `e.deck_card.board.is_active()`
- `e.deck_card.maybeboard` (as filter) → `e.deck_card.board.is_maybeboard()`

### Step 2: Add sideboard section to view screen

**File:** `zwiper/src/lib/inbound/screens/deck/card/view.rs`

Add `show_sideboard` signal alongside `show_maybeboard`:
```rust
let mut show_sideboard: Signal<bool> = use_signal(|| false);
```

Add sideboard chip to show row:
```rust
if sb_count > 0 {
    button {
        class: if show_sideboard() { "chip selected" } else { "chip" },
        onclick: move |_| show_sideboard.set(!show_sideboard()),
        "sideboard ({sb_count})"
    }
}
```

Compute sideboard entries:
```rust
let sideboard_entries: Vec<DeckEntry> = deck_entries()
    .into_iter()
    .filter(|e| e.deck_card.board.is_sideboard())
    .collect();
let sb_count = sideboard_entries.len();
```

Render sideboard section (between maybeboard and command zone):
```rust
if show_sideboard() && !sideboard_entries.is_empty() {
    div { class: "card-group row-enter",
        div { class: "card-group-header", "sideboard ({sb_count})" }
        for entry in sideboard_entries.iter() {
            // CardRow with on_board_change handler
            // "to deck" button for sideboard cards
        }
    }
}
```

### Step 3: Update move handlers

Replace `move_to_maybeboard` and `move_to_deck` with a generic `move_to_board`:

```rust
let mut move_to_board = move |card_id: Uuid, target: Board| {
    session.upkeep(client);
    let Some(session) = session() else { return };

    // Optimistic: set the board
    let old_board = deck_entries.peek().iter()
        .find(|e| e.card.scryfall_data.id == card_id)
        .map(|e| e.deck_card.board)
        .unwrap_or(Board::Deck);

    if let Some(entry) = deck_entries.write().iter_mut()
        .find(|e| e.card.scryfall_data.id == card_id)
    {
        entry.deck_card.board = target;
    }
    // trigger re-filter...

    let request = HttpUpdateDeckCard::new(None, Some(target.display_name().to_string()));
    spawn(async move {
        if let Err(e) = client().update_deck_card(deck_id, card_id, &request, &session).await {
            // Rollback to old_board...
        }
    });

    toast.info(
        format!("moved to {}", target.display_name()),
        ToastOptions::default().duration(Duration::from_millis(1500)),
    );
};
```

### Step 4: Update CardRow move buttons

**File:** `zwiper/src/lib/inbound/screens/deck/card/components/card_row.rs`

Replace the single `on_maybeboard_toggle` + `maybeboard_label` props with a more flexible approach:

```rust
// Props
on_move_to: EventHandler<Board>,
current_board: Board,
```

In expanded view, show contextual move buttons:
- **Active deck cards:** "to maybe" and "to sideboard" buttons
- **Maybeboard cards:** "to deck" and "to sideboard" buttons
- **Sideboard cards:** "to deck" and "to maybe" buttons

### Step 5: Update add screen maybeboard handler

**File:** `zwiper/src/lib/inbound/screens/deck/card/add.rs`

Replace `Some(true)` (maybeboard) with `Some("maybeboard".to_string())` in the create request. The up-swipe still adds to maybeboard — no sideboard on add screen.

Update `HttpCreateDeckCard::new()` calls to pass `board` string instead of `maybeboard` bool.

---

## Phase 3: Frontend — Remove Screen

### Step 1: Expand filter enum

**File:** `zwiper/src/lib/inbound/screens/deck/card/remove.rs`

Replace `MaybeboardFilter` with `BoardFilter`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum BoardFilter {
    #[default]
    Deck,
    Maybeboard,
    Sideboard,
    All,
}
```

### Step 2: Update filter chips

```rust
div { class: "chip-row",
    span { class: "chip-row-label", "show:" }
    // [deck] [maybeboard] [sideboard] [all]
    for (label, variant) in [
        ("deck", BoardFilter::Deck),
        ("maybeboard", BoardFilter::Maybeboard),
        ("sideboard", BoardFilter::Sideboard),
        ("all", BoardFilter::All),
    ] {
        button {
            class: if board_filter() == variant { "chip selected" } else { "chip" },
            onclick: move |_| {
                board_filter.set(variant);
                // trigger re-filter...
            },
            "{label}"
        }
    }
}
```

### Step 3: Update filter application

```rust
let filtered: Vec<Card> = entries
    .iter()
    .filter(|e| match board_filter {
        BoardFilter::Deck => e.deck_card.board.is_active(),
        BoardFilter::Maybeboard => e.deck_card.board.is_maybeboard(),
        BoardFilter::Sideboard => e.deck_card.board.is_sideboard(),
        BoardFilter::All => true,
    })
    .map(|e| e.card.clone())
    .collect();
```

### Step 4: Up-swipe on remove screen

Currently up-swipe on remove moves card to maybeboard. With board enum, the up-swipe should still move to maybeboard (consistent with add screen). No change needed beyond updating the API call from `maybeboard: Some(true)` to `board: Some("maybeboard")`.

---

## Phase 4: Frontend — Export/Import

### Step 1: Export

**File:** `zwiper/src/lib/inbound/screens/deck/export.rs`

Add `include_sideboard` toggle alongside `include_maybeboard`:

```rust
let mut include_sideboard: Signal<bool> = use_signal(|| false);

let has_sideboard = deck_resource()
    .and_then(|r| r.ok())
    .is_some_and(|d| d.entries.iter().any(|e| e.deck_card.board.is_sideboard()));
```

Export text generation adds `// Sideboard` section:

```rust
// Active deck
for entry in deck.entries.iter().filter(|e| e.deck_card.board.is_active()) {
    lines.push(format!("{} {}", *entry.deck_card.quantity, entry.card.scryfall_data.name));
}

// Sideboard
if include_sideboard() {
    let sb: Vec<_> = deck.entries.iter().filter(|e| e.deck_card.board.is_sideboard()).collect();
    if !sb.is_empty() {
        lines.push(String::new());
        lines.push("// Sideboard".to_string());
        for entry in sb {
            lines.push(format!("{} {}", *entry.deck_card.quantity, entry.card.scryfall_data.name));
        }
    }
}

// Maybeboard
if include_maybeboard() {
    // ... existing logic but with board.is_maybeboard()
}
```

Toggle chips:
```rust
div { class: "chip-row",
    span { class: "chip-row-label", "include:" }
    if has_sideboard {
        button { /* sideboard chip */ }
    }
    if has_maybeboard {
        button { /* maybeboard chip */ }
    }
}
```

### Step 2: Import

Already handled in Phase 1 Step 11 — `// Sideboard` header sets `current_board = Board::Sideboard`.

### Step 3: Buy links

**File:** `zwiper/src/lib/outbound/buy_links.rs` (or the caller)

Buy links should include sideboard cards by default (you're buying the full deck + sideboard for a tournament). Add a toggle if desired, but the default behavior is: active deck + sideboard, exclude maybeboard.

Filter entries at call site:
```rust
let buy_entries: Vec<&DeckEntry> = deck.entries.iter()
    .filter(|e| e.deck_card.board.is_active() || e.deck_card.board.is_sideboard())
    .collect();
```

---

## Verification Checklist

### Phase 1 (Data Model)
- [ ] `Board` enum with Deck, Maybeboard, Sideboard in zwipe-core
- [ ] `DeckCard.board: Board` replaces `maybeboard: bool`
- [ ] Migration: `board TEXT NOT NULL DEFAULT 'deck'` with CHECK constraint
- [ ] DatabaseDeckCard uses `board: String` with TryFrom
- [ ] All SQL queries updated (create, get, update, bulk, count, profile)
- [ ] HTTP contracts use `board: Option<String>` instead of `maybeboard: Option<bool>`
- [ ] Domain requests use `Board` enum
- [ ] validate_deck filters by `board.is_active()`
- [ ] validate_deck checks sideboard limits (15 cards for 60-card formats)
- [ ] validate_deck warns if sideboard used in commander formats
- [ ] `Format::has_sideboard()` and `Format::sideboard_max()` methods
- [ ] DeckMetrics filters by `board.is_active()`
- [ ] Import parser detects `// Sideboard` header
- [ ] `cargo sqlx prepare --workspace` succeeds
- [ ] All existing tests updated

### Phase 2 (Deck Card View)
- [ ] All `maybeboard` references replaced with `board` enum
- [ ] Sideboard section with show toggle chip
- [ ] `move_to_board` generic handler replaces `move_to_maybeboard`/`move_to_deck`
- [ ] CardRow shows contextual move buttons per board state
- [ ] Active cards: "to maybe" + "to sideboard"
- [ ] Maybeboard cards: "to deck" + "to sideboard"
- [ ] Sideboard cards: "to deck" + "to maybe"

### Phase 3 (Remove Screen)
- [ ] `BoardFilter` enum: Deck, Maybeboard, Sideboard, All
- [ ] Four filter chips in show row
- [ ] Filter correctly applied to displayed cards

### Phase 4 (Export/Import/Buy)
- [ ] Export: `// Sideboard` section with toggle
- [ ] Import: `// Sideboard` header parsed correctly
- [ ] Buy links include sideboard by default, exclude maybeboard

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwipe-core/.../deck/models/board.rs` | **NEW** — Board enum |
| `zwipe-core/.../deck/models/mod.rs` | Register board module |
| `zwipe-core/.../deck/models/deck_card.rs` | `board: Board` replaces `maybeboard: bool` |
| `zwipe-core/.../deck/models/format.rs` | Add `has_sideboard()`, `sideboard_max()` |
| `zwipe-core/.../deck/models/validate_deck.rs` | Filter by `board.is_active()`, add `check_sideboard_limits` |
| `zwipe-core/.../deck/models/deck_metrics.rs` | Filter by `board.is_active()` |
| `zwipe-core/.../deck/requests/create_deck_card.rs` | `board: Board` replaces `maybeboard: bool` |
| `zwipe-core/.../deck/requests/update_deck_card.rs` | `board: Option<Board>` replaces `maybeboard: Option<bool>` |
| `zwipe-core/.../deck/requests/import_deck_cards.rs` | `board: Board` on ImportLine, parse `// Sideboard` |
| `zwipe-core/src/http/contracts/deck_card.rs` | `board: Option<String>` replaces `maybeboard` on both structs |
| `zerver/migrations/20250810194459_create_deck_cards.sql` | `board TEXT` replaces `maybeboard BOOLEAN` |
| `zerver/.../outbound/sqlx/deck/models.rs` | `board: String` with TryFrom |
| `zerver/.../outbound/sqlx/deck/mod.rs` | Update all SQL queries |
| `zerver/.../http/handlers/deck_card/*.rs` | Pass board instead of maybeboard |
| `zwiper/.../deck/card/view.rs` | Sideboard section, `move_to_board`, show toggle |
| `zwiper/.../deck/card/add.rs` | Board string in create request |
| `zwiper/.../deck/card/remove.rs` | `BoardFilter` with 4 variants |
| `zwiper/.../deck/card/components/card_row.rs` | Contextual move buttons per board |
| `zwiper/.../deck/card/components/action_history.rs` | SwipeAction unchanged (Maybeboard variant stays) |
| `zwiper/.../deck/export.rs` | Sideboard toggle + section |
| `zwiper/.../outbound/buy_links.rs` | Caller filters by board |
