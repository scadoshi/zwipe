# Partner, Background & Signature Spell — Phase 4: Search UX Overhaul

**Depends on:** Phase 3 (field visibility) must be merged first. Phases 1-2 complete (`e08a218c`).

Replace the dropdown search results below the text input with **chips above the search bar** showing the top 10 results. This prevents the iOS keyboard from blocking search results.

---

## Problem

Currently, commander search shows a dropdown list of 5 results **below** the text input. On iOS, the on-screen keyboard covers the bottom half of the screen, which often overlaps or completely hides the dropdown. Users can't see what they're searching for.

This problem will get worse with 4 searchable card fields (commander, partner, background, signature spell).

## Solution

Show search results as **horizontal scrollable chips ABOVE the search bar**. As the user types, chips bubble up showing the top results. They can tap any chip to select it immediately, without the keyboard blocking visibility.

---

## Step 1: Create CommanderSearchChips Component

**File:** Create `zwiper/src/lib/inbound/screens/deck/components/card_search_chips.rs`

A reusable component that works for all 4 card search fields:

### Props

```rust
#[derive(Props, Clone, PartialEq)]
pub struct CardSearchChipsProps {
    /// Label for the field (e.g., "commander", "partner", "background", "signature spell")
    label: String,
    /// Current search query text
    search_query: Signal<String>,
    /// Search results to display as chips
    search_results: Signal<Vec<Card>>,
    /// Whether currently searching (shows loading indicator)
    is_searching: Signal<bool>,
    /// Callback when a card is selected
    on_select: EventHandler<Card>,
    /// Callback when selection is cleared
    on_clear: EventHandler<()>,
    /// Currently selected card (None if no selection)
    selected: Signal<Option<Card>>,
    /// Whether the filter toggle is on
    filter_enabled: Signal<bool>,
    /// Callback when filter toggle is tapped
    on_toggle_filter: EventHandler<()>,
    /// Label for the filter (e.g., "commander filter", "partner filter")
    filter_label: String,
}
```

### Layout (top to bottom)

```
┌──────────────────────────────────────┐
│ Selected: [card name]           [×]  │  ← Only when a card is selected
├──────────────────────────────────────┤
│ [chip1] [chip2] [chip3] [chip4] ··→  │  ← Horizontal scroll, above input
├──────────────────────────────────────┤
│ 🔍 [search input___________]        │  ← Text input
├──────────────────────────────────────┤
│ [filter label: on]                   │  ← Filter toggle button
└──────────────────────────────────────┘
```

### Chip Design

Each chip shows:
- Card name (truncated if long)
- Tap to select

Chips should be:
- Compact (small font, minimal padding)
- Horizontally scrollable (overflow-x: auto)
- Visually distinct from the filter toggle

### States

1. **No selection, no query:** Show label and empty input, no chips
2. **No selection, typing:** Show chips as results arrive, show spinner while searching
3. **No selection, results empty:** Show "no results" text in chip area
4. **Selected:** Show selected card name with clear button, hide chips and input
5. **Searching:** Show loading indicator in chip area

---

## Step 2: Increase Result Limit

**Current:** 5 results per search query
**New:** 10 results per search query

Update the search limit in the debounced search effect:
```rust
builder.set_limit(10);
```

---

## Step 3: Replace Dropdown in DeckFields

**File:** `zwiper/src/lib/inbound/screens/deck/components/deck_fields.rs`

Remove the old dropdown pattern (show_dropdown signal, dropdown div, etc.) and replace with `CardSearchChips` for each field.

### Commander field
```rust
if show_commander() {
    CardSearchChips {
        label: "commander".to_string(),
        search_query: commander_search_query,
        search_results: commander_results,
        is_searching: commander_searching,
        on_select: move |card| { /* set commander */ },
        on_clear: move |_| { /* clear commander */ },
        selected: commander,
        filter_enabled: commander_filter_enabled,
        on_toggle_filter: move |_| { /* toggle */ },
        filter_label: "commander filter".to_string(),
    }
}
```

### Partner field
```rust
if show_partner() {
    CardSearchChips {
        label: "partner".to_string(),
        // ... same pattern
        filter_label: "partner filter".to_string(),
    }
}
```

### Background field
```rust
if show_background() {
    CardSearchChips {
        label: "background".to_string(),
        // ... same pattern
        filter_label: "background filter".to_string(),
    }
}
```

### Signature spell field
```rust
if show_signature_spell() {
    CardSearchChips {
        label: "signature spell".to_string(),
        // ... same pattern
        filter_label: "spell filter".to_string(),
    }
}
```

---

## Step 4: Debounced Search Per Field

Each field needs its own debounced search effect. Extract the current debounced search pattern into a reusable hook or duplicate it for each field:

```rust
fn use_card_search(
    query: Signal<String>,
    filter_builder_fn: impl Fn(&str) -> CardFilterBuilder,
    // ... session, etc.
) -> (Signal<Vec<Card>>, Signal<bool>)
```

Or simply duplicate the effect 4 times with different signals. The duplication is acceptable since each field has slightly different filter configuration.

**Search configurations per field:**

| Field | Base filter | Additional filter |
|-------|-----------|------------------|
| Commander | `with_name_contains(query)` | `set_is_commander_in_format(format)` when toggle ON |
| Partner | `with_name_contains(query)` | `set_is_partner(true)` when toggle ON |
| Background | `with_name_contains(query)` | `set_is_background(true)` when toggle ON |
| Signature Spell | `with_name_contains(query)` | `set_is_signature_spell(true)` + `set_color_identity_within(oathbreaker_colors)` when toggle ON |

---

## Step 5: CSS for Chip Bar

The chip bar needs:
- `display: flex`, `flex-direction: row`, `overflow-x: auto`
- `gap: 8px` between chips
- Hide scrollbar: `-webkit-scrollbar: display none` or equivalent
- Chips: `border-radius` rounded, background color, compact padding
- Max height constraint so the chip area doesn't grow unbounded
- Smooth transitions when chips appear/disappear

---

## Step 6: Keyboard Behavior

On iOS/mobile:
- Tapping the search input focuses it and shows keyboard
- Chips remain visible above the input (they scroll with the page, not under the keyboard)
- Tapping a chip should dismiss the keyboard and set the selection
- The input should not lose focus while typing (chips update live via debounce)

If the keyboard still causes layout issues, consider adding scroll-into-view behavior on focus.

---

## Verification Checklist

- [ ] Commander search shows chips above input, not dropdown below
- [ ] Partner search shows chips above input
- [ ] Background search shows chips above input
- [ ] Signature spell search shows chips above input
- [ ] Chips are horizontally scrollable when >10 results
- [ ] Up to 10 results shown per field
- [ ] Tapping a chip selects the card and hides chips/input
- [ ] Clear button restores the search input
- [ ] Filter toggle works for each field
- [ ] Chips update live as user types (debounced)
- [ ] iOS keyboard does not block chips
- [ ] Loading spinner shown during search
- [ ] "No results" shown when search returns empty
- [ ] Old dropdown code removed

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwiper/.../deck/components/card_search_chips.rs` | **NEW** — reusable chip-based card search component |
| `zwiper/.../deck/components/mod.rs` | Register new module |
| `zwiper/.../deck/components/deck_fields.rs` | Replace dropdown with CardSearchChips for all 4 fields |
| CSS/styling files | Add chip bar styles |
