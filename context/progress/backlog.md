# Backlog - Future Development

Planned features and improvements for after App Store launch.

---

## High Priority

- **Deck Migration — Archidekt SHIPPED (2026-06-10), Moxfield DENIED**: Archidekt URL import landed in 1.0.5 (see `context/plans/deck_import.md`). Moxfield support denied API access (2026-06-10) — policy excludes deckbuilding apps. They plan a scoped deck-export endpoint for such services (no ETA, announced via their help pages when live); periodically check their help pages and re-request access then. The text-paste importer covers Moxfield users meanwhile.
- **recommander.cards integration — gated on a dedicated API key.** https://recommander.cards/ is a third-party card-suggestion engine we'd like Zwipe to consume for recommendation data. **Finding (2026-06-23): the public endpoint's rate limit is far too low to be viable — on the order of ~10 requests/hour.** Two ways it breaks: (1) all Zwipe traffic would funnel through our single backend, exhausting that hourly cap in seconds; (2) if instead clients called it directly, an IP-keyed limit collides for mobile users sharing a Wi-Fi network (same public IP), throttling each other. So the integration is **only viable with a dedicated API key carrying production-grade limits.** Until then, don't build against it (we already have our own recommendation data to fall back on). Outreach is in progress; specifics are kept out of this public repo (local notes only). (noted 2026-06-09; rate-limit constraint added 2026-06-23)
- **Deck import atomicity (#7) — SHIPPED 2026-07-27** (`b4cc65bb`): `apply_import_batch` runs lock + limit-check + upsert + replace-reconcile in one tx (`FOR UPDATE` closes the concurrent-import TOCTOU; `create_deck_card` got the same fix). 5 new `#[sqlx::test]` cases incl. the race, plus a live E2E pass (real text imports + the real Archidekt Satya deck, set-equality-verified). Plan archived at `context/plans/archive/import_atomicity.md`.
**Done & removed:** Split `CardFilter` into `CardCriteria` + `CardQuery` + `Cards` — executed 2026-07-02 (`e681e58f`), wire unchanged, on main awaiting the next release. Outcome in `overview.md`; plan doc deleted.

---

## Weekly Badges + Stats / Share Cards (gamification; pairs with future social)

**Promoted to a full plan 2026-07-06: [`../plans/social-features/`](../plans/social-features/overview.md)
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

## AI Card Categorization — Layer 2 & 3 (CLOSED 2026-07-27: superseded by oracle tags)

Layers 2 (LLM classification client) and 3 (fine-tuned model) were the
improvement path for the Layer-1 oracle-text heuristic. The whole ladder is
obsolete: Scryfall's community-maintained oracle tags now provide the
human-accurate tagging these layers were meant to approximate, card roles
derive from otag subtrees (`classify.rs` deleted 2026-07-13), and the only
surviving text heuristic is the 4-category `oracle_tag_gaps` fallback. No AI
categorization is planned; owner call 2026-07-27.
- Target accuracy: 95-99%
- Build when: Layer 2 has run multiple cycles and tags have been spot-checked

**Why three layers:** Rule-based heuristics get you launched. LLM classification corrects the 20-30% that heuristics miss. A fine-tuned model makes it self-sustaining without ongoing API costs. Each layer builds on the last.

See `context/plans/mechanical-category.md` for full implementation plan including taxonomy and schema.

---

## Production Hardening
- **Zerver app-role split ("Phase 3" of zervice least privilege, idea 2026-07-29)**: give zerver its own scoped Postgres role — write on user/deck/auth/signal tables, read-only on the card catalog (which zervice + owner alone write after `zcripts/server/sql/zervice_role.sql`). Sound hardening, deliberately deferred: all tables are owned by `zwipe` (an owner can't be restricted by grants), CI sources the same `.env` `DATABASE_URL` for migrations so the split forces two URLs + deploy-pipeline changes, and every future migration needs grant discipline (`ALTER DEFAULT PRIVILEGES` automates most of it) or the serve path 500s — a worse failure mode than a failed nightly sync. Take up deliberately, not as a drive-by.
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
