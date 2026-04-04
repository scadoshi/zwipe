# Commander Filter — Phase 3: Default Commander Filtering on Create/Edit

**Depends on:** Phase 2 (frontend filter) must be merged first.

Add a default-ON commander filtering toggle to the commander search on deck create and edit screens. When a commander-format deck has a format selected, the commander search automatically filters for valid commanders.

---

## Context

The create (`zwiper/src/lib/inbound/screens/deck/create.rs`) and edit (`zwiper/src/lib/inbound/screens/deck/edit.rs`) screens both use the `DeckFields` component (`zwiper/src/lib/inbound/screens/deck/components/deck_fields.rs`). This component has a commander search dropdown that appears when the deck's format has a commander.

Currently, the commander search uses `CardFilterBuilder::with_name_contains(query)` with limit=5 to search by name. It does NOT filter for commander eligibility — any card can appear.

This phase adds a toggle button near the commander search that:
- Defaults to ON when a commander format is selected
- When ON: automatically sets `is_commander_in_format` on the search filter
- When OFF: searches all cards (existing behavior)
- Label: "Commander filter: on" / "Commander filter: off"

---

## Step 1: Add Toggle State to DeckFields

**File:** `zwiper/src/lib/inbound/screens/deck/components/deck_fields.rs`

Add a signal for the toggle:
```rust
let mut commander_filter_enabled = use_signal(|| true); // ON by default
```

This signal should reset to `true` whenever the format changes (so switching from Commander to Brawl re-enables filtering for the new format's rules).

---

## Step 2: Wire Toggle into Commander Search Query

In the commander search logic (where `CardFilterBuilder::with_name_contains(query)` is built), conditionally add the commander filter:

```rust
let mut builder = CardFilterBuilder::with_name_contains(query);
if *commander_filter_enabled.read() {
    if let Some(format) = current_format {
        builder.set_is_commander_in_format(format);
    }
}
builder.set_limit(5);
let filter = builder.build();
```

This means when the toggle is ON and the deck has format Commander, the search only returns legendary creatures (etc.). When OFF, it returns any card matching the name.

---

## Step 3: Render the Toggle Button

Place a small toggle button near the commander search input. It should be:
- Visually compact (not a full chip bar, just a small toggleable text button)
- Labeled: "Commander filter: on" when enabled, "Commander filter: off" when disabled
- Styled to indicate active/inactive state (e.g., primary color when on, muted when off)
- Only visible when the deck has a commander format selected

**Suggested placement:** Directly above or below the commander search input, aligned to the right.

---

## Step 4: Validate Behavior

Users can still select any card as commander even with the filter ON by:
1. Turning the filter OFF
2. Searching for any card
3. Selecting it

When they do this with an invalid commander, `validate_deck` (from Phase 1) will show a warning like "sol ring is not a valid commander for commander format." This is intentional — the filter is a convenience, not a hard gate.

---

## Verification Checklist

- [ ] Toggle button appears on create screen when a commander format is selected
- [ ] Toggle button appears on edit screen when a commander format is selected
- [ ] Toggle defaults to ON
- [ ] When ON, commander search only shows valid commanders for the selected format
- [ ] When OFF, commander search shows all cards matching the name query
- [ ] Switching formats resets toggle to ON
- [ ] Selecting a non-commander format hides the toggle entirely
- [ ] Can still select an invalid commander with toggle OFF (validate_deck warns)
- [ ] Toggle state doesn't persist across screen navigation (always resets to ON)

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwiper/.../deck/components/deck_fields.rs` | Add toggle signal, wire into search, render toggle button |
