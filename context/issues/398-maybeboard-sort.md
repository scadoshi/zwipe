# 398. The Commander maybeboard has no ordering

State: fixing | Owner: laptop | PR: #601 | Build seen: release after #599

## Report
Scotty, via the composer, 2026-11-11: give the Commander maybeboard ordering with the Decks list's Sort chip row (same component, no new look): Newest saved (the default), Name, Mana value and Color. Remember the choice, on the web and in the app.

## Fix
laptop, #601, web and app:
- **The order.** `zwipe_ui::commander::maybeboard::{SortBy, Sort}` hold the keys and the comparison, with ties falling back to the name:
  - Newest saved is the saved order, newest first ascending;
  - Mana value orders by `cmc`;
  - Color puts fewer colors first, then WUBRG order.
  `Sort::key` and `from_key` store it as text.
- **The screen model.** `screens::commander_maybeboard::State` carries the sort (`with_sort`, `Action::Sort`, `Effect::RememberSort`), and `shown` applies it before any sort the card filter picks.
- **Web.** A Sort row after Show uses the Decks list's chip row and direction chip, with a ghost row while loading. `MaybeboardSortStore` keeps the choice in `localStorage` under `zwiper-web.maybeboard-sort`.
- **App.** The same row and ghost. `maybeboard_sort_store` keeps the choice across launches, in the keyring on Apple and desktop and in a file on Android, as the count scope does.
- **Tests:** each key's order and name tie-break, the key round trip, the model's row and remembered sort, and both stores.

Headless on the web at 1422: every key and the direction reorder the list as expected, and a reload keeps the choice. Shot: `target/398-shots/` on the laptop.

## Verification
-
