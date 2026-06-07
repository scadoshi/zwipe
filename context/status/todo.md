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

## Post-Launch Polish (Live on App Store)

Momentum work now that Zwipe TCG is live. Roughly ordered by user-visible impact.

### Marketing & Discovery
- [x] **Refresh App Store imagery** (2026-06-06) — screenshots recaptured from build 19 (1.0.1) on iPhone 11 Pro Max simulator (1242×2688px). 12 screenshots in `context/ops/ios/app-store-submission/`.
- [ ] **Update App Store icon to gruvbox** — current 1024×1024 master uses pre-gruvbox palette. Regenerate from current theme set so the home screen icon matches the in-app default.
- [ ] **Marketing posts** — Reddit (r/EDH, r/magicTCG, r/CompetitiveEDH) and X. Lead with the swipe demo video, link the App Store listing.
- [ ] **Tutorial?** — decide whether to ship a guided first-run flow or let users discover the swipe UX themselves. Lean toward letting them figure it out unless analytics later say otherwise.

### Web/Zite Polish
- [ ] **Increase `Z` logo size on zwipe.net** — current ASCII logo reads small; bump scale or font-size on the landing hero.
- [ ] **Mobile testing pass on zwipe.net** — verify landing, about, privacy, verify/reset token pages on iOS Safari + Android Chrome. Check the sticky nav and entrance animations under narrow viewports.
- [ ] **Everforest theme review** — possibly too green; sample real card art against it and consider desaturating the background or shifting the accent.

### UX Regressions (zwiper)
- [ ] **Show password eye icon** — password inputs (login, register, reset, change password) should toggle a visibility eye so users can verify what they typed. Currently masked-only.
- [ ] **Alert dialog dark overlay missing** — at some point AlertDialog/Dialog stopped rendering the dim backdrop behind the modal. Likely fallout from the same `dioxus-primitives` bump that broke toast classes (`02801f27`). Inspect the `[role="dialog"]` / `data-state` attributes the new build emits and re-style the overlay in CSS the same way toast was fixed.
- [x] **Card filter broken in deck screens** (2026-06-07) — exhaustive per-filter pass found four distinct bugs in `zwipe-core/src/domain/card/models/search_card/filter_cards.rs` (the in-deck predicate the deck cards / remove cards screens run against the loaded deck — backend SQL search unaffected). Plus a rarity sort fix.
  - **Basic types include/exclude** — `card_type_contains_any`, `card_type_contains_all`, `card_type_excludes_any` compared a lowercased `type_line` against `CardType::to_string()` (capitalized: `"Creature"`), so `"legendary creature — dragon".contains("Creature")` was `false`. Include hid everything; exclude excluded nothing. Added `.to_lowercase()` to each `ct.to_string()`.
  - **Set filter include/exclude** — predicate compared against `sd.set` (the lowercase 3-letter code, e.g. `"mh2"`), but the UI (`extract_sets`) and backend `/sets` endpoint both send `set_name` (e.g. `"Modern Horizons 2"`). Switched predicate to `sd.set_name`. Fixed stale setter doc that still said `"MH2", "ONE"`.
  - **"Is commander in <format>"** — predicate field/getter/setter/UI all existed and the backend SQL adapter honored it, but `filter_cards.rs` had **zero** references to `is_commander_in_format`. Toggle did nothing on deck screens. Added predicate using existing `commander_eligibility::is_valid_commander(card, format)` helper.
  - **"Is legal in <format>"** — same shape as commander: `legalities_contains_any` set everywhere, backend SQL honored it (`legalities->>format IN ('legal', 'restricted')` OR-joined per format), `filter_cards.rs` ignored it. Added predicate mirroring backend semantics — card passes if any chosen format is `Legal` or `Restricted`, parsing format strings via `Format::try_from`.
  - **Rarity sort alphabetical** — `OrderByOption::Rarity` compared `to_long_name()` strings, giving `Common → Mythic → Rare → Uncommon`. Derived `Ord, PartialOrd, Eq` on `Rarity` (variant declaration is already in tier order so discriminant comparison gives Common < Uncommon < Rare < Mythic < Bonus < Special). Both sort sites updated to compare enum values directly. Added a `do not reorder` comment on the enum.
- [ ] **Round-trip filter coverage test** (follow-up from the audit above) — build a `CardFilter` with every field populated and assert that both the backend SQL adapter and `filter_cards.rs::filter_by` accept/reject the same fixture set. Would catch the "field exists end-to-end except in the frontend predicate" pattern that hid the commander and legality bugs for an unknown amount of time. Currently `filter_cards.rs` mirrors backend logic by hand, with no compile-time linkage to the `CardFilter` field list.
- [ ] **Deck metrics skeleton state** — metrics block on deck view shows "possible but loading" briefly before resolving. Replace the in-between flash with a proper skeleton placeholder.

### Card Rendering Bugs
- [x] **Cards missing from search** (2026-06-06) — `Kibo, Uktabi Prince` and `Wear // Tear` were invisible to card/commander search and in-deck filtering. Two root causes, both fixed: (1) `latest_cards` materialized view's `DISTINCT ON ... ORDER BY released_at DESC` picked digital-only and promo-flagged printings whenever they were the most recent — the new ORDER BY prefers paper, non-promo, non-oversized, non-content-warning rows first. (2) `CardFilterBuilder::default()` hardcoded `promo: Some(false)`, which over-filtered Jumpstart, Secret Lair, and UB-bonus printings where many cards only exist in promo form — relaxed to `None`. Migration `20260606120000_latest_cards_prefer_real_printings.sql` also remaps existing `deck_cards.scryfall_data_id` and `decks.{commander,partner_commander,background,signature_spell}_id` references so existing decks switch to the preferred printing on deploy.
- [x] **DFC handling — render front face for transform/MDFC + add flipper for all double-faced layouts** (2026-06-06) — landed in two commits:
  - **Step 1 — front face renders**: `ScryfallData::primary_image_url(ImageSize)` in zwipe-core now falls back from top-level `image_uris` to `card_faces[0].image_uris` so transform/MDFC cards surface in search and on every image render site. Also fixed the client-side filter in `add.rs` that was dropping cards with no top-level `image_uris` (the actual reason `Delver of Secrets` never reached the renderer despite the backend returning it).
  - **Step 2 — flipper**: new `FlippableCardImage` component (`zwiper/.../components/flippable_card_image.rs`) wraps `<img>` + a "Flip" squircle button. Wraps every image surface: swipe stack (top card only, peeking cards stay plain), printing carousel + single-printing view, image preview modal. Wrapper gets `aspect-ratio: 5/7` only when flippable so the button hugs the card edge at every call site; single-faced cards keep natural sizing.
  - **Meld handling not added** — meld pieces (Urza, Lord Protector et al.) already render correctly because each piece is a separate single-faced Scryfall row. Their "back" (the melded result, e.g. Urza, Planeswalker) is a separate scryfall row, not in `card_faces`. Cross-piece flipping via `all_parts` could surface that, but it's a different feature and not in scope for the current fix.
- [ ] **Investigate Delver not showing in existing deck after MDFC fix landed** — observed 2026-06-06 on an airplane (slow wifi may be a factor): a deck containing `Delver of Secrets` returned zero rows from `SELECT users.username, decks.name, scryfall_data.name FROM users JOIN decks ... JOIN deck_cards ... JOIN scryfall_data WHERE scryfall_data.name ILIKE 'delver of secrets'`. Card and combination definitely existed before. Fresh deck testing worked fine, so probably state-specific. Could be the migration's deck-card remap pointed the old reference at a row that no longer exists, OR the connection was timing out and the join silently returned empty. Worth re-running the query off airplane wifi first; if it still returns zero, dig into the remap audit.

### Infrastructure (Reactive)
- [ ] **Home server may struggle under marketing load** — current host is a single Ubuntu box behind Cloudflare Tunnel. If real users surface latency or 5xx spikes, evaluate: Redis for session/rate-limit/search cache, or migrate zerver to a cheap VPS (Hetzner/Fly). Don't pre-optimize — only act on observed pain.

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
- **Investigate why `zervice` isn't running daily on zerver** — background sync (nightly Scryfall delta, materialized view refresh, refresh-token cleanup, session enforcement) should fire every day. Most recent log on prod is `/var/log/zwipe/zervice.2026-04-12.log` — over a month stale as of 2026-06-06. Earlier crontab fix (2026-05-26: added `SHELL=/bin/bash` and appended `>> /var/log/zwipe/zervice-cron.log 2>&1`) doesn't appear to have taken. Check `systemctl list-timers`, `journalctl -u zervice`, `crontab -l`, and confirm the `.env` is being sourced cleanly under whichever runner is configured. The card visibility fix migration includes an inline `REFRESH MATERIALIZED VIEW latest_cards`, so the view itself is fresh — but nightly Scryfall sync, expired-token cleanup, and session enforcement haven't run in weeks.
- **Pin other git deps** (optional follow-up) — `dioxus-primitives` is now pinned to rev `02801f27` (commit `b40d2019`). Audit remaining workspace deps: `grep "git = " **/Cargo.toml`. Currently no other floating git deps, but worth a periodic check.
