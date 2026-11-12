# 364. A melded card's list row thumbnail and hover preview show the half

State: fixing | Owner: scotland | PR: #563 | Build seen: live, web, after #559

## Report
surfer via composer, 2026-11-12, web, deck page List: with Bruna, the Fading Light melded (its Meld on), the row's thumbnail and its hover preview still show Bruna. Shot `target/scotty-shots/559-list-row-art-bruna-while-melded.jpg`.

## Cause
`CardRow` drew its thumbnail (the art crop) and its hover preview from the half's own data. Since #559 every other place a half shows reads its Meld from the shared `MeldSource`; the row's head did not.

## Fix
scotland, #563 (components/src/card_row.rs): the row reads the half's Meld (`use_meld`). While it is on, the thumbnail and the hover preview show the melded card, on its one face. A preview held under the cursor swaps when Meld turns on or off, as it does for a flip. The row's name, price and stats stay the half's.

Failing-first test `a_melded_half_s_list_row_shows_the_melded_card_s_art` (`components/tests/meld_row.rs`). Headless on scotland: with Art on, hovering Bruna's row and pressing `v` turns the thumbnail and the held preview to Brisela, and `v` again turns them back. Shot `target/shots-364/364-row.png`.
