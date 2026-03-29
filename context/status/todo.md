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

### ✓ 1. Fix App Name (shows "Main" on home screen)

Binary is named `main`, so iOS displays the app as "Main". Fix in `zwiper/Dioxus.toml`:
- Add `name = "Zwipe"` to the `[application]` section

### 2. iOS App Icon

Current `zwiper/assets/favicon/` icons are web favicons — iOS ignores them. App Store requires:
- **1024×1024** — App Store listing
- **180×180** (`@3x`) — iPhone home screen
- **120×120** (`@2x`) — older iPhones

Base: `android-chrome-512x512.png`. Dioxus config likely via `[bundle] icon = [...]` in `Dioxus.toml` — needs research.

### 3. zwipe.net Web Client

✅ Live at https://zwipe.net — deployed via GitHub Pages (workflow: `.github/workflows/deploy-zweb.yml`). HTTPS active. See `ops/cicd.md` for deploy details.

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

✅ Live at https://zwipe.net — GitHub Pages via `.github/workflows/deploy-zweb.yml`. `api.zwipe.net` → Cloudflare Tunnel → zerver.

**Pages:**
- ✅ `/` — home/landing with ASCII logo, tagline, App Store link
- ✅ `/about` — about page
- ✅ `/contribute` — Stripe, Buy Me a Coffee, and GitHub Sponsors links
- ✅ `/privacy` — privacy policy
- ✅ `/verify/:token` — POST to `POST /api/auth/verify-email`, shows success/error (token in path segment, not query param — Dioxus Router strips query params on SPA init)
- ✅ `/reset/:token` — new password form, POST to `POST /api/auth/reset-password`

Token links in emails use path segments: `https://zwipe.net/verify/{token}` and `https://zwipe.net/reset/{token}`. SPA routing handled by `404.html` (copy of `index.html`) in the deploy workflow.

---

## UX Polish

10. **Full screen integration pass** — walk every screen on device.

---

## Rate Limiting

### Password Reset (Partial)
- ✅ Forgot password (`POST /api/auth/forgot-password`) — IP-level governor, ~5 req/hr
- ✅ Reset password (`POST /api/auth/reset-password`) — IP-level governor
- ✅ Change password (authenticated, `PUT /api/user/change-password`) — burst 2, then 1 req/30min
- ✅ Change username (authenticated, `PUT /api/user/change-username`) — burst 2, then 1 req/30min
- ✅ Change email (authenticated, `PUT /api/user/change-email`) — burst 2, then 1 req/30min

### Search Cards
- ✅ `POST /api/card/search` — burst 5, then 1 req/10s (100-card batches make higher rate unrealistic)
- Future: key by authenticated user ID instead of IP for per-user fairness.

---

## Limits (Pre-Subscription Groundwork)

11. ✅ **Deck count limit per user** — 20 decks max. Enforced in service layer on `create_deck_profile`.

12. ✅ **Per-deck card limit** — 250 cards (sum of quantities) max. Enforced on `create_deck_card` and `import_deck_cards`. Constants in `domain/deck/mod.rs`.

---

## Binary Versioning

Add a version string to `zerver` and `zervice` that prints on startup — makes it immediately obvious after a manual or CI deploy whether the new binary is live.

- Add `version` to workspace `Cargo.toml` (e.g. `0.1.0`)
- Print version on startup: `zerver v0.1.0 starting...` / `zervice v0.1.0 starting...` using `env!("CARGO_PKG_VERSION")`
- Scrape codebase for all hardcoded version strings (e.g. `"0.1.0"`, `version:`) and replace with `env!("CARGO_PKG_VERSION")`
- Expose on health endpoint: `GET /` already returns a `version` field — verify it uses `CARGO_PKG_VERSION` and not a hardcoded string

---

## Project Structure Doc

Add a `context/architecture/structure.md` walking through the full directory tree with explanations — useful for onboarding and AI context.

---

## CI/CD

✅ **zerver/zervice**: GitHub Actions workflow on push → builds release binaries → SSHes into server → restarts `zerver` systemd service. See `ops/cicd.md`.

✅ **zweb**: GitHub Actions workflow on push (when `zweb/**` changes) → `dx build --release --platform web` → deploys to GitHub Pages at `zwipe.net`. See `ops/cicd.md`.

---

## Donate Button

✅ Done. Contribute page live at `zwipe.net/contribute` with Stripe Payment Link, Buy Me a Coffee, and GitHub Sponsors. FUNDING.yml in repo adds Sponsor button to GitHub repo. GitHub Sponsors application pending approval.

---

## Extract Password Validation into a Shared Crate

`zweb/src/pages/reset.rs` currently duplicates the password policy rules from
`zerver/src/lib/domain/auth/models/password/mod.rs`. Both must be updated together
if the policy changes.

The right fix is a new minimal crate (e.g. `zwipe-domain` or `zwipe-validation`) that:
- Lives at the workspace root alongside `zerver`/`zwiper`
- Has no heavy deps (no reqwest, no axum, no argon2)
- Compiles cleanly for both native (`zerver`, `zwiper`) and WASM (`zweb`)
- Exports `Password::new()` and the `InvalidPassword` error type
- Can optionally be published to crates.io so `zweb` can depend on it without
  being added to the main workspace

The `zerver` crate would then depend on this crate instead of owning the type.

For now the rules are duplicated in `zweb` with a comment pointing at the source.

---

## Database Backups

The server runs PostgreSQL locally. If the server dies, the database goes with it. Need a
periodic backup job that gets data off the machine.

Things to figure out:
- **Backup tool** — `pg_dump` is the standard; outputs a SQL file or custom format
- **Schedule** — nightly cron (similar to zervice)
- **Destination** — off-machine storage: S3/R2, Backblaze B2, or a remote SSH target
- **Retention** — how many days/weeks to keep
- **Restore runbook** — document how to restore from a backup on a fresh server

Rough sketch:
```bash
# Example cron: nightly pg_dump to a compressed file, upload to cloud storage
pg_dump -U zwipe zwipe | gzip > ~/backups/zwipe-$(date +%Y%m%d).sql.gz
# then rclone/aws-cli/restic to upload offsite
```

Related: if a self-hosted GitHub Actions runner is set up, the runner workspace on the
server contains the full repo source — that's already in GitHub. The database is the only
critical stateful data that isn't replicated elsewhere.

---

## Testing

- **Integration tests** — SQLx repository tests require a real PostgreSQL instance. Unit test phase complete (250+ tests). Remaining gap: outbound adapters have no coverage.
