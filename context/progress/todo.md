# Todo

**Primary goal: grow the user base — marketing + tester-feedback intake.** (iOS App Store: LIVE. Android Play Store: LIVE (production). Full webapp at zwipe.net: in progress.)

Only open, actionable items live here. When something ships, its outcome moves to
[`overview.md`](overview.md) and leaves this list. Older completed work is archived
at `context/archive/complete_2026_q1.md`.

---

## Next Up

- [ ] **1.7.0 adoption cleanup (gated).** 1.7.0 has been **live since ~2026-07-15**, with **1.7.1** (live) and **1.7.2** (submitted for review 2026-07-20) shipped on top — all carry the `deck_id`-only Phase 5S signal. Once the `<1.7.0` clients have drained, floor `MIN_CLIENT_VERSION` to 1.7.0 → unlocks the **Phase 5S step-3 cleanup** (drop the legacy `commander_oracle_id` wire + server fallback + client's internal commander resolution; bump). Then **Phase 6** serving (data-gated).
- [ ] **Read the anonymous funnel once data accrues.** Anonymous funnel metrics (app-open / register-viewed / register-submitted) have shipped in prod since 1.3.1. When enough sessions accrue, read the funnel with `zcripts` (distinct sessions per kind vs. `user_events.register`) — these numbers gate the sign-in-with-Google decision.
- [ ] **Privacy follow-ups for per-user collection.** The policy text shipped 2026-07-02 (`b1ee1b11`, discloses per-account activity + deck skip memory). Remaining owner passes: update the App Store privacy "nutrition label" + Play data-safety form to reflect per-account analytics, and send the policy-change notification email to users.
- [ ] **Suggestion signal — Phase 3c (pair-level ranking).** Phases 3a+3b **shipped 2026-07-06** (server 1.3.2): the default synergy ordering now blends base score + pooled net-rate (`added + 0.5·maybed − removed`, shrunk/centered) + per-(deck, day) seeded jitter — different decks serve differently, the same deck stays stable within a day, and crowd favorites drift up as signal accrues. Remaining: the commander-specific pair-level term, gated on pair-depth (baseline 2026-07-06: 0 pairs ≥20 impressions — re-run the readiness queries after the user base grows). Plan: [`../plans/archive/suggestion_signal.md`](../plans/archive/suggestion_signal.md).

---

## Bugs

- [ ] **Deck share screen: mana value groups order weirdly (investigate).** (owner, 2026-07-22) On the zwipe.net deck share screen, the mana-value groupings render out of order (e.g. 1 → 4 → 5 → 3 instead of ascending). Sort the groups numerically by mana value.
- [ ] **Filter bottom-sheet Cancel doesn't revert (investigate).** (owner, 2026-07-21) The filter sheet's **Cancel** should discard any changes made while the sheet was open and restore the filter to its state on open — but it commits instead. Repro: open the filter, tap **Reset filter**, then tap **Cancel** → the filter stays reset rather than reverting. Cancel needs to snapshot the filter state on open and restore it on cancel; the backdrop-tap-to-close path should behave the same. Filter sheet is in zwiper (the Cancel button was added 2026-07 this session).

Recently resolved (outcomes in [`overview.md`](overview.md)):
the **pre-1.6.0 "connection error" wire break** (fixed by flooring `MIN_CLIENT_VERSION=1.6.0`,
2026-07-13; root cause fully removed 2026-07-14 when the Phase M sunset dropped the
`mechanical_categories` dual-emit), and **app version in session data** (shipped `ce8abcad`,
recorded per-session on the refresh-token row).

Completed fixes are archived to
[`archive/complete_2026_q3.md`](../archive/complete_2026_q3.md) (hashes stay searchable there).

---

## Features — queued (owner 2026-07-11)

- [ ] **Commander shortlist / dedicated commander-swiping area** — "save for later" while swiping commanders. **Feature request** ([`../plans/commander_shortlist.md`](../plans/commander_shortlist.md)): recommend a dedicated Commanders browse space with a per-user shortlist + "start a deck with this," decoupled from the deck-creation picker (kills the "where did it go?" of an in-flow up-swipe). Open decisions: storage (server vs local), placement, commander scope. Not specced.
- [ ] **Deck folders** — let users organize the deck list into folders/groups. **Spec'd** ([`../plans/deck_folders.md`](../plans/deck_folders.md)): custom user-named folders, one per deck, collapsible grouped list; `FolderName` reuses `DeckName` validation; ~1–1.5 days (client UI is the bulk, backend is mechanical). Not started.
- [ ] **Oracle tags (otags) — HORIZON, big.** Ingest Scryfall's community-maintained functional tags (hundreds; daily `zervice` sync → `card_otags`), let players select strategy otags per deck (reconciled with deck tags), show the distribution, and use them as a new algorithmic serving axis (commander + otags, MVP otags, non-EDH formats via color-identity + otags + swipe data). Community-accurate replacement/complement for our heuristic `mechanical_categories`. Full vision + open research questions in [`../plans/otags.md`](../plans/otags.md).
- [ ] **Otag-selector search over descriptions (queued, small).** From Discord feedback: make the deck otag **selector** search over *descriptions* too (today slug + label). (The companion "Dictionary link on the card-filter otag section" shipped in 1.7.1.)
- [ ] **Deck share screen: add charts (investigate).** (owner, 2026-07-22) Bring the deck stats charts (distributions etc.) onto the zwipe.net deck share screen to make it more capable. Not specced.
- [ ] **Deck stats: average power/toughness (small).** (owner, 2026-07-22) Add the deck's average power and toughness to the "Distributions" deck stats chart.
- [ ] **Mana pip-count filter (investigate).** (owner idea, Discord, 2026-07-21) Let players filter the card pool by the exact count of colored pips of a given color in the mana cost — e.g. "has 2 blue pips and 1 red pip." Per-color pip counts are derivable from `mana_cost`. Open: UI (per-color count steppers? which colors shown?), match semantics (exact vs min/max), and whether it stacks with the existing color/mana filters. Not specced.
- [ ] **Oracle-tag descriptions — bulk authoring (ongoing, ~1,100 / 4,500 done).** Mechanism SHIPPED 2026-07-13 (`0114cb38`): `zervice` overlays our `ORACLE_TAG_DESCRIPTIONS` const (`zerver/.../helpers/oracle_tag_descriptions.rs`) into `oracle_tags.description` each sync (ours always wins). **1,100 authored as of 2026-07-13**, every one drafted + adversarially verified against real card oracle text, highest card-population first — the whole high-traffic head is covered; the remaining ~3,400 are the low-population long tail. Goal = describe all, fully replacing Scryfall. Repeatable loop (fan out subagents to draft + verify, then splice): **runbook** [`../development/runbooks/otag_description_authoring.md`](../development/runbooks/otag_description_authoring.md). Add lines, push, next `zervice` writes them in.

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
- **Business cards — reference for reprints** (task done; card at `context/marketing/business_card.html`, `cebe5d91`). Standard playing-card stock is **2.5×3.5in (63×88mm)**. For a true card feel use a card printer (MakePlayingCards/MPC, PrinterStudio, Ad Magic, or a local shop with 300–350gsm + matte/linen) — order a custom card / "custom card game" SKU. Export: open the HTML, print to PDF at 100% (no scaling), "background graphics" ON; add 1/8in bleed if the printer wants it. QR holds down to ~0.8in (current is well above).

---

## Monetization

- [ ] **TCGplayer affiliate** — application submitted 2026-06-23, **In Review** on Impact (impact.com). When approved: wire the tracking ID into `tcgplayer_url()` (`zwiper/.../outbound/buy_links.rs`) — zero UI change — then add per-card **"Buy ↗"** links (currently buy is whole-deck only).
- [ ] **Card Kingdom affiliate** — **no public self-serve program**; it's a direct-outreach partnership. Email CK when ready (cite the Archidekt `?partner=` precedent). `cardkingdom_url()` stays untracked until then.
- Detail + saved signup copy: `context/product/affiliate/tcgplayer.md`.

---

## Web/Zite Polish

- [ ] **About page (`/about`) visual overhaul — larger redesign wanted.** A partial alignment pass landed 2026-07-21 (`51f69d72`, `a007ac9f`, `c33bf479`): five-crate diagram with `zwipe-components`, the enrichment card rewritten for community oracle tags (roles derived from otag subtrees, not the retired heuristic), refreshed test counts, and the whole thing brought toward the app's tag/chip grammar (colored theme chips cycling accent 1–3 / success / warning / error, single-line wrapping header, tech stacks as chip rows, linkified imports). Owner still wants a fuller visual redesign of this section. Open bits from the pass: single-label subtitles (Scryfall "external service", PostgreSQL "primary datastore", the two foundation bands) → chips for full consistency; and the tagline comma (moot now the tagline is gone).
- [ ] **Keep zwipe.net in sync as the app grows.** The guides knowledge base shipped (12 guides under `/guides`, sitemap + per-guide `Article` JSON-LD landed 2026-07-08). No committed appetite for the demand-first SEO guides ("best mobile MTG deck builder", etc.) — leave them optional. The standing task is just to update the site (guides, feature pages, screenshots) as the app becomes more feature-rich. (SEO-guides plan archived at [`../archive/seo_guides.md`](../archive/seo_guides.md).)

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

- [ ] **Android target API level — hard deadline 2026-08-31 (Play Console alert, seen 2026-07-21).** Google Play flagged Zwipe TCG as targeting an old Android version ("Action by Aug 31"). From **2026-08-31**, if `targetSdk` isn't within one year of the latest Android release, you **can't publish app updates** (existing installs unaffected). Fix: bump `targetSdk` (currently **35**; `compileSdk` is 36) to the required level — likely **36** — in the Android release's gradle `build.gradle.kts` patch step, then cut and submit an AAB before the deadline. Details in Play Console → Zwipe TCG → the "Update your target API level" notification (View details).
- [ ] **Turn on Dependabot alerts** (repo Settings → Code security → Dependabot alerts) — the passive GitHub-Advisory backstop to the active weekly `cargo audit` workflow (`audit.yml`). Zero code, zero noise; optionally enable "Dependabot security updates" for auto-fix PRs, but skip *version* updates (the noisy weekly-bump firehose). Owner-only (a settings toggle, not a file).
- **sqlx 0.8 → 0.9** — major bump. 0.9 has breaking changes around type mappings and connection options. Needs a dedicated branch where the integration tests run against a real Postgres before merge (the suite now exists to gate it — `zerver/tests/`).
- **keyring 3 → 4** (zwiper) — major bump. Used for iOS Keychain on `apple-native`. Needs on-device test before merging; don't ship blind.
- **Pin other git deps** (optional follow-up) — `dioxus-primitives` is now pinned to rev `02801f27` (commit `b40d2019`). Audit remaining workspace deps: `grep "git = " **/Cargo.toml`. Currently no other floating git deps, but worth a periodic check.
