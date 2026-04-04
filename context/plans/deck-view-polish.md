# Deck View Polish — Single Session

Five small UX improvements to the deck view screen. All frontend-only, no backend changes, no dependencies on other plans.

---

## Change 1: Rename "avg price / card" label

**File:** `zwiper/src/lib/inbound/screens/deck/components/deck_stats.rs`
**Line ~56**

Change:
```rust
span { class: "info-row-label", "avg price / card" }
```
To:
```rust
span { class: "info-row-label", "avg card price" }
```

That's it. One line.

---

## Change 2: Toast "card removed" on quantity decrement to 0

**File:** `zwiper/src/lib/inbound/screens/deck/card/view.rs`
**Lines ~209-221** — the `should_delete` branch in `change_quantity`

Currently, when quantity decrements to 0, the card is optimistically removed and deleted via API, but **no success toast is shown**. Add one:

```rust
if should_delete {
    // Optimistic: remove from local state
    quantity_map.write().remove(&card_id);
    deck_cards.write().retain(|c| c.scryfall_data.id != card_id);
    // Trigger re-filter
    let current = *filter_reset_counter.peek();
    filter_reset_counter.set(current + 1);

    // NEW: toast on removal
    toast.info(
        "card removed",
        ToastOptions::default().duration(Duration::from_millis(1500)),
    );

    spawn(async move {
        if let Err(e) = client().delete_deck_card(deck_id, card_id, &session).await {
            toast.error(e.to_string(), ToastOptions::default());
        }
    });
}
```

The toast fires immediately (optimistic) rather than waiting for the API response. If the API fails, the error toast will follow.

---

## Change 3: Toast "card removed" on warning remove button

**File:** `zwiper/src/lib/inbound/screens/deck/components/deck_warnings.rs`
**Line ~46** — the `Ok(())` branch of the remove button handler

Currently calls `on_remove(())` with no toast. Add one:

```rust
match result {
    Ok(()) => {
        toast.info(
            "card removed",
            ToastOptions::default().duration(Duration::from_millis(1500)),
        );
        on_remove(());
    }
    Err(e) => {
        toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
    }
}
```

---

## Change 4: "fix to N" button on copy limit warning

This is the most involved change. The copy limit warning says `"sol ring exceeds copy limit (3/1)"`. We need to add a button that sets the quantity to the legal max.

### 4a. Add action metadata to DeckWarning (zwipe-core)

**File:** `zwipe-core/src/domain/deck/models/deck_warning.rs`

The current `DeckWarning` is a struct with `message` and `scryfall_data_id`. To support different action buttons per warning type, add an optional action field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarningAction {
    /// Remove the card from the deck entirely
    Remove,
    /// Set the card's quantity to this value
    FixQuantity(i32),
    /// Clear the commander from the deck profile
    ClearCommander,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeckWarning {
    message: String,
    scryfall_data_id: Option<Uuid>,
    action: Option<WarningAction>,       // NEW
}
```

Add constructor:
```rust
impl DeckWarning {
    // ... existing new(), with_card()

    /// Creates a card-specific warning with a suggested action.
    pub fn with_action(
        message: impl Into<String>,
        scryfall_data_id: Uuid,
        action: WarningAction,
    ) -> Self {
        Self {
            message: message.into(),
            scryfall_data_id: Some(scryfall_data_id),
            action: Some(action),
        }
    }

    /// Returns the suggested action, if any.
    pub fn action(&self) -> Option<&WarningAction> {
        self.action.as_ref()
    }
}
```

Update existing constructors `new()` and `with_card()` to set `action: None`.

### 4b. Update check_copy_limits to emit FixQuantity action

**File:** `zwipe-core/src/domain/deck/models/validate_deck.rs`

In `check_copy_limits`, change the warning from `with_card` to `with_action`:

```rust
if qty > max {
    warnings.push(DeckWarning::with_action(
        format!(
            "{} exceeds copy limit ({}/{})",
            entry.card.scryfall_data.name.to_lowercase(), qty, max
        ),
        entry.card.scryfall_data.id,
        WarningAction::FixQuantity(max as i32),
    ));
}
```

### 4c. Update check_commander_eligibility to emit ClearCommander action

**File:** `zwipe-core/src/domain/deck/models/validate_deck.rs`

In `check_commander_eligibility` (added by Commander Filter Phase 1), change the warning to use `ClearCommander`:

```rust
if !is_valid_commander(card, format) {
    warnings.push(DeckWarning::with_action(
        format!(
            "{} is not a valid commander for {}",
            card.scryfall_data.name.to_lowercase(),
            format.display_name().to_lowercase()
        ),
        card.scryfall_data.id,
        WarningAction::ClearCommander,
    ));
}
```

**Note:** If Commander Filter Phase 1 hasn't been merged yet, skip this sub-step. The plan for that phase can include this action when it adds the eligibility check.

### 4d. Update DeckWarnings component to render different buttons per action

**File:** `zwiper/src/lib/inbound/screens/deck/components/deck_warnings.rs`

Currently, every card-specific warning gets a "remove" button. Replace with action-aware rendering:

```rust
if let Some(card_id) = warning.scryfall_data_id() {
    match warning.action() {
        // "fix to N" button — sets quantity to legal max
        Some(WarningAction::FixQuantity(target_qty)) => {
            let target_qty = *target_qty;
            rsx! {
                button {
                    class: "btn-xs",
                    style: "color: var(--color-warning); border-color: var(--border-warning); margin-bottom: 0;",
                    onclick: move |_| {
                        on_fix_quantity((card_id, target_qty));
                    },
                    "fix to {target_qty}"
                }
            }
        }
        // "clear" button — clears the commander
        Some(WarningAction::ClearCommander) => {
            rsx! {
                button {
                    class: "btn-xs",
                    style: "color: var(--color-warning); border-color: var(--border-warning); margin-bottom: 0;",
                    onclick: move |_| {
                        on_clear_commander(());
                    },
                    "clear"
                }
            }
        }
        // Default: "remove" button (existing behavior)
        _ => {
            rsx! {
                button {
                    class: "btn-xs",
                    style: "color: var(--color-warning); border-color: var(--border-warning); margin-bottom: 0;",
                    onclick: move |_| {
                        // ... existing delete logic
                    },
                    "remove"
                }
            }
        }
    }
}
```

### 4e. Add new event handlers to DeckWarnings props

The component needs two new callbacks:

```rust
#[component]
pub(crate) fn DeckWarnings(
    warnings: Vec<DeckWarning>,
    deck_id: Uuid,
    on_remove: EventHandler<()>,
    on_fix_quantity: EventHandler<(Uuid, i32)>,      // NEW: (card_id, target_qty)
    on_clear_commander: EventHandler<()>,             // NEW
) -> Element
```

### 4f. Implement fix_quantity handler in deck view

**File:** `zwiper/src/lib/inbound/screens/deck/card/view.rs` (or wherever DeckWarnings is rendered)

The parent screen needs to handle the `on_fix_quantity` callback:

```rust
on_fix_quantity: move |(card_id, target_qty): (Uuid, i32)| {
    let current_qty = quantity_map.peek().get(&card_id).copied().unwrap_or(1);
    let delta = target_qty - current_qty;  // Negative delta to reduce

    // Optimistic update
    quantity_map.write().entry(card_id).and_modify(|q| *q = target_qty);

    let request = HttpUpdateDeckCard::new(delta);
    spawn(async move {
        session.upkeep(client);
        let Some(session) = session() else { return };
        match client().update_deck_card(deck_id, card_id, &request, &session).await {
            Ok(_) => {
                toast.info(
                    format!("quantity set to {target_qty}"),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            Err(e) => {
                // Rollback
                quantity_map.write().entry(card_id).and_modify(|q| *q = current_qty);
                toast.error(e.to_string(), ToastOptions::default());
            }
        }
    });
}
```

### 4g. Implement clear_commander handler

```rust
on_clear_commander: move |_| {
    spawn(async move {
        session.upkeep(client);
        let Some(session) = session() else { return };
        let request = HttpUpdateDeckProfile::new(
            None,                           // no name change
            Opdate::Set(None),             // clear commander
            Opdate::Unchanged,             // no format change
        );
        match client().update_deck_profile(deck_id, &request, &session).await {
            Ok(_) => {
                toast.info(
                    "commander cleared",
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
                // Refresh the deck data
                // (trigger deck resource reload or navigate)
            }
            Err(e) => {
                toast.error(e.to_string(), ToastOptions::default());
            }
        }
    });
}
```

**Import needed:** Add `HttpUpdateDeckProfile`, `Opdate`, and `ClientUpdateDeckProfile` to the imports in the file where this handler lives.

---

## Change 5: Audit other warnings for convenience actions

Based on the analysis, here's the complete action mapping:

| Warning type | Current | New action | Rationale |
|-------------|---------|-----------|-----------|
| Card count (deck-level) | no button | no button | No single-card fix possible |
| Commander required (deck-level) | no button | no button | User must pick one on edit screen |
| Not legal / Banned | "remove" | "remove" | Only fix is removal |
| Exceeds copy limit | "remove" | **"fix to N"** | Better than removing entirely |
| Outside color identity | "remove" | "remove" | Only fix is removal |
| Invalid commander | "remove" | **"clear"** | Clears commander from deck profile |

No other warnings need new actions. The audit is complete — document this in the todo as done.

---

## Verification Checklist

- [ ] "avg card price" label on stats table
- [ ] Toast "card removed" when decrementing quantity to 0
- [ ] Toast "card removed" when using warning remove button
- [ ] `WarningAction` enum with `Remove`, `FixQuantity(i32)`, `ClearCommander`
- [ ] Copy limit warning has "fix to N" button
- [ ] "fix to N" correctly sets quantity via delta update
- [ ] Toast "quantity set to N" after fix
- [ ] Invalid commander warning has "clear" button
- [ ] "clear" sends update_deck_profile with `commander_id: Set(None)`
- [ ] Toast "commander cleared" after clear
- [ ] All other card-specific warnings still show "remove"
- [ ] All remove buttons show "card removed" toast
- [ ] Optimistic updates with rollback on error

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwipe-core/.../deck/models/deck_warning.rs` | Add `WarningAction` enum, `with_action()` constructor, `action()` getter |
| `zwipe-core/.../deck/models/validate_deck.rs` | Copy limit uses `FixQuantity`, commander eligibility uses `ClearCommander` |
| `zwiper/.../deck/components/deck_stats.rs` | Rename label to "avg card price" |
| `zwiper/.../deck/components/deck_warnings.rs` | Action-aware button rendering, new props, "card removed" toast |
| `zwiper/.../deck/card/view.rs` | Toast on delete, fix_quantity handler, clear_commander handler |
| `zwiper/.../deck/view.rs` | Pass new event handlers to DeckWarnings component |
