# 313. Decks list shows no price per deck

State: fixing | Owner: scotland | PR: #465 | Build seen: 00ed27f59

## Report
Scotty, 2026-11-08 (screenshot of web Decks sorted by Price, descending): each deck entry shows colors, card count, format, power and commander, but not its total price, so a price sort has nothing to read. Add the deck's total price as a chip after the card count, styled like the deck page's price chip ($27.80), app and web. A deck with no priced cards shows no price chip.
