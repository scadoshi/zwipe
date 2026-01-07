# Language Filter Implementation Plan

## Goal
Implement language filter for CardFilter to default to English cards only, following the pattern established by other boolean filters.

## Database Schema
- Field: `lang` (TEXT NOT NULL)
- Location: `scryfall_data` table
- Current value: String (e.g., 'en', 'es', 'fr', etc.)

## Implementation Steps

### 1. Create Language Domain Enum
**File**: `zerver/src/lib/domain/card/models/scryfall_data/language.rs`

Create a new `Language` enum following the Rarity pattern:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "en")]
    English,
}

impl Language {
    pub fn to_code(&self) -> &'static str {
        match self {
            Language::English => "en",
        }
    }

    pub fn all() -> Vec<Language> {
        vec![Language::English]
    }
}

impl TryFrom<&str> for Language {
    type Error = InvalidLanguage;
    // ...
}

impl Display for Language {
    // ...
}
```

**Error type**: `InvalidLanguage` (in same file)

### 2. Update Domain Module
**File**: `zerver/src/lib/domain/card/models/scryfall_data/mod.rs`

Add:
```rust
pub mod language;
```

### 3. Add language Field to CardFilter
**File**: `zerver/src/lib/domain/card/models/search_card/card_filter/mod.rs`

Add field:
```rust
// printing section
language: Option<Language>,
```

### 4. Update CardFilterBuilder Default
**File**: `zerver/src/lib/domain/card/models/search_card/card_filter/builder/mod.rs`

In `Default` impl:
```rust
language: Some(Language::English),
```

Add field to struct definition.

Add constructor:
```rust
pub fn with_language(language: Language) -> CardFilterBuilder {
    CardFilterBuilder {
        language: Some(language),
        ..CardFilterBuilder::default()
    }
}
```

Update `build()` to include `language: self.language`.

### 5. Add Getter
**File**: `zerver/src/lib/domain/card/models/search_card/card_filter/getters.rs`

```rust
pub fn language(&self) -> Option<Language> {
    self.language
}
```

### 6. Add Setters
**File**: `zerver/src/lib/domain/card/models/search_card/card_filter/builder/setters.rs`

```rust
pub fn set_language(&mut self, language: Language) -> &mut Self {
    self.language = Some(language);
    self
}

pub fn unset_language(&mut self) -> &mut Self {
    self.language = None;
    self
}
```

Update `retain_config()` to preserve `language: self.language`.

### 7. Implement SQL Filter
**File**: `zerver/src/lib/outbound/sqlx/card/mod.rs`

Add filter logic:
```rust
if let Some(language) = request.language() {
    sep.push("scryfall_data.lang = ");
    sep.push_bind_unseparated(language.to_code());
}
```

## Design Decisions

1. **Default to Some(Language::English)**: Matches next.md spec and ensures only English cards are served by default
2. **Preserve in retain_config()**: Language is a config-like filter (like digital, oversized, etc.)
3. **Start with English only**: Only implement Language::English variant initially; can add other languages later
4. **Use to_code() method**: Converts enum to database string representation
5. **Follow Rarity pattern**: Similar enum structure with TryFrom, Display, Serialize/Deserialize

## Files Modified

1. Create: `zerver/src/lib/domain/card/models/scryfall_data/language.rs`
2. Update: `zerver/src/lib/domain/card/models/scryfall_data/mod.rs`
3. Update: `zerver/src/lib/domain/card/models/search_card/card_filter/mod.rs`
4. Update: `zerver/src/lib/domain/card/models/search_card/card_filter/builder/mod.rs`
5. Update: `zerver/src/lib/domain/card/models/search_card/card_filter/getters.rs`
6. Update: `zerver/src/lib/domain/card/models/search_card/card_filter/builder/setters.rs`
7. Update: `zerver/src/lib/outbound/sqlx/card/mod.rs`

## Testing Approach

- Verify language filter defaults to English only
- Test building filter with different language values
- Verify SQL generation produces correct lang = 'en' clause
- Clippy should pass with no warnings
