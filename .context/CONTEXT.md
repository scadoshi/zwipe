# Session Context for Next AI

## Project Overview
**zwipe** - A Rust full-stack application for Magic: The Gathering card management
- **Frontend**: Dioxus 0.7 (Rust reactive framework, compiles to WASM)
- **Backend**: Axum (Rust web framework)
- **Architecture**: Hexagonal (domain/inbound/outbound separation)
- **Database**: SQLx with PostgreSQL

## Key Directories
- `zwiper/` - Frontend Dioxus application
- `zerver/` - Backend Axum server
- `zwiper/assets/` - CSS files (main.css, accordion.css, alert-dialog.css, toast.css)
- `zwiper/src/lib/inbound/screens/deck/card/filter/` - Filter components

## What Was Accomplished This Session

### 1. CSS Color Variable System
Implemented CSS custom properties across all CSS files in `zwiper/assets/`:
- `:root` variables for backgrounds, text, borders, status colors
- Solid hex colors (user rejected rgba opacity approach)
- Key variables: `--bg-primary`, `--text-primary`, `--border-primary`, etc.

### 2. Stepper Controls Implementation
Replaced text inputs with stepper controls `[ - ] value [ + ]` for integer filters:

**Files Modified:**
- `zwiper/src/lib/inbound/screens/deck/card/filter/combat.rs` - Power/toughness steppers
- `zwiper/src/lib/inbound/screens/deck/card/filter/mana/mod.rs` - CMC kept as text input (supports decimals)
- `zwiper/assets/main.css` - Stepper CSS styles

**Key Decision:** CMC remains text input because it supports decimal values (e.g., 2.5). Power/toughness are always integers so use steppers.

### 3. Generic FilterMode Enum
Created `zwiper/src/lib/inbound/screens/deck/card/filter/filter_mode.rs`:
```rust
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum FilterMode {
    #[default]
    Exact,
    Within,
}
```
- Replaces the old `ColorIdentityFilterMode`
- Used by CMC, power, toughness, and color identity filters
- Has `toggle()` method and Display impl

### 4. Option<i32> Pattern for Filters
Critical pattern where `None` = no filter active, `Some(value)` = filter with value:
```rust
let mut power_equals = use_signal(|| filter_builder().power_equals()); // Option<i32>
```
- Clear button sets to `None`, not `Some(0)`
- Display shows "-" when None
- Steppers default to 0 when incrementing from None

### 5. Stepper Size Reduction
Reduced by ~20% to fit within 85% accordion width:
- Button padding: `0.4rem 0.8rem` → `0.3rem 0.6rem`
- Font sizes: `1rem` → `0.85rem`
- Value width: `3rem` → `2rem`

## Current Bugs (Unresolved)

### Bug 1: Combat Flickering in Within Mode
**Symptom:** Setting min/max on power or toughness in Within mode causes cards to flick through continuously (not infinite API calls, just cycling through batch).

**Attempted Fix:** Batch `filter_builder.write()` calls by holding the write guard:
```rust
let mut fb = filter_builder.write();
fb.unset_power_equals();
fb.set_power_range(...);
// fb drops here - single notification
```
**Result:** Did not fix the issue.

**Root Cause Theory:** Might be a reactive loop between:
1. "sync TO filter_builder" effect (local signals → filter_builder)
2. "sync FROM filter_builder" effect (filter_builder → local signals)

Each triggers the other, causing continuous re-renders.

**Alternative Approach to Try:** Don't allow partial range values. Only sync to filter_builder when BOTH min AND max are set. If user only sets one, keep the range as None until both are set.

### Bug 2: CMC Filter Can't Set Values
**Symptom:** Typing into CMC input doesn't work - values get cleared immediately.

**Root Cause:** The "sync FROM" effect clears local string when `filter_builder().cmc_equals()` is None. But it's always None until blur (when parsing happens). So typing triggers effect, effect sees None, clears input.

**Attempted Fix:** Track previous filter_builder state with signals, only clear if value WAS set and is NOW None (external clear).
```rust
let mut prev_cmc_equals = use_signal(|| filter_builder().cmc_equals());
// Only clear if prev was Some and current is None
if prev_cmc_equals().is_some() && current_cmc_equals.is_none() {
    cmc_equals_string.set(String::new());
}
prev_cmc_equals.set(current_cmc_equals);
```
**Result:** Did not fix the issue.

**Alternative Approaches to Try:**
1. Remove sync FROM for CMC entirely, handle clear differently
2. Use a "dirty" flag when user is typing
3. Don't sync CMC at all - let the input manage its own state

## File Reference

### `zwiper/src/lib/inbound/screens/deck/card/filter/combat.rs`
Power and toughness filter with steppers. Key sections:
- Lines 11-32: Signal initialization with Option<i32>
- Lines 34-61: Sync TO effect for power (batched writes)
- Lines 63-90: Sync TO effect for toughness (batched writes)
- Lines 92-105: Sync FROM effect (handles external clear)
- Lines 140-211: Stepper UI for power (Exact and Within modes)
- Lines 236-306: Stepper UI for toughness (Exact and Within modes)

### `zwiper/src/lib/inbound/screens/deck/card/filter/mana/mod.rs`
CMC and color identity filter. Key sections:
- Lines 14-39: CMC signal initialization (strings for text input)
- Lines 41-69: CMC parsing functions (try_parse_cmc_equals, try_parse_cmc_range)
- Lines 71-89: Color identity signal initialization
- Lines 91-125: Sync TO effect for color identity
- Lines 127-156: Sync FROM effect (with prev state tracking for CMC)
- Lines 171-227: CMC input UI (text inputs, not steppers)
- Lines 251-270: Color identity chip UI

### `zwiper/assets/main.css`
- CSS variables at top in `:root`
- Stepper styles: `.stepper`, `.stepper-btn`, `.stepper-value`
- Input styles: `.input-compact`, `.input-narrow`

### `zwiper/assets/accordion.css`
- `.accordion-content .flex-col { width: 85%; }` - User set this width

## Dioxus 0.7 Specifics
- `use_signal` - Creates reactive signal
- `use_effect` - Runs when dependencies change (auto-tracked)
- `use_context` - Gets signal from parent provider
- `signal.write()` - Returns mutable ref, notifies on drop
- `signal()` - Reads current value, subscribes to changes
- `signal.set(value)` - Sets value, notifies subscribers

## User Preferences
- Accordion content at 85% width
- Clear button should set to None, not zero
- Prefers solid hex colors over rgba opacity
- Wants steppers to fit within accordion borders
- CMC should stay as text input (supports decimals)

## Remaining Tasks
1. Fix combat flickering bug
2. Fix CMC filter bug
3. Update toast styles for overlay stacking
4. Toasts should dissipate when changing screens
