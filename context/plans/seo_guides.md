# SEO Guides — A Guide for Every Page of Zwipe

**Status: PLAN ONLY — do not ship yet.** Needs a full read-through of the app to
write each guide accurately. This doc is the blueprint; building it out is a
later, deliberate effort.

## Goal

Capture organic search and convert it to downloads by publishing long-form,
indexable guides on `zwipe.net`. Two things at once:

1. **A how-to guide for every user-facing page/feature of the app** — the vision
   here. Someone searching "how to import an Archidekt deck to mobile" or "MTG
   card filter by keyword" lands on a Zwipe guide that shows them exactly how,
   with a download CTA.
2. **MTG-topic guides** that ride existing search demand ("best mobile MTG deck
   builder", "how to build a Commander deck on your phone") and funnel to the app.

These are static pages prerendered by zite's existing SSG pass, so they inherit
the per-route `<head>` meta, canonical URLs, and near-zero hosting cost we already
have. No CMS, no new infra.

## Why this is deferred, not quick

Each guide must be *accurate* — screenshots/clips of the real current UI, correct
step order, correct button labels. That means walking each screen in `zwiper`
before writing. Shipping thin or stale guides is worse than none (Google discounts
low-value content, and wrong steps erode trust). So: read-through first, write
second, ship in batches.

---

## Content inventory — one guide per page

Derived from the app's screen tree (`zwiper/src/lib/inbound/screens/`). Grouped
by area; each bullet is a candidate guide. Not all need to ship day one — start
with the high-intent, high-traffic ones (marked ★).

### Getting started / account
- ★ Getting started with Zwipe (the swipe model: right add, left skip, up maybe, down undo)
- Create an account (`auth/register`)
- Log in (`auth/login`)
- Reset a forgotten password (`auth/forgot_password`)
- Verify your email (`profile/components/email_verification`)

### Building a deck (the core loop)
- ★ Build a deck by swiping (`deck/card/add`) — the flagship guide
- ★ Pick your commander by swiping (`deck/components/swipe_select`)
- Choose a format: Commander, Oathbreaker, partners, backgrounds (`deck/components/format_select`)
- Remove cards from a deck (`deck/card/remove`)
- Undo / swipe history (`deck/card/components/action_history`)
- The maybeboard (swipe up)

### Finding the right cards (filters — big cluster, big search demand)
- ★ Filter the card pool (overview: stack, clear, match modes — `deck/card/filter/*`)
- Filter by color identity (`filter/mana/color_identity`)
- Filter by mana value / CMC (`filter/mana/cmc`) and produced mana (`filter/mana/produced_mana`)
- Filter by keyword ability (`filter/oracle_text/keywords`) and rules text (`text_contains`, `oracle_words`)
- Filter by type / subtype (`filter/types/*`)
- Filter by rarity, set, artist, flavor text, price (`filter/rarity|set|artist|flavor_text|price`)
- Filter by power / toughness (`filter/combat/*`)
- Filter by mechanical category (`filter/category`)
- Sort the card pool (`filter/sort`)

### Inspecting cards
- Read a card: oracle text, real mana symbols, P/T, loyalty (`deck/card/components/card_info`, `oracle_text`)
- Understand keywords (tap-a-keyword) (`keyword_chips`, `keyword_hint`)
- Browse printings and art (`printing_sheet`, `flippable_card_image`, `image_preview`)

### Managing decks
- ★ Import a decklist — Archidekt URL or paste (`deck/import`)
- Export a deck (`deck/export`)
- Your decks, synced across sessions (`deck/list`)
- Tag decks by archetype (`deck/components/tag_select`)
- Deck stats, charts, and warnings (`deck/components/deck_stats|deck_charts|deck_warnings`)
- Clone a deck (`deck/components/clone_deck_dialog`)
- Set a land target / price target (shipped 1.1.4 — ties to the App Store "set land amount" request)

### Profile & preferences
- ★ Themes & color-blind modes (13+ themes, 3 CVD modes) (`profile/preferences`)
- Change username / email / password (`profile/change_*`)
- Delete your account (`profile/components/delete_account_dialog`)

### MTG-topic guides (demand-first, not page-first)
- ★ Best mobile MTG deck builder (2026)
- ★ How to build a Commander deck on your phone
- How to import an Archidekt deck to mobile
- Building an EDH deck with synergy-ranked suggestions
- Oathbreaker / partners / backgrounds on mobile

---

## Tech implementation (zite)

- **Routing:** add `#[route("/guides")] Guides {}` (index) and
  `#[route("/guides/:slug")] Guide { slug: String }` to `zite/src/main.rs`.
  Dynamic-segment routes are excluded from `Route::static_routes()`, so to get
  each guide prerendered, either (a) enumerate guide slugs and register them as
  concrete static routes, or (b) extend the SSG route source to emit each slug.
  Simplest to start: one component per guide + a static route each.
- **Content:** guides as Rust/RSX components (like the current pages) or as a
  small data table (slug → title, description, body sections). Keep bodies in
  their own module so they don't bloat `pages/`.
- **Meta:** reuse `PageMeta` — pass a keyword-rich `title`, `description`, and
  `path: "/guides/<slug>"`. Already produces canonical + OG/Twitter tags.
- **Structured data:** add `Article` (or `HowTo` for step guides) JSON-LD per
  guide and `BreadcrumbList` (Home › Guides › <title>), mirroring the
  `MobileApplication` JSON-LD now on the home page (`pages/home.rs`).
- **Sitemap:** add each guide path to `ROUTES` in `zite/build.rs` (the generator
  added in the SEO batch) so they land in `sitemap.xml` automatically.
- **Media:** reuse the demo clips in `zite/assets/demo/` and add per-step
  screenshots; lazy-load below the fold.

## Per-guide structure (template)

1. `<h1>` — the exact search phrase ("How to import an Archidekt deck into Zwipe").
2. One-paragraph answer up top (featured-snippet bait).
3. Numbered steps with a screenshot or demo clip each.
4. A short "why it works this way" or tips aside.
5. Cross-links to 2–3 related guides (internal linking).
6. Download CTA (App Store + Play) + the live stats strip.

## Cross-linking

- Link the home feature cards ("Swipe to Build", "Filter & Inspect", etc.) to
  their matching guide.
- Guides link to each other by area.
- Footer or nav gets a "Guides" entry once the index exists.

## Rollout

1. Read-through pass: walk each ★ screen, capture current labels + clips.
2. Ship the `/guides` index + the ~6 ★ guides first.
3. Fill in the long tail in batches; measure in Search Console before scaling.

## Open questions

- Guides as hand-written components vs. a lightweight content-data layer? (Lean
  data layer if the count grows past ~10.)
- Do guides live logged-out only, or also surface as in-app help? (Could reuse
  copy for the in-app report/help surface.)
- Screenshot maintenance: guides go stale when UI changes — decide a refresh
  cadence or generate shots from a scripted UI pass.
