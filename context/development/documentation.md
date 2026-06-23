# Documentation Philosophy

## Core Principle

**Document intent, not implementation details that are obvious from naming.**

Documentation should add information that cannot be inferred from well-chosen names and clear domain context. Every doc comment should answer: "What does this tell the reader that they don't already know?"

---

## When to Document

### ✅ Always Document

1. **Non-obvious behavior**
   ```rust
   /// Shared by all printings of the same card.
   pub oracle_id: Option<Uuid>,
   ```
   The field name doesn't convey that this ID is consistent across reprints.

2. **External API schemas**
   ```rust
   /// This card's Arena ID, if any. A large percentage of cards are not available on Arena and do not have this ID.
   pub arena_id: Option<i32>,
   ```
   API fields have specific semantics defined by the external service.

3. **Surprising edge cases**
   ```rust
   /// The card's mana value (converted mana cost). Some funny cards have fractional mana costs.
   pub cmc: Option<f64>,
   ```
   The `f64` type might seem odd—documentation explains why.

4. **Business logic and constraints**
   ```rust
   /// Lower values indicate more popular.
   pub edhrec_rank: Option<i32>,
   ```
   The ranking direction is non-obvious.

5. **Public APIs**
   - All public modules, functions, types, and traits need doc comments
   - Explain purpose, parameters, return values, and errors

---

## When NOT to Document

### ❌ Skip Documentation

1. **Self-explanatory domain concepts**
   ```rust
   /// Magic: The Gathering's five colors.
   #[allow(missing_docs)]
   pub enum Color {
       White,  // ❌ Don't add: /// The white color.
       Blue,   // ❌ Don't add: /// The blue color.
       // ...
   }
   ```
   In the MTG domain, color names are universally recognized and unambiguous.

2. **Schema fields that restate the name**
   ```rust
   // ❌ BAD
   /// The card's name.
   pub name: String,

   // ✅ GOOD (only if there's additional context)
   /// The name of this card. Multiple faces separated by ␣//␣.
   pub name: String,
   ```

3. **Obvious standard library patterns**
   ```rust
   // ❌ Don't document getters that just return a field
   /// Returns the color.
   pub fn color(&self) -> Color { self.color }

   // ✅ But DO document if there's transformation or logic
   /// Returns the single-letter color code (e.g., "W", "U", "B", "R", "G").
   pub fn to_short_name(&self) -> String { /* ... */ }
   ```

---

## Using `#[allow(missing_docs)]`

Apply `#[allow(missing_docs)]` strategically **at the container level** when all contained items are self-documenting.

### ✅ Good Use Cases

**Domain enums with obvious variants:**
```rust
/// Card rarity classification in Magic: The Gathering.
#[allow(missing_docs)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
    Bonus,
    Special,
}
```

**Why this works:**
- Container-level doc explains the purpose
- Variant names are standard MTG terminology
- Target audience (MTG developers) instantly understands meaning
- Adding `/// Common rarity.` would be pure noise

### ❌ Don't Use For

- Structs with many fields (document fields instead)
- Types with non-obvious semantics
- Public APIs where external users need guidance
- Anything you'd have to explain to a new team member

---

## Documentation Style Guide

### 1. Be Concise and Precise

```rust
// ❌ Verbose
/// This function returns the long name representation of the color as a String type
pub fn to_long_name(&self) -> String

// ✅ Concise
/// Returns the full color name (e.g., "White", "Blue").
pub fn to_long_name(&self) -> String
```

### 2. Use Examples for Clarity

```rust
/// Converts to Scryfall mana notation (e.g., "{2}{R}{R}").
pub fn to_scryfall_notation(&self) -> String
```

### 3. Document Guarantees and Invariants

```rust
/// Returns all five colors in WUBRG order.
pub fn all() -> [Self; 5]
```

### 4. Explain Trade-offs and Design Decisions

Use container-level docs for architectural notes:
```rust
/// # Design Note
///
/// This struct derives `sqlx::FromRow` (when `zerver` feature is enabled) to avoid
/// maintaining separate domain and database models for this large structure.
/// If the database ever changes, only the derive macro needs replacement.
pub struct ScryfallData { /* ... */ }
```

---

## Balancing Signal vs. Noise

**Good documentation has high signal-to-noise ratio:**

- **High Signal**: "Lower values indicate more popular" (non-obvious ranking direction)
- **Low Signal**: "The white color" (restates the name "White")

**Ask yourself:**
1. Would a competent developer in this domain be confused without this doc?
2. Does this doc add information beyond the name and type?
3. Would I want to read this doc, or would it feel like clutter?

If you answer "no" to all three, consider using `#[allow(missing_docs)]` or improving the naming instead of adding docs.

---

## Special Cases

### API Schema Mirroring

When a struct directly mirrors an external API (like Scryfall), documentation serves as:
1. **Integration reference** for the API contract
2. **Field semantics** that may not be obvious from names
3. **Change detection** when the upstream API evolves

**Always document these comprehensively**, even if some field names seem obvious. The docs capture the external contract's specific meanings.

### Wrapper Types

```rust
/// Collection of card colors.
///
/// Empty collection means colorless.
pub struct Colors(Vec<Color>);
```

Document the semantic meaning of empty/special states.

---

## Summary

| Scenario | Action |
|----------|--------|
| External API fields | ✅ Document all with API semantics |
| Non-obvious behavior | ✅ Document the surprise |
| Domain-specific enums (MTG colors/rarities) | ✅ Container doc + `#[allow(missing_docs)]` |
| Public APIs | ✅ Always document |
| Self-explanatory getters | ❌ Skip or use `#[allow(missing_docs)]` |
| Restating the obvious | ❌ Never do this |

**Golden Rule:** If documentation makes you think "well, obviously," it's probably noise. If it makes you think "oh, I didn't know that," it's valuable signal.
