# Todo

**Primary goal: Ship Zwipe as a webapp at zwipe.net. (iOS App Store: LIVE as of 2026-06-06. Android: next.)**

Completed work archived at `context/archive/complete-2026-Q1.md` (swept 2026-05-27).

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

## Android — Near Submission Ready

Android build compiles and runs. Remaining polish before Play Store submission:

- [ ] Card images show white corners — the white is baked into the image data from Scryfall (white-bordered card editions). iOS clips correctly via WKWebView; Android WebView does not honor `overflow: hidden` + `border-radius` on `object-fit: contain` images. Tried: `overflow: hidden` on img, wrapper div with `border-radius` + `overflow: hidden`, `-webkit-mask-image` hack. None work on Android WebView. Options: crop with `object-fit: cover` (loses card edges), mask SVG overlay, or accept as-is for black-bordered cards (majority) and revisit for white-bordered.
- [ ] Swipe gesture doesn't tilt the card — cards should rotate slightly during drag like they do on iOS
- [ ] Lock screen orientation to portrait — need `android:screenOrientation="portrait"` on main activity. Dioxus may support this via `[android.raw.manifest]` or activity-level config. Test on Pixel once available.

---

## EDHREC Integration

Closed API — must request access at edhrec.com/api. Full scope pending what they expose.

- [ ] Request API access
- [ ] Salt score import, display per card and aggregate per deck, filtering and sorting on card search
- [ ] Synergy scores — surface cards with high synergy to the deck's commander
- [ ] Popularity data — most-played cards for a given commander
- [ ] Evaluate other EDHREC data (themes, combos, etc.) once API access granted

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
- **Verify zervice cron fix** — root cause: crontab used `source /home/scadoshi/zwipe/.env`, but cron defaults to `/bin/sh` (dash on Ubuntu), where `source` is not a command. Config loading failed before tracing init, so no log file and no stderr capture (no redirect on the cron line). Fix applied 2026-05-26: added `SHELL=/bin/bash` to the crontab and appended `>> /var/log/zwipe/zervice-cron.log 2>&1` to the zervice line. **Today is 2026-05-27 — check `/var/log/zwipe/zervice.2026-05-27.log` exists and `zervice-cron.log` is clean from the 04:00 UTC run.**
