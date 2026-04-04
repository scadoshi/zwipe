# Partner, Background & Signature Spell — Phase 2: Search Filters + Validation

**Depends on:** Phase 1 (database + model) must be merged first.

Add CardFilter fields for searching partner-eligible cards, backgrounds, and signature spells. Add validate_deck warnings for invalid selections.

---

## Partner Rules Reference

Partner has 4 distinct variants that are **NOT cross-compatible**:

| Variant | Keyword/Text | Compatible With |
|---------|-------------|-----------------|
| **Partner** (generic) | `keywords` array contains "Partner" AND oracle text does NOT contain "Partner with" | Any other generic Partner card |
| **Partner with [Name]** | Oracle text contains "Partner with [Name]" | Only the specifically named card |
| **Friends Forever** | `keywords` array contains "Friends forever" | Any other Friends Forever card |
| **Doctor's Companion** | `keywords` array contains "Doctor's companion" | Cards with creature type "Time Lord Doctor" |

All partner variants require the card to also be a valid commander (legendary creature, etc.).

## Background Rules Reference

- First commander must have oracle text containing "Choose a Background"
- Background card must have type_line containing "Background" (legendary enchantment subtype)
- Background is mutually exclusive with partner — a commander cannot have both

## Signature Spell Rules Reference

- Must be an instant or sorcery
- Must be within the oathbreaker planeswalker's color identity
- No CMC restriction
- Can only be cast when oathbreaker is on the battlefield (game rule, not deckbuilding rule)
- Starts in the command zone alongside the oathbreaker

---

## Step 1: Add Filter Fields to CardFilter (zwipe-core)

**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/mod.rs`

Add three new fields in a `// partner/background/spell` section:

```rust
// partner/background/spell
is_partner: Option<bool>,
is_background: Option<bool>,
is_signature_spell: Option<bool>,
```

**Why booleans, not format-parameterized?**
- `is_partner: true` means "show cards with any partner variant keyword that are also valid commanders." The specific compatibility check (generic Partner vs named Partner vs Friends Forever) happens in validation, not search. The search casts a wide net, validation narrows.
- `is_background: true` means "show legendary enchantments with Background subtype."
- `is_signature_spell: true` means "show instants and sorceries." Color identity restriction is already handled by the existing `color_identity_within` filter — the frontend composes both.

### 1a. Add to CardFilter struct, builder struct, Default impl, build() method

Follow the exact pattern of `is_token` — it's the closest analog (a boolean filter on card characteristics).

### 1b. Add getters to both CardFilter and CardFilterBuilder

```rust
pub fn is_partner(&self) -> Option<bool> { self.is_partner }
pub fn is_background(&self) -> Option<bool> { self.is_background }
pub fn is_signature_spell(&self) -> Option<bool> { self.is_signature_spell }
```

### 1c. Add setters/unsetters to CardFilterBuilder

```rust
pub fn set_is_partner(&mut self, is_partner: bool) -> &mut Self {
    self.is_partner = Some(is_partner);
    self
}
pub fn unset_is_partner(&mut self) -> &mut Self {
    self.is_partner = None;
    self
}

pub fn set_is_background(&mut self, is_background: bool) -> &mut Self {
    self.is_background = Some(is_background);
    self
}
pub fn unset_is_background(&mut self) -> &mut Self {
    self.is_background = None;
    self
}

pub fn set_is_signature_spell(&mut self, is_signature_spell: bool) -> &mut Self {
    self.is_signature_spell = Some(is_signature_spell);
    self
}
pub fn unset_is_signature_spell(&mut self) -> &mut Self {
    self.is_signature_spell = None;
    self
}
```

### 1d. Update `retain_config` and `is_empty_ignoring_deck_context`

These filters are auto-set from deck context, similar to `is_commander_in_format`. They should NOT count as user-set filters for emptiness checks. Update `is_empty_ignoring_deck_context` (or whatever Phase 1 named it) to also unset these.

`retain_config` already zeros them via `..Self::default()`, so no change needed there.

---

## Step 2: Add SQL Filter Implementations (zerver)

**File:** `zerver/src/lib/outbound/sqlx/card/mod.rs`

Add filter blocks in `search_scryfall_data`:

### 2a. Partner filter

```rust
if let Some(true) = request.is_partner() {
    // Cards with any partner variant keyword that are also legendary creatures
    // This is a broad search — validation checks specific compatibility
    sep.push("(");
    sep.push_unseparated(
        "type_line ILIKE '%Legendary%' AND type_line ILIKE '%Creature%' AND (\
         keywords @> ARRAY['Partner']::text[] \
         OR keywords @> ARRAY['Friends forever']::text[] \
         OR keywords @> ARRAY['Doctor''s companion']::text[] \
         OR oracle_text ILIKE '%partner with%' \
         OR oracle_text ILIKE '%choose a background%')"
    );
    sep.push_unseparated(")");
}
```

Note: We include "Choose a Background" creatures here because the partner search field is used on the same screen. The UI presents this as the "partner" slot, which also serves for backgrounds. If you'd prefer to keep them strictly separate, remove the `choose a background` line. But including it means a user searching the partner field will also see background-eligible commanders, which may reduce confusion.

Actually, on reflection — keep them separate. The `is_partner` filter should NOT include "Choose a Background" creatures. Those are found via the commander filter. The partner filter finds the SECOND card for the partner slot. Similarly, `is_background` finds the SECOND card for the background slot.

Revised:

```rust
if let Some(true) = request.is_partner() {
    // Cards that can serve as a second commander via any partner variant
    sep.push("(");
    sep.push_unseparated(
        "type_line ILIKE '%Legendary%' AND type_line ILIKE '%Creature%' AND (\
         keywords @> ARRAY['Partner']::text[] \
         OR keywords @> ARRAY['Friends forever']::text[] \
         OR keywords @> ARRAY['Doctor''s companion']::text[] \
         OR oracle_text ILIKE '%partner with%')"
    );
    sep.push_unseparated(")");
}
```

### 2b. Background filter

```rust
if let Some(true) = request.is_background() {
    // Legendary enchantments with Background subtype
    sep.push("(");
    sep.push_unseparated(
        "type_line ILIKE '%Legendary%' AND type_line ILIKE '%Enchantment%' AND type_line ILIKE '%Background%'"
    );
    sep.push_unseparated(")");
}
```

### 2c. Signature spell filter

```rust
if let Some(true) = request.is_signature_spell() {
    // Instants and sorceries (color identity restriction handled separately)
    sep.push("(");
    sep.push_unseparated(
        "type_line ILIKE '%Instant%' OR type_line ILIKE '%Sorcery%'"
    );
    sep.push_unseparated(")");
}
```

---

## Step 3: Add Eligibility Pure Functions (zwipe-core)

**File:** `zwipe-core/src/domain/card/models/search_card/commander_eligibility.rs`

Add these functions alongside the existing `is_valid_commander`:

### 3a. Partner validation

```rust
/// The kind of partner ability a card has, if any.
pub enum PartnerKind {
    /// Generic "Partner" keyword — compatible with any other Generic partner
    Generic,
    /// "Partner with [Name]" — compatible only with the named card
    Named(String),
    /// "Friends forever" — compatible with any other FriendsForever
    FriendsForever,
    /// "Doctor's companion" — compatible with Time Lord Doctor cards
    DoctorsCompanion,
}

/// Returns the partner kind for a card, if it has one.
pub fn partner_kind(card: &Card) -> Option<PartnerKind> {
    let sd = &card.scryfall_data;
    let oracle_text = sd.oracle_text.as_deref().unwrap_or("");
    let keywords = &sd.keywords;

    // Check named partner first (more specific)
    if oracle_text.to_lowercase().contains("partner with ") {
        // Extract the partner name from "Partner with [Name]"
        if let Some(name) = extract_named_partner(oracle_text) {
            return Some(PartnerKind::Named(name));
        }
    }

    if keywords.iter().any(|k| k == "Friends forever") {
        return Some(PartnerKind::FriendsForever);
    }

    if keywords.iter().any(|k| k == "Doctor's companion") {
        return Some(PartnerKind::DoctorsCompanion);
    }

    // Generic Partner — has "Partner" keyword but NOT "Partner with"
    if keywords.iter().any(|k| k == "Partner") {
        return Some(PartnerKind::Generic);
    }

    None
}

/// Checks whether two cards can be partners.
pub fn are_valid_partners(card_a: &Card, card_b: &Card) -> bool {
    let kind_a = partner_kind(card_a);
    let kind_b = partner_kind(card_b);

    match (kind_a, kind_b) {
        (Some(PartnerKind::Generic), Some(PartnerKind::Generic)) => true,
        (Some(PartnerKind::Named(name)), _) => {
            card_b.scryfall_data.name.to_lowercase() == name.to_lowercase()
        }
        (_, Some(PartnerKind::Named(name))) => {
            card_a.scryfall_data.name.to_lowercase() == name.to_lowercase()
        }
        (Some(PartnerKind::FriendsForever), Some(PartnerKind::FriendsForever)) => true,
        (Some(PartnerKind::DoctorsCompanion), _) => {
            // Check if card_b is a Time Lord Doctor
            card_b.scryfall_data.type_line.as_deref().unwrap_or("")
                .contains("Time Lord Doctor")
        }
        (_, Some(PartnerKind::DoctorsCompanion)) => {
            card_a.scryfall_data.type_line.as_deref().unwrap_or("")
                .contains("Time Lord Doctor")
        }
        _ => false,
    }
}

/// Extracts the partner name from oracle text like "Partner with Brallin, Skyshark Rider"
fn extract_named_partner(oracle_text: &str) -> Option<String> {
    let lower = oracle_text.to_lowercase();
    if let Some(pos) = lower.find("partner with ") {
        let after = &oracle_text[pos + "partner with ".len()..];
        // Partner name ends at newline, period, or parenthesis
        let end = after.find(['\n', '('])
            .unwrap_or(after.len());
        let name = after[..end].trim().trim_end_matches('.');
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}
```

### 3b. Background validation

```rust
/// Whether a card has "Choose a Background" (making it background-eligible as commander).
pub fn has_choose_a_background(card: &Card) -> bool {
    card.scryfall_data.oracle_text.as_deref().unwrap_or("")
        .to_lowercase()
        .contains("choose a background")
}

/// Whether a card is a Background enchantment.
pub fn is_background_card(card: &Card) -> bool {
    let type_line = card.scryfall_data.type_line.as_deref().unwrap_or("");
    type_line.contains("Legendary")
        && type_line.contains("Enchantment")
        && type_line.contains("Background")
}
```

### 3c. Signature spell validation

```rust
/// Whether a card is a valid signature spell (instant or sorcery).
pub fn is_valid_signature_spell_type(card: &Card) -> bool {
    let type_line = card.scryfall_data.type_line.as_deref().unwrap_or("");
    type_line.contains("Instant") || type_line.contains("Sorcery")
}

/// Whether a signature spell is within the oathbreaker's color identity.
pub fn is_signature_spell_in_color_identity(spell: &Card, oathbreaker: &Card) -> bool {
    let ob_colors = &oathbreaker.scryfall_data.color_identity;
    for color in spell.scryfall_data.color_identity.iter() {
        if !ob_colors.contains(color) {
            return false;
        }
    }
    true
}
```

### Tests

Add comprehensive tests for each function. Key test cases:
- Generic Partner + Generic Partner: valid
- Generic Partner + Named Partner: invalid
- Named Partner + correct named card: valid
- Named Partner + wrong card: invalid
- Friends Forever + Friends Forever: valid
- Friends Forever + Generic Partner: **invalid** (not cross-compatible)
- Doctor's Companion + Time Lord Doctor: valid
- Background eligibility checks
- Signature spell type + color identity checks

---

## Step 4: Add validate_deck Warnings (zwipe-core)

**File:** `zwipe-core/src/domain/deck/models/validate_deck.rs`

Add three new check functions called from `validate_deck`. These need access to the actual Card data for the partner/background/signature_spell, similar to how `commander_card` is passed in.

### 4a. Update function signature

The function needs more card data. Two approaches:

**Option A (recommended):** Pass a struct of optional cards:

```rust
pub struct DeckCommandZone {
    pub commander: Option<Card>,
    pub partner_commander: Option<Card>,
    pub background: Option<Card>,
    pub signature_spell: Option<Card>,
}

pub fn validate_deck(
    deck_profile: &DeckProfile,
    entries: &[DeckEntry],
    command_zone: &DeckCommandZone,
) -> Vec<DeckWarning>
```

**Option B:** Keep adding Optional<&Card> parameters (gets unwieldy with 4 cards).

Go with Option A. This is a breaking change to the function signature — update all call sites. Search for `validate_deck(` across the workspace.

### 4b. Partner validation warning

```rust
fn check_partner_validity(
    format: &Format,
    profile: &DeckProfile,
    command_zone: &DeckCommandZone,
    warnings: &mut Vec<DeckWarning>,
) {
    let Some(_partner_id) = profile.partner_commander_id else { return };

    if !format.supports_partner() {
        warnings.push(DeckWarning::new(format!(
            "{} does not support partner commanders",
            format.display_name().to_lowercase()
        )));
        return;
    }

    if let (Some(commander), Some(partner)) = (&command_zone.commander, &command_zone.partner_commander) {
        if !are_valid_partners(commander, partner) {
            warnings.push(DeckWarning::with_card(
                format!(
                    "{} and {} cannot be partners",
                    commander.scryfall_data.name.to_lowercase(),
                    partner.scryfall_data.name.to_lowercase()
                ),
                partner.scryfall_data.id,
            ));
        }
    }
}
```

### 4c. Background validation warning

```rust
fn check_background_validity(
    format: &Format,
    profile: &DeckProfile,
    command_zone: &DeckCommandZone,
    warnings: &mut Vec<DeckWarning>,
) {
    let Some(_bg_id) = profile.background_id else { return };

    if !format.supports_background() {
        warnings.push(DeckWarning::new(format!(
            "{} does not support backgrounds",
            format.display_name().to_lowercase()
        )));
        return;
    }

    // Check commander has "Choose a Background"
    if let Some(commander) = &command_zone.commander {
        if !has_choose_a_background(commander) {
            warnings.push(DeckWarning::with_card(
                format!(
                    "{} does not have 'choose a background'",
                    commander.scryfall_data.name.to_lowercase()
                ),
                commander.scryfall_data.id,
            ));
        }
    }

    // Check background card is actually a Background
    if let Some(bg) = &command_zone.background {
        if !is_background_card(bg) {
            warnings.push(DeckWarning::with_card(
                format!(
                    "{} is not a valid background enchantment",
                    bg.scryfall_data.name.to_lowercase()
                ),
                bg.scryfall_data.id,
            ));
        }
    }

    // Mutual exclusivity: can't have both partner and background
    if profile.partner_commander_id.is_some() {
        warnings.push(DeckWarning::new(
            "a commander cannot have both a partner and a background"
        ));
    }
}
```

### 4d. Signature spell validation warning

```rust
fn check_signature_spell_validity(
    format: &Format,
    profile: &DeckProfile,
    command_zone: &DeckCommandZone,
    warnings: &mut Vec<DeckWarning>,
) {
    let Some(_spell_id) = profile.signature_spell_id else {
        // Warn if format requires one but none selected
        if format.has_signature_spell() {
            warnings.push(DeckWarning::new("oathbreaker format requires a signature spell"));
        }
        return;
    };

    if !format.has_signature_spell() {
        warnings.push(DeckWarning::new(format!(
            "{} does not use signature spells",
            format.display_name().to_lowercase()
        )));
        return;
    }

    if let Some(spell) = &command_zone.signature_spell {
        // Must be instant or sorcery
        if !is_valid_signature_spell_type(spell) {
            warnings.push(DeckWarning::with_card(
                format!(
                    "{} must be an instant or sorcery to be a signature spell",
                    spell.scryfall_data.name.to_lowercase()
                ),
                spell.scryfall_data.id,
            ));
        }

        // Must be within oathbreaker's color identity
        if let Some(oathbreaker) = &command_zone.commander {
            if !is_signature_spell_in_color_identity(spell, oathbreaker) {
                warnings.push(DeckWarning::with_card(
                    format!(
                        "{} is outside the oathbreaker's color identity",
                        spell.scryfall_data.name.to_lowercase()
                    ),
                    spell.scryfall_data.id,
                ));
            }
        }
    }
}
```

### 4e. Update color identity check

Color identity should be the **union** of both commanders' color identities when a partner or background is present:

```rust
fn commander_color_identity(
    profile: &DeckProfile,
    entries: &[DeckEntry],
    command_zone: &DeckCommandZone,
) -> Option<Colors> {
    // Start with primary commander's colors
    let mut colors = find_commander_colors(profile.commander_id, entries, command_zone.commander.as_ref())?;

    // Union with partner's colors
    if let Some(partner) = &command_zone.partner_commander {
        for color in partner.scryfall_data.color_identity.iter() {
            colors.insert(color.clone());
        }
    }

    // Union with background's colors
    if let Some(bg) = &command_zone.background {
        for color in bg.scryfall_data.color_identity.iter() {
            colors.insert(color.clone());
        }
    }

    Some(colors)
}
```

---

## Step 5: Run SQLx Prepare + Tests

```bash
cargo sqlx prepare --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

---

## Verification Checklist

- [ ] `CardFilter` has `is_partner`, `is_background`, `is_signature_spell` fields
- [ ] Builder has setters, unsetters, getters for all three
- [ ] SQL filter for partner finds cards with Partner/Friends Forever/Doctor's Companion/Partner with keywords
- [ ] SQL filter for background finds legendary enchantments with Background type
- [ ] SQL filter for signature spell finds instants and sorceries
- [ ] `partner_kind()` correctly identifies all 4 partner variants
- [ ] `are_valid_partners()` enforces compatibility rules (no cross-variant pairing)
- [ ] `validate_deck` warns on invalid partner pairings
- [ ] `validate_deck` warns on background without "Choose a Background" commander
- [ ] `validate_deck` warns on non-background card in background slot
- [ ] `validate_deck` warns on partner + background mutual exclusivity
- [ ] `validate_deck` warns on missing signature spell for Oathbreaker
- [ ] `validate_deck` warns on non-instant/sorcery signature spell
- [ ] `validate_deck` warns on signature spell outside oathbreaker's color identity
- [ ] Color identity check unions both commanders' colors
- [ ] Comprehensive tests for all eligibility functions
- [ ] All existing tests updated for new `validate_deck` signature

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwipe-core/.../card_filter/mod.rs` | Add 3 filter fields |
| `zwipe-core/.../card_filter/getters.rs` | Add 3 getters |
| `zwipe-core/.../card_filter/builder/mod.rs` | Add 3 fields + default + build |
| `zwipe-core/.../card_filter/builder/setters.rs` | Add 6 setters/unsetters |
| `zwipe-core/.../card_filter/builder/getters.rs` | Add 3 getters |
| `zwipe-core/.../search_card/commander_eligibility.rs` | Add PartnerKind, partner/background/spell functions + tests |
| `zwipe-core/.../deck/models/validate_deck.rs` | Add DeckCommandZone, 3 new check functions, update color identity |
| `zerver/.../outbound/sqlx/card/mod.rs` | Add 3 SQL filter blocks |
