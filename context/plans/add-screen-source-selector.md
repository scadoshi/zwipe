# Add Screen Source Selector: Search vs Maybeboard

## Context

The add screen currently only supports searching for new cards via the API. Users who have built up a maybeboard want to swipe through those cards to promote them to the deck without navigating to the deck view. Add a source selector (chip row) at the top of the add screen — identical in style to the remove screen's board filter — that toggles between "search" (default, current behavior) and "maybeboard" (swipe through the deck's maybeboard entries).

## Behavior

### Source modes

| Mode | Card source | Right swipe | Left swipe | Up swipe | Down swipe |
|------|------------|-------------|------------|----------|------------|
| **search** (default) | API search, paginated | Add to deck | Skip | Add to maybeboard | Undo |
| **maybeboard** | Deck's maybeboard entries, local | Move to deck (update board) | Skip (circular) | Add to maybeboard (no-op, already there) | Undo |

- Up-swipe in maybeboard mode: no action (card is already on the maybeboard). Could show a toast "already on maybeboard" or just skip.
- Undo in maybeboard mode: reverse the last move-to-deck (update board back to maybeboard).

### Filter sheet

- Works in both modes. In maybeboard mode, filters apply client-side to the maybeboard cards (same pattern as the remove screen's Effect 2).
- The filter dot indicator and clear button work identically in both modes.

## Implementation

### File: `zwiper/src/lib/inbound/screens/deck/card/add.rs`

#### 1. Add source enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum AddSource {
    #[default]
    Search,
    Maybeboard,
}
```

#### 2. Add signals

```rust
let mut add_source: Signal<AddSource> = use_signal(AddSource::default);
let mut mb_entries: Signal<Vec<DeckEntry>> = use_signal(Vec::new);
let mut mb_displayed_cards: Signal<Vec<Card>> = use_signal(Vec::new);
let mut mb_current_index: Signal<usize> = use_signal(|| 0);
let mut mb_action_history: Signal<Vec<SwipeAction>> = use_signal(Vec::new);
```

#### 3. Load maybeboard entries on mount

In the existing `use_effect` that fetches the deck (line ~362), also collect maybeboard entries:

```rust
// After deck_cards_ids.set(ids):
let mb: Vec<DeckEntry> = deck.entries
    .iter()
    .filter(|e| e.deck_card.board.is_maybeboard())
    .cloned()
    .collect();
mb_entries.set(mb);
```

#### 4. Add filter effect for maybeboard mode

New `use_effect` that reads `add_source`, `filter_builder`, and a reset counter. When in maybeboard mode, applies client-side filtering + sorting (same pattern as remove screen Effect 2):

```rust
use_effect(move || {
    if add_source() != AddSource::Maybeboard { return; }
    let entries = mb_entries.peek().clone();
    let builder = filter_builder.peek().clone();

    let cards: Vec<Card> = entries.iter().map(|e| e.card.clone()).collect();
    let mut filtered = if builder.is_empty() {
        cards
    } else {
        // build filter, apply filter_by
    };
    filtered.sort_by_filter(&builder);
    mb_displayed_cards.set(filtered);
    mb_current_index.set(0);
});
```

#### 5. Add chip row to RSX

Insert above the `form-container` div, same style as remove screen:

```rust
div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
    div { class: "chip-row",
        span { class: "chip-row-label", "from:" }
        for (label, variant) in [("search", AddSource::Search), ("maybeboard", AddSource::Maybeboard)] {
            button {
                class: if add_source() == variant { "chip selected" } else { "chip" },
                onclick: move |_| {
                    add_source.set(variant);
                    // reset indices, clear action history for the new mode
                },
                "{label}"
            }
        }
    }
}
```

#### 6. Conditional rendering

The `form-container` content switches based on `add_source()`:
- **Search**: existing SwipeStack with `cards`, `current_index`, search-mode callbacks (unchanged)
- **Maybeboard**: SwipeStack with `mb_displayed_cards`, `mb_current_index`, maybeboard-mode callbacks

#### 7. Maybeboard swipe callbacks

- **Right (promote to deck)**: call `update_deck_card` with board="deck", remove from `mb_entries` and `mb_displayed_cards`, push `SwipeAction::Do` to `mb_action_history`
- **Left (skip)**: circular advance like remove screen, push `SwipeAction::Skip`
- **Up**: toast "already on maybeboard" (no-op)
- **Down (undo)**: pop `mb_action_history`, reverse the last action (re-insert card, update board back to maybeboard on backend)

#### 8. Util bar adjustments

- **Refresh button**: in search mode, works as before (re-fetch from API). In maybeboard mode, re-fetches the deck and reloads maybeboard entries.
- **Filter button**: works in both modes (filter sheet is shared).
- **Clear filter**: works in both modes.

### New imports needed

```rust
use zwipe_core::domain::deck::{Board, DeckEntry};
use zwipe_core::http::contracts::deck_card::HttpUpdateDeckCard;
use zwipe_core::domain::card::search_card::filter_cards::SortCards;
```

(`HttpUpdateDeckCard` may already be imported depending on current state; `DeckEntry` and `Board` are new to this file.)

### No changes needed

- **zwipe-core**: no domain changes. `HttpUpdateDeckCard` with board="deck" already exists.
- **Filter sheet**: already works with local card lists via the `DeckCards` context provider.
- **Undo**: reuses existing `SwipeAction` enum (Do/Skip/Maybeboard variants).

## Edge cases

- **Empty maybeboard**: show the existing `CardSkeleton` placeholder (no cards state).
- **Switching modes**: reset index and action history for the new mode. Search mode cards persist across switches (existing behavior via `last_search_filter` check).
- **Promoting last maybeboard card**: after the last card is moved to deck, show empty state.

## Text Search: Trim + Punctuation Stripping (2026-04-11)

All text "contains" filter fields now strip punctuation and trim whitespace at input. Server SQL uses `regexp_replace` on DB columns. Client-side `filter_by` uses `strip_punctuation()` on card data. Exact-match fields (artist, set, keywords) only trim.

- `strip_punctuation()` utility in `zwipe-core/.../card_filter/mod.rs` with 6 unit tests
- Punctuation-insensitive filter tests: name (apostrophe, comma), oracle text, whitespace trimming
- Server SQL: `regexp_replace(col, '[^a-zA-Z0-9 ]', '', 'g') ILIKE` on name, type_line, oracle_text, flavor_text

## Verification

1. Open a deck with maybeboard cards → add screen → switch to "maybeboard"
2. Confirm maybeboard cards appear in the swipe stack
3. Right-swipe → card moves to deck, toast "moved to deck"
4. Down-swipe → undo restores card to maybeboard
5. Left-swipe → skip, circular navigation
6. Apply a filter → maybeboard cards filter correctly
7. Switch back to "search" → existing search behavior unchanged
8. Switch to maybeboard on a deck with no maybeboard cards → empty state
