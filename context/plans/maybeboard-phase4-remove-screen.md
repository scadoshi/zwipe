# Maybeboard — Phase 4: Remove Screen (Tri-State Filter)

**Depends on:** Phase 3 (deck view) must be merged first.

Add a tri-state maybeboard filter to the remove screen's config section, allowing users to swipe through active cards, maybeboard cards, or all cards.

---

## Context

The remove screen (`zwiper/src/lib/inbound/screens/deck/card/remove.rs`) loads the full deck, populates `deck_cards`, then lets users swipe right to remove cards. It has a CardFilterSheet with a config section for flag filters.

---

## Step 1: Add Tri-State Maybeboard Filter

### Define the filter type

This can be a simple enum in the filter module or reuse an existing pattern:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MaybeboardFilter {
    /// Show only active deck cards (default)
    #[default]
    No,
    /// Show only maybeboard cards
    Yes,
    /// Show all cards regardless of maybeboard status
    Any,
}
```

### Add signal to remove screen

```rust
let mut maybeboard_filter: Signal<MaybeboardFilter> = use_signal(MaybeboardFilter::default);
```

---

## Step 2: Add Filter UI to Config Section

**File:** `zwiper/src/lib/inbound/screens/deck/card/filter/config.rs`

Add a "maybeboard" row in the config filter section with three chips:

```
Maybeboard: [no] [yes] [any]
```

- "no" = active deck only (default, highlighted)
- "yes" = maybeboard only
- "any" = show all

Use the same chip component pattern as existing config toggles. The chips are mutually exclusive (radio-style, not toggle).

---

## Step 3: Apply Filter to Displayed Cards

In the remove screen's filtering effect (Effect #2 that processes `filter_reset_counter`), add the maybeboard filter:

```rust
// After applying CardFilter to get filtered cards:
let filtered_by_maybeboard: Vec<Card> = match *maybeboard_filter.read() {
    MaybeboardFilter::No => filtered.into_iter()
        .filter(|card| {
            // Find the DeckEntry for this card and check maybeboard=false
            quantity_map.read().get(&card.scryfall_data.id).is_some()
            && !is_maybeboard(&card.scryfall_data.id, &deck_entries)
        })
        .collect(),
    MaybeboardFilter::Yes => filtered.into_iter()
        .filter(|card| is_maybeboard(&card.scryfall_data.id, &deck_entries))
        .collect(),
    MaybeboardFilter::Any => filtered,
};
```

You'll need access to the maybeboard status per card. Since `deck_entries` contains the full `DeckEntry` with `DeckCard.maybeboard`, create a helper:

```rust
fn is_maybeboard(card_id: &Uuid, entries: &[DeckEntry]) -> bool {
    entries.iter()
        .find(|e| e.card.scryfall_data.id == *card_id)
        .is_some_and(|e| e.deck_card.maybeboard)
}
```

Or maintain a `maybeboard_set: Signal<HashSet<Uuid>>` populated on load for O(1) lookups.

---

## Step 4: Swipe Behavior Per Filter State

When swiping right to remove:
- **Active deck card:** Delete from deck entirely (existing behavior)
- **Maybeboard card:** Delete from maybeboard entirely (same delete endpoint — removes the row)
- **Toast:** "card removed" in both cases

The swipe-right handler doesn't need to change — `delete_deck_card` removes the row regardless of maybeboard status. The toast can optionally say "removed from maybeboard" when the card was on the maybeboard, but "card removed" is fine for both.

---

## Step 5: Active Indicator for Maybeboard Filter

The CardFilterSheet shows active dots on sections with filters applied. The config section's active indicator should light up when `maybeboard_filter != MaybeboardFilter::No` (i.e., the user has changed from the default).

---

## Step 6: Update Filter Reset

When "clear filters" is tapped, reset `maybeboard_filter` to `MaybeboardFilter::No` (default).

---

## Verification Checklist

- [ ] Tri-state chips appear in config section: no | yes | any
- [ ] Default is "no" (active deck only)
- [ ] "yes" shows only maybeboard cards
- [ ] "any" shows all cards
- [ ] Swiping right removes card regardless of maybeboard status
- [ ] Active indicator dot shows when filter is changed from default
- [ ] Clear filters resets to "no"
- [ ] Filter composes correctly with other filters (name, type, etc.)
- [ ] Card count updates correctly after removal

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwiper/.../deck/card/remove.rs` | Add maybeboard_filter signal, apply to displayed cards |
| `zwiper/.../deck/card/filter/config.rs` | Add maybeboard tri-state chips |
| `zwiper/.../deck/card/filter/card_filter_sheet.rs` | Update active indicator for config section |
