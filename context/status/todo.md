# Todo

**Primary goal: Get Zwipe publicly available on the App Store.**

---

## Server Migration (2026-03-27)

Moving from Raspberry Pi 5 to Ubuntu Server (Intel i5, 32GB RAM, x86_64). Full setup guide: `ops/server.md`.

- [x] Disassemble desktop, remove GPU, reassemble
- [x] Install Ubuntu Server (headless)
- [x] PostgreSQL installed, `zwipe` DB + user created
- [x] Create `/var/log/zwipe/` log directory (`sudo mkdir -p /var/log/zwipe && sudo chown $USER /var/log/zwipe`)
- [x] Install Rust, clone repo, build and deploy binaries
- [x] Configure `.env`, run SQLx migrations
- [x] systemd unit for zerver — running and enabled
- [x] Install cloudflared, configure tunnel to `api.zwipe.net`
- [x] Cron entry for zervice (nightly 4am)
- [x] Run `zervice` once manually to seed Scryfall card data
- [x] Self-hosted GitHub Actions runner installed, registered, running as systemd service
- [x] CI/CD pipeline live — push to main deploys zerver automatically
- [ ] Verify iOS app hits `api.zwipe.net` successfully

---

## App Store Submission

### 1. Fix App Name (shows "Main" on home screen)

App still shows as "Main" on the iOS home screen after deploy. Needs investigation.

Known starting point from earlier research:
- Binary is named `main`, so iOS reads that as the display name
- Attempted fix: add `name = "Zwipe"` to `[application]` section in `zwiper/Dioxus.toml`
- **Status: not confirmed working** — still shows "Main" on device as of 2026-03-29

#### Investigation needed

1. Check `zwiper/Dioxus.toml` — does `[application] name = "Zwipe"` exist?
2. If not, add it and rebuild
3. If it does exist, the fix may need to go in `Info.plist` instead:
   - `CFBundleDisplayName` = `Zwipe`
   - `CFBundleName` = `Zwipe`
   - Dioxus generates `Info.plist` at build time — check if it respects the `Dioxus.toml` name field or if it needs to be set elsewhere
4. Check if `dx build` output contains an `Info.plist` at
   `target/dx/main/debug/ios/Main.app/Info.plist` and inspect `CFBundleDisplayName`

### 2. Account Deletion (App Store Required)

Apple guideline 5.1.1 **requires** apps with account creation to offer in-app account
deletion. This is a hard blocker for App Store approval.

#### Backend — `DELETE /api/user`

New authenticated endpoint that:
1. Verifies the JWT (same as all other authenticated routes)
2. Deletes all of the user's deck cards (cascade likely handles this via FK)
3. Deletes all of the user's decks
4. Deletes all of the user's refresh tokens
5. Deletes the user record itself
6. Returns `200 OK`

Rate limit: low burst, long refill (same pattern as change-password) to prevent abuse.

#### Frontend (zwiper)

- Add "Delete Account" button to the profile screen (bottom of util-bar or a dedicated
  danger zone section)
- Require a confirmation dialog: "This will permanently delete your account and all
  decks. This cannot be undone."
- On confirm: call `DELETE /api/user`, clear the local session, navigate to login screen
- Button should be visually distinct — red or muted, not the same style as normal actions

#### Notes

- No email confirmation step required, but the confirmation dialog is essential UX
- Apple reviewers will specifically look for this — it must be discoverable in the app,
  not buried or hidden
- Data deletion must be immediate (or near-immediate), not "submitted for deletion"

---

### 3. iOS App Icon

Current `zwiper/assets/favicon/` icons are web favicons — iOS ignores them.

#### Required sizes

| Size | Usage |
|------|-------|
| **1024×1024** | App Store listing (required, no alpha channel) |
| **180×180** | iPhone home screen @3x (iPhone 6 Plus and newer) |
| **120×120** | iPhone home screen @2x (older iPhones) |
| **87×87** | Spotlight @3x |
| **80×80** | Spotlight @2x |
| **60×60** | Notification @3x |
| **40×40** | Notification @2x |

The 1024×1024 is the hard requirement — App Store Connect will reject the build without
it. The others improve the appearance on-device but missing them won't block submission.

#### How to produce them

Start from a single high-resolution master (at least 1024×1024, ideally vector/SVG):
- **Figma** (free) — design at 1024×1024, export all sizes at once
- **Sketch** / **Affinity Designer** — same approach
- **makeappicon.com** — upload a 1024×1024 PNG, download a zip with all sizes
- **ImageMagick** (CLI) — `convert master.png -resize 180x180 icon-180.png`

#### Dioxus config

Add to `zwiper/Dioxus.toml` under `[bundle]`:
```toml
[bundle]
icon = ["assets/icons/icon-1024.png"]
```

Dioxus/Tauri will handle resizing for the other slots from the single source image.
Confirm exact config key by checking the Dioxus mobile docs — this may have changed
between versions.

#### Current base asset

`zwiper/assets/favicon/android-chrome-512x512.png` is the closest existing asset — only
512×512 which is borderline. Better to create a clean 1024×1024 master.

### 4. zwipe.net Web Client

✅ Live at https://zwipe.net — deployed via GitHub Pages (workflow: `.github/workflows/deploy-zweb.yml`). HTTPS active. See `ops/cicd.md` for deploy details.

### 5. App Store Connect Setup

1. [appstoreconnect.apple.com](https://appstoreconnect.apple.com) — sign in with Apple ID
2. Create app: Bundle ID `com.scadoshi.zwipe`, name "Zwipe", English
3. Fill out: description, keywords (MTG, Magic the Gathering, deck builder, commander), screenshots (6.7" iPhone required), privacy policy URL, support URL, age rating 4+, category: Games > Card Games

### 6. Build for Distribution

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

✅ **zerver/zervice**: GitHub Actions workflow on push → builds release binaries → stops zerver, copies binaries, starts zerver via self-hosted runner. See `ops/cicd.md`.

✅ **zweb**: GitHub Actions workflow on push (when `zweb/**` changes) → `dx build --release --platform web` → deploys to GitHub Pages at `zwipe.net`. See `ops/cicd.md`.

---

## GitHub Actions Node.js 20 Deprecation

Actions running on Node.js 20 will be **forced to Node.js 24** starting **June 2, 2026**.
Node.js 20 will be **removed from runners entirely** on **September 16, 2026**.

The following actions in `.github/workflows/deploy-zerver.yml` are affected:
- `actions/checkout@v4`
- `actions/cache@v4`

### What to do

Before June 2, 2026, bump both to a version that ships with Node.js 24 support.
Check the release notes for each action — newer patch/minor releases of `@v4` are
expected to add Node 24 support before the deadline.

```yaml
# In .github/workflows/deploy-zerver.yml:
- uses: actions/checkout@v4        # bump to latest @v4 patch when Node 24 lands
- uses: actions/cache@v4           # same
```

To opt in early (test Node 24 compatibility now), add this env var to the workflow or
runner:
```yaml
env:
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true
```

`deploy-zweb.yml` uses the same actions and will need the same update.

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
