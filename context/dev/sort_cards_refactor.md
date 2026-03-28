# Refactor: Extract `SortCards` Trait from Inline Sort Logic

## Problem

`view.rs` and `remove.rs` both contain an identical ~50-line inline sort block
to handle the case where the user sets a sort order but no other filter criteria.
This is a workaround for `CardFilterBuilder::is_empty()` treating `order_by` as
a config field (not a search criterion), which causes `build()` to return `Err`
and the filter effect to bypass `filter_by` entirely.

The workaround works but duplicates the sort logic that already lives in
`filter_cards.rs`. Any future `OrderByOption` variant addition requires changes
in three places.

---

## Proposed Solution

Add a `SortCards` extension trait to the shared domain, alongside the existing
`FilterCards` trait in `filter_cards.rs`.

### Trait Definition

```rust
/// Extension trait for sorting a `Vec<Card>` in-place using a `CardFilterBuilder`.
pub trait SortCards {
    fn sort_by_filter(&mut self, builder: &CardFilterBuilder);
}

impl SortCards for Vec<Card> {
    fn sort_by_filter(&mut self, builder: &CardFilterBuilder) {
        use OrderByOption::*;
        let Some(order_by) = builder.order_by() else { return };

        if order_by == Random {
            use rand::seq::SliceRandom;
            self.shuffle(&mut rand::rng());
            return;
        }

        let ascending = builder.ascending();
        self.sort_by(|a, b| {
            let sd_a = &a.scryfall_data;
            let sd_b = &b.scryfall_data;
            let ord = match order_by {
                Name => sd_a.name.cmp(&sd_b.name),
                Cmc => {
                    let ca = sd_a.cmc.unwrap_or(f64::MAX);
                    let cb = sd_b.cmc.unwrap_or(f64::MAX);
                    ca.partial_cmp(&cb).unwrap_or(Ordering::Equal)
                }
                Power => {
                    let pa = sd_a.power.as_deref().and_then(|p| p.parse::<i32>().ok()).unwrap_or(i32::MAX);
                    let pb = sd_b.power.as_deref().and_then(|p| p.parse::<i32>().ok()).unwrap_or(i32::MAX);
                    pa.cmp(&pb)
                }
                Toughness => {
                    let ta = sd_a.toughness.as_deref().and_then(|t| t.parse::<i32>().ok()).unwrap_or(i32::MAX);
                    let tb = sd_b.toughness.as_deref().and_then(|t| t.parse::<i32>().ok()).unwrap_or(i32::MAX);
                    ta.cmp(&tb)
                }
                Rarity => sd_a.rarity.to_long_name().cmp(&sd_b.rarity.to_long_name()),
                ReleasedAt => sd_a.released_at.cmp(&sd_b.released_at),
                PriceUsd => {
                    let pa = sd_a.prices.usd.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    let pb = sd_b.prices.usd.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    pa.partial_cmp(&pb).unwrap_or(Ordering::Equal)
                }
                PriceEur => {
                    let pa = sd_a.prices.eur.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    let pb = sd_b.prices.eur.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    pa.partial_cmp(&pb).unwrap_or(Ordering::Equal)
                }
                PriceTix => {
                    let pa = sd_a.prices.tix.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    let pb = sd_b.prices.tix.as_deref().and_then(|p| p.parse::<f64>().ok()).unwrap_or(f64::MAX);
                    pa.partial_cmp(&pb).unwrap_or(Ordering::Equal)
                }
                Random => Ordering::Equal,
            };
            if ascending { ord } else { ord.reverse() }
        });
    }
}
```

### Usage After Refactor

Both filter effects become:

```rust
let mut filtered = if builder.is_empty() {
    all_cards
} else {
    let mut b = builder.clone();
    b.set_limit(10_000);
    b.set_offset(0);
    match b.build() {
        Ok(filter) => all_cards.filter_by(&filter),
        Err(_) => deck_cards.peek().clone(),
    }
};

// Sort-only case: filter_by was bypassed when builder.is_empty()
if builder.is_empty() {
    filtered.sort_by_filter(&builder);
}
```

The inline 50-line sort blocks in `view.rs` and `remove.rs` are replaced with
a single `filtered.sort_by_filter(&builder)` call.

---

## Files to Modify

| File | Change |
|---|---|
| `zerver/src/lib/domain/card/models/search_card/filter_cards.rs` | Add `SortCards` trait + impl |
| `zwiper/src/lib/inbound/screens/deck/card/view.rs` | Replace inline sort block with `filtered.sort_by_filter(&builder)` |
| `zwiper/src/lib/inbound/screens/deck/card/remove.rs` | Same as above |

---

## Notes

- The sort logic should NOT be de-duplicated inside `filter_by` itself (i.e., extract
  a private `sort_vec` helper that both `filter_by` and `sort_by_filter` call) to
  avoid changing the `filter_by` signature or internal structure unnecessarily.
- `SortCards` is a no-op when `builder.order_by()` is `None`, making it safe to
  always call without the `is_empty()` guard.
- `OrderByOption` and `CardFilterBuilder` are both already in scope in `filter_cards.rs`.
