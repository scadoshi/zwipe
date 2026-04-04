# Maybeboard — Phase 5: Export, Import & Buy Links

**Depends on:** Phase 1 (core + backend) must be merged. Can run in parallel with Phases 2-4 if careful.

Add maybeboard support to deck export, import parsing, and buy link generation.

---

## Part A: Export

### Context

**File:** `zwiper/src/lib/inbound/screens/deck/export.rs`

The export screen fetches the deck and formats entries as `"{qty} {name}"` joined by newlines.

### Step 1: Add "Include Maybeboard" Toggle

Add a toggle signal:

```rust
let mut include_maybeboard: Signal<bool> = use_signal(|| false);
```

Render a toggle checkbox/button above or below the export text:

```
[x] include maybeboard    ← defaults OFF
```

Use the same visual style as the edit screen's form toggles.

### Step 2: Regenerate Export Text When Toggle Changes

The export text should be a memo that reacts to both the deck data and the toggle:

```rust
let export_text = use_memo(move || {
    let Some(Ok(deck)) = deck_resource() else { return String::new() };

    let mut lines: Vec<String> = Vec::new();

    // Active deck cards
    for entry in deck.entries.iter().filter(|e| !e.deck_card.maybeboard) {
        lines.push(format!("{} {}", *entry.deck_card.quantity, entry.card.scryfall_data.name));
    }

    // Maybeboard section (only if toggled on AND cards exist)
    if include_maybeboard() {
        let maybeboard: Vec<_> = deck.entries.iter()
            .filter(|e| e.deck_card.maybeboard)
            .collect();

        if !maybeboard.is_empty() {
            lines.push(String::new());           // blank line separator
            lines.push("// Maybeboard".to_string());
            for entry in maybeboard {
                lines.push(format!("{} {}", *entry.deck_card.quantity, entry.card.scryfall_data.name));
            }
        }
    }

    lines.join("\n")
});
```

### Step 3: Format Compatibility

The `// Maybeboard` header is the standard format used by:
- Moxfield: `// Maybeboard`
- Archidekt: `// Maybeboard`
- MTGO: not supported (no maybeboard concept)

This ensures exported decklists can be imported back into Zwipe or other tools.

---

## Part B: Import

### Context

**File:** `zwipe-core/src/domain/deck/requests/import_deck_cards.rs`

Phase 1 already updated the parser to detect `// Maybeboard` headers and tag ImportLines with `maybeboard: bool`. The bulk import endpoint passes this flag through to the INSERT.

### Verification

Confirm Phase 1's import changes work end-to-end:
1. Import text with `// Maybeboard` header
2. Cards before header have `maybeboard: false`
3. Cards after header have `maybeboard: true`
4. Verify via get_deck that entries have correct maybeboard flags

### Edge Cases to Test

- Multiple `// Maybeboard` headers (only the first matters — everything after any maybeboard header is maybeboard)
- Variations: `// Maybeboard`, `//Maybeboard`, `// MAYBEBOARD` (case-insensitive)
- Empty maybeboard section (header with no cards after it)
- Maybeboard header at the start (no active cards — unusual but valid)
- Re-import when cards already exist: ON CONFLICT should update both quantity AND maybeboard flag

---

## Part C: Buy Links

### Context

**File:** `zwiper/src/lib/outbound/buy_links.rs`

`tcgplayer_url()` and `cardkingdom_url()` take `&[DeckEntry]` and build URLs from all entries.

**File:** `zwiper/src/lib/inbound/screens/deck/components/more_buttons.rs` (or wherever buy links are rendered)

### Step 1: Filter Entries Before Generating URLs

The buy link functions currently receive all entries. The caller should filter:

```rust
let buy_entries: Vec<&DeckEntry> = if include_maybeboard_in_buy() {
    deck.entries.iter().collect()
} else {
    deck.entries.iter().filter(|e| !e.deck_card.maybeboard).collect()
};
```

Either:
- Change the buy link functions to accept `&[&DeckEntry]`, or
- Filter and collect into `Vec<DeckEntry>` before passing

### Step 2: Add "Include Maybeboard" Toggle in Buy Section

The buy links appear in a bottom sheet or "more" section on the deck view screen. Add a toggle:

```
[Buy on TCGplayer]
[Buy on Card Kingdom]
[ ] include maybeboard    ← defaults OFF
```

```rust
let mut include_maybeboard_in_buy: Signal<bool> = use_signal(|| false);
```

When toggled, the URLs regenerate to include/exclude maybeboard cards.

### Step 3: Update URL Generation

The URLs are likely computed in a memo. Ensure the memo reacts to the toggle signal:

```rust
let tcg_url = use_memo(move || {
    let entries = if include_maybeboard_in_buy() {
        &deck_entries
    } else {
        &active_entries  // pre-filtered
    };
    tcgplayer_url(entries)
});
```

---

## Verification Checklist

### Export
- [ ] "Include maybeboard" toggle appears on export screen (defaults OFF)
- [ ] Export without toggle: only active deck cards
- [ ] Export with toggle: active cards + `// Maybeboard` header + maybeboard cards
- [ ] Blank line before `// Maybeboard` header
- [ ] Toggle reactively updates the export text

### Import
- [ ] `// Maybeboard` header detected (case-insensitive)
- [ ] Cards before header imported as active
- [ ] Cards after header imported as maybeboard
- [ ] Re-import updates both quantity and maybeboard flag
- [ ] Edge cases handled (empty section, multiple headers, header at start)

### Buy Links
- [ ] "Include maybeboard" toggle in buy section (defaults OFF)
- [ ] URLs exclude maybeboard by default
- [ ] Toggle ON includes maybeboard cards in URLs
- [ ] URLs update reactively when toggle changes

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwiper/.../deck/export.rs` | Add include_maybeboard toggle, update export text memo |
| `zwiper/.../outbound/buy_links.rs` | May need signature update for filtered entries |
| `zwiper/.../deck/components/more_buttons.rs` | Add include_maybeboard toggle in buy section |
| `zwiper/.../deck/view.rs` or parent | Pass toggle signal to buy links component |
