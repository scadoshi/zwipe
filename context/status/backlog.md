# Backlog - Future Development

Planned features and improvements for after App Store launch.

---

## High Priority

- **Deck Migration — Archidekt SHIPPED (2026-06-10), Moxfield remains**: Archidekt URL import landed in 1.0.5 (see `context/plans/deck-import.md`). Moxfield URL import is ToS-gated — needs an authorized User-Agent from support@moxfield.com; the text-paste importer covers Moxfield users meanwhile.
- **Reach out to recommander.cards dev**: https://recommander.cards/ — card suggestion engine built by a local dev. Just open the conversation and see what he thinks about integrating. (noted 2026-06-09)

---

## AI Card Categorization — Layer 2 & 3

**Deferred until there's a user base and a premium tier to fund it.**

Layer 1 (oracle text heuristics, ~70-80%) ships with the mechanical category feature. Layers 2 and 3 are post-launch improvements.

**Layer 2: AI Classification Client**
- Standalone Rust binary (`zort`) in its own workspace crate, connecting directly to Postgres
- Reads cards in batches, sends (name, type_line, oracle_text) to LLM API (Claude Haiku)
- Writes category tags back via UPDATE on card_profiles.mechanical_categories
- Subcommands: `zort classify` (untagged), `zort reclassify` (all), `zort delta` (changed), `zort audit` (compare vs heuristics)
- Cost: ~$5-15 for full 35k card run
- Target accuracy: 90-95%
- High-level thought (2026-06-09): https://pioneer.ai/ might be a good AI platform for this — evaluate when Layer 2 work starts

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
- **Android KeyStore**: Verify keyring configuration
- **Android Build**: Test and polish Android target

## Future Features
- **EDHREC Integration**: Synergy scores for commander decks (undocumented API, complex)
- **Collection Management**: User card ownership tracking
- **Social Features**: Deck sharing, public deck browser
- **Multi-Language UI**: i18n for application text (card language infra already complete)

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
