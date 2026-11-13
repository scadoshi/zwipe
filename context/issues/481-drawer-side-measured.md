# 481. A card's details drawer opens on the left only when a right drawer would cover the card

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: bf1bba725

## Report
Scotty, through the composer, 2026-11-13, replacing 474's halves rule: the drawer opens on the right by default and on the left only when the card would actually be under a right drawer, measured against where the right drawer sits. Same on the phone, wide web and the app; featured cards follow the same test. With the drawer open, the cursor or a click keeps it where it is unless the new card would be under it. On a phone, where either drawer covers nearly every card (the right one 38 to 380 of 390, the left 10 to 352), the composer approved: left only when the card overlaps the right drawer's place and would overlap the left one less.

## Before
474's halves rule, headless: at 1440 (six columns, the right drawer at 1040 to 1424) columns 3 to 5 opened on the left, column 3 (730 to 945) clear of the right drawer; at 1100 (four columns) columns 2 and 3, both under it. With the drawer open, every swap took the new card's half, wherever the drawer was.

## Cause
474 decided the side by the card's column (`columns::drawer_side(column, columns)`, the right half opening left), not by where the drawer would sit.

## Fix
scotland:
- `zwipe_ui::deck_cards::columns::drawer_side(card, right, width, now)`: from the drawer's side `now` (the right for a closed one), it moves to the other edge only when it would cover the card and the other would cover it less; the left drawer's place mirrors the right one's.
- `StackDetails` measures as a card opens or swaps in: the stack card or featured card's rect and the drawer's place (its offsets, which leave out the slide), then shows the drawer on that side. 474's halves set (`opens_left`) and its column work in `DeckStacks` are gone. A close still shuts the drawer at once.

## Verification
- Headless 1440, closed: columns 0 to 3 open on the right, 4 and 5 (under the right drawer) on the left.
- 1440, open, the cursor stepping h from column 5: the drawer stays on the left through column 2 and goes right at column 1 (257 to 477, under the left drawer at 16 to 400); stepping l back it stays right through column 3 and goes left at column 4.
- 390 with touch: the left column opens right, the right column left, a featured card (108 to 283) right; a left drawer drags left to close.
