# Backlog - Future Development

Planned features and improvements for after hosting is live.

---

## AI Card Categorization (Post-Hosting, High Priority)

Batch-classify all 35k cards with Claude API to tag strategic categories:
burn, recursion, ramp, removal, counterspells, draw, tutors, board wipes, lifegain, tokens, flyers

**Why AI:** Rule-based pattern matching breaks down fast — "destroy target creature", "exile target nonland permanent", "deals 3 damage to any target", and "target creature gets -3/-3" are all removal but share no keywords. Oracle text variance across thousands of cards is fundamentally non-deterministic.

**Approach:**
- Run as batch job in `zervice` sync pipeline (after Scryfall data upsert)
- Send oracle text in batches to Claude API, receive category tags
- Store as jsonb column on `card_profiles` (e.g. `categories: ["removal", "burn"]`)
- Re-run only on new/changed cards (delta sync)
- Expose as `CardFilter` option + `GroupByOption` variant on frontend

**Cost:** Low — oracle texts are short strings, 35k cards in batches of ~100 = ~350 API calls.

---

## Production Hardening
- **Rate Limiting**: Request throttling, abuse prevention
- **Caching Layer**: Redis for card data and query results
- **Monitoring**: Structured logging, health monitoring
- **Database Optimization**: Query performance, indexing strategy

## Mobile & Deployment
- **iOS Keychain Entitlements**: Configure for persistent session storage
- **Android KeyStore**: Verify keyring configuration
- **App Signing**: iOS/Android code signing
- **Store Submission**: App Store / Play Store

## Future Features
- **Multi-Copy Add Flow**: Quantity picker on swipe-right for standard decks
- **EDHREC Integration**: Synergy scores for commander decks (undocumented API, complex)
- **Deck Validation**: Format legality checking beyond copy-max
- **Collection Management**: User card ownership tracking
- **Social Features**: Deck sharing, public deck browser
- **Legality Filter**: Filter by format legality (needs design work)
- **Multi-Language UI**: i18n for application text (card language infra already complete)
