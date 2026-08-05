# Clear (×) button on every floating-results search bar

**Status: DONE (2026-08-05).** All 14 inputs across the 7 host files wired
(each file's includes AND excludes bars, plus all four command-zone pickers):
input wrapped in a flex row, `clear-btn` shown only while the query is
non-empty, clearing query + typing/dropdown state (pickers also clear their
display text). Rides 1.7.6. Note kept from the design discussion: the
pickers' pre-existing label-row × clears the *selection*; the new inline ×
clears the *search text* — deliberately separate controls.

**One sentence:** every search bar that floats selectable results above/over the
screen gets the same × clear button quick add already has (shown only when the
bar has content; clears the text and closes the results).

## The pattern to copy

`quick_add.rs` (~line 279): a `button.clear-btn` rendered only when the query is
non-empty; onclick sets the query to `""`, empties results, hides the dropdown.
The `.clear-btn` CSS already exists.

## Where it goes (the search-float hosts, from `grep -rl search-float`)

- [ ] `deck/card/filter/oracle_tags.rs`
- [ ] `deck/card/filter/set.rs`
- [ ] `deck/card/filter/types/other_types.rs`
- [ ] `deck/card/filter/artist.rs`
- [ ] `deck/card/filter/oracle_text/keywords.rs`
- [ ] `deck/card/filter/oracle_text/oracle_words.rs`
- [ ] `deck/components/deck_fields.rs` (commander/partner pickers)
- [x] `deck/card/components/quick_add.rs` — already has it (the reference)

Check each host's local state names (query signal, results, dropdown visibility)
and wire the same three resets. If the bars share enough shape, consider
extracting a small shared input+clear component — but don't force it; seven
hand-wired copies of a 10-line button is fine for a fast pass.
