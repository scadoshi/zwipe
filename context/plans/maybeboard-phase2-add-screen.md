# Maybeboard — Phase 2: Add Screen (Swipe Up to Maybeboard)

**Depends on:** Phase 1 (core + backend) must be merged first.

Add up-swipe gesture on the add screen to add cards to the maybeboard. Cards already in the deck (active or maybeboard) should not appear in search results.

---

## Step 1: Add SwipeAction::Maybeboard Variant

**File:** `zwiper/src/lib/inbound/screens/deck/card/components/action_history.rs`

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum SwipeAction {
    Skip(Box<Card>),       // Swipe left — skip card
    Do(Box<Card>),         // Swipe right — add to deck
    Maybeboard(Box<Card>), // Swipe up — add to maybeboard    // NEW
}
```

---

## Step 2: Add Up Direction to SwipeConfig

**File:** `zwiper/src/lib/inbound/screens/deck/card/add.rs`

The add screen's SwipeConfig currently allows Left, Right, and Down:

```rust
SwipeConfig::new(
    vec![Direction::Left, Direction::Right, Direction::Down],
    150.0,
    5.0
)
```

Add `Direction::Up`:

```rust
SwipeConfig::new(
    vec![Direction::Left, Direction::Right, Direction::Up, Direction::Down],
    150.0,
    5.0
)
```

---

## Step 3: Add Up-Swipe Handler

In the add screen's swipe event handling, add the up-swipe case:

```rust
on_swipe_up: move |_| {
    let Some(card) = current_card() else { return };
    let card_id = card.scryfall_data.id;

    // Defensive: if card is somehow already in deck, just add to maybeboard
    // (shouldn't happen due to filtering in Step 4, but be safe)
    spawn(async move {
        let request = HttpCreateDeckCard::new(
            &card_id.to_string(),
            1,
            Some(true),  // maybeboard = true
        );
        match client().create_deck_card(deck_id, &request, &session).await {
            Ok(deck_card) => {
                // Track in deck_cards_ids so it's excluded from future results
                deck_cards_ids.write().insert(card_id);
                // Record action for undo
                action_history.write().push(SwipeAction::Maybeboard(Box::new(card.clone())));
                toast.info(
                    "added to maybeboard",
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            Err(e) => {
                toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
            }
        }
    });
    // Advance to next card
    current_index += 1;
}
```

---

## Step 4: Exclude Maybeboard Cards from Search Results

The add screen tracks `deck_cards_ids: Signal<HashSet<Uuid>>` to skip cards already in the deck. On mount, it fetches the deck and populates this set.

Currently it only adds active deck cards to this set. Update to include maybeboard cards too:

```rust
// On mount, when populating deck_cards_ids:
for entry in deck.entries.iter() {
    deck_cards_ids.write().insert(entry.card.scryfall_data.id);
    // This now includes both active AND maybeboard cards
    // because DeckEntry includes all cards from get_deck
}
```

Since get_deck now returns all entries (active + maybeboard) with the `maybeboard` flag on DeckCard, the existing logic of "add all entry IDs to the skip set" already works. Just verify the code doesn't filter by maybeboard before building the skip set.

---

## Step 5: Undo Support for Maybeboard

The Down-swipe (undo) handler needs to handle the new `SwipeAction::Maybeboard` variant:

```rust
Direction::Down => {
    // Undo last action
    if let Some(last_action) = action_history.write().pop() {
        match last_action {
            SwipeAction::Skip(card) => {
                // Re-add to card stack (existing behavior)
                current_index -= 1;
            }
            SwipeAction::Do(card) => {
                // Delete from deck (existing behavior)
                spawn(async move {
                    client().delete_deck_card(deck_id, card.scryfall_data.id, &session).await.ok();
                    deck_cards_ids.write().remove(&card.scryfall_data.id);
                });
                current_index -= 1;
            }
            SwipeAction::Maybeboard(card) => {
                // Delete from maybeboard (same as undo-add — card gets removed entirely)
                spawn(async move {
                    client().delete_deck_card(deck_id, card.scryfall_data.id, &session).await.ok();
                    deck_cards_ids.write().remove(&card.scryfall_data.id);
                });
                current_index -= 1;
            }
        }
    }
}
```

---

## Step 6: Visual Indicator for Up-Swipe

Users need to know up-swipe is available. Options:
- A small text hint below the card: "swipe up → maybeboard"
- A visual arrow or icon during the swipe gesture
- The card could tilt upward slightly during an up-swipe (matching the existing tilt behavior for left/right)

Follow the existing swipe hint pattern on the add screen. If there's already a hint for "swipe left to skip, right to add", extend it to include "up for maybeboard".

---

## Step 7: Update Client API

> **NOTE: No separate toggle client was created. Maybeboard toggling uses the existing `ClientUpdateDeckCard::update_deck_card()` with `HttpUpdateDeckCard::new(None, Some(true/false))`. The `HttpCreateDeckCard` struct includes `maybeboard: Option<bool>` for creating cards directly onto the maybeboard.**

---

## Verification Checklist

- [ ] SwipeAction::Maybeboard variant exists
- [ ] Up-swipe adds card with `maybeboard: true`
- [ ] Toast shows "added to maybeboard" on up-swipe
- [ ] Cards already in deck (active OR maybeboard) excluded from search results
- [ ] Undo (down-swipe) correctly deletes maybeboard cards
- [ ] Visual hint for up-swipe present
- [ ] Client sends maybeboard flag on create
- [ ] Maybeboard toggling uses existing `update_deck_card` client (no separate toggle method)

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwiper/.../deck/card/components/action_history.rs` | Add SwipeAction::Maybeboard |
| `zwiper/.../deck/card/add.rs` | Add Direction::Up, up-swipe handler, undo for maybeboard |
| `zwiper/.../deck/card/remove.rs` | Add Direction::Up, move-to-maybeboard handler via update_deck_card, undo |
| `zwiper/assets/main.css` | Add card-exit-up animation |
