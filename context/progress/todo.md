# Todo

**Primary goal: grow the user base — production launch + marketing + tester-feedback intake.** (iOS App Store: LIVE. Android: closed testing LIVE ~400 testers, production access next. Full webapp at zwipe.net: in progress.)

Only open, actionable items live here. When something ships, its outcome moves to
[`overview.md`](overview.md) and leaves this list. Older completed work is archived
at `context/archive/complete_2026_q1.md`.

---

## Next Up

- [ ] **Read the anonymous funnel once data accrues.** Anonymous funnel metrics (app-open / register-viewed / register-submitted) have shipped in prod since 1.3.1. When enough sessions accrue, read the funnel with `zcripts` (distinct sessions per kind vs. `user_events.register`) — these numbers gate the sign-in-with-Google decision.
- [ ] **Privacy follow-ups for per-user collection.** The policy text shipped 2026-07-02 (`b1ee1b11`, discloses per-account activity + deck skip memory). Remaining owner passes: update the App Store privacy "nutrition label" + Play data-safety form to reflect per-account analytics, and send the policy-change notification email to users.
- [ ] **Suggestion signal — Phase 3c (pair-level ranking).** Phases 3a+3b **shipped 2026-07-06** (server 1.3.2): the default synergy ordering now blends base score + pooled net-rate (`added + 0.5·maybed − removed`, shrunk/centered) + per-(deck, day) seeded jitter — different decks serve differently, the same deck stays stable within a day, and crowd favorites drift up as signal accrues. Remaining: the commander-specific pair-level term, gated on pair-depth (baseline 2026-07-06: 0 pairs ≥20 impressions — re-run the readiness queries after the user base grows). Plan: [`../plans/suggestion_signal.md`](../plans/suggestion_signal.md).

**The next big two (of the original three set 2026-06-11; Android clock is now DONE):**
1. **Marketing — get users.** Business cards for LGSs, Reddit/X posts (see Marketing & Discovery below).
2. **VPS — stabilize.** Cutover is done (see `overview.md`); remaining follow-ups are the two items below.

- [ ] **Verify the VPS crons fired.** First unattended run was 2026-06-14 (zervice 4am + backup 5am UTC). Check: `ssh root@100.114.251.8` (or `zerver` alias), then `tail /var/log/zwipe/zervice-cron.log /var/log/zwipe/backup.log` and `rclone lsl r2:zwipe-backups/ | tail -3` — a recent dated dump should be present. If the zervice log is empty, recheck the `SHELL=/bin/bash` line (the dash-vs-bash trap from `operations/infrastructure/server.md`).
- [ ] **Repurpose home box + rotate R2 keys — after ~1–2 clean VPS weeks.** Home is powered off but intact as the rollback (boot → flip `api.zwipe.net` CNAME to home tunnel `70ba169b-…` → `systemctl start zerver`). Once the VPS has a clean run, rebuild home as a gaming box (a different Linux distro) — that reinstall wipes the old prod secrets on its disk. Then rotate the still-shared R2 keys (JWT/DB/Resend are already fresh on the VPS, so only R2 carries over from home). Closing the rollback window = remove old home tunnel `70ba169b-…` + old Tailscale device.

---

## Bugs (owner-reported 2026-07-11)

- [ ] **Printing shifting bug** — [zwiper] visual layout shift when a card's printing changes. Owner-observed; needs repro + which surface (card detail / printing picker) and whether it's an image-size / reflow issue.
- [x] **Flip-card down-shift — FIXED 2026-07-11** (`02aab440`). Wasn't the button padding; the DFC-only `aspect-ratio: 5/7` wrapper rule made double-faced cards render a few px smaller/lower than single-faced ones. Now the button pins to an image-sized `.flip-face` (absolute, top-right) and DFC/single-face size identically. Sim-verified.

---

## Features — queued (owner 2026-07-11)

- [ ] **Maybeboard for commander select** — let users "maybe" a commander during Zwipe-select and revisit a list of maybed commanders (mirror of the deck maybeboard; would tie into the commander-select signal already collected).
- [ ] **Metric capture for non-commander decks** — the suggestion/usage signal is commander-keyed today (`commander_card_signal` keyed by `commander_oracle_id`); non-commander formats (Standard/Modern/etc.) have no commander to key on, so their building generates no signal. Design a capture path for them.
- [ ] **Deck folders** — let users organize the deck list into folders/groups.
- [ ] **Oracle tags (otags) — HORIZON, big.** Ingest Scryfall's community-maintained functional tags (hundreds; daily `zervice` sync → `card_otags`), let players select strategy otags per deck (reconciled with deck tags), show the distribution, and use them as a new algorithmic serving axis (commander + otags, MVP otags, non-EDH formats via color-identity + otags + swipe data). Community-accurate replacement/complement for our heuristic `mechanical_categories`. Full vision + open research questions in [`../plans/otags.md`](../plans/otags.md).

---

## Android — production access next

Decision (2026-06-10): grow the user base before monetizing — Android ships
before the premium tier. Closed-testing clock **complete 2026-07-09** (~400 active
testers via a hired service + organic, past the 12-tester / 14-continuous-day gate).
`com.scadoshi.zwipe` live to testers across 176 countries. Build pipeline + gotchas
in `operations/android/play-store/submission/`.

- [~] **Android production launch — SUBMITTED FOR REVIEW 2026-07-11** (Submission 21,
  "Production", **all countries** selected; status: In review). 14-day closed-testing
  cycle confirmed complete 2026-07-09 (QA partner Teekam Suthar / 12testers);
  questionnaire answers in [`../operations/android/play-store/submission/production_access.md`](../operations/android/play-store/submission/production_access.md).
  Gotcha hit: the **Production track needs its own country list** (separate from closed
  testing's 176) — set via Test and release → Production → Countries/regions, not the
  release page or the bundle. Now: **wait for Google review**, then it goes live on Play.
  Once live, do the website link + announcement item below.
- [ ] **Intake tester feedback → `feature_requests.md`.** ~400 testers + hired
  testers are generating suggestions; triage them into the weighted request queue
  ([`feature_requests.md`](feature_requests.md)) and surface anything actionable
  into this list. This is the near-term driver now that the launch gate is cleared.
- [ ] **Android store announcement + website link** (owner 2026-07-11) — once Android is live on Play: announce it across the marketing waves, and add a Play Store download link/button on zwipe.net (zite has `/download/ios` today — add the Android equivalent). Gated on production launch above.

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

## Marketing & Discovery

- [ ] **Marketing posts — keep the cadence going.**
  - **Wave 1 — 2026-06-25 (Day 333):** build-in-public text posts to X, Reddit, Bluesky (1.1.0 features).
  - **Wave 2 — 2026-06-28 (Day 336): the swipe-demo video.** A vertical 9:16 screen-recording (Tinder-for-Magic hook: swipe a commander → build by swiping → filter → import → finished deck), captions + an end card built from `context/marketing/video_end_card.html`. Posted natively to **Instagram** (new `@scadoshi`), **YouTube Shorts**, **X**, **Bluesky**, and **Reddit r/EDH** (video + pinned dev comment, feedback-framed). **TikTok pending** — the old 8-yr personal account was compromised (password breach; rogue sessions cleared, no third-party apps, recovering via phone+email, needs new pw + 2FA before use); lean toward a fresh branded handle. Post-copy templates were drafted then deleted (kept out of the repo).
  - r/EDH and r/magicTCG *posts* are karma/age-gated (comment first, build toward posting); reuse the r/EDH title + pinned comment for r/custommagic, r/SideProject, r/buildinpublic.
- [ ] **Business cards for LGSs** (noted 2026-06-10, designed 2026-06-12; **PRINTED + distributing as of 2026-06-28** — cards out on local bulletin boards + 1 LGS so far, working on store-owner relationships; ongoing) — self-contained HTML/CSS mockup at `context/marketing/business_card.html` (committed `cebe5d91`). It's a real MTG-style **creature card** (not a token): 2.5×3.5in, gruvbox by default with a dark-theme picker, front + back shown together. Front = title bar with WUBRG mana cost (real mana-font symbols, toggle for coded `{W}` text), theme-tinted **QR** (defaults to zwipe.net; App Store option) as the art, type line `Artifact Creature — Mobile App` with the pixel-Z as the set symbol, oracle text with keyworded features (**ETB swipe** left-red/right-green, **Synergy**, **Import**, **Free** — "Free forever. No ads.", deliberately no "subscriptions" so premium stays open), `*/*` P/T, selectable flavor (default banger: "Every great deck begins with a single swipe."). Back = full Zwipe wordmark + `zwipe.net`, nothing else. Oracle text auto-fits its box. Flat (no gradients). **Next: keep distributing — more LGSs + bulletin boards, build store-owner relationships** (see printing notes below). Ask store owners before leaving stacks; commander-night crowds are the target.
  - **Printing notes**: standard playing-card stock is **2.5×3.5in (63×88mm)**. For a true card feel use a card printer (MakePlayingCards/MPC, PrinterStudio, Ad Magic, or a local print shop with 300–350gsm + matte/linen finish) — order a custom card or "custom card game" SKU. For cheap-and-fast, a local shop can do business-card stock but it's thinner. Export: open the HTML, print to PDF at 100% (no scaling) with "background graphics" ON; add 1/8in bleed if the printer wants it. QR scannability holds down to ~0.8in — current QR is well above that.

---

## Monetization

- [ ] **TCGplayer affiliate** — application submitted 2026-06-23, **In Review** on Impact (impact.com). When approved: wire the tracking ID into `tcgplayer_url()` (`zwiper/.../outbound/buy_links.rs`) — zero UI change — then add per-card **"Buy ↗"** links (currently buy is whole-deck only).
- [ ] **Card Kingdom affiliate** — **no public self-serve program**; it's a direct-outreach partnership. Email CK when ready (cite the Archidekt `?partner=` precedent). `cardkingdom_url()` stays untracked until then.
- Detail + saved signup copy: `context/product/affiliate/tcgplayer.md`.

---

## Web/Zite Polish

- [ ] **Guides SEO leftovers.** The guides knowledge base shipped (12 guides live under `/guides`) and the sitemap + per-guide `Article` JSON-LD landed 2026-07-08. Still unshipped: demand-first MTG-topic guides ("best mobile MTG deck builder", "how to build a Commander deck on your phone") that ride search volume rather than app screens. (SEO-guides plan archived at [`../archive/seo_guides.md`](../archive/seo_guides.md).)
- [ ] **Mobile testing pass on zwipe.net** — verify landing, about, privacy, verify/reset token pages on iOS Safari + Android Chrome. Check the sticky nav and entrance animations under narrow viewports.

---

## Synergy & Popularity Data

The cache-first synergy layer shipped (see `overview.md`); these are the consumers built on top of it.

- [ ] Synergy scores — surface cards with high synergy to the deck's commander
- [ ] Popularity data — most-played cards for a given commander
- [ ] Salt score, display per card and aggregate per deck, filtering and sorting on card search
- [ ] Evaluate further data (themes, combos, etc.) as the layer matures

---

## Mechanical Category — Heuristic Refinement

Phases 1+2 shipped (see archive). ~73% classification rate today; refinement targets are below. Layers 2+3 (AI classifier + fine-tuned model) tracked in `backlog.md`.

**Testing approach (owner idea 2026-07-09):** assemble a set of known special-case / edge cards, run each through the heuristic classifier, and have a **cheap model (Haiku)** grade whether the classification is right. Where Haiku flags a miss, **escalate that card to a stronger model** to propose the heuristic/regex fix. Turns "audit a sample by hand" into a semi-automated find-and-fix loop; do this before hand-tuning the individual patterns below.

- [ ] Add more test cases for edge cases and false positives/negatives
- [ ] Audit a sample of classified cards per category to find misclassifications
- [ ] Lands should NOT be classified as ramp (fixed: removed `type_line.contains("land")` from ramp fallback) — verify still holds
- [ ] Tune regex proximity windows (e.g. blink regex was too narrow, widened to 80 chars)
- [ ] Consider additional ramp patterns (e.g. treasure token creators, rituals like Dark Ritual)
- [ ] Consider additional removal patterns (e.g. "exile target" with qualifiers, fight mechanics)
- [ ] Burn heuristic excludes creatures — should it include creatures with ETB damage?
- [ ] Stax heuristic may false-positive on cards that say "can't" in reminder text

---

## Maintenance

- [ ] **Turn on Dependabot alerts** (repo Settings → Code security → Dependabot alerts) — the passive GitHub-Advisory backstop to the active weekly `cargo audit` workflow (`audit.yml`). Zero code, zero noise; optionally enable "Dependabot security updates" for auto-fix PRs, but skip *version* updates (the noisy weekly-bump firehose). Owner-only (a settings toggle, not a file).
- **sqlx 0.8 → 0.9** — major bump. 0.9 has breaking changes around type mappings and connection options. Needs a dedicated branch where the integration tests run against a real Postgres before merge (the suite now exists to gate it — `zerver/tests/`).
- **keyring 3 → 4** (zwiper) — major bump. Used for iOS Keychain on `apple-native`. Needs on-device test before merging; don't ship blind.
- **Pin other git deps** (optional follow-up) — `dioxus-primitives` is now pinned to rev `02801f27` (commit `b40d2019`). Audit remaining workspace deps: `grep "git = " **/Cargo.toml`. Currently no other floating git deps, but worth a periodic check.
