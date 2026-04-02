# Plan: Buy Deck Links (TCGplayer + CardKingdom)

## Goal

Allow users to tap a "buy" button on the deck view screen and open their full decklist in TCGplayer or CardKingdom's bulk purchase tools, pre-populated with all cards and quantities.

## Data Available

Each `DeckEntry` has:
- `card.scryfall_data.name` — card name (e.g., "Lightning Bolt", "Urza, Lord High Artificer")
- `deck_card.quantity` — number of copies (1–99)

The deck entries are already loaded in `view.rs` via `deck_resource`.

## TCGplayer Mass Entry

**URL format (confirmed working):**
```
https://www.tcgplayer.com/massentry?c=QUANTITY+CARD+NAME||QUANTITY+CARD+NAME||...
```

Example:
```
https://www.tcgplayer.com/massentry?c=1+Sol+Ring||4+Lightning+Bolt||1+Urza%2C+Lord+High+Artificer
```

- Entries separated by `||`
- Each entry: `QUANTITY+CARD+NAME` (spaces become `+`)
- Card names must be URL-encoded (commas → `%2C`, apostrophes → `%27`, etc.)
- This is a GET request — all data in the query string

**Research needed:**
- Maximum URL length (browsers cap around 2000–8000 chars; 100-card Commander decks with long names could exceed this)
- Whether TCGplayer supports specifying set/edition (useful for getting the right printing/price)
- Affiliate link parameters (the Archidekt example URL includes `irclickid`, `utm_source=impact`, `utm_medium=affiliate` params)

## CardKingdom Builder

**URL:** `https://www.cardkingdom.com/builder`

The Archidekt example uses short query params: `?partner=archidekt&utm_source=archidekt&utm_medium=affiliate&utm_campaign=archidekt&partner_args=bulkBuy`

This is NOT a mass entry URL like TCGplayer — the card data is NOT in the URL. CardKingdom's builder likely works via:
1. A POST form submission (hidden form with card data, submitted to their endpoint)
2. A JavaScript-based paste/import in their builder UI
3. A session/cookie-based mechanism where card data is sent via API first

**Research needed:**
- Reverse-engineer how Archidekt (or Moxfield) sends card data to CardKingdom builder
- Check if CardKingdom has a documented API or import format
- Test whether a form POST redirect would work from a mobile app (may need to open browser with pre-filled clipboard instead)
- Check their affiliate/partner program for Zwipe

## Implementation Approach

### UI
- Add a "buy" button to the deck view screen (currently in the util bar, but may move to a more natural location per the util bar redesign roadmap item)
- On tap, show a bottom sheet or simple dialog with two options: "TCGplayer" and "CardKingdom"
- Each option constructs the appropriate link and opens it in the system browser

### Code
- Create a utility module (e.g., `zwiper/src/lib/outbound/buy_links.rs` or similar) with functions:
  - `fn tcgplayer_url(entries: &[DeckEntry]) -> String` — builds the mass entry URL
  - CardKingdom approach TBD based on research
- Use Dioxus's `web_sys` or platform-specific API to open URLs in external browser

### Edge Cases to Handle
- Cards with special characters: commas (`Urza, Lord High Artificer`), apostrophes (`Akroma's Will`), slashes (`Wear // Tear`)
- Double-faced cards: use front face name only
- Very large decks: if URL exceeds browser limits, consider chunking or showing a copyable text list as fallback
- Cards not available on the platform: TCGplayer/CK will handle missing cards gracefully (they show "not found" for individual entries)

## Key Files
- `zwiper/src/lib/inbound/screens/deck/view.rs` — add buy button
- New utility module for URL construction
- `zwiper/src/lib/domain/deck/models/deck.rs` — `DeckEntry` struct (read-only reference)

## Dependencies
- Should be done AFTER the component extraction (deck-view-component-extraction plan), since view.rs will be restructured
- No backend changes needed — this is purely frontend
