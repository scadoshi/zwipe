# Partner, Background & Signature Spell — Phase 3: Frontend Field Visibility

**Depends on:** Phase 2 (search filters + validation) must be merged first.

Change the create/edit deck screens to **hide** commander-related fields until they're relevant, instead of showing them greyed out. Add the new partner, background, and signature spell fields with the same show/hide behavior.

---

## Current Behavior

The `DeckFields` component always shows the commander input field. When the selected format doesn't have a commander, the field is **disabled and greyed out** with placeholder text "select a commander format."

## New Behavior

Fields are **hidden entirely** until the format makes them relevant. When format changes, the appropriate fields animate in/out.

---

## Field Visibility Rules

| Field | Visible when | Filter toggle behavior |
|-------|-------------|----------------------|
| **Commander** | `format.has_commander()` | "Commander filter: on/off" (from Phase 3 of commander filter) |
| **Partner Commander** | `format.supports_partner()` AND commander has a partner variant keyword | "Partner filter: on/off" — filters for cards with compatible partner keyword |
| **Background** | `format.supports_background()` AND commander has "Choose a Background" | "Background filter: on/off" — filters for Background enchantments |
| **Signature Spell** | `format.has_signature_spell()` | "Spell filter: on/off" — filters for instants/sorceries within oathbreaker's color identity |

**Partner vs Background mutual exclusivity:** Only ONE of partner/background should be visible at a time, determined by the primary commander's abilities:
- If commander has "Choose a Background" → show Background field, hide Partner field
- If commander has any Partner variant → show Partner field, hide Background field
- If commander has neither → hide both (most commanders)
- If no commander selected yet → hide both

This requires reading the commander card's oracle text/keywords. The `commander` signal already holds the full `Card` object, so the data is available.

---

## Step 1: Add Signals for New Fields

**File:** `zwiper/src/lib/inbound/screens/deck/components/deck_fields.rs`

Add signals mirroring the existing commander pattern:

```rust
// Existing
let commander: Signal<Option<Card>>
let commander_display: Signal<String>

// New
let partner_commander: Signal<Option<Card>>
let partner_commander_display: Signal<String>

let background: Signal<Option<Card>>
let background_display: Signal<String>

let signature_spell: Signal<Option<Card>>
let signature_spell_display: Signal<String>
```

These need to be passed in as props from the create/edit screens (same pattern as `commander`).

---

## Step 2: Add Visibility Memos

Replace the existing `commander_enabled` memo with computed visibility signals:

```rust
let show_commander = use_memo(move || {
    selected_format().is_some_and(|f| f.has_commander())
});

let show_partner = use_memo(move || {
    selected_format().is_some_and(|f| f.supports_partner())
        && commander().is_some_and(|c| {
            let sd = &c.scryfall_data;
            let keywords = &sd.keywords;
            let oracle = sd.oracle_text.as_deref().unwrap_or("");
            keywords.iter().any(|k| k == "Partner" || k == "Friends forever" || k == "Doctor's companion")
                || oracle.to_lowercase().contains("partner with ")
        })
});

let show_background = use_memo(move || {
    selected_format().is_some_and(|f| f.supports_background())
        && commander().is_some_and(|c| {
            c.scryfall_data.oracle_text.as_deref().unwrap_or("")
                .to_lowercase()
                .contains("choose a background")
        })
});

let show_signature_spell = use_memo(move || {
    selected_format().is_some_and(|f| f.has_signature_spell())
});
```

---

## Step 3: Update DeckFields Component Layout

Currently the layout is:
1. Deck name
2. Commander (always visible, sometimes disabled)
3. Format chips

New layout:
1. Deck name
2. Format chips (moved UP — format must be selected first to know which fields to show)
3. Commander (hidden until format has commander)
4. Partner Commander OR Background (hidden until commander reveals which)
5. Signature Spell (hidden until format is Oathbreaker)

**Why move format above commander?** The user needs to pick a format FIRST for the system to know which special fields to show. This is a natural flow: "What format?" → "Who leads the deck?" → "Do they have a partner/background?" → "What's the signature spell?"

---

## Step 4: Render Hidden Fields with Conditional Rendering

Use Dioxus conditional rendering (not CSS display:none) to completely remove hidden fields from the DOM:

```rust
if show_commander() {
    // Commander input + filter toggle
    // ... existing commander UI with hide/show instead of enable/disable
}

if show_partner() {
    // Partner commander input + filter toggle
    // Same search UI pattern as commander
    // Filter auto-sets is_partner: true
}

if show_background() {
    // Background input + filter toggle
    // Same search UI pattern as commander
    // Filter auto-sets is_background: true
}

if show_signature_spell() {
    // Signature spell input + filter toggle
    // Same search UI pattern as commander
    // Filter auto-sets is_signature_spell: true
    // Also auto-sets color_identity_within from oathbreaker's color identity
}
```

---

## Step 5: Add Filter Toggles for Each Field

Each new field gets a "filter: on/off" toggle identical to the commander filter toggle (from Commander Filter Phase 3):

### Partner filter toggle
- Default: ON
- When ON: search sets `is_partner: true`
- When OFF: search shows all cards

### Background filter toggle
- Default: ON
- When ON: search sets `is_background: true`
- When OFF: search shows all cards

### Signature spell filter toggle
- Default: ON
- When ON: search sets `is_signature_spell: true` AND `color_identity_within` to oathbreaker's colors
- When OFF: search shows all cards (still respects name query)

---

## Step 6: Clear Cascading

When format changes:
- Clear commander, partner, background, signature spell
- Reset all filter toggles to ON

When commander changes:
- Clear partner and background (they depend on commander's abilities)
- Do NOT clear signature spell (it depends on format, not commander)

When commander is cleared:
- Clear partner and background

---

## Step 7: Update Create Screen

**File:** `zwiper/src/lib/inbound/screens/deck/create.rs`

Add signals for the 3 new fields. Extract IDs when submitting:

```rust
let partner_commander_id = partner_commander().map(|c| c.scryfall_data.id);
let background_id = background().map(|c| c.scryfall_data.id);
let signature_spell_id = signature_spell().map(|c| c.scryfall_data.id);
```

Pass to `HttpCreateDeckProfile::new(...)` with the new fields.

---

## Step 8: Update Edit Screen

**File:** `zwiper/src/lib/inbound/screens/deck/edit.rs`

Add signals + original state tracking for the 3 new fields. Same pattern as commander:
- Load original card from API when deck loads
- Track changes via `Optdate`
- Only send changed fields in update request

The edit screen needs to fetch the partner/background/signature_spell cards by ID on load, similar to how it fetches the commander card.

---

## Verification Checklist

- [ ] Format chips appear above commander (layout reorder)
- [ ] Commander field hidden when format has no commander
- [ ] Commander field appears when commander format selected
- [ ] Partner field hidden by default, appears only when commander has partner keyword
- [ ] Background field hidden by default, appears only when commander has "Choose a Background"
- [ ] Partner and Background never visible simultaneously
- [ ] Signature Spell field hidden by default, appears only when format is Oathbreaker
- [ ] Each field has a filter toggle (on/off) defaulting to ON
- [ ] Clearing format clears all card fields
- [ ] Changing commander clears partner and background
- [ ] Create screen passes all new IDs to API
- [ ] Edit screen loads, tracks, and saves all new fields
- [ ] All fields properly cleared on format/commander change

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwiper/.../deck/components/deck_fields.rs` | Major rewrite: add 3 field groups, visibility memos, filter toggles, layout reorder |
| `zwiper/.../deck/create.rs` | Add 3 new signals, pass IDs to create request |
| `zwiper/.../deck/edit.rs` | Add 3 new signals + original tracking, fetch cards on load, pass to update request |
