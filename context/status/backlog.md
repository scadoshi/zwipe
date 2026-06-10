# Backlog - Future Development

Planned features and improvements for after App Store launch.

---

## AI Card Categorization — Layer 2 & 3

Layer 1 (oracle text heuristics, ~70-80%) ships with the mechanical category feature. Layers 2 and 3 are post-launch improvements.

**Layer 2: AI Classification Client** (post-launch, high priority)
- Standalone Rust binary (`zort`) in its own workspace crate, connecting directly to Postgres
- Reads cards in batches, sends (name, type_line, oracle_text) to LLM API (Claude Haiku)
- Writes category tags back via UPDATE on card_profiles.mechanical_categories
- Subcommands: `zort classify` (untagged), `zort reclassify` (all), `zort delta` (changed), `zort audit` (compare vs heuristics)
- Cost: ~$5-15 for full 35k card run
- Target accuracy: 90-95%

**Layer 3: Fine-Tuned Lightweight Model** (future, when Layer 2 data is mature)
- Train a small model on Layer 2's corrected tags as training data
- Input: oracle_text + type_line → Output: category tags
- Runs locally, no API costs, embeddable in zervice sync pipeline
- Target accuracy: 95-99%
- Build when: Layer 2 has run multiple cycles and tags have been spot-checked

**Why three layers:** Rule-based heuristics get you launched. LLM classification corrects the 20-30% that heuristics miss. A fine-tuned model makes it self-sustaining without ongoing API costs. Each layer builds on the last.

See `context/plans/mechanical-category.md` for full implementation plan including taxonomy and schema.

---

## Production Hardening
- **Caching Layer**: Redis for card data and query results
- **Monitoring**: Structured logging (done), health monitoring dashboard
- **Database Optimization**: Query performance, indexing strategy
- **Credential-stuffing defense**: Layer a second governor on `/login`, `/forgot-password`, `/verify-email`, `/reset-password` keyed by the submitted email/username (normalized lowercase) in addition to the existing IP-keyed governor. IP alone doesn't catch a distributed botnet hitting one email across many IPs; account lockout is the strict per-account version of this but kicks in late. Requires a small `KeyExtractor` that peeks at the JSON body (or runs as middleware before governor and stuffs the key into request extensions). See `inbound/http/routes.rs:71-114` for the existing IP-keyed configs to stack against. (Per-user-id keying on authenticated routes is **already done** via `UserIdKeyExtractor` in `middleware.rs`.)

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
