# Todo

**Primary goal: Get Zwipe publicly available on the App Store.**

---

## Server Migration (2026-03-27)

Moving from Raspberry Pi 5 to Ubuntu Server (Intel i5, 32GB RAM, x86_64). Full setup guide: `ops/server.md`.

- [ ] Disassemble desktop, remove GPU, reassemble
- [ ] Install Ubuntu Server (headless)
- [ ] PostgreSQL installed, `zwipe` DB + user created
- [ ] Create `/var/log/zwipe/` log directory (`sudo mkdir -p /var/log/zwipe && sudo chown $USER /var/log/zwipe`)
- [ ] Install Rust, clone repo, build and deploy binaries
- [ ] Configure `.env`, run SQLx migrations
- [ ] systemd unit for zerver — running and enabled
- [ ] Install cloudflared, configure tunnel to `api.zwipe.net`
- [ ] Cron entry for zervice (nightly 4am)
- [ ] Run `zervice` once manually to seed Scryfall card data
- [ ] Verify iOS app hits `api.zwipe.net` successfully

---

## App Store Submission

### 1. Fix App Name (shows "Main" on home screen)

Binary is named `main`, so iOS displays the app as "Main". Fix in `zwiper/Dioxus.toml`:
- Add `name = "Zwipe"` to the `[application]` section

### 2. iOS App Icon

Current `zwiper/assets/favicon/` icons are web favicons — iOS ignores them. App Store requires:
- **1024×1024** — App Store listing
- **180×180** (`@3x`) — iPhone home screen
- **120×120** (`@2x`) — older iPhones

Base: `android-chrome-512x512.png`. Dioxus config likely via `[bundle] icon = [...]` in `Dioxus.toml` — needs research.

### 3. zwipe.net Web Client

Minimal static site hosted on Cloudflare Pages. Full plan: `status/todo.md` web client section below.

### 4. App Store Connect Setup

1. [appstoreconnect.apple.com](https://appstoreconnect.apple.com) — `scottyfermo17@gmail.com`
2. Create app: Bundle ID `com.scadoshi.zwipe`, name "Zwipe", English
3. Fill out: description, keywords (MTG, Magic the Gathering, deck builder, commander), screenshots (6.7" iPhone required), privacy policy URL, support URL, age rating 4+, category: Games > Card Games

### 5. Build for Distribution

Current build uses Development profile. App Store requires:
- Distribution certificate (Apple Distribution)
- App Store provisioning profile
- Archive and upload via `xcrun altool` or Transporter

### 6. Submit

- Export compliance: no encryption beyond HTTPS — answer No
- Submit for review — typical 1–3 days

---

## zwipe.net Web Client

Static site on Cloudflare Pages. `zwipe.net` → Pages, `api.zwipe.net` → Cloudflare Tunnel → zerver.

**Pages needed:**
- `zwipe.net/verify?token=<hex>` — POST to `POST /api/auth/verify-email`, show success/error
- `zwipe.net/reset?token=<hex>` — new password form, POST to `POST /api/auth/reset-password`
- `zwipe.net/privacy` — privacy policy (required for App Store)
- `zwipe.net/` — everything else redirects to App Store listing

Not a full web app — just the token-handling pages, a privacy policy, and an App Store redirect. Static HTML + minimal JS.

---

## UX Polish

5. **Filter active count badges** — each filter accordion group should show active filter count (e.g. "mana (2)") so the user knows where to clear without opening every section.

6. **Filter clear empties card hand** — clearing filters on the add-cards screen should reset the `Vec<Cards>` in hand, not leave stale results.

7. **Loading indicators** — audit all screens that trigger async fetches and ensure a loading state is shown. Hard to notice locally (127.0.0.1 is instant) but visible on device hitting `api.zwipe.net`.

8. **Stop-words: move to zerver** — `ORACLE_STOP_WORDS` and `TYPE_STOP_WORDS` in `zwiper/src/lib/inbound/screens/deck/card/filter/deck_cards.rs` should be defined in zerver and shared, not duplicated. Backend uses the same stop words in queries.

9. **Password change/reset session alert** — both `change_password_and_revoke_sessions` and `reset_password` now revoke all sessions. Frontend should show a confirmation before submitting: "Changing your password will log you out on all other devices."

10. **Util bar button tap feedback** — `.util-btn` should animate on press (brief scale-down or opacity dip via `:active`). Remove any `:hover` styles — they're meaningless on touch and can leave buttons stuck after a tap on iOS.

11. **Full screen integration pass** — walk every screen on device. For each async operation add a skeleton or spinner. Add transitions between screens and loading/loaded states — nothing heavy, just enough to feel intentional.

---

## Rate Limiting (Planned)

### Password Reset
Two-layer protection:
1. **IP-level** — dedicated `tower_governor` on `POST /api/auth/forgot-password`. Target: ~5 req/hr per IP.
2. **Per-email cooldown** — already implemented (5-min window in domain logic).

### Search Cards
`GET /api/cards` is the heaviest DB operation. Add dedicated governor:
- ~1 req/3s replenishment, burst of 5 (~20/min ceiling)
- Future: key by authenticated user ID instead of IP for per-user fairness.

---

## Limits (Pre-Subscription Groundwork)

11. **Deck count limit per user** — cap decks per account in anticipation of a subscription tier that would raise the limit.

12. **Per-deck card limit** — same idea for cards per deck.

---

## Testing

- **Integration tests** — SQLx repository tests require a real PostgreSQL instance. Unit test phase complete (250+ tests). Remaining gap: outbound adapters have no coverage.
