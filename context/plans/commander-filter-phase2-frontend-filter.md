# Commander Filter — Phase 2: Frontend Filter UI

**Depends on:** Phase 1 (backend) must be merged first.

Add the `is_commander_in_format` filter to the Format section of the CardFilterSheet, showing only commander formats as selectable chips.

---

## Context

The CardFilterSheet (`zwiper/src/lib/inbound/screens/deck/card/filter/card_filter_sheet.rs`) is a bottom-sheet modal with accordion sections. One of those sections is the **Format** filter (`zwiper/src/lib/inbound/screens/deck/card/filter/format.rs`), which currently shows format legality chips.

This phase adds a **Commander Eligibility** subsection within the Format filter group. It should only show the 8 commander formats as chips. When one is selected, it sets `is_commander_in_format` on the builder.

---

## Step 1: Add Commander Eligibility UI to Format Filter Section

**File:** `zwiper/src/lib/inbound/screens/deck/card/filter/format.rs`

Add a new subsection within the Format filter accordion, labeled something like "Commander Eligibility" or "Show Commanders For". This appears **above** the existing format legality chips.

**UI Design:**
- Label: "Commander Eligibility" (or "Commanders For")
- Chips: Only the 8 commander formats from `Format::commander_formats()`
- Single-select (only one format at a time — you can't be a commander in two formats simultaneously)
- Selecting a chip calls `filter_builder.write().set_is_commander_in_format(format)`
- Deselecting (tapping the active chip) calls `filter_builder.write().unset_is_commander_in_format()`
- Show the currently selected format as highlighted

**Pattern to follow:** Look at how the rarity chips or legality chips work in the existing filter sections. Same chip component, single-select behavior.

---

## Step 2: Active Indicator

The CardFilterSheet shows small dots on accordion sections that have active filters. Ensure the Format section's active indicator accounts for `is_commander_in_format` being set.

**File:** `zwiper/src/lib/inbound/screens/deck/card/filter/card_filter_sheet.rs`

Check how the Format section determines its active state and include `is_commander_in_format().is_some()` in that check.

---

## Step 3: Update `is_empty_ignoring_deck_context` Usage

If Phase 1 renamed `is_empty_ignoring_legalities` to `is_empty_ignoring_deck_context`, update all frontend call sites. Search for `is_empty_ignoring_legalities` in the zwiper crate and replace.

If Phase 1 kept the old name and added the new field to its ignore list, no changes needed here.

---

## Step 4: Clear Behavior

When the user taps "Clear Filters", the `is_commander_in_format` field should be cleared along with everything else. This should already work via the `clear()` method on `CardFilterBuilder` (which resets to defaults, and the default is `None`).

Verify this by testing:
1. Set a commander format filter
2. Tap "Clear Filters"
3. Confirm the commander chip is deselected

---

## Verification Checklist

- [ ] Commander format chips appear in Format filter section
- [ ] Only the 8 commander formats are shown (not Standard, Modern, etc.)
- [ ] Selecting a chip sets `is_commander_in_format` on the filter builder
- [ ] Deselecting clears it
- [ ] Active indicator dot appears when commander filter is set
- [ ] Clear filters resets the commander selection
- [ ] Filter works end-to-end: selecting "Commander" shows only legendary creatures + "can be your commander" cards
- [ ] Can combine with other filters (name search + commander filter)

---

## Files Modified (Summary)

| File | Change |
|------|--------|
| `zwiper/.../filter/format.rs` | Add commander eligibility chip section |
| `zwiper/.../filter/card_filter_sheet.rs` | Update active indicator for Format section |
| Possibly other filter files | Update `is_empty_ignoring_legalities` references if renamed |
