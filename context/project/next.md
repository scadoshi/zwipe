# Next Immediate Priorities 🎯

Planned work after completing current tasks.

**Primary goal: Get Zwipe publicly available on the App Store.**

---

## App Store Submission Checklist

Everything needed to go from running on one iPhone to publicly listed on the App Store.

### 1. Fix App Name (shows "Main" on home screen)

The binary is named `main`, so iOS displays the app as "Main". Fix in `zwiper/Dioxus.toml`:
- Add `name = "Zwipe"` to the `[application]` section

### 2. iOS App Icon

The current `zwiper/assets/favicon/` icons are web favicons — iOS ignores them for the home screen icon. The App Store requires a dedicated icon set:
- **1024×1024** — required for App Store listing
- **180×180** (`@3x`) — iPhone home screen
- **120×120** (`@2x`) — older iPhones
- These go in an `AppIcon.appiconset` in the Xcode assets catalog, or via Dioxus bundle config

The current icon is ASCII-art-based — needs converting to a proper rasterized image at these sizes. Consider generating from the existing logo at `android-chrome-512x512.png` as a base.

Dioxus bundle icon config is likely via `[bundle] icon = [...]` in `Dioxus.toml` or a direct `Assets.xcassets` approach — needs research/testing.

### 3. App Store Connect Setup

1. Log into [appstoreconnect.apple.com](https://appstoreconnect.apple.com) with `scottyfermo17@gmail.com`
2. Create new app: Bundle ID `com.scadoshi.zwipe`, name "Zwipe", primary language English
3. Fill out:
   - **App description** — see `context/decisions/app-store-listing.md` (TBD)
   - **Keywords** — MTG, Magic the Gathering, deck builder, commander, card game
   - **Screenshots** — minimum 1 per required device size (6.7" iPhone required)
   - **Privacy Policy URL** — required for App Store, even for free apps (need a hosted page)
   - **Support URL** — can be a simple page or GitHub
   - **Age rating** — 4+ (no objectionable content)
   - **Category** — Games > Card Games, or Utilities

### 4. Build for Distribution (not Development)

Current build uses a Development provisioning profile. App Store submission requires:
- **Distribution certificate** (Apple Distribution, not Apple Development)
- **App Store provisioning profile** (not iOS App Development)
- Build with `dx build --release --platform ios --device "scotland-mobile"`
- Archive and upload via `xcrun altool` or Transporter

### 5. Privacy Policy

App Store requires a privacy policy URL. Zwipe collects email + deck data. A simple hosted page (GitHub Pages, Notion, etc.) is sufficient. Must state:
- What data is collected (email, deck contents)
- Where it's stored (Pi backend at `api.zwipe.net`)
- No third-party data sharing

### 6. Submit

- Upload build via Transporter or `xcrun altool`
- Fill out export compliance (no encryption beyond standard HTTPS — answer No to encryption questions)
- Submit for review — typical review time 1–3 days

---

## User notes about minor tweaks

~~1. Silent-omit documentation on `find_cards_by_exact_names`~~ — **DONE** (2026-03-25). `ports.rs` now documents that missing names are silently omitted (no error). Rename deferred/dropped.

~~2. Accordion scroll-to-focus~~ — **DONE** (2026-03-25). Each `AccordionItem` in the add/view/remove filter bottom sheets fires `on_change` and calls `document::eval` with a 50ms-deferred `scrollIntoView({ behavior: 'smooth', block: 'start' })` targeting the opened item. Delay prevents phantom-open touch events and lets layout settle.

~~3. Config filter labels: add "is" or "has" prefix to boolean fields so they read naturally — "is playable", "is digital only", "is oversized", "is promo", "has content warning".~~

~~4. **Card image size** — card images need to expand to near full-screen on mobile.~~ — **DONE** (2026-03-27). `.card-image` `max-height` changed from `42vh` to `calc(100svh - 13rem)`, added `max-width: calc(100% - 2rem)`, and reduced bottom margin. `.card-shape` (text fallback) height updated to match. Add/remove screens switched from `.centered` to `.card-swipe` (`overflow-y: hidden; justify-content: flex-start`) — eliminates vertical scroll competition with swipe gestures and fixes card-top clip caused by flex centering overflow above the scroll origin.

5. **Filter active count badges** — each filter accordion group should show how many filters are currently active (e.g. "mana (2)") so the user knows where to go to turn things off without opening every section.

6. **Filter clear empties card hand** — when clearing filters on the add-cards screen, the current `Vec<Cards>` in hand should be emptied, not left stale. User should start fresh from an unfiltered fetch.

7. **Loading indicators** — anywhere async loading occurs, a spinner or skeleton should be visible. Example: `zwiper/src/lib/inbound/screens/deck/card/view.rs` may not show a loading state while cards are fetching. Hard to notice locally (127.0.0.1 is instant) but will be visible on device hitting `api.zwipe.net`. Audit all screens that trigger async fetches and ensure a loading state is shown.

8. ORACLE_STOP_WORDS and TYPE_STOP_WORDS in zwiper/src/lib/inbound/screens/deck/card/filter/deck_cards.rs should be maintained by the zerver lib and passed to the frontend. Generally domain models or business logic should be defined there and then utilized by the frontend rather than built and maintained in the frontend. This is especially true since the backend uses the very same stop words in its queries. We should define shared logic and then use that shared logic in both places so we don't have to maintain the content in two places!

9. **Util bar button tap feedback** — `.util-btn` should animate on press (e.g. brief scale-down or opacity dip via `active` pseudo-class) but have no hover effect. Hover states are meaningless on touch screens and can leave buttons visually stuck after a tap on iOS. Remove any existing `:hover` styles on `.util-btn` and replace with a `:active` transition only.

10. **Full screen integration pass: transitions + loading states** — walk every screen end-to-end on device and audit for missing loading feedback and abrupt state changes. For each async operation (data fetch, form submit, route change) add either a skeleton placeholder or a spinner where appropriate. Also add tasteful transitions between screens and between loading/loaded states — nothing heavy, just enough to make the app feel intentional rather than janky. Goal is that no screen ever appears to flash blank or jump content in.

11. **Deck Count Limit** ---implement limit to the number of decks that any single user can have created against their profile. Pre-emptive limitation in anticipation for a subscription tier which would offer higher deck limits.

12. **Per Deck Card Limit** ---same idea as above but for cards per deck.

13. **Numeric ID for all entities** ---Accompanying uuid-based operations for security, numeric IDs will make human identification of entries easier. Consider adding to all entities. Numeric ID is represented by a naturally incrementing integer (1, 2, 3, 4, etc.) while uuid is represented as, well, a uuid. This makes operations more secure against enumeration attacks while entries remain easily found using numeric IDs. Probably a waste of time but worth considering.

~~5. Deck-aware filter dropdowns (view/remove screens)~~ — **DONE** (2026-03-25). `DeckCards` newtype context provided by view/remove screens. Filter components (artist, set, types, oracle words, keywords) use `try_use_context::<DeckCards>()` to derive selectable values from the loaded deck's cards instead of fetching from server. Add screen continues fetching from server (no context provided). Commander now also respects the active filter — hidden from the pinned slot when filtered out.

~~6. Lowercase import screen text~~ — **DONE** (2026-03-25). Placeholder sample card names and post-import result card names (imported + unresolved) are now lowercase.

---

## Testing & Stability

1. **Integration Tests** - Repository tests with real PostgreSQL (longer-term — requires real DB infrastructure)
   - Unit testing phase complete: filter_cards (34 tests), group_cards (15 tests), copy_max (9 tests), quantity, SwipeState (32 tests)
   - Remaining gap: outbound SQLx repositories have no test coverage (integration tests only viable path)

2. **Bug Fixes** - ~~Layout shift after deck creation~~, ~~iOS keyboard push issues~~ (fixed via unified `.screen` layout — see Bugs section for details)
    - ~~Quantity is not built to affect deck profile view screen dashboard metrics and it should~~ — **FIXED** (2026-03-24). `DeckMetrics::from_entries(&[DeckEntry])` replaces `ComputeMetrics` trait; each card counted by its quantity. ViewDeck fetches `Vec<DeckEntry>` instead of discarding quantities.

---

---

## Enhancements

### Deck Composition & Card Management

1. ~~**Deck Import (Text List)**~~ — **DONE** (2026-03-24). Parses Moxfield (`qty name`) and Archidekt (`qtyx name (set) collector# [tags]`) formats. Exact-name batch SQL resolution via CTE dedup. Copy-max clamping (basic lands exempt). Atomic bulk upsert with `ON CONFLICT DO UPDATE`. Import screen with results display (imported + unresolved). Export button on ViewDeck copies deck to clipboard. `ScryfallData::is_basic_land()` helper used across call sites.

2. ~~**Deck Export Screen**~~ — **DONE** (2026-03-24). Dedicated `ExportDeck` screen with readonly textarea + "copy" button with toast feedback. Replaces inline clipboard-copy on ViewDeck.

3. ~~**"Show Lands" Toggle on ViewDeckCard**~~ — **DONE** (2026-03-24). Toggle chip in group-by chip row (right-aligned). Filters lands from displayed groups reactively. Uses `ScryfallData::is_land()`. `ScryfallData::is_spell()` may come later.

~~2. **Multi-Copy Add Flow**~~ — **NIXED**. Users can adjust quantity in the view deck screen via +/− controls. Swipe-right adding 1 at a time is intentional and sufficient.

2. ~~**CopyMax Enforcement (Frontend + Backend)**~~ — **DONE** (2026-03-24). Backend: `UpdateDeckCard` guard query enforces copy_max before applying delta. `UpdateDeckProfile` truncates existing card quantities when copy_max becomes more restrictive (single UPDATE in same transaction). Frontend: ViewDeckCard +/- controls respect copy_max with toast feedback. EditDeck shows truncation warning dialog only when actual card quantities exceed the new limit.

3. ~~**Change Quantity in View Deck Screen**~~ — **DONE** (2026-03-24). Inline +/- quantity controls in ViewDeckCard expanded rows. Wired to `UpdateDeckCard` with optimistic updates and rollback on error. Singleton decks show only − (which deletes). Qty column in compact rows, omitted for singleton decks.

4. ~~**Deck Metrics View**~~ — **DONE** (2026-03-23). `DeckMetrics` in deck domain, `ComputeMetrics` trait generic over `IntoIterator<Item = &Card>`. Stats (cards, avg cmc, lands), ASCII mana curve, type/color distributions rendered on ViewDeck screen.

~~5. **Mana Pip Balance**~~ — **DONE** (2026-03-25). `DeckMetrics` extended with `pip_consumed` and `pip_produced` per color, computed in a single pass in `deck_metrics.rs`. Rendered in ViewDeck as CSS vertical bar charts with surplus checkmark indicator. ASCII chart representation replaced with CSS bars across all metric sections.

~~6. **Deck Profile Enhancements (ViewDeck screen)**~~ — **DONE** (2026-03-25).

### Card Filter: Oracle Keywords

~~4. **Oracle Text Keyword Filter**~~ — **DONE** (2026-03-25). Backend (`oracle_text_contains_any`, `get_oracle_keywords` endpoint, oracle words pipeline) and frontend (client, `keywords.rs` filter component, accordion registration) fully complete.

### Post-MVP Backlog (deferred)

- Card keyword categories (import-time tagging: burn, ramp, removal, etc.)
- EDHREC synergy integration
- Deck list screen redesign
- CardFilter: serve only playable cards by default
- Set type filter (hide funny/memorabilia/token)
- Legality/format filter (needs design work)
- Cross-deck card ownership indicator
- Stop-words shared between zerver and zwiper (item 4 from user tweaks)

---

## Rate Limiting (Planned)

### Password Reset Rate Limiting

Two-layer protection to prevent Resend quota exhaustion and enumeration:

1. **IP-level (`tower_governor`)** — dedicated governor on `POST /api/auth/forgot-password`, tighter than the blanket private-route limit. Target: ~5 requests per hour per IP.
2. **Per-email cooldown (domain logic)** — before issuing a reset token, check `last_reset_requested_at` on the `password_reset_tokens` table. If a token was issued for that email within the last 5 minutes, skip sending (return the same generic response). Prevents bypass via rotating IPs/VPNs.

Both layers return the identical generic response ("if that email exists, a link was sent") — no oracle leakage.

### Search Cards Rate Limiting

The search endpoint is the heaviest DB operation (full-text search across 35k+ cards, 100 results/page). Add a dedicated `tower_governor` instance on `GET /api/cards` with tighter params than the blanket private-route limiter:

- **Target:** ~1 request per 3 seconds replenishment, burst allowance of 5 (effectively ~20/min ceiling, ~1,200/hour theoretical max)
- 100 pages × 100 cards = 10,000 cards viewable per 5 minutes at max burst — more than enough for legitimate use
- Current limiter uses `PeerIpKeyExtractor` (IP-keyed). Future improvement: key by authenticated user ID for per-user fairness instead of per-IP.

---

## Infrastructure & Shipping

### iOS Session Persistence (Keychain Entitlement)

**Status:** Partially complete (2026-03-26). App is running on device hitting production backend. Keychain entitlement is configured in `zwiper/Entitlements.plist` and provisioning profile has Keychain Sharing enabled.

**Remaining:** Verify the `errSecMissingEntitlement` error is gone on cold start — log in, kill the app, reopen and confirm session persists without re-login.

**Deploy command (for future builds):**
```bash
cd zwiper
dx build --platform ios --device "scotland-mobile"
cp ~/Downloads/zwipedev.mobileprovision target/dx/main/debug/ios/Main.app/embedded.mobileprovision
codesign -f -s "F421F2E0FF6575A04BB18520C1A699A3F9CCEB45" \
  --entitlements zwiper/Entitlements.plist \
  target/dx/main/debug/ios/Main.app
ios-deploy --bundle target/dx/main/debug/ios/Main.app
```

**Key facts:**
- Must use `--device "scotland-mobile"` flag — otherwise dx builds for simulator (platform 7, crashes on device)
- Signing cert fingerprint: `F421F2E0FF6575A04BB18520C1A699A3F9CCEB45`
- Team ID: `VV74WQ89GD`, Bundle ID: `com.scadoshi.zwipe`
- Provisioning profile at `~/Downloads/zwipedev.mobileprovision`

### Backend Hosting

**Decided: Ubuntu Server (headless, x86_64) via Cloudflare Tunnel (2026-03-27)**

Moved off Raspberry Pi 5 (4GB RAM, aarch64) to a repurposed desktop machine: 32GB RAM, x86_64, Ubuntu Server (no desktop UI — headless, managed via SSH). GPU removed before OS install.

- **Hardware:** x86_64 desktop, 32GB RAM, Ubuntu Server (headless)
- **Domain:** `zwipe.net` — Cloudflare DNS
- **Networking:** Cloudflare Tunnel (`cloudflared`) — no port forwarding, TLS at Cloudflare's edge
- **Database:** PostgreSQL — `zwipe` DB + user
- **Process manager:** `systemd` for zerver; cron for nightly `zervice`

**Migration status (2026-03-27):**
- [ ] Disassemble desktop, remove GPU, reassemble
- [ ] Install Ubuntu Server (headless)
- [ ] PostgreSQL installed, `zwipe` DB + user created
- [ ] Cloudflare Tunnel configured, running as systemd service
- [ ] Cross-compile zerver + zervice for x86_64 on Mac (via `cargo zigbuild`)
- [ ] Deploy binaries, configure `.env`, run SQLx migrations
- [ ] systemd unit for zerver — running, enabled
- [ ] Cron entry for zervice (nightly 4am)
- [ ] Create `/var/log/zwipe/` for rolling logs
- [ ] Run zervice once manually to seed Scryfall card data
- [ ] Verify iOS app hits `api.zwipe.net` successfully
- [ ] Verify iOS Keychain session persistence across cold launches

**Cross-compile zerver for x86_64 (on Mac):**
```bash
rustup target add x86_64-unknown-linux-gnu
cargo zigbuild --release --bin zerver --bin zervice --target x86_64-unknown-linux-gnu
scp target/x86_64-unknown-linux-gnu/release/zerver zervice <user>@<server-ip>:~/zwipe/
ssh <user>@<server-ip> "sudo systemctl restart zerver"
```

**Key prod config notes:**
- iOS requires HTTPS — Cloudflare Tunnel provides this automatically
- iOS native clients don't send an `Origin` header, so CORS won't block them
- Refresh token cleanup: `zervice` runs `delete_expired_sessions` nightly at 4am, `token_cleanup` event logged on each run
- See `context/decisions/hosting.md` for full config reference

### File-Based Logging for zerver

**Implemented (2026-03-27).** Rolling daily log files written to `/var/log/zwipe/` alongside stdout. 30-day retention via `tracing-appender` non-blocking writer. Structured `event =` audit logs added at all key auth points (login success/failure/lockout, register, token refresh failures, token cleanup).

**Pi log directory already created:**
```bash
# Already run on Pi (2026-03-27):
sudo mkdir -p /var/log/zwipe
sudo chown scottyfermo:scottyfermo /var/log/zwipe
```

Log files appear as `/var/log/zwipe/zerver.YYYY-MM-DD.log`. Inspect with:
```bash
tail -f /var/log/zwipe/zerver.$(date +%Y-%m-%d).log
grep '"token_refresh_failure"' /var/log/zwipe/zerver.$(date +%Y-%m-%d).log
```

### Email Verification + Password Reset (next security item)

**Status:** Not started. One remaining open security item — see `security.md`.

Both features share the same Resend integration, so they should be built together:
- **Email verification**: on register, send a confirmation email. Block full account access until confirmed.
- **Password reset**: forgot-password flow with a 15-min single-use token. Full design in `security.md`.

**Resend setup** (resend.com — 3k emails/month free tier):
1. Add `resend` Rust crate (or use `reqwest` directly against Resend's REST API)
2. Add `RESEND_API_KEY` to zerver `.env`
3. New DB table: `email_verification_tokens (id, user_id, token_hash, expires_at, used_at)`
4. New DB table: `password_reset_tokens (id, user_id, token_hash, expires_at, used_at)` (or reuse above with a `kind` column)
5. New endpoints: `POST /api/auth/verify-email`, `POST /api/auth/forgot-password`, `POST /api/auth/reset-password`
6. New `users` column: `email_verified_at TIMESTAMP` — null until confirmed

### Dockerized Backend Dev Environment (deferred)

- `Dockerfile.dev` for zerver/zervice + compose + postgres — useful for onboarding other devs but not needed for solo shipping

---

## Bugs

1. ~~**Layout Shift After Deck Creation**~~ — **FIXED** (2026-03-23)

   **Root cause:** 14 screens used 5 different layout patterns (`position: sticky` on header/footer + `height: 100vh` content divs). This created layouts taller than the viewport, and scroll/positioning state leaked across route changes via Dioxus DOM patching.

   **Fix:** Unified all screens under a single `.screen` fixed-frame layout (`position: fixed; inset: 0` + flexbox). Header and footer are `flex-shrink: 0` items, content fills remaining space with `flex: 1; overflow-y: auto`. No body scroll, no sticky positioning, no per-screen inline layout styles.

2. **iOS Keychain Session Persistence** — `errSecMissingEntitlement (-34018)` on cold start. `keyring` crate can't access iOS Keychain without `keychain-access-groups` entitlement + provisioning profile. User must log in on every app launch. Fix: see Infrastructure section.

3. ~~**iOS Keyboard Pushes Content Down**~~ — **FIXED** (2026-03-23)

~~4. **Swipe-up triggers page scroll on card viewing screens**~~ — **FIXED** (2026-03-27). Add/remove screens switched from `.centered` to `.card-swipe` (`overflow-y: hidden`). Vertical scroll is fully locked on these screens — no longer competes with swipe gestures. Also fixed the related card-top clip bug where `justify-content: center` was pushing overflow above the scroll origin.
