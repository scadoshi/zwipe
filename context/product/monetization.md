# Monetization Decision

**Decided: Freemium with subscription for AI/intelligence features (2026-03-26)**

---

## Model

**Free tier** — full core functionality, no paywall:
- Account + persistent sessions
- Unlimited decks
- Swipe-to-build interface across the full Scryfall card database
- Deck metrics (mana curve, color breakdown, pip balance)
- Import/export (Moxfield, Archidekt formats)
- Advanced card filtering

**Paid tier** — intelligence layer, subscription only:
- AI card suggestions: "given your commander and strategy, here are cards you're missing"
- EDHREC synergy integration — synergy scores, theme/archetype suggestions
- Possibly: price awareness, collection tracking

**No ads** — ever. Ads look terrible, tank reviews, and CPMs for a niche hobby audience are lousy.

## Pricing Target

- $3–5/month or $20–25/year
- One-time purchase is off the table — a dedicated Commander player using this weekly will pay $3/month, and 6 months in you've earned twice what a $5 purchase would have returned

## Rationale

- MTG players are willing to pay for good tools (they buy $50 booster boxes)
- Free tier is the acquisition funnel — App Store discoverability for MTG tools is real, Commander players are always looking for something better than the clunky mobile experience of Moxfield/Archidekt
- AI card suggestions are defensible — requires our backend, our card data, and Claude API calls. Not trivially copyable
- Subscription > one-time on App Store: recurring revenue, better LTV, aligns incentives (we keep improving it)

## Technical Path

1. Ship free tier publicly on App Store first — get real users and reviews
2. Layer in subscription via Apple's `StoreKit` + a server-side entitlement flag on the JWT
3. Claude API calls for suggestions live in `zervice`/`zerver` — paid users get a route, free users get a 402

## Status

Not yet implemented. Subscription tier is post-App Store launch.
