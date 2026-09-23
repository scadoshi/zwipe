# The add screen can't clear a filter back to default

**Status: FIXED 2026-09-22 (`8ff6e790`), owner-tested on device. Ships with 1.10.2. Pre-existing, not from the 2026-09-21 refactors.**

What landed, beyond the diagnosis below:

- Apply reads the **transition**, not the draft alone: clearing a filter that is on commits, while an empty apply from an already-empty state is still refused. The decision is a pure `apply_action` fn with tests covering all seven states, because the first attempt got it wrong twice (it refused a deliberate clear, then labeled every apply over an existing filter a clear).
- The refusal returns **before** closing the sheet. Closing with the snapshot still armed was what made a refusal also revert the draft and raise the second toast.
- An **unedited apply no longer refetches**: opening the sheet, changing nothing and applying leaves the card stack where the user had it (owner request, 2026-09-22).
- Reset keeps its toast. It was briefly removed as redundant, but the sheet stays open with sections collapsed, so a silent Reset reads as a dead button.

**One sentence:** on the add screen, applying a cleared filter is refused as "empty" and then silently reverted, so there is no way back out of a filter once one is on.

## What the user sees

Apply a filter (say a mana pip), then try to clear it and apply. Two toasts land on top of each other: "Filter is empty, nothing applied" and "Filter changes discarded". The filter comes back. Reset does the same thing.

## Mechanism

In `card_filter_sheet.rs`, the Apply handler:

```rust
if validate_before_apply && !filter_builder.read().has_search_intent() {
    toast.warning("Filter is empty, nothing applied", ...);
} else {
    bump_filter();
    applied_snapshot.set(None);   // only the commit path drops the snapshot
    toast.success("Filter applied", ...);
}
open.set(false);                  // runs either way, outside the if/else
```

The refusal path skips `bump_filter()` **and** leaves `applied_snapshot` intact, then closes the sheet regardless. The close effect (same file, the `use_effect` on `open()`) still holds a snapshot, sees the draft differs from it, restores the old filter and fires the second toast. So the refusal and the revert are two separate mechanisms landing together.

Reset is caught by the same trap: its handler only *stages* the default ("Stages the default; Apply commits it"), and Apply then refuses to commit it.

`validate_before_apply` is only true on the add screen with `add_source() == AddSource::Search` (`add.rs:1589`), which is why the remove screen behaves differently.

Synergy counts as intent (`has_search_intent` is `!is_empty_ignoring_deck_context()
|| sort.is_some() || synergy`), so this only bites with synergy off. On a
commander deck with synergy on, a cleared filter still reads as intent and applies fine. That's why it hid for so long.

## The conflict to resolve

The guard exists to stop an accidental firehose: an empty filter on the add screen means "search the whole catalog". That's worth preventing. But the same condition describes a deliberate "clear my filter", which is the only route back to an unfiltered pool. One predicate is being asked to tell two opposite intents apart.

## Suggested fix

Judge the transition, not the destination. An empty apply is meaningless only when nothing was applied before; clearing a filter that *is* currently applied is deliberate. `applied_snapshot` already holds the pre-edit state, so the Apply handler can compare: if the snapshot had intent and the draft doesn't, commit it.

Whatever shape it takes, the refusal path must stop leaving the snapshot armed while closing the sheet. Either keep the sheet open on refusal (the contradiction branch above already does exactly that, and returns early), or drop the snapshot before closing. Matching the contradiction branch is the smaller change and gives the user a chance to fix the filter in place.

## Also seen in the same screenshot

- Two toasts stack and overlap, leaving neither readable. Worth checking whether the toast host should queue rather than stack, since any double-toast path hits this.
- "Filter is empty, nothing applied" truncates to "Filter is empty," at iPhone 11 width.

## Verification

- Apply a filter with synergy off, clear it, apply: the pool returns to unfiltered and only one toast appears.
- Reset then Apply does the same.
- A fresh empty apply with nothing previously applied is still refused, and the sheet stays open.
- The contradiction guard (include and exclude the same value) still keeps the sheet open and applies nothing.
