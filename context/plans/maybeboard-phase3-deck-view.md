# Maybeboard — Phase 3: Deck View (Toggle + Move Buttons)

**Depends on:** Phase 2 (add screen) must be merged first.

Add maybeboard display and card movement controls to the deck card view screen.

---

## Context

The deck card view screen (`zwiper/src/lib/inbound/screens/deck/card/view.rs`) currently has:
- **Grouping chips:** CardType | Cmc | Color
- **Show toggles:** "show lands" | "show tokens"
- **Column headers:** Qty | Name | CMC | P/T | Colors
- **Sections rendered:** Tokens (if toggled) → Commander (pinned) → Grouped cards

Display priority per the todo: tokens → maybeboard → lands.

---

## Step 1: Add Show Maybeboard Toggle Signal

**File:** `zwiper/src/lib/inbound/screens/deck/card/view.rs`

Add signal alongside existing toggles:

```rust
let mut show_lands: Signal<bool> = use_signal(|| false);
let mut show_tokens: Signal<bool> = use_signal(|| false);
let mut show_maybeboard: Signal<bool> = use_signal(|| false);  // NEW
```

---

## Step 2: Separate Active and Maybeboard Entries

When the deck loads, separate entries into two collections:

```rust
// After deck loads and entries are populated:
let active_cards: Signal<Vec<Card>> = use_memo(move || {
    deck_entries()
        .iter()
        .filter(|e| !e.deck_card.maybeboard)
        .map(|e| e.card.clone())
        .collect()
});

let maybeboard_cards: Signal<Vec<Card>> = use_memo(move || {
    deck_entries()
        .iter()
        .filter(|e| e.deck_card.maybeboard)
        .map(|e| e.card.clone())
        .collect()
});
```

The existing `deck_cards` signal should now only contain active cards. Filtering, grouping, and display use the active cards. Maybeboard cards render in their own section.

---

## Step 3: Add Toggle Button to Controls Bar

The controls bar currently has grouping chips + show toggles. Add maybeboard toggle in display priority order:

```
[CardType] [Cmc] [Color] | [tokens] [maybeboard] [lands]
```

The todo says: "Consolidate grouping/show controls bar to accommodate the new toggle without crowding." If the bar is tight, consider:
- Moving show toggles to a second row
- Using smaller chip styling for toggles
- Using icons instead of text

The maybeboard toggle button:
- Label: "maybeboard" (or "maybe")
- Active state: highlighted when `show_maybeboard` is true
- Shows count: "maybeboard (3)" if 3 cards are on the maybeboard
- Hidden entirely if maybeboard is empty (no point showing a toggle for an empty section)

---

## Step 4: Render Maybeboard Section

When `show_maybeboard` is true, render maybeboard cards between tokens and lands (per display priority):

```
[Tokens section]           ← if show_tokens
[Maybeboard section]       ← if show_maybeboard    // NEW
[Commander]                ← pinned
[Grouped active cards]     ← filtered by show_lands
```

The maybeboard section:
- Section header: "maybeboard" (styled like the tokens section header)
- Lists maybeboard cards using the same `CardRow` component
- Each card shows quantity from `quantity_map`
- Each card shows a **"to deck"** button (see Step 5)
- Cards are sorted alphabetically or by the current group_by_option (follow active card grouping)

---

## Step 5: Add "To Deck" and "To Maybeboard" Buttons

### On maybeboard cards: "To Deck" button

When tapped:
1. Call `client.update_deck_card(deck_id, scryfall_data_id, &HttpUpdateDeckCard::new(None, Some(false)), &session)` — sets maybeboard=false
2. On success: move card from maybeboard_cards to active cards in local state (optimistic update)
3. Toast: "added to deck"

### On active deck cards: "To Maybeboard" button

When tapped:
1. Call `client.update_deck_card(deck_id, scryfall_data_id, &HttpUpdateDeckCard::new(None, Some(true)), &session)` — sets maybeboard=true
2. On success: move card from active to maybeboard in local state
3. Toast: "moved to maybeboard"

**UI placement for these buttons:** Add to the expanded CardRow view. When a card row is tapped/expanded, it currently shows full card details. Add a small button in the expanded area:
- Active cards: show "to maybeboard" button
- Maybeboard cards: show "to deck" button

Alternatively, add it as an icon button in the compact row (next to the +/- quantity buttons). Choose whichever fits the existing layout better.

---

## Step 6: Quantity Controls on Maybeboard Cards

Maybeboard cards should have the same +/- quantity controls as active deck cards. The user might want to maybeboard 3 copies of a card to remember they need 3 if they decide to include it.

Use the same `on_qty_change` handler pattern as active cards — it calls `update_deck_card` with a quantity delta.

---

## Step 7: Maybeboard Count in Deck Stats

**File:** `zwiper/src/lib/inbound/screens/deck/components/deck_stats.rs`

Consider adding a "maybeboard" row to the stats table showing the count of maybeboard cards. This is optional and low priority — the toggle button count (Step 3) may be sufficient.

---

## Verification Checklist

- [ ] "Maybeboard" toggle appears in controls bar (only when maybeboard has cards)
- [ ] Toggle shows card count: "maybeboard (N)"
- [ ] Maybeboard section renders between tokens and grouped cards
- [ ] Maybeboard cards use same CardRow component
- [ ] "To deck" button on maybeboard cards works (toggles flag, moves card)
- [ ] "To maybeboard" button on active cards works
- [ ] Toast "added to deck" on to-deck action
- [ ] Toast "moved to maybeboard" on to-maybeboard action
- [ ] Quantity +/- controls work on maybeboard cards
- [ ] Active card metrics/grouping exclude maybeboard cards
- [ ] Maybeboard cards don't appear in grouped active cards section
- [ ] Empty maybeboard hides the toggle entirely

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwiper/.../deck/card/view.rs` | Add maybeboard signal, section rendering, entry separation |
| `zwiper/.../deck/card/components/card_row.rs` | Add "to deck" / "to maybeboard" button in expanded view |
| `zwiper/.../deck/components/deck_stats.rs` | Optional: add maybeboard count |
