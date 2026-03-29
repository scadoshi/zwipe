# Backlog - Future Development

Planned features and improvements for after App Store launch.

---

## AI Card Categorization (Post-Launch, High Priority)

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
- **Caching Layer**: Redis for card data and query results
- **Monitoring**: Structured logging (done), health monitoring dashboard
- **Database Optimization**: Query performance, indexing strategy
- **Per-user rate limiting**: Key by authenticated user ID instead of IP for per-user fairness

## Mobile & Deployment
- **iOS Keychain Entitlements**: Configure for persistent session storage
- **Android KeyStore**: Verify keyring configuration
- **Android Build**: Test and polish Android target

## Future Features
- **Card Stack Peek Effect**: When swiping a card, the next card in the deck should already be visible underneath it — creating a physical card stack feel. Prior attempt (2026-03-27) caused auto-swiping due to interaction between Dioxus `key`-triggered remounts and `SwipeState`. Refactor plan for `SortCards` trait extraction first (`context/dev/sort_cards_refactor.md`) before retrying this.
- **Multi-Copy Add Flow**: Quantity picker on swipe-right for standard decks
- **EDHREC Integration**: Synergy scores for commander decks (undocumented API, complex)
- **Deck Validation**: Format legality checking beyond copy-max
- **Collection Management**: User card ownership tracking
- **Social Features**: Deck sharing, public deck browser
- **Legality Filter**: Filter by format legality (needs design work)
- **Multi-Language UI**: i18n for application text (card language infra already complete)

## User Metrics
Start simple — don't reach for Mixpanel/Amplitude until you know what questions to ask.

- **Web traffic**: Plausible or Fathom (privacy-friendly, no GDPR/cookie banner headache)
- **API activity**: structured logs already exist — add a `user_events` table for key
  actions (registration, deck created, card added) that can be queried directly
- **Dashboard**: query the DB directly to start; build reporting later if needed

## Patch Discipline
The App Store review cycle is 1–3 days per iOS submission. Backend patches ship in
minutes via CI/CD. That asymmetry shapes everything:

- Keep the iOS client **defensive** — handle unexpected server responses gracefully so
  the server can be patched without forcing an app update
- **Never edit existing migration files** — always add a new migration forward
- **Semantic versioning**: `MAJOR.MINOR.PATCH` — bump PATCH for bug fixes, MINOR for
  new features, MAJOR for breaking changes
- **Deprecate before removing**: leave old endpoints alive for at least one app version
  cycle before pulling them
- **API versioning**: don't add `/v2/` preemptively — only version when you have an
  actual breaking change and need both versions live simultaneously
- **Breaking change checklist**: before removing or changing an endpoint signature,
  check what version of zwiper is in the wild and whether old clients will break
