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

10. **Full screen integration pass** — walk every screen on device. For each async operation add a skeleton or spinner. Add transitions between screens and loading/loaded states — nothing heavy, just enough to feel intentional.

---

## Rate Limiting

### Password Reset (Partial)
- ✅ Forgot password (`POST /api/auth/forgot-password`) — IP-level governor, ~5 req/hr
- ✅ Reset password (`POST /api/auth/reset-password`) — IP-level governor
- ❌ Change password (authenticated, `PUT /api/user/change-password`) — NEEDS rate limiting
- ❌ Change email (authenticated, `PUT /api/user/change-email`) — NEEDS rate limiting

### Search Cards (Outstanding)
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
