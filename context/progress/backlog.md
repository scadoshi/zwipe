# Backlog - Future Development

Planned features and improvements for after App Store launch.

---

## High Priority

- **Deck Migration — Archidekt SHIPPED (2026-06-10), Moxfield DENIED**: Archidekt URL import landed in 1.0.5 (see `context/plans/deck_import.md`). Moxfield support denied API access (2026-06-10) — policy excludes deckbuilding apps. They plan a scoped deck-export endpoint for such services (no ETA, announced via their help pages when live); periodically check their help pages and re-request access then. The text-paste importer covers Moxfield users meanwhile.
- **recommander.cards integration — gated on a dedicated API key.** https://recommander.cards/ is a third-party card-suggestion engine we'd like Zwipe to consume for recommendation data. **Finding (2026-06-23): the public endpoint's rate limit is far too low to be viable — on the order of ~10 requests/hour.** Two ways it breaks: (1) all Zwipe traffic would funnel through our single backend, exhausting that hourly cap in seconds; (2) if instead clients called it directly, an IP-keyed limit collides for mobile users sharing a Wi-Fi network (same public IP), throttling each other. So the integration is **only viable with a dedicated API key carrying production-grade limits.** Until then, don't build against it (we already have our own recommendation data to fall back on). Outreach is in progress; specifics are kept out of this public repo (local notes only). (noted 2026-06-09; rate-limit constraint added 2026-06-23)
- **Deck import atomicity (#7)** (deferred, low priority, needs test DB / testable frontend): replace-mode insert commits before the delete-not-in runs → a crash leaves a hybrid board; the limit check is also a separate read (concurrent-import TOCTOU). Chosen fix Option A: lock + limit-check + insert + reconcile in one tx (`apply_import_batch`). Full design in `context/plans/import_atomicity.md`. (noted 2026-06-19)
**Done & removed:** Split `CardFilter` into `CardCriteria` + `CardQuery` + `Cards` — executed 2026-07-02 (`09d39a20`), wire unchanged, on main awaiting the next release. Outcome in `overview.md`; plan doc deleted.

---

## Weekly Badges + Stats / Share Cards (gamification; pairs with future social)

**Promoted to a full plan 2026-07-06: [`../plans/social_features/`](../plans/social_features/overview.md)
(weekly badges + owner-curated featured decks with MVPs). The plan carries
these decisions forward; this section stays as the original rationale.**

**Backlogged 2026-07-02.** A weekly retention loop: at week close, categorize each
active user's week into **1–3 badges** ("Swipe King" volume, "The Controller"
taste, "Ultimate Indecision" quirk), surfaced as a "Your week" recap on next open
plus a badge-history/stats page. The recap doubles as a **shareable card**
(Wrapped-style, terminal aesthetic) — viral value without social infrastructure.

- **Derive, don't collect.** Almost every badge/stat is a *join*, not new
  collection: per-user card signal × `mechanical_categories` (archetypes), ×
  `color_identity`, × `cmc` (curve taste), × `prices` (budget), × `edhrec_rank`
  (hipster/meta). Lifetime volume badges are computable **today** from
  `user_lifetime_counters` / `user_daily_activity` / `user_events`. Rule: only
  add a counter when a named consumer exists.
- **Data prerequisite: weekly windowing — ✅ BUILT (2026-07-02, on main).**
  Ingest now bumps `user_week_signal` (directional swipes, searches,
  added/skipped/maybed/removed per ISO week) and `user_week_facet_signal`
  (accepts by mechanical category and color identity). One row per active user
  per week; history accrues from the moment the server deploys.
- **Badge job**: week-close cron (zervice pattern) computes 1–3 badges per
  active user (v1: threshold rules + priority order, cap 3, ≥1 for any
  activity) into `user_week_badges (user_id, week, badges)`.
- **Social pairing (later)**: public profiles / leaderboards / seeing others'
  badges is the natural extension, but it's a real subsystem (opt-in
  visibility, moderation, blocking) and another privacy-posture change — the
  private recap + share card ships first and stands alone.

Related: `archive/swipe_memory.md` (the flush-ingest surface all of this rides
on, executed 2026-07-02) and the now-live per-user `user_card_signal`
collection.

---

## Security — Account Enumeration Hardening (deferred, matters at larger scale)

Both are low-risk now, fine to leave; revisit with a bigger user base. Context: login timing was equalized via a dummy-hash verify (commit pending 2026-06-19), so these are the *remaining* enumeration surfaces.

- **`AccountLocked` returns 429 while bad-password returns 401** (`zerver/.../handlers/auth/authenticate_user.rs`): distinguishable status lets an attacker learn an account exists *and* is locked. Kept as-is deliberately — the 429 gives locked-out real users useful "wait and retry" UX. Option if it ever matters: fold `AccountLocked` into the generic 401. (noted 2026-06-19)
- **Registration enumerates existing accounts**: `register` returns 422 "user with that username or email already exists." Genuinely hard to fully close (can't silently allow a duplicate), and many large apps surface "username taken" too, so likely won't change — logged for completeness. (noted 2026-06-19)

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
- **Synergy scores**: per-commander synergy data for commander decks (prioritized 2026-06-10 — see `todo.md` Next Up)
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
