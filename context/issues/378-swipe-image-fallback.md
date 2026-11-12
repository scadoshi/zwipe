# 378. A failed swipe card image shows the card underneath

State: fixing | Owner: laptop | PR: - | Build seen: production, web

## Report
Surfer, via the composer, 2026-11-12: when a swipe card's image fails to load (Scryfall down), the top swipe card goes transparent and the next card shows through, while the details still name the top card. Swipe cards (Add, Remove, Commander swipe, Suggested commanders, Examples) should get a fallback, so a failed image never reveals the card underneath. Web and app.

## Cause
components' `FlippableCardImage`, which every swipe stack on the web and in the app draws its cards with, shows an image only once it has decoded. A failed load only announced itself and left the hidden image in place: an empty, see-through card.

## Fix
laptop: `FlippableCardImage` remembers an image URL that failed to load and then draws as a card with no art: its card-shaped text proxy (`no-image-card`: name, mana cost, type line, rarity and set, rules and flavor) in the image's footprint, opaque. It is the same proxy a card without art already gets, and it applies everywhere the component draws: every swipe stack, the image preview, the featured cards and the app's printing sheet, web and app alike.

## Verification
-
