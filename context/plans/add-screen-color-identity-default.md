# Add Screen — Default Color Identity Filter

Pre-populate the color identity filter on the add screen to the commander's (or commanders' collective) color identity when the deck is in a format that enforces color identity.

## Current Behavior

The add screen already auto-sets **one** default filter on mount:
- **Legality:** `legalities_contains_any` set to the deck's format (add.rs:386-392)
- `is_empty_ignoring_deck_context()` ignores legality for the "is filter empty?" check
- `clear_filters()` re-applies legality after clearing

Color identity is NOT defaulted — the user must manually open the filter sheet and select their commander's colors every time.

## Target Behavior

When the deck's format enforces color identity (`format.checks_color_identity()` is true) and a commander is set:

1. **On mount:** auto-set `color_identity_within` to the union of all command zone cards' color identities (commander + partner + background)
2. **Filter dot:** the filter dot in the util bar should NOT light up from this default (same as legality)
3. **Clear filter:** re-applies color identity default alongside legality default
4. **Empty check:** `is_empty_ignoring_deck_context()` should also ignore auto-set color identity
5. **On leave (back button):** if the user never modified the filter beyond defaults, remove all defaults so the shared filter signal is clean for next entry

## Resolving Commander Color Identity

The add screen already fetches the deck on mount (add.rs:372-401). The `Deck` response contains `entries` and `deck_profile`. The commander card might be in `entries` (if it's also a deck card) or might only exist as an ID on the profile.

**Approach:** After fetching the deck, resolve color identity:

```rust
// 1. Try to find commander in entries
let commander_colors = deck.deck_profile.commander_id.and_then(|cmd_id| {
    deck.entries.iter()
        .find(|e| e.card.scryfall_data.id == cmd_id)
        .map(|e| e.card.scryfall_data.color_identity.clone())
});

// 2. If not in entries, fetch the card
let commander_colors = if commander_colors.is_none() {
    if let Some(cmd_id) = deck.deck_profile.commander_id {
        client().get_card(cmd_id, &session).await.ok()
            .map(|c| c.scryfall_data.color_identity)
    } else { None }
} else { commander_colors };
```

Then union with partner/background colors:
```rust
let mut identity_colors = commander_colors.unwrap_or_default();
// Same pattern for partner_commander_id, background_id
// Union all colors into identity_colors
```

**5-color commanders:** Apply the filter anyway — it's correct (all cards are within WUBRG) and keeps the UX consistent. The filter just won't restrict anything.

**Colorless commanders:** Apply — filters to only colorless cards, which is the correct deck-building constraint.

## Implementation

### Step 1: Extend `is_empty_ignoring_deck_context()`

**File:** `zwipe-core/src/domain/card/models/search_card/card_filter/builder/mod.rs:197-206`

Rename to keep the same name, but also ignore `color_identity_within`:

```rust
pub fn is_empty_ignoring_deck_context(&self) -> bool {
    let mut test = self.clone();
    test.unset_legalities_contains_any();
    test.unset_color_identity_within();
    test.is_empty()
}
```

**Wait — problem.** This would ignore ALL `color_identity_within` settings, including ones the user explicitly set via the filter UI. We need to distinguish "auto-set by deck context" from "user-set."

**Two options:**

**Option A: Separate flag.** Add a `color_identity_is_deck_default: bool` field to CardFilterBuilder. Set it true when auto-populating, false when user changes it via the filter UI. `is_empty_ignoring_deck_context` checks this flag.

**Option B: Accept the tradeoff.** If the user manually sets `color_identity_within` to the exact same colors as the commander, then clears filters, the dot disappears. This is actually fine UX — the filter is "effectively default" whether auto-set or manually set to the same value.

**Recommendation: Option B.** It's simpler, mirrors how legality already works (if the user manually sets the exact same legality as the deck format, `is_empty_ignoring_deck_context` still ignores it). No extra state to track.

### Step 2: Auto-populate color identity on mount

**File:** `zwiper/src/lib/inbound/screens/deck/card/add.rs` — in the mount `use_effect` (line 366)

After the existing format population block (lines 386-393), add:

```rust
// Pre-populate color identity filter from commander
if deck.deck_profile.format.as_ref().is_some_and(|f| f.checks_color_identity()) {
    let mut identity = Colors::default();

    // Resolve commander colors (from entries or fetch)
    // ... (see resolution approach above)

    // Union partner/background colors
    // ...

    if filter_builder.peek().color_identity_within().is_none() {
        filter_builder.write().set_color_identity_within(identity);
    }
}
```

The `if filter_builder.peek().color_identity_within().is_none()` guard prevents overwriting if the user already set a color identity filter (e.g. navigating back from the filter sheet).

### Step 3: Re-apply color identity on clear

**File:** `zwiper/src/lib/inbound/screens/deck/card/add.rs` — `clear_filters` closure (line 347)

After re-applying legality (lines 354-358), re-apply color identity:

```rust
// Re-apply color identity after clear
if let Some(colors) = deck_color_identity() {
    filter_builder.write().set_color_identity_within(colors);
}
```

This needs a new signal `deck_color_identity: Signal<Option<Colors>>` to cache the resolved colors from mount.

### Step 4: Clean up defaults on leave

**File:** `zwiper/src/lib/inbound/screens/deck/card/add.rs`

Track whether the user modified the filter beyond defaults:

```rust
let user_modified_filter = use_signal(|| false);
```

Set this to `true` when the filter sheet closes (in the `CardFilterSheet` on_apply or when filter_builder changes from user interaction).

On component drop / back button:
```rust
// On back/unmount: if user never touched the filter, remove defaults
if !user_modified_filter() {
    filter_builder.write().clear();
}
```

**Simpler alternative:** Always clear the filter on back. The legality/color identity defaults are cheap to re-apply on next entry. This prevents stale defaults from leaking.

**Simplest alternative:** Don't clean up at all. The existing behavior doesn't clean up legality defaults either, and `is_empty_ignoring_deck_context` handles the "is this empty?" check. The filter state persists so re-entering the add screen preserves the user's filter work — which is a feature, not a bug. Only clean up if the filter is still at defaults (never modified).

### Step 5: Store resolved colors in signal

New signal in the Add component:
```rust
let deck_color_identity: Signal<Option<Colors>> = use_signal(|| None);
```

Set during mount after resolving commander colors. Used by `clear_filters` to re-apply.

## File Changes Summary

| File | Action |
|------|--------|
| `zwipe-core/.../card_filter/builder/mod.rs` | Edit `is_empty_ignoring_deck_context()` to also unset `color_identity_within` |
| `zwiper/.../deck/card/add.rs` | Add color identity resolution on mount, re-apply on clear, new `deck_color_identity` signal |

## Edge Cases

- **No commander set:** Format has color identity but no commander assigned yet → don't default, let user see all cards
- **Commander in entries vs not:** Resolution tries entries first (no extra API call), falls back to `get_card`
- **User changes commander after entering add screen:** Stale default. Acceptable — user can clear filter or go back and re-enter. Fixing this would require watching for profile changes.
- **Deck with no format:** `checks_color_identity()` returns false → no default applied
- **Non-commander formats:** Standard, Modern, etc. → no color identity default

## Not Changing

- Remove screen — no defaults (user is removing cards they already own)
- View screen — no search filter involved
- Backend — no changes needed, `color_identity_within` already works in SQL queries
