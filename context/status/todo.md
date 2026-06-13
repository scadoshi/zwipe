# Todo

**Primary goal: Ship Zwipe as a webapp at zwipe.net. (iOS App Store: LIVE as of 2026-06-06. Android: next.)**

Completed work archived at `context/archive/complete-2026-Q1.md` (swept 2026-05-27).

---

## Next Up

**The next big three (set 2026-06-11, in order):**
1. **Android — get the clock ticking.** Confirm the Play account's closed-testing requirement and start the 14-day clock immediately; polish items ride alongside (see Android section below).
2. **Marketing — get users.** Business cards for LGSs, Reddit/X posts (see Marketing & Discovery below). The first wave doubles as the Android closed-test tester pool.
3. **VPS — get stable.** Un-defer once the first two are moving; plan is ready in `context/plans/vps-migration.md` (home server retirement is coming).

- [x] **Synergy data layer (cache-first) — SHIPPED 2026-06-11** — full stack live in prod: migration applied, separate worker service deployed under a least-privilege DB role (own runner + systemd unit), drip-seed warming the top-1,000 commanders, deck-aware search endpoint live, synergy-ordered add screen in client build 32. zerver writes nothing; DB is the only interface. Plan: `context/plans/synergy-data-layer.md`. Product framing: `context/product/premium/smart-stack.md`. *AI assistants: check local memory for the data-source strategy before extending this.*
- [x] **Moxfield import (phase 2) — DENIED, shelved** (2026-06-10) — Moxfield support (Rob) replied same day: policy is no API access for sites/apps that offer deckbuilding. They plan a scoped deck-export endpoint for such services eventually (no ETA; will be announced on their help pages) and invited a re-request once it's live. Until then Moxfield users use the text-paste importer. Watch item moved to `backlog.md`; see `context/plans/deck-import.md`.
- [x] **Backup restore drill — PASSED 2026-06-13** (first ever run; doubled as VPS Phase 0 step 6). Fresh manual dump (`zwipe-20260613.sql.gz`) restored onto the Hetzner VPS: PG 17.10→18 clean, all counts present (115,805 cards, 24 users, 37 decks, 1,627 deck_cards, 22 migrations), zerver booted against it, `/health` green, auth rows intact. Only noise was `synergy_worker` GRANT errors (role not on VPS yet — cosmetic for zerver). Result logged in `context/plans/backup-restore-drill.md`. **Cutover prereq surfaced: `createuser synergy_worker` on the VPS before the zynegry worker runs.**
- [ ] **Migrate prod off the home server to a VPS — CUTOVER DONE 2026-06-13; cleanup remaining.** `api.zwipe.net` now serves from Hetzner CPX31 (Hillsboro OR, Ubuntu 26.04, PG 18, tailnet `100.114.251.8`, hostname `zerver-prod`). Hardened (key-only `scadoshi`, ufw deny-all + `tailscale0` only). All three services live as boot-enabled systemd units: `zerver` (→ api), `zynergy` (worker, `synergy_worker` role), `cloudflared` (tunnel `zwipe-vps` = `2b5d54b3-…`). Data restored from a fresh final dump (PG17→18 clean, 0 errors): 24 users / 37 decks / 1,627 deck_cards / 115,805 cards / 22 migrations. Verified on live mobile: login, decks, writes. Cutover = new-tunnel + proxied-CNAME flip, ~2 min downtime. Full result + the two gotchas (use `127.0.0.1` not `localhost` in tunnel ingress on IPv6 hosts; `createuser synergy_worker` before restore) in `context/plans/vps-migration.md`. **MIGRATION COMPLETE 2026-06-13.** CI runners on VPS (`zerver-prod`, `zynergy-prod`); home runners removed; crons on VPS (zervice 4am + backup 5am), home crons disabled; VPS backup proven (pg_dump 18 → R2). `WHERE TRUE` fix pushed + deployed via the new runner pipeline (validated end-to-end: push → `zerver-prod` runner → build/migrate/restart → 200). Mac `zerver` alias repointed to `100.114.251.8`. **Email restored with a NEW Resend key** (home was already off when we tried to copy the old one — generated fresh in the Resend dashboard; verification email tested working end-to-end via `zwipe.net`→`api.zwipe.net`→VPS). Home is **powered off** (intact = rollback: boot → flip CNAME to home tunnel `70ba169b-…` → start zerver). **`zwipe.net` hostnames carried over unchanged — no IP/host was hardcoded anywhere.** *Lesson:* don't pipe `ssh home '...' | ssh vps` once home may be off — the empty pipe stripped the .env RESEND lines and 502'd prod for ~1 min until re-added. **Only "later" items left:** (a) tighten `scadoshi` NOPASSWD-ALL on the VPS to least-privilege; (b) when repurposing home as a gaming box, that reinstall wipes the disk — do it after ~1–2 clean VPS weeks, and rotate any still-shared secrets (R2 keys — JWT/DB are already fresh on the VPS).
- [x] **`POST /api/card/search` 500 on a condition-less filter — ROOT-CAUSED + FIXED 2026-06-13** (found during VPS Phase-0 smoke test). Not a panic: `search_scryfall_data_deck_aware` (`zerver/src/lib/outbound/sqlx/card/mod.rs`) baked `WHERE ` into the `QueryBuilder` unconditionally, so a `CardFilter` that pushes **zero** conditions produced `... WHERE  LIMIT $1 OFFSET $2` → Postgres `syntax error at or near "LIMIT"` → `SearchScryfallDataError`. **Fix:** seed `sep.push("TRUE")` as the first separated clause — empty filter degrades to valid `WHERE TRUE`, and each real condition still gets its ` AND `. No-op for non-empty filters (`WHERE TRUE AND <cond>` ≡ `WHERE <cond>`). Verified on the VPS box: `{}` now 200 (25 rows), `name_contains` still 200. **Prod impact ~nil** — the client always sends ≥1 condition (commander autocomplete carries a search term), so the zero-condition path was never hit in practice; this was latent. **Committed on `main`; needs push to deploy to prod** (CI auto-deploys home server on push). Minor follow-up (not blocking): the error didn't surface in `/var/log/zwipe` under the default `RUST_LOG` during first repro yet did under `sqlx=debug` — worth a quick look at whether `log_500` output is reliably hitting the file appender, and consider giving `CatchPanicLayer` a logging handler for true panics.
- [ ] **Track 1.0.6 (build 35) App Store review + propagation** — builds 32/33 (2026-06-11) superseded by 34 (hints, resend cooldown, warning remedies, skeleton + flash fixes) then **35 (submitted for review 2026-06-11)**, which adds the remove-screen stage/promote swipe toggle and the local board-sync fix. Headline feature is synergy-ordered card suggestions (deck-aware add screen), plus Archidekt import, the min-version gate, the 401/health fixes, and first-run hints. Supersedes 1.0.5 (build 31). Once approved + propagated it's the floor for `MIN_CLIENT_VERSION` gating. Watch ASC review states; What's New text in `context/ops/ios/app-store-submission/form-fields.md`.
- [x] **Deck-aware card search → client adoption — SHIPPED in build 32 (2026-06-11)** — `POST /api/deck/{deck_id}/card/search` excludes in-deck cards (all boards + profile slots) and defaults to synergy ordering when no explicit sort; old `/api/card/search` untouched. Client add-cards screen consumes it and auto-serves on open. See `context/plans/synergy-data-layer.md`.
- [x] **Missing-auth responses return 500, should be 401** (fixed 2026-06-11) — the 500 came from the user-id-keyed `tower_governor` layer (it runs before the auth extractor; `UnableToExtractKey` mapped to 500 by default). Added an `error_handler` remapping that one case to 401 on all four user-keyed limiters, delegating 429/everything else to the library default; also moved the `AuthenticatedUser` extractor's missing-header rejection 400→401. Live-tested: unauth private routes → 401, authed → 200, rate limiting still 429.
- [x] **`/health` path drifted** (fixed 2026-06-11) — `GET /health` now runs the combined server+database check (`/health/server` and `/health/database` unchanged); added `health_route()` helper in core paths. `GET /` still returns status+version.
- [x] **Resend-verification throttle + refresh — BUILT 2026-06-11** (per `context/plans/resend-verification-throttle.md`; server `3692c410`, client `ae916671`) — server: dedicated per-route limiter on `/api/auth/resend-verification` (burst 1, then 1/60s per user, `unauthorized_on_missing_key` handler); live-tested locally: 5 rapid calls → 1 through + 4×429, no-auth → 401. Deploys with the next zerver deploy (CI on push). Client (profile email row): Resend greys out with a 60s countdown matching the server window (optimistic; cleared on non-429 failure, kept on 429 with a gentle toast), plus a "Check again" button that re-fetches the user and flips the badge to Verified in place. Rides the next App Store build (after 33).
- [x] **Security notification emails on profile changes — BUILT 2026-06-11** (noted 2026-06-10) — `AuthService::change_username/change_email/change_password_and_revoke_sessions` now send a security email after the repo change commits, fire-and-forget like registration's verification email. Email changes notify the OLD address (the new one is what an attacker controls) with the new address named; username changes name old and new; password changes note that sessions were revoked. Three new templates in `zerver/.../auth/email_templates/` (`email_changed`, `username_changed`, `password_changed`) matching the existing monospace style, each with a "wasn't you?" footer (reset via Forgot password, or Discord for email changes since the old address can no longer receive reset links). User-controlled values are HTML-escaped (usernames have no character whitelist). Server-side only via Resend; deploys with the next zerver deploy.
- [x] **Min-version gate client build (1.0.5)** (2026-06-10) — built per `context/plans/min-version-gate.md` and shipped: public `/api/client/min-version` driven by `MIN_CLIENT_VERSION` env (live on prod at `0.0.0` = open), `zwipe_core::version` compare (fails open), 60s client poll, blocking "Update required" screen. In build 31.
- [x] **Deck migration from Archidekt** (2026-06-10) — `POST /api/deck/{deck_id}/import/archidekt` imports an Archidekt URL's cards into an existing deck by Scryfall printing UID (name fallback), identical semantics to text import. Both importers gained Add/Replace `ImportMode` (board-scoped replace) and the import screen got From/Mode/Board chips. Server deployed + smoke-tested on prod; client in build 31. See `context/plans/deck-import.md`.
- [x] **Reach out to recommander.cards dev** (2026-06-10) — sent friendly intro to Michael Celani (GamesfreakSA) via Discord + X, pitching a coffee chat about Zwipe consuming Recommander's recommendation API (good-ergonomics client + good-recs API), with traffic + credit driven back to him. Awaiting reply. See `backlog.md` High Priority for talking points.

---

## Pending Gated Merges — server flips waiting on iOS propagation (~2026-06-23)

Two prepared branches sit on origin, each a server-side flip that would break
older iOS clients. **Merge only after ASC Analytics → App Versions shows the
gating build at ~100% of active installs.** Both can land together once 1.0.4 (build 30)
has propagated (1.0.4 implies Build 25's gate too). 1.0.5 (build 31, submitted
2026-06-10) supersedes 30 and carries the min-version gate — once 1.0.5 is the
floor, stragglers on these flips can be force-updated via `MIN_CLIENT_VERSION`
instead of waited out.

| Branch | What it does | Gate | Plan |
|---|---|---|---|
| `feat/wire-format-rfc3339` | Server emits RFC3339 `Z` timestamps, deletes `wire_time` adapter from zwipe-core | Build 25+ (1.0.3) propagated | `context/plans/timestamptz-migration.md` (phase 2) |
| `feat/refresh-token-hardening` | Strict single-use refresh rotation (`FOR UPDATE` + delete check) | 1.0.4 (build 30, submitted 2026-06-09: single-flight refresh + new icon + toast copy) propagated | `context/plans/refresh-token-hardening.md` |

Per-branch verification and rollback steps live in the linked plans. Merging
either triggers an automatic zerver deploy via CI.

---

## Web App — Ship Full App via Zite at zwipe.net

Build the full deck builder into zite so `zwipe.net` serves both marketing pages (logged out) and the authenticated app experience (logged in). See `architecture/decisions.md` for rationale.

### Wasm Build Blockers

Zwiper doesn't compile to `wasm32-unknown-unknown` yet. Two issues (discovered 2026-04-06):

1. **`getrandom` needs `wasm_js` feature** — `getrandom` 0.4+ requires explicit `features = ["wasm_js"]` for wasm32 targets. Zite already has this. Zwiper needs it too, but it goes in `zwiper/Cargo.toml` (NOT the workspace root — virtual manifests can't have `[target]` sections).

2. **`tokio` pulls in `mio`, which doesn't compile to wasm32** — Tokio's full runtime uses OS-level I/O via mio, which has no wasm support. Zwiper uses tokio in 4 places, all for timers:
   - `zwiper/src/lib/inbound/screens/profile/components/delete_account_dialog.rs` — `tokio::time::sleep`
   - `zwiper/src/lib/inbound/components/auth/session_upkeep.rs` — `tokio::time::interval`
   - `zwiper/src/lib/inbound/screens/deck/components/deck_fields.rs` — `tokio::time::sleep`
   - `zwiper/src/lib/inbound/screens/deck/card/components/image_preview.rs` — `tokio::time::sleep`

   Options for wasm-compatible timers:
   - `gloo_timers::future::sleep` for wasm, `tokio::time::sleep` for native (behind `#[cfg]`)
   - `dioxus-sdk-time` (already a dependency) if it provides cross-platform timers
   - `web_sys::setTimeout` wrapped in a future

   Tokio itself should be gated behind non-web features in zwiper's `Cargo.toml`, or the timer calls need platform abstraction.

### Build the App into Zite

Once wasm compiles, build the authenticated experience into zite:

- [ ] Resolve wasm build blockers (getrandom feature + tokio/mio platform abstraction)
- [ ] Add login/register screens to zite
- [ ] Add authenticated routes: deck list, deck view, card search/swipe, profile, preferences
- [ ] Dual input for card selection: swipe gestures for mobile browsers, arrow buttons for desktop
- [ ] Add `zwipe.net` to zerver's `ALLOWED_ORIGINS` for CORS
- [ ] Session storage for web (localStorage or similar — no keyring on web)
- [ ] Test full auth flow: register, verify email, login, refresh token rotation
- [ ] Test deck CRUD, card search, card add/remove via both swipe and arrow buttons
- [ ] Rework `/download` page — still useful for iOS users, but less central

### Architecture Notes

- **Single domain**: `zwipe.net` — no subdomain split. Marketing and app coexist.
- **Security posture unchanged**: Same JWT auth, rate limiting, account lockout. Browser is just another API client.
- **Ship both**: Webapp ships first for immediate reach. iOS submits to App Store in parallel.
- **Reuse**: zite already depends on `zwipe-core`. Domain types, validation, and shared CSS (`shared/themes.css`) are ready.

---

## App Store Submission — LIVE 🎉 (2026-06-06)

**Zwipe is live on the App Store as "Zwipe TCG":** https://apps.apple.com/us/app/zwipe-tcg/id6761341603

Build 15 cleared review after the metadata scrub. The path there:
- Rejected 2026-06-03 under Guideline 4.1(a) Copycats — metadata referenced Magic: The Gathering without authorization from Wizards of the Coast. Apple's complaint was scoped to **metadata** (app name, description, keywords, screenshots), not in-app behavior.
- Resubmitted 2026-06-04 after scrub: renamed "Zwipe MTG" → **"Zwipe TCG"**, removed all MTG/Magic/Commander/EDH/Planeswalker/Scryfall references from the listing, reworded as generic "trading card game deck builder". Screenshots still showed MTG card art but it passed anyway.

Details:
- Bundle ID `com.scadoshi.zwipe`, App Store name "Zwipe TCG"
- Distribution certificate + App Store provisioning profile in place
- Export compliance: no encryption beyond HTTPS
- zite: iOS "App Store" nav link now points straight to the live listing; the old pending `/download/ios` page was removed

Standing risk: WotC could still C&D the app for trademark/copyright at any time. Their [Fan Content Policy](https://company.wizards.com/en/legal/fancontentpolicy) explicitly excludes applications. Long-term risk; independent of the App Store.

---

## Post-Launch Polish (Live on App Store)

Momentum work now that Zwipe TCG is live. Roughly ordered by user-visible impact.

### Marketing & Discovery
- [x] **Refresh App Store imagery** (2026-06-06) — screenshots recaptured from build 19 (1.0.1) on iPhone 11 Pro Max simulator (1242×2688px). 12 screenshots in `context/ops/ios/app-store-submission/`.
- [x] **Update App Store icon to gruvbox** (2026-06-07) — new 1024×1024 master, gruvbox tan Z on `#282828`, alpha stripped, sized to 40/60/80/87/120/180/1024. Shipped in 1.0.2 build 21. Repack process documented in `context/ops/ios/appstore-icon-update.md`.
- [ ] **Marketing posts** — Reddit (r/EDH, r/magicTCG, r/CompetitiveEDH) and X. Lead with the swipe demo video, link the App Store listing.
- [ ] **Business cards for LGSs** (noted 2026-06-10; **design DRAFTED 2026-06-12, awaiting feedback + a printer**) — self-contained HTML/CSS mockup at `context/marketing/business-card.html` (committed `cebe5d91`). It's a real MTG-style **creature card** (not a token): 2.5×3.5in, gruvbox by default with a dark-theme picker, front + back shown together. Front = title bar with WUBRG mana cost (real mana-font symbols, toggle for coded `{W}` text), theme-tinted **QR** (defaults to zwipe.net; App Store option) as the art, type line `Artifact Creature — Mobile App` with the pixel-Z as the set symbol, oracle text with keyworded features (**ETB swipe** left-red/right-green, **Synergy**, **Import**, **Free** — "Free forever. No ads.", deliberately no "subscriptions" so premium stays open), `*/*` P/T, selectable flavor (default banger: "Every great deck begins with a single swipe."). Back = full Zwipe wordmark + `zwipe.net`, nothing else. Oracle text auto-fits its box. Flat (no gradients). **Next: gather feedback from folks, then find a printer** (see options below). Ask store owners before leaving stacks; commander-night crowds are the target.
  - **Printing notes**: standard playing-card stock is **2.5×3.5in (63×88mm)**. For a true card feel use a card printer (MakePlayingCards/MPC, PrinterStudio, Ad Magic, or a local print shop with 300–350gsm + matte/linen finish) — order a custom card or "custom card game" SKU. For cheap-and-fast, a local shop can do business-card stock but it's thinner. Export: open the HTML, print to PDF at 100% (no scaling) with "background graphics" ON; add 1/8in bleed if the printer wants it. QR scannability holds down to ~0.8in — current QR is well above that.
- [ ] **Commander search debounce hint** (real user confusion 2026-06-11: racebeard thought "Dina, Soul Steeper" was missing from commander search because nothing appeared while typing) — commander search on the create/edit deck screen requires **3+ characters** then waits **800 ms** before searching (`deck_fields.rs:149,158`; partner/background fields fire at 1 char). Nothing tells the user this, so a slow reveal reads as "card missing". Fix: lightweight inline hint, NOT a dialog — e.g. persistent helper text under the commander field ("Type at least 3 letters, results take a moment") or a "searching..." indicator while the debounce timer runs (probably the better fix: feedback beats instructions). Rides the next build.
- [x] **First-run hints — BUILT 2026-06-11** (server `06f0b441`, client `298c4848` + polish through `f8cd8008`; evolved from "tutorial toasts" to one-time dialogs during design). Shipped: **`hints_shown` jsonb on users** (additive migration; rides every user-returning query: login, get-user, change-username/email) + `PUT /api/user/hint` marking a key shown (idempotent, returns updated user; key constants + validation in `zwipe-core/.../hints.rs`). Client: `use_one_time_hint(key)` hook (mount-time) + `open_and_record_hint` (for async gates) + composable `HintDialog` shell (`components/hint_dialog.rs`: `HintLine`, `HintBullets`/`HintBullet`, `HintKey` inert accent button chips, `HintColored` color-coded words, optional dividers) — auto-opens on first visit per account, reports shown (fire-and-forget, optimistic), persists session so it survives restarts. **Six auto-once hints live**: `first_login` (home: Decks, Profile, verify-email emphasis in warning color), `profile` (Change / Preferences / Delete account), `first_deck` deck-profile tour (Cards, Edit, More full action list, Buy, warnings), `deck_cards` browsing (tap-to-expand + Boards/Show chips; fires only once the deck HAS cards), `add_swipes` and `remove_swipes` (color-coded swipe vocabulary: green right / red left / yellow up-maybeboard / accent down-undo). Every main screen carries a grayed top-right "?" in the page header that reopens its hint forever; the Decks list one (`f8cd8008`) is **on-demand only** (no key, never auto-opens) and is the future home for bulk-operations bullets. New hints = new keys, no migration. Related (built same day): below-minimum deck warning row carries Add cards / Import remedy buttons. Live-tested: register → `{}`, marks accumulate + idempotent, 422 bad key, 401 no auth. NOTE: hint persistence needs the server deploy; until then dialogs re-open per screen visit and mark calls 404 harmlessly.

### Web/Zite Polish
- [ ] **Increase `Z` logo size on zwipe.net** — current ASCII logo reads small; bump scale or font-size on the landing hero.
- [ ] **Mobile testing pass on zwipe.net** — verify landing, about, privacy, verify/reset token pages on iOS Safari + Android Chrome. Check the sticky nav and entrance animations under narrow viewports.
- [ ] **Everforest theme review** — possibly too green; sample real card art against it and consider desaturating the background or shifting the accent.

### UX Regressions (zwiper)
- [x] **"Update required" screen flashes when applying filters on add/remove cards** (observed + FIXED 2026-06-11, ships in build 33) — root cause was a Dioxus context **type collision**, not the version poll: `CardFilterSheet` looked up its collapse flag via `try_use_context::<Signal<bool>>()`, and on add/remove (which provide no closer `Signal<bool>`) that resolved to the root `upgrade_required` gate — so Apply literally set the min-version gate true. Deck-cards was immune because `view.rs` provides its own `Signal<bool>` (the diagnostic clue). The flash self-healed because the gate flip re-rendered the root, which re-ran `spawn_upkeeper` and spawned a fresh poll loop whose immediate first tick reset the gate — i.e. every flash also leaked an extra 60s polling loop. Fix: newtype both context signals (`CollapseExpanded` in the filter sheet, `UpgradeRequired` in session upkeep — same pattern as the existing `DeckCards` newtype) and move the upkeep loop + usage flusher behind `use_future`/`use_hook` so root re-renders can't multiply background tasks. Commit `598e45a5`.
- [x] **Show password eye icon** (2026-06-07) — Show/Hide text button wired into `TextInput` so every password field (login, register, change password, change username, change email confirm, delete account) gets it automatically. Shipped in 1.0.2.
- [x] **Alert dialog dark overlay missing** (2026-06-07) — confirmed via reading the upstream `dioxus-primitives` source: `AlertDialogRoot` deliberately does *not* emit an overlay (docs say "you can use it to create a backdrop for the dialog if needed"). Our wrapper now renders `<div class="alert-dialog-overlay" data-state="open">` as a sibling when open. Shipped in 1.0.2.
- [x] **Card filter broken in deck screens** (2026-06-07) — exhaustive per-filter pass found four distinct bugs in `zwipe-core/src/domain/card/models/search_card/filter_cards.rs` (the in-deck predicate the deck cards / remove cards screens run against the loaded deck — backend SQL search unaffected). Plus a rarity sort fix.
  - **Basic types include/exclude** — `card_type_contains_any`, `card_type_contains_all`, `card_type_excludes_any` compared a lowercased `type_line` against `CardType::to_string()` (capitalized: `"Creature"`), so `"legendary creature — dragon".contains("Creature")` was `false`. Include hid everything; exclude excluded nothing. Added `.to_lowercase()` to each `ct.to_string()`.
  - **Set filter include/exclude** — predicate compared against `sd.set` (the lowercase 3-letter code, e.g. `"mh2"`), but the UI (`extract_sets`) and backend `/sets` endpoint both send `set_name` (e.g. `"Modern Horizons 2"`). Switched predicate to `sd.set_name`. Fixed stale setter doc that still said `"MH2", "ONE"`.
  - **"Is commander in <format>"** — predicate field/getter/setter/UI all existed and the backend SQL adapter honored it, but `filter_cards.rs` had **zero** references to `is_commander_in_format`. Toggle did nothing on deck screens. Added predicate using existing `commander_eligibility::is_valid_commander(card, format)` helper.
  - **"Is legal in <format>"** — same shape as commander: `legalities_contains_any` set everywhere, backend SQL honored it (`legalities->>format IN ('legal', 'restricted')` OR-joined per format), `filter_cards.rs` ignored it. Added predicate mirroring backend semantics — card passes if any chosen format is `Legal` or `Restricted`, parsing format strings via `Format::try_from`.
  - **Rarity sort alphabetical** — `OrderByOption::Rarity` compared `to_long_name()` strings, giving `Common → Mythic → Rare → Uncommon`. Derived `Ord, PartialOrd, Eq` on `Rarity` (variant declaration is already in tier order so discriminant comparison gives Common < Uncommon < Rare < Mythic < Bonus < Special). Both sort sites updated to compare enum values directly. Added a `do not reorder` comment on the enum.
- [ ] **Round-trip filter coverage test** (follow-up from the audit above) — build a `CardFilter` with every field populated and assert that both the backend SQL adapter and `filter_cards.rs::filter_by` accept/reject the same fixture set. Would catch the "field exists end-to-end except in the frontend predicate" pattern that hid the commander and legality bugs for an unknown amount of time. Currently `filter_cards.rs` mirrors backend logic by hand, with no compile-time linkage to the `CardFilter` field list.
- [x] **Deck metrics skeleton state** (2026-06-07) — replaced the spinner/blank flash with skeletons across deck list, deck view (profile + stats with bordered info-list rendition), deck cards list, edit deck form, printing sheet, and home flavor text. Stats skeleton uses a shared `SkeletonInfoList` component that mirrors the real `.info-list` border, radius, padding, and per-row dividers. Shipped in 1.0.2.

### Card Rendering Bugs
- [x] **Cards missing from search** (2026-06-06) — `Kibo, Uktabi Prince` and `Wear // Tear` were invisible to card/commander search and in-deck filtering. Two root causes, both fixed: (1) `latest_cards` materialized view's `DISTINCT ON ... ORDER BY released_at DESC` picked digital-only and promo-flagged printings whenever they were the most recent — the new ORDER BY prefers paper, non-promo, non-oversized, non-content-warning rows first. (2) `CardFilterBuilder::default()` hardcoded `promo: Some(false)`, which over-filtered Jumpstart, Secret Lair, and UB-bonus printings where many cards only exist in promo form — relaxed to `None`. Migration `20260606120000_latest_cards_prefer_real_printings.sql` also remaps existing `deck_cards.scryfall_data_id` and `decks.{commander,partner_commander,background,signature_spell}_id` references so existing decks switch to the preferred printing on deploy.
- [x] **DFC handling — render front face for transform/MDFC + add flipper for all double-faced layouts** (2026-06-06) — landed in two commits:
  - **Step 1 — front face renders**: `ScryfallData::primary_image_url(ImageSize)` in zwipe-core now falls back from top-level `image_uris` to `card_faces[0].image_uris` so transform/MDFC cards surface in search and on every image render site. Also fixed the client-side filter in `add.rs` that was dropping cards with no top-level `image_uris` (the actual reason `Delver of Secrets` never reached the renderer despite the backend returning it).
  - **Step 2 — flipper**: new `FlippableCardImage` component (`zwiper/.../components/flippable_card_image.rs`) wraps `<img>` + a "Flip" squircle button. Wraps every image surface: swipe stack (top card only, peeking cards stay plain), printing carousel + single-printing view, image preview modal. Wrapper gets `aspect-ratio: 5/7` only when flippable so the button hugs the card edge at every call site; single-faced cards keep natural sizing.
  - **Meld handling not added** — meld pieces (Urza, Lord Protector et al.) already render correctly because each piece is a separate single-faced Scryfall row. Their "back" (the melded result, e.g. Urza, Planeswalker) is a separate scryfall row, not in `card_faces`. Cross-piece flipping via `all_parts` could surface that, but it's a different feature and not in scope for the current fix.
- [ ] **Investigate Delver not showing in existing deck after MDFC fix landed** — observed 2026-06-06 on an airplane (slow wifi may be a factor): a deck containing `Delver of Secrets` returned zero rows from `SELECT users.username, decks.name, scryfall_data.name FROM users JOIN decks ... JOIN deck_cards ... JOIN scryfall_data WHERE scryfall_data.name ILIKE 'delver of secrets'`. Card and combination definitely existed before. Fresh deck testing worked fine, so probably state-specific. Could be the migration's deck-card remap pointed the old reference at a row that no longer exists, OR the connection was timing out and the join silently returned empty. Worth re-running the query off airplane wifi first; if it still returns zero, dig into the remap audit.

### Infrastructure (Reactive)
- [x] **Cloudflare edge caching for immutable card endpoints** (2026-06-07) — probed via `zcripts/latency/probe.sh`: backend healthy (~52ms LOCAL search, 110k+ cards via materialized view + GIN trigram), tunnel floor ~125ms. Redis would buy ~50ms on 250ms — not worth complexity. CF edge caching does. Backend: moved 8 GET card metadata routes (`/api/card/{id}`, `/{oracle_id}/printings`, `sets`, `types`, `keywords`, `oracle-words`, `artists`, `languages`) from `private_routes` to `public_routes` with IP-keyed rate limit (60/s burst); `POST /api/card/search` stays private. Frontend: dropped `bearer_auth` + `session` param on all 8 client methods so CF doesn't `BYPASS` cache for authenticated requests. CF dashboard: one Cache Rule with `starts_with(http.request.uri.path, "/api/card/")`, Eligible for cache, Ignore origin Cache-Control, Edge TTL 24h. Verified end-to-end via `zcripts/latency/cf_cache_verify.sh` — converged to 6/6 HIT across POPs after warmup. Shipped in zerver `5666a86b` and zwiper `a721e413`; iOS client ships in 1.0.2 build 22. Roadmap doc: `context/ops/latency-optimization.md`.
- [x] **HTTP response compression** (2026-06-07) — `tower-http`'s `CompressionLayer` added to the public stack in `zerver/src/lib/inbound/http/mod.rs`. gzip+brotli enabled via `tower-http` features. `/api/card/search` body 39690b → 16444b on the wire (59% smaller); `/api/deck` body 3996b → 727b (82% smaller). Verified via `--compressed` curl in `zcripts/latency/probe.sh`.
- [x] **HTTP/2 client multiplexing** (2026-06-07) — workspace reqwest gained `http2` feature. Reqwest auto-negotiates h2 via ALPN with CF. Benefits burst flows like `deck/card/view.rs` (4 parallel `get_card` calls for command zone). Ships in iOS 1.0.2 build 23.
- [x] **Lower default search page size** (2026-06-07) — `CardFilter::default_limit()` 100 → 25 in zwipe-core; swipe stack's `pagination_limit` matched at 25, `load_more_threshold` 15 → 5. Compounding win: DB query returns 4× fewer rows, serialization is 4× cheaper, then gzip on top. LOCAL search dropped 52ms → 5ms. Ships in iOS 1.0.2 build 23.
- [ ] **Home server may struggle under marketing load** — current host is a single Ubuntu box behind Cloudflare Tunnel. If real users surface latency or 5xx spikes beyond what edge caching absorbs, evaluate migrating zerver to a cheap VPS in a major POP (Hetzner/Fly). Don't pre-optimize — only act on observed pain.
- [ ] **Migrate TIMESTAMP columns to TIMESTAMPTZ** — every timestamp column in the schema is plain `TIMESTAMP` (no time zone), so values stored = whatever the writing session's `TIMEZONE` resolves to. Today the SQLx pool pins `SET TIME ZONE 'UTC'` on every connection (`zerver/src/lib/outbound/sqlx/postgres.rs::after_connect`) as a backstop, so writes land canonically UTC, but the type system doesn't enforce it. Plan and execution captured in [context/ops/timestamptz-migration.md](../ops/timestamptz-migration.md). Schedule after metrics PR burns in for a week; not coupled to any feature work.
- [x] **Public stats strip on zwipe.net** (2026-06-07) — shipped on `zite-metrics-dashboard` branch. `StatsStrip` component in `zite/src/components/stats_strip.rs` fetches `GET /api/marketing/stats` during SSR and renders `Cards swiped · Searches run · Decks created` under the tagline in JetBrains Mono. CF Cache Rule live at 2h Edge TTL (free-plan minimum) covering `/api/marketing/*`. Numbers refresh on each GH Pages rebuild — acceptable for vanity; if staleness matters later, scheduled rebuild cron is the next move.

---

## Android — Near Submission Ready (PRIORITIZED 2026-06-10: ship before premium)

Decision (2026-06-10): grow the user base before monetizing — Android ships
before the premium tier. Rationale: the port is days of polish away (Dioxus),
premium revenue scales with users, MTG pods are mixed-device so adoption
spreads through playgroups, and shipping Android first means premium later
launches on both stores in one pass (StoreKit + Play Billing designed together).

- [ ] **FIRST: finish Play developer account verification** — account
  CREATED + $25 PAID 2026-06-11 (new personal account, so the closed-testing
  requirement — 12 testers, 14 continuous days before production access —
  definitely applies). Remaining sequence: **complete the identity
  verification tasks (can take days on Google's side — do them promptly so
  the wait runs unattended)** → upload a build to a closed track → recruit
  12 testers (LGS regulars + Reddit) → 14-day clock runs while the polish
  items below are finished.

Android build compiles and runs. Remaining polish before Play Store submission:

- [ ] Card images show white corners — the white is baked into the image data from Scryfall (white-bordered card editions). iOS clips correctly via WKWebView; Android WebView does not honor `overflow: hidden` + `border-radius` on `object-fit: contain` images. Tried: `overflow: hidden` on img, wrapper div with `border-radius` + `overflow: hidden`, `-webkit-mask-image` hack. None work on Android WebView. Options: crop with `object-fit: cover` (loses card edges), mask SVG overlay, or accept as-is for black-bordered cards (majority) and revisit for white-bordered.
- [ ] Swipe gesture doesn't tilt the card — cards should rotate slightly during drag like they do on iOS
- [ ] Lock screen orientation to portrait — need `android:screenOrientation="portrait"` on main activity. Dioxus may support this via `[android.raw.manifest]` or activity-level config. Test on Pixel once available.

---

## Synergy & Popularity Data

Per-commander synergy/popularity layer — the cache-first item at the top of
Next Up is the foundation; these are the consumers once it exists.

- [ ] Synergy scores — surface cards with high synergy to the deck's commander
- [ ] Popularity data — most-played cards for a given commander
- [ ] Salt score, display per card and aggregate per deck, filtering and sorting on card search
- [ ] Evaluate further data (themes, combos, etc.) as the layer matures

---

## Mechanical Category — Heuristic Refinement

Phases 1+2 shipped (see archive). ~73% classification rate today; refinement targets are below. Layers 2+3 (AI classifier + fine-tuned model) tracked in `backlog.md`.

- [ ] Add more test cases for edge cases and false positives/negatives
- [ ] Audit a sample of classified cards per category to find misclassifications
- [ ] Lands should NOT be classified as ramp (fixed: removed `type_line.contains("land")` from ramp fallback) — verify still holds
- [ ] Tune regex proximity windows (e.g. blink regex was too narrow, widened to 80 chars)
- [ ] Consider additional ramp patterns (e.g. treasure token creators, rituals like Dark Ritual)
- [ ] Consider additional removal patterns (e.g. "exile target" with qualifiers, fight mechanics)
- [ ] Burn heuristic excludes creatures — should it include creatures with ETB damage?
- [ ] Stax heuristic may false-positive on cards that say "can't" in reminder text

---

## Zwipe for Commander (Phase 3, UX Enhancement)

A dedicated swiping flow for commander selection. Future work — only build if users want it.

- [ ] On create/edit screen, when format has a commander, show "Zwipe for Commander" button
- [ ] Opens the swiping interface pre-filtered to valid commanders for the selected format
- [ ] User can adjust filters (colors, mana cost, set, etc.) and swipe through candidates
- [ ] First swipe-right sets the commander and returns to the create/edit screen
- [ ] Format filter defaults to deck's format but user can change it
- [ ] Works on both create and edit screens

---

## Testing

- **Integration tests** — SQLx repository tests require a real PostgreSQL instance. Unit test phase complete (308+ tests, ~100 in zwipe-core). Remaining gap: outbound adapters have no coverage.

---

## Maintenance

- **sqlx 0.8 → 0.9** — major bump. 0.9 has breaking changes around type mappings and connection options. Needs a dedicated branch where the integration tests run against a real Postgres before merge.
- **keyring 3 → 4** (zwiper) — major bump. Used for iOS Keychain on `apple-native`. Needs on-device test before merging; don't ship blind.
- **GitHub Actions Node.js 20 deprecation** — forced to Node.js 24 on June 2, 2026. All workflows already on latest major versions. No action needed — monitor for v5 releases.
- **Pin other git deps** (optional follow-up) — `dioxus-primitives` is now pinned to rev `02801f27` (commit `b40d2019`). Audit remaining workspace deps: `grep "git = " **/Cargo.toml`. Currently no other floating git deps, but worth a periodic check.
