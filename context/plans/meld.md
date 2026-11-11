# Meld support

Decided with Scotty on 2026-11-11.

## What we have

A meld half (Gisela, the Broken Blade) has `layout = "meld"`, and its `all_parts` names the result (`component: "meld_result"`, Brisela, Voice of Nightmares, by Scryfall id) and both halves (`meld_part`). The result is its own card in our database, with full oracle text, type, power/toughness and its own full card image from Scryfall, so nothing needs half an image.

## Behavior

1. A meld icon in the top-right of a meld half's card, the same place and style as the flip icon. Tapping it shows the melded card's full image; tapping again shows the half.
2. Card details get the same toggle: it switches the details to the melded card (name, type, full text, P/T), with one line, "Melds with <other half>", naming the other half.
3. Both halves show the icon and the same result.
4. App and web alike.

## Data

The client needs the result card for a meld half: either sent with the card, or fetched by its id. Pick the lighter way. Any change to what the server sends or accepts is a wire change: additive only, checked against shipped clients per `context/development/api_evolution.md`, server first.

## Not in scope

Meld results are not swipeable or addable on their own (they only exist by melding).
