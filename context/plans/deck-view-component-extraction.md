# Plan: Extract Deck View Screen into Components

## Goal

`zwiper/src/lib/inbound/screens/deck/view.rs` is ~480 lines with profile info, stats, four chart types, util bar, and delete dialog all inline. Extract into logical sub-components so each piece is manageable and the view screen orchestrates rather than renders everything.

## What to Extract

Three new files under `zwiper/src/lib/inbound/screens/deck/`:

### 1. `deck_profile_section.rs`

**Component:** `DeckProfileSection`

**Props:**
- `deck_profile: DeckProfile`
- `commander: Option<Card>`
- `warnings: Vec<DeckWarning>`

**Renders** (currently lines 204–245 of `view.rs`):
- `label { class: "label", "profile" }`
- Info-list with name, format, commander rows
- The `has_commander()` conditional for showing commander row
- Warnings section (if non-empty)

### 2. `deck_stats_section.rs`

**Component:** `DeckStatsSection`

**Props:**
- `metrics: DeckMetrics`

**Renders** (currently lines 249–263 of `view.rs`):
- `label { class: "label", "stats" }`
- Info-list with cards, avg cmc, lands rows
- NOTE: After this extraction is done, a follow-up task will add price stats and currency chips here. For now, just extract what exists.

### 3. `deck_charts.rs`

**Component:** `DeckCharts`

**Props (pre-computed bar data — parent computes, child renders):**
- `mana_curve_bars: [(usize, u32); 7]`
- `type_bars: Option<Vec<(&'static str, usize, u32)>>`
- `color_bars: Option<Vec<(&'static str, usize, u32)>>`
- `mana_balance_rows: Option<Vec<(&'static str, usize, usize, u32, bool)>>`

**Renders** (currently lines 266–361 of `view.rs`):
- Mana curve chart with bar labels
- Type distribution chart (if present)
- Color distribution chart (if present)
- Mana balance / cost fulfillment chart (if present)

**Also move these helpers into this file** (currently at bottom of `view.rs`):
- `fn abbreviate_type(label: &str) -> &str` (line 453)
- `fn abbreviate_color(label: &str) -> &str` (line 467)

## What Stays in `view.rs`

- All resource fetching (`deck_profile_resource`, `commander_resource`, `deck_resource`)
- All `use_effect` hooks
- All metric/bar pre-computation (lines 121–189)
- `Bouncer` wrapper, page header
- Util bar with navigation buttons (tightly coupled to router/state)
- Delete dialog (`AlertDialogRoot` and friends)

## Imports Pattern

The extracted components are private to the deck module. In `view.rs`, import via:
```rust
use super::deck_profile_section::DeckProfileSection;
use super::deck_stats_section::DeckStatsSection;
use super::deck_charts::DeckCharts;
```

## Update `deck/mod.rs`

Add three module declarations (private — only used by `view.rs`):
```rust
/// Deck profile info and warnings section for the view screen.
mod deck_profile_section;
/// Deck stats summary section for the view screen.
mod deck_stats_section;
/// Deck chart visualizations for the view screen.
mod deck_charts;
```

## Dioxus Component Props Notes

- Use `#[component]` macro on each function
- Props that are `Vec<T>` or `Option<Vec<T>>` work fine — Dioxus clones them
- `&'static str` works in props (satisfies `'static` bound) — no need to convert to `String`
- Arrays like `[(usize, u32); 7]` are `Copy` so they pass by value
- `DeckProfile`, `Card`, `DeckWarning`, `DeckMetrics` all derive `Clone`

## Existing Patterns to Follow

- See `zwiper/src/lib/inbound/components/tri_toggle.rs` for single-file component pattern
- CSS classes used: `label`, `info-list`, `info-row`, `info-row-label`, `info-row-value`, `text-muted`, `chip`, `chip selected`
- Component modules follow `mod.rs` + `component.rs` pattern in `components/`, but since these are screen-specific sections (not reusable), single `.rs` files under `screens/deck/` is the right call

## Verification

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
dx serve  # visually verify deck view renders identically
```

The deck view should look exactly the same before and after — this is a pure refactor.
