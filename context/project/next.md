# Next Immediate Priorities 🎯

Planned work after completing current tasks.

---

## User notes about minor tweaks

~~1. Silent-omit documentation on `find_cards_by_exact_names`~~ — **DONE** (2026-03-25). `ports.rs` now documents that missing names are silently omitted (no error). Rename deferred/dropped.

~~2. Accordion scroll-to-focus~~ — **DONE** (2026-03-25). Each `AccordionItem` in the add/view/remove filter bottom sheets fires `on_change` and calls `document::eval` with a 50ms-deferred `scrollIntoView({ behavior: 'smooth', block: 'start' })` targeting the opened item. Delay prevents phantom-open touch events and lets layout settle.

~~3. Config filter labels: add "is" or "has" prefix to boolean fields so they read naturally — "is playable", "is digital only", "is oversized", "is promo", "has content warning".~~

4. ORACLE_STOP_WORDS and TYPE_STOP_WORDS in zwiper/src/lib/inbound/screens/deck/card/filter/deck_cards.rs should be maintained by the zerver lib and passed to the frontend. Generally domain models or business logic should be defined there and then utilized by the frontend rather than built and maintained in the frontend. This is especially true since the backend uses the very same stop words in its queries. We should define shared logic and then use that shared logic in both places so we don't have to maintain the content in two places!

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

## Infrastructure & Shipping

### iOS Session Persistence (Keychain Entitlement)

**Bug:** `Platform secure storage failure: A required entitlement isn't present` on every cold start. The `keyring` crate targets the iOS Keychain, which requires the `keychain-access-groups` entitlement in the app's provisioning profile. Without it, `infallible_load()` silently returns `None` — user must log in every launch.

**Status:** Apple Developer subscription purchased 2026-03-25, payment processing (up to 48h). `Dioxus.toml` already updated with `identifier = "com.scottyrayfermo.zwipe"`. Wait for portal access before doing anything in Xcode — no App ID exists yet to sign against.

**Next steps once payment clears:**
1. developer.apple.com → Identifiers → + → App IDs → App
2. Description: `zwipe`, Bundle ID (Explicit): `com.scottyrayfermo.zwipe`, enable **Keychain Sharing** → Register
3. In `zwiper/`: `dx build --platform ios` → generates `gen/apple/` Xcode project
4. `open gen/apple/zwiper.xcodeproj`
5. Target → Signing & Capabilities → **Automatically manage signing**, set Team
6. **+ Capability** → Keychain Sharing
7. Run — `errSecMissingEntitlement` gone, sessions persist across launches

### Backend Hosting

**Recommended stack:**
- **VPS**: DigitalOcean Droplet ($6–12/mo, 1–2GB RAM; Rust binary is tiny)
- **Database**: PostgreSQL on the same VPS (or DigitalOcean Managed DB for reliability)
- **TLS/proxy**: Caddy — one-line HTTPS with automatic Let's Encrypt cert renewal
- **Process manager**: `systemd` service for `zerver`; `systemd.timer` (or cron) for daily `zervice` Scryfall sync

**Key config changes when moving to prod:**
- `BACKEND_URL` in `zwiper/.env` → `https://your-domain.com`
- `ALLOWED_ORIGINS` in `zerver/.env` → iOS native clients don't send a browser Origin header, but set this to `https://your-domain.com` for web clients
- iOS **requires HTTPS** (App Transport Security blocks plain HTTP) — Caddy handles this automatically
- Refresh token cleanup: `zervice` or a cron job should prune expired refresh tokens from the DB periodically

**Rough deployment steps:**
1. Provision VPS, install PostgreSQL + Caddy
2. `cargo build --release --bin zerver` → copy binary to server
3. Run migrations (`sqlx migrate run`)
4. Write `systemd` unit for zerver, enable + start
5. Configure Caddy reverse proxy to `127.0.0.1:3000`
6. Set DNS A record → VPS IP, Caddy auto-provisions cert
7. Schedule `zervice` via cron (`0 4 * * * /path/to/zervice`) for nightly Scryfall sync

### Dockerized Backend Dev Environment (deferred)

- `Dockerfile.dev` for zerver/zervice + compose + postgres — useful for onboarding other devs but not needed for solo shipping

---

## Bugs

1. ~~**Layout Shift After Deck Creation**~~ — **FIXED** (2026-03-23)

   **Root cause:** 14 screens used 5 different layout patterns (`position: sticky` on header/footer + `height: 100vh` content divs). This created layouts taller than the viewport, and scroll/positioning state leaked across route changes via Dioxus DOM patching.

   **Fix:** Unified all screens under a single `.screen` fixed-frame layout (`position: fixed; inset: 0` + flexbox). Header and footer are `flex-shrink: 0` items, content fills remaining space with `flex: 1; overflow-y: auto`. No body scroll, no sticky positioning, no per-screen inline layout styles.

2. **iOS Keychain Session Persistence** — `errSecMissingEntitlement (-34018)` on cold start. `keyring` crate can't access iOS Keychain without `keychain-access-groups` entitlement + provisioning profile. User must log in on every app launch. Fix: see Infrastructure section.

3. ~~**iOS Keyboard Pushes Content Down**~~ — **FIXED** (2026-03-23)

   **Root cause:** Same as above — `sticky top-0` + `justify-content: center` + `h-screen` caused layout reflow when iOS keyboard changed the viewport height.

   **Fix:** `position: fixed` on `.screen` is immune to viewport resize from keyboard. Safe-area insets moved from `body` to `.screen` via `env(safe-area-inset-top/bottom)`.
