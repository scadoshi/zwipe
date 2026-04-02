# Plan: Deck Price Stats (Total Cost + Avg Cost with Currency Selection)

## Goal

Show total deck price and average price per card in the deck view stats section, with selectable currency chips (USD / EUR / TIX).

## Dependency

**Do this AFTER** the component extraction plan (`deck-view-component-extraction.md`) is complete. The price stats will be added to the extracted `DeckStatsSection` component rather than the monolithic `view.rs`.

## Step 1: Add Price Fields to DeckMetrics

**File:** `zerver/src/lib/domain/deck/models/deck_metrics.rs`

Add six fields to `DeckMetrics`:
```rust
pub total_price_usd: Option<f64>,
pub avg_price_usd: Option<f64>,
pub total_price_eur: Option<f64>,
pub avg_price_eur: Option<f64>,
pub total_price_tix: Option<f64>,
pub avg_price_tix: Option<f64>,
```

Add a helper function:
```rust
fn parse_price(price: &Option<String>) -> Option<f64> {
    price.as_ref().and_then(|s| s.parse::<f64>().ok())
}
```

In `from_entries`, add accumulators before the loop:
```rust
let mut usd_sum = 0.0f64;
let mut usd_count = 0usize;
let mut eur_sum = 0.0f64;
let mut eur_count = 0usize;
let mut tix_sum = 0.0f64;
let mut tix_count = 0usize;
```

Inside the loop, for each entry:
```rust
// Price: prefer nonfoil → foil → etched
let prices = &card.scryfall_data.prices;
if let Some(p) = parse_price(&prices.usd)
    .or_else(|| parse_price(&prices.usd_foil))
    .or_else(|| parse_price(&prices.usd_etched))
{
    usd_sum += p * qty as f64;
    usd_count += qty;
}
// Same pattern for eur, tix
```

After the loop:
```rust
let total_price_usd = if usd_count > 0 { Some(usd_sum) } else { None };
let avg_price_usd = if usd_count > 0 { Some(usd_sum / usd_count as f64) } else { None };
// Same for eur, tix
```

### Tests to Add

In the existing `tests` module:

1. **`price_aggregation_usd`** — two entries with `usd: Some("1.50")`, qty 2 and 3. Assert `total_price_usd == Some(7.50)`, `avg_price_usd == Some(1.50)`.

2. **`price_none_when_no_prices`** — entries with all prices `None`. Assert all six price fields are `None`.

3. **`price_mixed_availability`** — one card has `usd: Some("2.00")`, another has `usd: None`. Assert total/avg reflect only the priced card.

4. **`price_foil_fallback`** — card with `usd: None`, `usd_foil: Some("5.00")`. Assert `total_price_usd == Some(5.00)`.

5. **`price_quantity_multiplied`** — card with `usd: Some("1.00")`, qty 4. Assert `total_price_usd == Some(4.00)`, `avg_price_usd == Some(1.00)`.

## Step 2: Display Price Stats in DeckStatsSection

**File:** `zwiper/src/lib/inbound/screens/deck/deck_stats_section.rs` (created by the extraction plan)

Add a `use_signal` for selected currency:
```rust
let mut selected_currency = use_signal(|| "usd");
```

After the cards/avg cmc/lands info-rows, add:

**Currency chips** (using existing `.chip` / `.chip.selected` CSS):
```rust
div { class: "chip-row",
    for (label, key) in [("USD", "usd"), ("EUR", "eur"), ("TIX", "tix")] {
        div {
            class: if selected_currency() == key { "chip selected" } else { "chip" },
            onclick: move |_| selected_currency.set(key),
            "{label}"
        }
    }
}
```

**Price display** — resolve total/avg based on selected currency:
```rust
let (total, avg, symbol) = match selected_currency() {
    "eur" => (metrics.total_price_eur, metrics.avg_price_eur, "€"),
    "tix" => (metrics.total_price_tix, metrics.avg_price_tix, "tix "),
    _ => (metrics.total_price_usd, metrics.avg_price_usd, "$"),
};
```

Then two info-rows:
- "total price" → `format!("{symbol}{:.2}", value)` or "n/a" if `None`
- "avg / card" → same format or "n/a"

## Key Files
- `zerver/src/lib/domain/deck/models/deck_metrics.rs` — add fields + computation + tests
- `zwiper/src/lib/inbound/screens/deck/deck_stats_section.rs` — add currency chips + price display
- `zwiper/assets/main.css` — no changes needed (`.chip`, `.chip.selected`, `.chip-row` already exist)

## Verification
```bash
cargo test --workspace          # new price tests pass
cargo clippy --workspace --all-targets -- -D warnings
dx serve                        # deck view shows price stats with working currency toggle
```
