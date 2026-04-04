# Commander Filter — Phase 1: Backend (Core + Server)

Add `is_commander_in_format: Option<Format>` to `CardFilter` so the backend can filter card search results to only valid commanders for a given format.

Also add a `validate_deck` warning when the selected commander is not a valid commander for the deck's format.

---

## Commander Eligibility Rules

Each commander format has different rules for what qualifies as a commander:

| Format | Rule |
|--------|------|
| Commander, Duel, PreDH | Legendary creature **OR** legendary with P/T (vehicle/spacecraft) **OR** oracle text contains "can be your commander" |
| Brawl, Standard Brawl, Historic Brawl | Legendary creature **OR** legendary planeswalker |
| Pauper Commander | Uncommon creature |
| Oathbreaker | Planeswalker (any, not just legendary) |

**Edge cases to handle:**
- "can be your commander" oracle text (e.g., Grist the Hunger Tide — a planeswalker that's also a creature everywhere except the stack)
- Legendary vehicles/spacecraft with power/toughness printed on the card (they have type_line containing "Vehicle" or "Spacecraft" and non-null power/toughness)
- Cards that are both legendary creature AND planeswalker (e.g., Grist) — should match both Brawl and Commander rules

---

## Step 1: Add `is_commander_in_format` to CardFilter (zwipe-core)

### 1a. Add field to `CardFilter` struct

**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/mod.rs`

Add to the struct, in a new `// commander` section after `// legalities`:

```rust
// commander
is_commander_in_format: Option<Format>,
```

You will need to add `Format` to the imports at the top:
```rust
use crate::domain::deck::Format;
```

### 1b. Add getter to `CardFilter`

**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/getters.rs`

```rust
// commander
pub fn is_commander_in_format(&self) -> Option<&Format> {
    self.is_commander_in_format.as_ref()
}
```

### 1c. Add field to `CardFilterBuilder` struct

**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/builder/mod.rs`

Add the field to the struct:
```rust
// commander
is_commander_in_format: Option<Format>,
```

Add to `Default` impl:
```rust
is_commander_in_format: None,
```

Add to `build()` method's `Ok(CardFilter { ... })`:
```rust
is_commander_in_format: self.is_commander_in_format,
```

**Important:** `is_commander_in_format` is a search criterion, NOT config. It should NOT be preserved in `retain_config()`. The existing `retain_config` in `setters.rs` uses `..Self::default()` to zero out non-config fields, so it will correctly reset to `None` with no changes needed.

However, like `legalities_contains_any`, this filter is auto-populated from deck context and should not count as a "user-set filter" for emptiness checks. Update `is_empty_ignoring_legalities` to also ignore this field (and consider renaming it to `is_empty_ignoring_deck_context` or adding a separate method). The simplest approach:

```rust
pub fn is_empty_ignoring_deck_context(&self) -> bool {
    let mut test = self.clone();
    test.unset_legalities_contains_any();
    test.unset_is_commander_in_format();
    test.is_empty()
}
```

Keep `is_empty_ignoring_legalities` as a deprecated alias or update all call sites to use the new name. Check the frontend for all call sites of `is_empty_ignoring_legalities` and update them.

### 1d. Add setter/unsetter to `CardFilterBuilder`

**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/builder/setters.rs`

```rust
// commander

/// Sets the commander eligibility filter for a specific format.
pub fn set_is_commander_in_format(&mut self, format: Format) -> &mut Self {
    self.is_commander_in_format = Some(format);
    self
}

/// Clears the commander eligibility filter.
pub fn unset_is_commander_in_format(&mut self) -> &mut Self {
    self.is_commander_in_format = None;
    self
}
```

Add `Format` import:
```rust
use crate::domain::deck::Format;
```

### 1e. Add getter to `CardFilterBuilder`

**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/builder/getters.rs`

```rust
/// Returns the commander eligibility format filter.
pub fn is_commander_in_format(&self) -> Option<&Format> {
    self.is_commander_in_format.as_ref()
}
```

---

## Step 2: Add `commander_formats` helper to Format (zwipe-core)

**File:** `zwipe-core/src/domain/deck/models/format.rs`

Add a static method that returns only the formats that have commanders. This will be useful for the frontend to populate the filter dropdown.

```rust
/// Formats that require a commander, alphabetical.
pub fn commander_formats() -> &'static [Format] {
    &[
        Self::Brawl,
        Self::Commander,
        Self::Duel,
        Self::HistoricBrawl,
        Self::Oathbreaker,
        Self::PauperCommander,
        Self::Predh,
        Self::StandardBrawl,
    ]
}
```

This is a convenience — it's the same as filtering `Format::all()` by `has_commander()`, but avoids recomputing.

---

## Step 3: Add commander eligibility pure function (zwipe-core)

**File:** Create `zwipe-core/src/domain/card/models/search_card/commander_eligibility.rs`

This function is purely for documentation and for `validate_deck` to call. The SQL layer implements the same logic in SQL for performance. Having both means we can test the logic in Rust and trust the SQL mirrors it.

```rust
use crate::domain::{card::Card, deck::Format};

/// Checks whether a card is a valid commander for the given format.
///
/// This is the authoritative definition of commander eligibility.
/// The SQL filter in `search_scryfall_data` must mirror this logic.
pub fn is_valid_commander(card: &Card, format: &Format) -> bool {
    let sd = &card.scryfall_data;
    let type_line = sd.type_line.as_deref().unwrap_or("");
    let oracle_text = sd.oracle_text.as_deref().unwrap_or("");

    match format {
        // Legendary creature, legendary vehicle/spacecraft with P/T,
        // or "can be your commander" oracle text
        Format::Commander | Format::Duel | Format::Predh => {
            let is_legendary = type_line.contains("Legendary");
            let is_creature = type_line.contains("Creature");
            let has_pt = sd.power.is_some() && sd.toughness.is_some();
            let can_be_commander = oracle_text
                .to_lowercase()
                .contains("can be your commander");

            (is_legendary && is_creature)
                || (is_legendary && has_pt)
                || can_be_commander
        }

        // Legendary creature OR legendary planeswalker
        Format::Brawl | Format::StandardBrawl | Format::HistoricBrawl => {
            let is_legendary = type_line.contains("Legendary");
            let is_creature = type_line.contains("Creature");
            let is_planeswalker = type_line.contains("Planeswalker");

            is_legendary && (is_creature || is_planeswalker)
        }

        // Uncommon creature (not legendary-required)
        Format::PauperCommander => {
            let is_creature = type_line.contains("Creature");
            let is_uncommon = sd.rarity.as_deref() == Some("uncommon");

            is_creature && is_uncommon
        }

        // Any planeswalker
        Format::Oathbreaker => {
            type_line.contains("Planeswalker")
        }

        // Non-commander formats — nothing is a valid commander
        _ => false,
    }
}
```

Register the module:

**File:** `zwipe-core/src/domain/card/models/search_card/mod.rs`

Add: `pub mod commander_eligibility;`

### Tests for commander eligibility

Add tests in the same file covering:
- Legendary creature passes Commander/Brawl
- Non-legendary creature fails Commander, passes Pauper Commander (if uncommon)
- Legendary planeswalker passes Brawl, fails Commander (unless "can be your commander")
- "Can be your commander" text passes Commander
- Legendary Vehicle with P/T passes Commander
- Regular planeswalker passes Oathbreaker, fails Commander
- Non-commander format returns false

Note: You'll need to construct test `Card` instances. Check how existing tests in `validate_deck.rs` or `deck_metrics.rs` build test cards and follow the same pattern.

---

## Step 4: Add SQL filter for commander eligibility (zerver)

**File:** `zerver/src/lib/outbound/sqlx/card/mod.rs`

In the `search_scryfall_data` function, add a new filter block after the legalities filter. The SQL needs to implement the same logic as `is_valid_commander` but in PostgreSQL:

```rust
if let Some(format) = request.is_commander_in_format() {
    match format {
        // Legendary creature, legendary vehicle with P/T, or "can be your commander"
        Format::Commander | Format::Duel | Format::Predh => {
            sep.push("(");
            sep.push_unseparated(
                "(type_line ILIKE '%Legendary%' AND type_line ILIKE '%Creature%') \
                 OR (type_line ILIKE '%Legendary%' AND power IS NOT NULL AND toughness IS NOT NULL) \
                 OR LOWER(oracle_text) LIKE '%can be your commander%'"
            );
            sep.push_unseparated(")");
        }
        // Legendary creature or legendary planeswalker
        Format::Brawl | Format::StandardBrawl | Format::HistoricBrawl => {
            sep.push("(");
            sep.push_unseparated(
                "type_line ILIKE '%Legendary%' AND \
                 (type_line ILIKE '%Creature%' OR type_line ILIKE '%Planeswalker%')"
            );
            sep.push_unseparated(")");
        }
        // Uncommon creature
        Format::PauperCommander => {
            sep.push("(");
            sep.push_unseparated(
                "type_line ILIKE '%Creature%' AND rarity = 'uncommon'"
            );
            sep.push_unseparated(")");
        }
        // Any planeswalker
        Format::Oathbreaker => {
            sep.push("type_line ILIKE '%Planeswalker%'");
        }
        // Non-commander formats: no filter (should not happen, but safe)
        _ => {}
    }
}
```

Add `Format` import at the top of the file:
```rust
use zwipe_core::domain::deck::Format;
```

---

## Step 5: Add `validate_deck` warning for invalid commander (zwipe-core)

**File:** `zwipe-core/src/domain/deck/models/validate_deck.rs`

Add a new check function called from `validate_deck`:

```rust
check_commander_eligibility(format, deck_profile, entries, commander_card, &mut warnings);
```

Implementation:

```rust
fn check_commander_eligibility(
    format: &Format,
    profile: &DeckProfile,
    entries: &[DeckEntry],
    commander_card: Option<&Card>,
    warnings: &mut Vec<DeckWarning>,
) {
    if !format.has_commander() {
        return;
    }

    let Some(commander_id) = profile.commander_id else {
        return; // Already warned by check_commander_required
    };

    // Find the commander card from entries or the provided card
    let commander = entries
        .iter()
        .find(|e| e.card.scryfall_data.id == commander_id)
        .map(|e| &e.card)
        .or(commander_card);

    let Some(card) = commander else {
        return; // Can't validate without the card data
    };

    use crate::domain::card::search_card::commander_eligibility::is_valid_commander;

    if !is_valid_commander(card, format) {
        warnings.push(DeckWarning::with_card(
            format!(
                "{} is not a valid commander for {}",
                card.scryfall_data.name.to_lowercase(),
                format.display_name().to_lowercase()
            ),
            card.scryfall_data.id,
        ));
    }
}
```

### Tests

Add tests for:
- Valid legendary creature commander: no warning
- Non-legendary creature as Commander format commander: warning generated
- Planeswalker as Brawl commander: no warning
- Planeswalker as Commander format commander: warning (unless "can be your commander")
- Uncommon creature as Pauper Commander: no warning
- Rare creature as Pauper Commander: warning

---

## Step 6: Run SQLx prepare

After all SQL changes:
```bash
cargo sqlx prepare --workspace
```

Commit the updated `.sqlx/` directory.

---

## Step 7: Run tests and clippy

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

---

## Verification Checklist

- [ ] `CardFilter` has `is_commander_in_format: Option<Format>` field
- [ ] `CardFilterBuilder` has setter, unsetter, getter for the field
- [ ] `retain_config()` does NOT preserve this field (defaults to None — already correct via `..Self::default()`)
- [ ] `is_empty_ignoring_legalities` (or renamed) also ignores `is_commander_in_format`
- [ ] `Format::commander_formats()` returns the 8 commander format variants
- [ ] `is_valid_commander(card, format)` pure function exists with tests
- [ ] SQL filter in `search_scryfall_data` mirrors the eligibility logic
- [ ] `validate_deck` warns when commander is not valid for format
- [ ] All existing tests pass
- [ ] New tests cover eligibility rules per format
- [ ] `cargo sqlx prepare --workspace` run after SQL changes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwipe-core/.../card_filter/mod.rs` | Add `is_commander_in_format` field |
| `zwipe-core/.../card_filter/getters.rs` | Add getter |
| `zwipe-core/.../card_filter/builder/mod.rs` | Add field, default, build mapping |
| `zwipe-core/.../card_filter/builder/setters.rs` | Add setter + unsetter |
| `zwipe-core/.../card_filter/builder/getters.rs` | Add getter |
| `zwipe-core/.../search_card/mod.rs` | Register new module |
| `zwipe-core/.../search_card/commander_eligibility.rs` | **NEW** — eligibility pure function + tests |
| `zwipe-core/.../deck/models/format.rs` | Add `commander_formats()` |
| `zwipe-core/.../deck/models/validate_deck.rs` | Add `check_commander_eligibility` |
| `zerver/.../outbound/sqlx/card/mod.rs` | Add SQL filter block |
