# Feature ideas

Scotty's backlog of larger features, brainstormed 2026-11-08. Not scheduled unless noted. Each gets its own plan before any building.

## Queued

- **ManaBox / collection tracking.** This week. Plan: [collection.md](collection.md).
- **Meld cards.** Show a meld card's melded result the way a double-sided card flips. After 2026-11-11.

## Social, light first

Where Scotty wants to start:
- Like decks, follow decks, follow people.
- Small recognition like a badge of the week (most-liked deck, most swipes).
- Public profiles built from what Zwipe already knows: shared decks, Playstyle, favorite formats.

Later, far out: playgroups and local game stores, then "looking for game", meaning swiping through nearby players matched by format, bracket and playstyle (the literal Tinder for Magic). Needs report, block and filtering (Apple's user-generated-content rules), an approximate-area-only location, opt-in, and a stance on minors. Keep interaction limited.

## Prices and market

- Price history and trends per card and deck (premium, per collection.md).
- Budget swaps: cheaper cards sharing the same Oracle tags.
- Price alerts on maybeboarded cards.
- Buy links (TCGplayer, Cardmarket); affiliate programs could fund hosting.

## AI deck review (premium)

A few dollars a month, rate-limited. Buttons only, no free text, to avoid prompt injection: mana base check, strategy summary, suggest 5 cards, what to cut, bracket estimate. The server builds the prompt from deck data; suggestions come back as card ids checked against Zwipe's database (no invented cards), optionally limited to the commander's candidate pool. Cheap model for quick checks, a stronger one for the full review. Built on the Claude API under Anthropic's commercial terms, which allow paid products on top of it.

## Others

- Combo detection (Commander Spellbook API).
- Deck version history and snapshots.
- Compare two decks.
- Opening-hand and goldfish sample draws.
