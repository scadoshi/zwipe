# Enhancements
- Remove card screen: build UI for removing cards from deck with filtering and swipe gestures
- Commander search: filter to only return valid commanders
    - Valid commanders:
        - Legendary creatures
        - Legendary vehicles/spacecraft with power/toughness (e.g., flagship, starship subtypes)
        - Cards with "can be your commander" text (some planeswalkers, special cards)
- Deck card search: exclude token cards from results
- Order by filter: add sorting option to filters pane (Name, Cmc, Power, Toughness, Rarity, ReleasedAt, Price, Random)

# Bugs
- Deck list nav bug: first deck creation navigates back but deck doesn't appear until navigating back again
- Set filter broken: not returning any results
- Create deck layout: commander image pushes "deck name" label into header area
