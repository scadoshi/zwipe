# Exclusive (Exclude/NOT) Filters

## Context

Zwipe's card filter system currently supports inclusive filtering: "has any/all of these." Users need the ability to *exclude* cards matching certain criteria — "show creatures with Flying but NOT Hexproof." Exclude fields are independent from include fields so both can operate simultaneously where it makes sense.

**Exclude semantics:** Always "has NONE of these" — if you exclude Flying and Trample, cards with either are rejected. No any/all distinction needed on the exclude side. Any/all toggle only applies to the include side.

**Punctuation stripping:** All text-based exclude inputs and their matched-against values strip punctuation, matching existing include behavior.

---

## UX Design: Two Categories of Filters

### Category 1: Single-value fields (include OR exclude, never both)

A card belongs to exactly ONE set, has ONE artist, ONE rarity. Including sets A, B, C already excludes everything else. Excluding set A already includes everything else. So these use a **mode toggle** — the filter section itself switches between include and exclude mode. The domain model still has separate fields, but the frontend only populates one at a time.

**Fields:** `set`, `artist`, `rarity`

**Frontend:** The existing chip UI gets an include/exclude mode toggle button. When in "exclude" mode, selected chips write to `_excludes_any` instead of `_equals_any`. Switching modes moves values between fields.

### Category 2: Multi-value fields (include AND exclude simultaneously)

A card can have MANY keywords, types, categories. "Include Flying AND exclude Hexproof" is a valid simultaneous query. These need both include and exclude to coexist.

**Fields:** `keywords`, `oracle_text` words, `type_line` other types, `card_type` basic types, `mechanical_categories`, `produced_mana`

**Frontend:** Each selected chip gets a small **include/exclude tag button** on it. When a value is selected (from search or grid), it appears as an include chip by default. Tapping the tag on any chip flips it to exclude (chip changes to `chip-exclude` style). Values are split: include-tagged → `_contains_any/all`, exclude-tagged → `_excludes`.

### Text contains fields (both inputs always visible)

Text search fields always show both a "contains" and a "not contains" input. These are independent — you can search for cards whose name contains "bolt" but whose oracle text does not contain "sacrifice."

**Fields:** `name`, `oracle_text`, `flavor_text`, `type_line`

---

## New Domain Fields (13 total)

### Multi-select excludes (Vec-based, "has none of these")
| Field | Type | Section | Category |
|-------|------|---------|----------|
| `keywords_excludes` | `Option<Vec<String>>` | keywords | multi-value |
| `oracle_text_excludes_any` | `Option<Vec<String>>` | oracle text | multi-value |
| `type_line_excludes_any` | `Option<Vec<String>>` | types | multi-value |
| `card_type_excludes_any` | `Option<Vec<CardType>>` | types | multi-value |
| `mechanical_categories_excludes` | `Option<Vec<String>>` | category | multi-value |
| `produced_mana_excludes` | `Option<Vec<String>>` | mana | multi-value |
| `set_excludes_any` | `Option<Vec<String>>` | set | single-value |
| `artist_excludes_any` | `Option<Vec<String>>` | artist | single-value |
| `rarity_excludes_any` | `Option<Rarities>` | rarity | single-value |

### Text not-contains (single string, "does NOT contain this substring")
| Field | Type | Section |
|-------|------|---------|
| `name_not_contains` | `Option<String>` | name |
| `oracle_text_not_contains` | `Option<String>` | oracle text |
| `flavor_text_not_contains` | `Option<String>` | flavor text |
| `type_line_not_contains` | `Option<String>` | types |

**Not included:** color_identity (within/equals already covers use cases), combat stats (range covers it), boolean flags, legalities, commander/partner filters, pagination/config.

---

## Phase 1: Domain Model (zwipe-core)

### 1a. CardFilter struct
**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/mod.rs`

Add 13 new fields grouped near their include counterparts:
- After `keywords_contains_all` (line 80): `keywords_excludes`
- After `oracle_text_contains_all` (line 77): `oracle_text_excludes_any`, `oracle_text_not_contains`
- After `name_contains` (line 74): `name_not_contains`
- After `type_line_contains_all` (line 86): `type_line_excludes_any`, `type_line_not_contains`
- After `card_type_contains_all` (line 88): `card_type_excludes_any`
- After `flavor_text_contains` (line 81): `flavor_text_not_contains`
- After `produced_mana_contains_all` (line 66): `produced_mana_excludes`
- After `rarity_equals_any` (line 68): `rarity_excludes_any`
- After `set_equals_any` (line 70): `set_excludes_any`
- After `artist_equals_any` (line 72): `artist_excludes_any`
- After `mechanical_categories_contains_all` (line 107): `mechanical_categories_excludes`

### 1b. CardFilterBuilder struct + Default + build()
**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/builder/mod.rs`

- Add same 13 fields to struct (lines 66-127)
- Add all as `None` in `Default::default()` (lines 129-177)
- Add 13 `.clone()` lines in `build()` (lines 586-631)
- `is_empty()` — no change needed (compares against `retain_config()` which uses `..Self::default()`)
- `retain_config()` — no change needed (uses `..Self::default()`)

### 1c. Setters
**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/builder/setters.rs`

Add new section "Exclude Filter Setters" with 13 set/unset pairs (26 methods):

**Vec<String> setters** (keywords_excludes, oracle_text_excludes_any, type_line_excludes_any, produced_mana_excludes, set_excludes_any, artist_excludes_any, mechanical_categories_excludes):
- Pattern: `IntoIterator<Item = impl Into<String>>`, `.trim()`, `.filter(!empty)`, wrap in Option
- `oracle_text_excludes_any` and `type_line_excludes_any`: also apply `strip_punctuation()`
- `keywords_excludes`, `set_excludes_any`, `artist_excludes_any`, `produced_mana_excludes`, `mechanical_categories_excludes`: trim only

**Vec<CardType> setter** (card_type_excludes_any):
- Pattern: `IntoIterator<Item = CardType>`, collect, wrap in Option

**Rarities setter** (rarity_excludes_any):
- Pattern: direct `Rarities` input, check `is_empty()`, wrap in Option

**String not-contains setters** (name_not_contains, oracle_text_not_contains, flavor_text_not_contains, type_line_not_contains):
- Pattern: `impl Into<String>`, `strip_punctuation()`, `.trim()`, empty = None

### 1d. Getters
**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/getters.rs`
**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/builder/getters.rs`

Add 13 getters to each file:
- Vec<String> → `Option<&[String]>`
- Vec<CardType> → `Option<&[CardType]>`
- Rarities → `Option<&Rarities>`
- String → `Option<&str>`

---

## Phase 2: Client-Side Filtering (zwipe-core)

**File:** `zwipe-core/src/domain/card/models/search_card/filter_cards.rs`

Add 13 exclude checks inside the `.filter()` closure. Each rejects cards matching ANY excluded value. Place each after its include counterpart.

**Pattern for Vec excludes:**
```rust
if let Some(values) = filter.keywords_excludes() {
    let excluded = match &sd.keywords {
        Some(kw) => values.iter().any(|v| kw.iter().any(|k| k.eq_ignore_ascii_case(v))),
        None => false,
    };
    if excluded { return false; }
}
```

**Pattern for text not-contains:**
```rust
if let Some(q) = filter.name_not_contains()
    && strip_punctuation(&sd.name).to_lowercase().contains(&q.to_lowercase())
{
    return false;
}
```

**Full list of checks:**
1. `name_not_contains` — strip_punctuation on name, reject if contains
2. `oracle_text_not_contains` — strip_punctuation on oracle_text, reject if contains
3. `oracle_text_excludes_any` — strip_punctuation on oracle_text, reject if any value matches
4. `keywords_excludes` — case-insensitive keyword match, reject if any matches
5. `type_line_not_contains` — strip_punctuation on type_line, reject if contains
6. `type_line_excludes_any` — strip_punctuation on type_line, reject if any value matches
7. `card_type_excludes_any` — strip_punctuation on type_line, reject if any CardType string matches
8. `mechanical_categories_excludes` — reject if any category matches
9. `produced_mana_excludes` — reject if any mana color matches
10. `rarity_excludes_any` — reject if rarity in excluded set
11. `set_excludes_any` — reject if set in excluded list
12. `artist_excludes_any` — reject if artist in excluded list
13. `flavor_text_not_contains` — strip_punctuation on flavor_text, reject if contains

---

## Phase 3: Backend SQL (zerver)

**File:** `zerver/src/lib/outbound/sqlx/card/mod.rs`

Add negated SQL clauses after each include counterpart. All compose with the existing `AND` separator.

**SQL patterns:**
| Include SQL | Exclude SQL |
|-------------|-------------|
| `keywords && ARRAY[...]::text[]` | `NOT (keywords && ARRAY[...]::text[])` |
| `STRIP_ORACLE ILIKE '%val%'` | `NOT (STRIP_ORACLE ILIKE '%val%')` |
| `STRIP_TYPE ILIKE '%val1%' OR ...` | `NOT (STRIP_TYPE ILIKE '%val1%' OR ...)` |
| `set_name = ANY($1)` | `NOT (set_name = ANY($1))` |
| `artist = ANY($1)` | `NOT (artist = ANY($1))` |
| `rarity = ANY($1)` | `NOT (rarity = ANY($1))` |
| `mechanical_categories ?| $1` | `NOT (mechanical_categories ?| $1)` |
| `produced_mana && ARRAY[...]::text[]` | `NOT (produced_mana && ARRAY[...]::text[])` |
| `STRIP_NAME ILIKE '%val%'` | `NOT (STRIP_NAME ILIKE '%val%')` |
| `regexp_replace(flavor_text...) ILIKE` | `NOT (regexp_replace(flavor_text...) ILIKE)` |

**Key:** text not-contains fields use the same `STRIP_*` constants and `regexp_replace` for punctuation-insensitive matching.

---

## Phase 4: Frontend UI (zwiper)

### 4a. Text input components — add "not contains" input

Both "contains" and "not contains" inputs always visible side by side.

**Files:**
- `zwiper/src/lib/inbound/screens/deck/card/filter/name.rs` — add `name_not_contains` input
- `zwiper/src/lib/inbound/screens/deck/card/filter/oracle_text/text_contains.rs` — add `oracle_text_not_contains` input
- `zwiper/src/lib/inbound/screens/deck/card/filter/flavor_text.rs` — add `flavor_text_not_contains` input

**Pattern:** Duplicate the existing input block with:
- Label: "{field} not contains"
- Reads from `filter_builder().{field}_not_contains()`
- Writes to `filter_builder.write().set_{field}_not_contains()`
- Clear button calls `unset_{field}_not_contains()`

### 4b. Multi-value chip components — per-chip include/exclude tag

Each selected chip gets a small include/exclude tag button. Tapping the tag flips the chip between include (normal `chip` class) and exclude (`chip-exclude` class). Values are split across include/exclude domain fields.

**Files:**
- `zwiper/src/lib/inbound/screens/deck/card/filter/oracle_text/keywords.rs` — `keywords_excludes`
- `zwiper/src/lib/inbound/screens/deck/card/filter/oracle_text/oracle_words.rs` — `oracle_text_excludes_any`
- `zwiper/src/lib/inbound/screens/deck/card/filter/types/other_types.rs` — `type_line_excludes_any`

**Implementation for searchable chips (keywords, oracle_words, other_types):**
- Track which values are excluded via a local `use_signal(HashSet<String>::new)` or by reading from the builder's exclude field
- When rendering selected chips, check if value is in exclude set → render with `chip-exclude` + tag button showing "incl"/"excl"
- Tapping the tag moves the value between include and exclude fields on the builder
- Search results filter out both included AND excluded values
- Any/All toggle only appears for include chips (exclude is always "none of these")
- read/write helpers split values: include-tagged → `_contains_any/all`, exclude-tagged → `_excludes`

**Implementation for fixed grid chips (basic_types, category, produced_mana):**
- `zwiper/src/lib/inbound/screens/deck/card/filter/types/basic_types.rs` — `card_type_excludes_any`
- `zwiper/src/lib/inbound/screens/deck/card/filter/category.rs` — `mechanical_categories_excludes`
- `zwiper/src/lib/inbound/screens/deck/card/filter/mana/produced_mana.rs` — `produced_mana_excludes`

Each chip in the grid has three states: unselected → include (tap) → exclude (tap again) → unselected (tap again). Visual:
- Unselected: default chip style
- Included: `chip selected` (existing)
- Excluded: `chip-exclude` (new style)

### 4c. Single-value chip components — include/exclude mode toggle

**Files:**
- `zwiper/src/lib/inbound/screens/deck/card/filter/set.rs` — `set_excludes_any`
- `zwiper/src/lib/inbound/screens/deck/card/filter/artist.rs` — `artist_excludes_any`
- `zwiper/src/lib/inbound/screens/deck/card/filter/rarity.rs` — `rarity_excludes_any`

**Implementation:** Add a mode toggle button (like the existing any/all toggle) that switches between "include" and "exclude" mode. When toggled:
- Move current values from `_equals_any` → `_excludes_any` (or vice versa)
- Chips render with `chip-exclude` style when in exclude mode
- Label changes: "include" / "exclude"
- All chips in the section follow the mode — no per-chip toggling needed

### 4d. Type line not-contains
**File:** `zwiper/src/lib/inbound/screens/deck/card/filter/types/other_types.rs`

Add a text input for `type_line_not_contains` within the Types section, following the same pattern as name/oracle_text/flavor_text not-contains inputs.

### 4e. Card filter sheet — active indicators and clear buttons
**File:** `zwiper/src/lib/inbound/screens/deck/card/filter/card_filter_sheet.rs`

**Active indicators** (lines 54-96) — add exclude field checks:
- `name_active`: `|| fb.name_not_contains().is_some()`
- `oracle_active`: `|| fb.oracle_text_not_contains().is_some() || fb.oracle_text_excludes_any().is_some() || fb.keywords_excludes().is_some()`
- `types_active`: `|| fb.type_line_not_contains().is_some() || fb.type_line_excludes_any().is_some() || fb.card_type_excludes_any().is_some()`
- `mana_active`: `|| fb.produced_mana_excludes().is_some()`
- `flavor_active`: `|| fb.flavor_text_not_contains().is_some()`
- `artist_active`: `|| fb.artist_excludes_any().is_some()`
- `rarity_active`: `|| fb.rarity_excludes_any().is_some()`
- `category_active`: `|| fb.mechanical_categories_excludes().is_some()`
- `set_active`: `|| fb.set_excludes_any().is_some()`

**Clear buttons** — add unset calls for exclude fields alongside existing include unsets in each section's onclick handler.

### 4f. CSS
**File:** `shared/themes.css`

Add `.chip-exclude` class:
```css
.chip-exclude {
    border-color: var(--accent-error, #c44);
    color: var(--accent-error, #c44);
}
```

---

## Phase 5: Testing & Verification

1. `cargo test --workspace` — all existing tests still pass
2. `cargo clippy --workspace --all-targets -- -D warnings` — no new warnings
3. Add unit tests in zwipe-core for filter_cards.rs:
   - Exclude keyword rejects matching card
   - Include + exclude on same field works (include Flying, exclude Hexproof)
   - Text not-contains with punctuation (e.g., "akromas" not-contains rejects "Akroma's Will")
   - Empty exclude field = no filtering
4. Manual SQL testing against dev database
5. Frontend: visual check of exclude chips in each filter section

---

## Implementation Order

```
Phase 1 (domain model) — all zwipe-core struct/setter/getter changes
    ↓
Phase 2 + Phase 3 (can parallelize)
  - Phase 2: filter_cards.rs client-side filtering
  - Phase 3: SQL query builder
    ↓
Phase 4 (frontend UI) — depends on setters/getters existing
    ↓
Phase 5 (testing)
```

**Recommended session split (per user's multi-session preference):**
- Session 1: Phase 1 (domain model complete) + Phase 2 (client-side filtering) + Phase 3 (SQL)
- Session 2: Phase 4 (frontend UI across all components)
- Session 3: Phase 5 (testing + polish)
